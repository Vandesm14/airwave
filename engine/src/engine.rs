use std::{
  collections::{HashMap, HashSet},
  time::Instant,
  usize, vec,
};

use glam::Vec2;
use internment::Intern;
use itertools::Itertools;
use petgraph::visit::{EdgeRef, IntoNodeReferences};
use serde::{Deserialize, Serialize};
use turborand::rng::Rng;

use crate::{
  DEFAULT_TICK_RATE_TPS, KNOT_TO_FEET_PER_SECOND, MAX_TAXI_SPEED,
  NAUTICALMILES_TO_FEET,
  assets::load_assets,
  entities::{
    aircraft::{
      Aircraft, AircraftState, FlightSegment, TCAS, TaxiingState,
      events::{AircraftEvent, EventKind, handle_aircraft_event},
    },
    airport::Airport,
    world::{Game, World},
  },
  geometry::{AngleDirections, angle_between_points, delta_angle, move_point},
  line::Line,
  pathfinder::{Node, NodeBehavior, NodeKind},
  wayfinder::VORData,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type", content = "value")]
/// UI Commands come from the frontend and are handled within the engine.
pub enum UICommand {
  Pause,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// UI Events are sent from the engine to the frontend.
pub enum UIEvent {
  Pause,
}

impl From<UICommand> for UIEvent {
  fn from(value: UICommand) -> Self {
    match value {
      UICommand::Pause => Self::Pause,
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
  Aircraft(AircraftEvent),
  UiEvent(UIEvent),
}

impl From<AircraftEvent> for Event {
  fn from(value: AircraftEvent) -> Self {
    Self::Aircraft(value)
  }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum EngineConfig {
  /// Runs no collision checks.
  Minimal,

  #[default]
  /// Runs all collision checks.
  Full,
}

impl EngineConfig {
  pub fn run_collisions(&self) -> bool {
    matches!(self, EngineConfig::Full)
  }

  pub fn show_logs(&self) -> bool {
    matches!(self, EngineConfig::Full)
  }
}

#[derive(Debug, Clone)]
pub struct Engine {
  pub airports: HashMap<String, Airport>,
  pub config: EngineConfig,
  pub rng: Rng,

  pub world: World,
  pub game: Game,

  pub events: Vec<Event>,

  pub last_tick: Instant,
  pub tick_counter: usize,
  pub tick_rate_tps: usize,
}

impl Default for Engine {
  fn default() -> Self {
    Self {
      airports: Default::default(),
      config: Default::default(),
      rng: Default::default(),
      world: Default::default(),
      game: Default::default(),
      events: Default::default(),
      last_tick: Instant::now(),
      tick_counter: Default::default(),
      tick_rate_tps: DEFAULT_TICK_RATE_TPS,
    }
  }
}

impl Engine {
  pub fn load_assets(&mut self) {
    let assets = load_assets();

    self.airports = assets.airports;
  }

  pub fn airport(&self, id: impl AsRef<str>) -> Option<&Airport> {
    self.airports.get(id.as_ref())
  }

  pub fn default_airport(&self) -> Option<&Airport> {
    self.airport("default")
  }

  pub fn add_aircraft(&mut self, mut aircraft: Aircraft) {
    while self.game.aircraft.iter().any(|a| a.id == aircraft.id) {
      aircraft.id = Intern::from(Aircraft::random_callsign(&mut self.rng));
    }

    self.game.aircraft.push(aircraft);
  }

  pub fn tick(&mut self) -> Vec<Event> {
    // TODO: use real DT.
    let dt = 1.0 / self.tick_rate_tps as f32;
    self.last_tick = Instant::now();

    let tick_span =
      tracing::span!(tracing::Level::TRACE, "tick", tick = self.tick_counter);
    let _tick_span_guard = tick_span.enter();

    let mut events: Vec<Event> = Vec::new();

    if !self.events.is_empty() {
      tracing::trace!("tick events: {:?}", self.events);
    }

    if self.config.run_collisions() {
      events.extend(self.handle_tcas());
    }

    for aircraft in self.game.aircraft.iter_mut() {
      let prev = aircraft.clone();

      // Run through all events
      for event in self.events.iter().filter_map(|e| match e {
        Event::Aircraft(aircraft_event) => Some(aircraft_event),
        Event::UiEvent(_) => None,
      }) {
        if event.id == aircraft.id {
          handle_aircraft_event(
            aircraft,
            &prev,
            &event.kind,
            &mut events,
            &self.world,
            &mut self.rng,
          );
        }
      }

      // Run through all effects

      // State effects
      aircraft.update_taxiing(&mut events, &self.world, dt);
      aircraft.update_landing(&mut events, dt);
      aircraft.update_flying(&mut events, dt);

      // General effects
      aircraft.update_from_targets(dt);
      aircraft.update_position(dt);
      aircraft.update_airspace(&self.world);
      aircraft.update_segment(&mut events, &self.world, self.tick_counter);
    }

    self.compute_available_gates();

    // ATC Automation
    self.update_auto_approach(&mut events);
    self.update_auto_ground(&mut events);

    if self.config.run_collisions() {
      self.taxi_collisions();
    }

    self.tick_counter += 1;

    self.events = events;
    self.events.clone()
  }
}

// Effects
impl Engine {
  pub fn compute_available_gates(&mut self) {
    for airport in self.world.airports.iter_mut() {
      for gate in airport
        .terminals
        .iter_mut()
        .flat_map(|t| t.gates.iter_mut())
      {
        let available = !self.game.aircraft.iter().any(|a| {
          a.airspace.is_some_and(|id| id == airport.id)
            && if let AircraftState::Parked { at, .. } = &a.state {
              at.name == gate.id
            } else if let AircraftState::Taxiing {
              current, waypoints, ..
            } = &a.state
            {
              waypoints
                .iter()
                .chain(core::iter::once(current))
                .any(|w| w.name == gate.id && w.kind == NodeKind::Gate)
            } else {
              false
            }
        });

        gate.available = available;
      }
    }
  }

  pub fn handle_tcas(&mut self) -> Vec<Event> {
    let mut events: Vec<Event> = Vec::new();
    let mut collisions: HashMap<Intern<String>, TCAS> = HashMap::new();
    for pair in self.game.aircraft.iter().combinations(2) {
      let aircraft = pair.first().unwrap();
      let other_aircraft = pair.last().unwrap();

      let distance = aircraft.pos.distance_squared(other_aircraft.pos);
      let vertical_distance =
        (aircraft.altitude - other_aircraft.altitude).abs();

      let both_are_flying = matches!(aircraft.state, AircraftState::Flying)
        && matches!(other_aircraft.state, AircraftState::Flying);
      let both_are_above =
        aircraft.altitude > 2000.0 && other_aircraft.altitude > 2000.0;

      if !both_are_flying || !both_are_above {
        continue;
      }

      let a_feet_to_descend = (500.0 / aircraft.climb_speed())
        * aircraft.speed
        * KNOT_TO_FEET_PER_SECOND;
      let b_feet_to_descend = (500.0 / other_aircraft.climb_speed())
        * other_aircraft.speed
        * KNOT_TO_FEET_PER_SECOND;
      let total_distance = a_feet_to_descend + b_feet_to_descend;

      let a_angle = delta_angle(
        angle_between_points(aircraft.pos, other_aircraft.pos),
        aircraft.heading,
      );
      let b_angle = delta_angle(
        angle_between_points(other_aircraft.pos, aircraft.pos),
        other_aircraft.heading,
      );

      let a_facing = a_angle.abs() < 90.0;
      let b_facing = b_angle.abs() < 90.0;
      let facing = a_facing || b_facing;

      let in_ta_threshold = vertical_distance < 2000.0
        && distance <= (total_distance * 2.0).powf(2.0);
      let in_ra_threshold =
        vertical_distance < 1000.0 && distance <= (total_distance).powf(2.0);

      // Class A: Facing
      if facing {
        // If they are in the RA threshold, provide an RA.
        if in_ra_threshold {
          if aircraft.altitude < other_aircraft.altitude {
            collisions.insert(aircraft.id, TCAS::Descend);
            collisions.insert(other_aircraft.id, TCAS::Climb);
          } else {
            collisions.insert(aircraft.id, TCAS::Climb);
            collisions.insert(other_aircraft.id, TCAS::Descend);
          }
        // If they are outside the threshold, provide a TA.
        } else if in_ta_threshold {
          // If we came from an RA, hold altitude until we are no longer facing.
          // Else, display a TA.
          if aircraft.tcas.is_ra() {
            collisions.insert(aircraft.id, TCAS::Hold);
          } else {
            collisions.insert(aircraft.id, TCAS::Warning);
          }

          if other_aircraft.tcas.is_ra() {
            collisions.insert(other_aircraft.id, TCAS::Hold);
          } else {
            collisions.insert(other_aircraft.id, TCAS::Warning);
          }
        }
      }
    }

    self.game.aircraft.iter_mut().for_each(|aircraft| {
      if let Some(tcas) = collisions.get(&aircraft.id) {
        aircraft.tcas = *tcas;
      } else if !aircraft.tcas.is_idle() {
        if aircraft.tcas.is_ra() {
          events.push(Event::Aircraft(AircraftEvent::new(
            aircraft.id,
            EventKind::CalloutTARA,
          )));
        }

        aircraft.tcas = TCAS::Idle;
      }
    });

    events
  }

  // FIXME: There's a bug here when aircraft land it spits out a ton of
  // TaxiContinue events. Not sure why.
  pub fn taxi_collisions(&mut self) -> Vec<Event> {
    let mut events: Vec<Event> = Vec::new();
    let mut collisions: HashSet<Intern<String>> = HashSet::new();
    for pair in self
      .game
      .aircraft
      .iter()
      .filter(|a| {
        matches!(
          a.state,
          AircraftState::Taxiing { .. } | AircraftState::Parked { .. }
        )
      })
      .combinations(2)
    {
      let aircraft = pair.first().unwrap();
      let other_aircraft = pair.last().unwrap();

      // Skip checking aircraft that are not in the same airspace.
      if aircraft.airspace != other_aircraft.airspace {
        continue;
      }

      // Skip checking aircraft that are both parked or not at the same airport.
      if matches!(aircraft.state, AircraftState::Parked { .. })
        && matches!(other_aircraft.state, AircraftState::Parked { .. })
      {
        continue;
      }

      // Skip checking aircraft within automated airports.
      if aircraft
        .airspace
        .is_some_and(|id| !self.world.airport_status(id).automate_ground)
      {
        continue;
      }

      let distance_squared = aircraft.pos.distance_squared(other_aircraft.pos);
      let diff_angle_a = delta_angle(
        aircraft.heading,
        angle_between_points(aircraft.pos, other_aircraft.pos),
      );
      let diff_angle_b = delta_angle(
        other_aircraft.heading,
        angle_between_points(other_aircraft.pos, aircraft.pos),
      );

      let rel_pos_a = Vec2::new(
        distance_squared * diff_angle_a.to_radians().sin().abs(),
        distance_squared * diff_angle_a.to_radians().cos(),
      );

      let rel_pos_b = Vec2::new(
        distance_squared * diff_angle_b.to_radians().sin().abs(),
        distance_squared * diff_angle_b.to_radians().cos(),
      );

      let min_forward_distance = 0.0;
      let forward_distance = 150.0_f32.powf(2.0);
      let side_distance = 120.0_f32.powf(2.0);

      // Aircraft
      if rel_pos_a.y >= min_forward_distance
        && rel_pos_a.x <= side_distance
        && rel_pos_a.y <= forward_distance
        && aircraft.speed <= MAX_TAXI_SPEED
      {
        collisions.insert(aircraft.id);
      }

      // Other Aircraft
      if rel_pos_b.y >= min_forward_distance
        && rel_pos_b.x <= side_distance
        && rel_pos_b.y <= forward_distance
        && other_aircraft.speed <= MAX_TAXI_SPEED
      {
        collisions.insert(other_aircraft.id);
      }
    }

    for aircraft in self.game.aircraft.iter_mut() {
      if let AircraftState::Taxiing { state, .. } = &mut aircraft.state {
        if collisions.contains(&aircraft.id) && state == &TaxiingState::Armed {
          *state = TaxiingState::Stopped;
          events.push(Event::Aircraft(AircraftEvent::new(
            aircraft.id,
            EventKind::TaxiHold { and_state: false },
          )));
        } else if !collisions.contains(&aircraft.id)
          && matches!(state, TaxiingState::Override | TaxiingState::Stopped)
        {
          if matches!(state, TaxiingState::Stopped) {
            events.push(Event::Aircraft(AircraftEvent::new(
              aircraft.id,
              EventKind::TaxiContinue,
            )));
          }

          *state = TaxiingState::Armed;
        }
      }
    }

    events
  }

  pub fn update_auto_approach(&mut self, events: &mut Vec<Event>) {
    let airspaces = self
      .game
      .aircraft
      .iter()
      .filter(|a| {
        matches!(a.segment, FlightSegment::Approach | FlightSegment::Landing)
      })
      .filter(|a| {
        a.airspace
          .is_some_and(|id| self.world.airport_status(id).automate_air)
      })
      .fold(
        HashMap::<_, Vec<(Intern<String>, f32)>>::new(),
        |mut map, aircraft| {
          let airspace = aircraft.airspace.unwrap();
          let key = (
            airspace,
            aircraft
              .flight_plan
              .active_waypoints()
              .last()
              .map(|w| w.name)
              .unwrap_or_default(),
          );
          let item = (
            aircraft.id,
            aircraft
              .flight_plan
              .distances(aircraft.pos)
              .last()
              .copied()
              .unwrap_or(0.0),
          );
          if let Some(entry) = map.get_mut(&key) {
            entry.push(item);
          } else {
            map.insert(key, vec![item]);
          }

          map
        },
      );

    let separation_distance = NAUTICALMILES_TO_FEET * 5.0;
    let min_approach_speed = 150.0;
    for (_, mut aircraft) in airspaces.into_iter() {
      aircraft.sort_by(|a, b| {
        a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
      });

      let mut aircraft = aircraft.into_iter();
      let Some(first) = aircraft.next() else {
        continue;
      };
      let mut current = first.1;

      let mut speeds: Vec<(Intern<String>, f32, f32)> = Vec::new();
      for (id, distance) in aircraft {
        let diff = distance - current;
        if diff < separation_distance {
          let half_sep = separation_distance * 0.5;
          if diff < half_sep {
            // For the first half, use min_approach_speed and scale offset from 30.0 to 0.0
            let offset = 30.0 * (1.0 - diff / half_sep);
            speeds.push((id, min_approach_speed, -offset));
          } else {
            // For the second half, interpolate speed up to 250.0, offset is 0
            let t = ((diff - half_sep) / half_sep).clamp(0.0, 1.0);
            let speed = min_approach_speed + t * (250.0 - min_approach_speed);
            speeds.push((id, speed.min(250.0), 0.0));
          }
        } else {
          speeds.push((id, 250.0, 0.0));
        }

        current = distance;
      }

      for (id, speed, offset) in speeds
        .into_iter()
        .chain(core::iter::once((first.0, 250.0, 0.0)))
      {
        if let Some(aircraft) =
          self.game.aircraft.iter_mut().find(|a| a.id == id)
        {
          // Only change speeds for aircraft on approach.
          if aircraft.segment == FlightSegment::Approach
            && aircraft.target.speed != speed
          {
            aircraft.flight_plan.course_offset = offset;
            events.push(
              AircraftEvent::new(aircraft.id, EventKind::Speed(speed)).into(),
            );
          }
        }
      }
    }

    for aircraft in self.game.aircraft.iter_mut() {
      if matches!(aircraft.segment, FlightSegment::Approach)
        && aircraft
          .airspace
          .is_some_and(|a| self.world.airport_status(a).automate_air)
      {
        if let Some(airport) = self
          .world
          .airports
          .iter()
          .find(|a| aircraft.airspace.is_some_and(|id| id == a.id))
        {
          let runway = if let Some(star) = aircraft
            .flight_plan
            .waypoints
            .iter()
            .find(|w| w.name == Intern::from_ref("STAR"))
          {
            airport
              .runways
              .iter()
              .min_by(|a, b| {
                let dist_a = star.data.pos.distance_squared(a.start);
                let dist_b = star.data.pos.distance_squared(b.start);
                dist_a
                  .partial_cmp(&dist_b)
                  .unwrap_or(std::cmp::Ordering::Equal)
              })
              .unwrap()
          } else {
            tracing::error!("No STAR, so no runway for {}!", aircraft.id);
            continue;
          };

          let directions = AngleDirections::new(runway.heading);
          let pattern_length = NAUTICALMILES_TO_FEET * 10.0;
          let final_fix =
            move_point(runway.start, directions.backward, pattern_length);

          let pattern_direction = directions.left;

          let base_fix = move_point(
            final_fix,
            pattern_direction,
            NAUTICALMILES_TO_FEET * 5.0,
          );

          let reverse_downwind = delta_angle(
            angle_between_points(aircraft.pos, final_fix),
            directions.forward,
          )
          .abs()
            < 90.0;
          let downwind_fix = if reverse_downwind {
            move_point(base_fix, directions.backward, pattern_length)
          } else {
            move_point(base_fix, directions.forward, pattern_length)
          };
          let crosswind_fix = move_point(
            downwind_fix,
            pattern_direction,
            -NAUTICALMILES_TO_FEET * 5.0,
          );

          let crosswind_wp = Node::default()
            .with_name(Intern::from_ref("CW"))
            .with_vor(VORData::new(crosswind_fix));
          let downwind_wp = Node::default()
            .with_name(Intern::from_ref(if reverse_downwind {
              "UW"
            } else {
              "DW"
            }))
            .with_vor(VORData::new(downwind_fix));
          let base_wp = Node::default()
            .with_name(Intern::from_ref("BS"))
            .with_vor(VORData::new(base_fix));
          let final_wp = Node::default()
            .with_name(runway.id)
            .with_vor(VORData::new(final_fix));

          let waypoints: Vec<Node<VORData>> =
            vec![crosswind_wp, downwind_wp, base_wp, final_wp];

          let altitude = 4000.0;
          let speed = 250.0;

          if aircraft.flight_plan.at_end() {
            aircraft.flight_plan.amend_end(waypoints);
            aircraft.flight_plan.start_following();
          }

          if aircraft.target.altitude >= altitude {
            events.push(
              AircraftEvent::new(aircraft.id, EventKind::Altitude(altitude))
                .into(),
            );
          }

          if aircraft.target.speed > speed {
            events.push(
              AircraftEvent::new(aircraft.id, EventKind::SpeedAtOrBelow(speed))
                .into(),
            );
          }

          if let Some(wp) = aircraft.flight_plan.waypoint() {
            if wp.data.pos == final_fix {
              let distance = wp.data.pos.distance_squared(aircraft.pos);
              let land_distance = (NAUTICALMILES_TO_FEET * 2.0).powf(2.0);

              if distance <= land_distance {
                events.push(
                  AircraftEvent::new(
                    aircraft.id,
                    EventKind::SpeedAtOrBelow(180.0),
                  )
                  .into(),
                );

                events.push(
                  AircraftEvent::new(aircraft.id, EventKind::Land(runway.id))
                    .into(),
                );
              }
            }
          }
        }
      }
    }
  }

  pub fn update_auto_ground(&mut self, events: &mut Vec<Event>) {
    for aircraft in self.game.aircraft.iter() {
      if aircraft
        .airspace
        .is_some_and(|a| self.world.airport_status(a).automate_ground)
      {
        if matches!(aircraft.segment, FlightSegment::TaxiArr)
          && aircraft.speed <= MAX_TAXI_SPEED
        {
          if let AircraftState::Taxiing {
            current, waypoints, ..
          } = &aircraft.state
          {
            if waypoints
              .iter()
              .chain(core::iter::once(current))
              .all(|w| w.kind != NodeKind::Gate)
            {
              if let Some(airport) = self
                .world
                .airports
                .iter()
                .find(|a| aircraft.airspace.is_some_and(|id| id == a.id))
              {
                let available_gate = airport
                  .terminals
                  .iter()
                  .flat_map(|t| t.gates.iter())
                  .find(|g| g.available);
                if let Some(gate) = available_gate {
                  events.push(
                    AircraftEvent::new(
                      aircraft.id,
                      EventKind::Taxi(vec![Node::new(
                        gate.id,
                        NodeKind::Gate,
                        NodeBehavior::Park,
                        (),
                      )]),
                    )
                    .into(),
                  );

                  // TODO: Instead of only scheduling one aircraft, keep a
                  // tally of gates we've sent aircraft to instead of relying
                  // on the `compute_available_gates` method which runs once
                  // per tick.
                  return;
                }
              }
            }
          }
        } else if matches!(aircraft.segment, FlightSegment::Parked) {
          if let AircraftState::Parked { .. } = &aircraft.state {
            if let Some(airport) = self
              .world
              .airports
              .iter()
              .find(|a| aircraft.airspace.is_some_and(|id| id == a.id))
            {
              let departure = self
                .world
                .airports
                .iter()
                .find(|a| a.id == aircraft.flight_plan.departing);
              let arrival = self
                .world
                .airports
                .iter()
                .find(|a| a.id == aircraft.flight_plan.arriving);
              if let Some((departure, arrival)) = departure.zip(arrival) {
                let departure_angle =
                  angle_between_points(departure.center, arrival.center);
                let runways = departure.runways.iter();

                let mut smallest_angle = f32::MAX;
                let mut closest = None;
                for runway in runways {
                  let diff = delta_angle(runway.heading, departure_angle).abs();
                  if diff < smallest_angle {
                    smallest_angle = diff;
                    closest = Some(runway);
                  }
                }

                // If an airport doesn't have a runway, we have other problems.
                let runway = closest.unwrap();
                let node_index = airport
                  .pathfinder
                  .graph
                  .node_references()
                  .find(|(_, w)| {
                    w.name_and_kind_eq(&Node::<Line>::from(runway))
                  })
                  .map(|(i, _)| i);
                if let Some(index) = node_index {
                  let mut points =
                    airport.pathfinder.graph.edges(index).collect::<Vec<_>>();
                  points.sort_by(|a, b| {
                    let dist_a = a.weight().distance_squared(runway.start);
                    let dist_b = b.weight().distance_squared(runway.start);
                    dist_a
                      .partial_cmp(&dist_b)
                      .unwrap_or(std::cmp::Ordering::Equal)
                  });

                  if let Some(closest) = points.first() {
                    let other = if closest.source() == index {
                      closest.target()
                    } else {
                      closest.source()
                    };
                    let other =
                      airport.pathfinder.graph.node_weight(other).unwrap();

                    // tracing::info!("taxi departure: {}", aircraft.id);
                    events.push(
                      AircraftEvent::new(
                        aircraft.id,
                        EventKind::Taxi(vec![other.into(), runway.into()]),
                      )
                      .into(),
                    );
                  }
                }
              }
            }
          }
        } else if matches!(aircraft.segment, FlightSegment::TaxiDep) {
          if let AircraftState::Taxiing {
            current, waypoints, ..
          } = &aircraft.state
          {
            if current.kind == NodeKind::Runway && waypoints.is_empty() {
              events.push(
                AircraftEvent::new(
                  aircraft.id,
                  EventKind::Takeoff(current.name),
                )
                .into(),
              );
            }
          }
        }
      }
    }
  }
}
