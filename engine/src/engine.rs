use std::{
  collections::{HashMap, HashSet},
  time::Instant,
};

use glam::Vec2;
use internment::Intern;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use turborand::rng::Rng;

use crate::{
  DEFAULT_TICK_RATE_TPS, KNOT_TO_FEET_PER_SECOND,
  assets::load_assets,
  entities::{
    aircraft::{
      Aircraft, AircraftState, TCAS, TaxiingState,
      events::{AircraftEvent, EventKind, handle_aircraft_event},
    },
    airport::Airport,
    world::{Game, World},
  },
  geometry::{angle_between_points, delta_angle},
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
    self.compute_available_gates();

    if self.config.show_logs() && !self.events.is_empty() {
      tracing::trace!("tick events: {:?}", self.events);
    }

    if self.config.run_collisions() {
      events.extend(self.handle_tcas());
    }

    for aircraft in self.game.aircraft.iter_mut() {
      // Capture the previous state
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
      aircraft.update_taxiing(&mut events, &self.world.airports, dt);
      aircraft.update_landing(&mut events, dt);
      aircraft.update_flying(&mut events, dt);

      // General effects
      aircraft.update_from_targets(dt);
      aircraft.update_position(dt);
      aircraft.update_segment(
        &mut events,
        &self.world.airports,
        self.tick_counter,
      );
      aircraft.update_airspace(&self.world.airports);
    }

    if self.config.run_collisions() {
      self.taxi_collisions();
    }

    // Capture the left over events and actions for next time
    if self.config.show_logs() && !events.is_empty() {
      // TODO: decide if we want to keep this or discard this.
      // tracing::info!("new events: {:?}", bundle.events);
    }

    self.events = events;
    self.events.clone()
  }
}

// Effects
impl Engine {
  pub fn compute_available_gates(&mut self) {
    for gate in self
      .world
      .airports
      .iter_mut()
      .flat_map(|a| a.terminals.iter_mut())
      .flat_map(|t| t.gates.iter_mut())
    {
      let aircraft = self.game.aircraft.iter().find(|a| {
        if let AircraftState::Parked { at, .. } = &a.state {
          at.name == gate.id && a.pos == gate.pos
        } else {
          false
        }
      });

      gate.available = aircraft.is_none();
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

      let a_feet_to_descend = (500.0 / aircraft.dt_climb_speed(1.0))
        * aircraft.speed
        * KNOT_TO_FEET_PER_SECOND;
      let b_feet_to_descend = (500.0 / other_aircraft.dt_climb_speed(1.0))
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

      // Skip checking aircraft that are both parked or not at the same airport.
      if matches!(aircraft.state, AircraftState::Parked { .. })
        && matches!(other_aircraft.state, AircraftState::Parked { .. })
        || aircraft.airspace != other_aircraft.airspace
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
      {
        collisions.insert(aircraft.id);
      }

      // Other Aircraft
      if rel_pos_b.y >= min_forward_distance
        && rel_pos_b.x <= side_distance
        && rel_pos_b.y <= forward_distance
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
}
