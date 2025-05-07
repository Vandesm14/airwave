use glam::Vec2;
use internment::Intern;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use turborand::TurboRand;

use crate::{
  APPROACH_ALTITUDE, ARRIVAL_ALTITUDE, EAST_CRUISE_ALTITUDE,
  NAUTICALMILES_TO_FEET, WEST_CRUISE_ALTITUDE,
  command::{CommandReply, CommandWithFreq, Task},
  engine::{Bundle, Event},
  entities::world::{
    AirspaceStatus, ArrivalStatus, closest_airport, closest_airspace,
  },
  geometry::{angle_between_points, delta_angle, inverse_degrees, move_point},
  heading_to_direction,
  pathfinder::{
    Node, NodeBehavior, NodeKind, Pathfinder, display_node_vec2,
    display_vec_node_vec2,
  },
  wayfinder::{VORLimit, VORLimits, new_vor},
};

use super::{
  Aircraft, AircraftKind, AircraftState, FlightSegment, LandingState,
  TaxiingState,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EventKind {
  // Any
  Speed(f32),
  SpeedAtOrBelow(f32),
  SpeedAtOrAbove(f32),
  Frequency(f32),
  NamedFrequency(String),

  // Flying
  Heading(f32),
  Altitude(f32),
  AltitudeAtOrBelow(f32),
  AltitudeAtOrAbove(f32),
  ResumeOwnNavigation {
    diversion: bool,
  },
  Direct(Intern<String>),

  // Transitions
  Land(Intern<String>),
  GoAround,
  Touchdown,
  Takeoff(Intern<String>),

  // Taxiing
  Taxi(Vec<Node<()>>),
  TaxiContinue,
  TaxiHold {
    and_state: bool,
  },
  LineUp(Intern<String>),

  // Requests
  Ident,

  // Callouts
  Callout(CommandWithFreq),
  CalloutTARA,

  // State
  Segment(FlightSegment),

  // Configuration and Automation
  /// Teleports an aircraft from its gate to the takeoff phase.
  QuickDepart,
  /// Teleports an aircraft from the approach phase to a gate.
  QuickArrive,

  // External
  // TODO: I think the engine can handle this instead internally.
  Delete,
}

impl From<Task> for EventKind {
  fn from(value: Task) -> Self {
    match value {
      Task::Altitude(x) => EventKind::Altitude(x),
      Task::Direct(s) => EventKind::Direct(s),
      Task::Frequency(x) => EventKind::Frequency(x),
      Task::GoAround => EventKind::GoAround,
      Task::Heading(x) => EventKind::Heading(x),
      Task::Ident => EventKind::Ident,
      Task::Land(x) => EventKind::Land(x),
      Task::NamedFrequency(x) => EventKind::NamedFrequency(x),
      Task::ResumeOwnNavigation => {
        EventKind::ResumeOwnNavigation { diversion: false }
      }
      Task::Speed(x) => EventKind::Speed(x),
      Task::Takeoff(x) => EventKind::Takeoff(x),
      Task::Taxi(x) => EventKind::Taxi(x),
      Task::TaxiContinue => EventKind::TaxiContinue,
      Task::TaxiHold => EventKind::TaxiHold { and_state: true },
      Task::LineUp(x) => EventKind::LineUp(x),
      Task::Delete => EventKind::Delete,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AircraftEvent {
  pub id: Intern<String>,
  pub kind: EventKind,
}

impl AircraftEvent {
  pub fn new(id: Intern<String>, kind: EventKind) -> Self {
    Self { id, kind }
  }
}

pub trait AircraftEventHandler {
  fn run(aircraft: &mut Aircraft, event: &EventKind, bundle: &mut Bundle);
}

pub struct HandleAircraftEvent;
impl AircraftEventHandler for HandleAircraftEvent {
  fn run(aircraft: &mut Aircraft, event: &EventKind, bundle: &mut Bundle) {
    match event {
      // Any
      EventKind::Speed(speed) => {
        aircraft.target.speed = *speed;
      }
      EventKind::SpeedAtOrBelow(speed) => {
        if aircraft.target.speed > *speed {
          aircraft.target.speed = *speed;
        }
      }
      EventKind::SpeedAtOrAbove(speed) => {
        if aircraft.target.speed < *speed {
          aircraft.target.speed = *speed;
        }
      }
      EventKind::Heading(heading) => {
        if let AircraftState::Flying = aircraft.state {
          aircraft.target.heading = *heading;

          // Cancel waypoints
          aircraft.flight_plan.stop_following();
        } else if let AircraftState::Landing { .. } = &aircraft.state {
          aircraft.target.heading = *heading;
        }
      }
      EventKind::Altitude(altitude) => {
        aircraft.target.altitude = *altitude;
      }
      EventKind::AltitudeAtOrBelow(altitude) => {
        if aircraft.target.altitude > *altitude {
          aircraft.target.altitude = *altitude;
        }
      }
      EventKind::AltitudeAtOrAbove(altitude) => {
        if aircraft.target.altitude < *altitude {
          aircraft.target.altitude = *altitude;
        }
      }
      EventKind::Frequency(frequency) => {
        aircraft.frequency = *frequency;
      }
      EventKind::NamedFrequency(frq) => {
        if let Some(frequency) =
          closest_airport(&bundle.world.airspaces, aircraft.pos)
            .and_then(|x| x.frequencies.try_from_string(frq))
        {
          aircraft.frequency = frequency;
        }
      }

      // Flying
      EventKind::ResumeOwnNavigation { diversion } => {
        // TODO: Reimplement
        if let AircraftState::Flying = aircraft.state {
          let departure = bundle
            .world
            .airspaces
            .iter()
            .find(|a| a.id == aircraft.flight_plan.departing);
          let arrival = bundle
            .world
            .airspaces
            .iter()
            .find(|a| a.id == aircraft.flight_plan.arriving);

          if let Some((departure, arrival)) = departure.zip(arrival) {
            let auto_approach = arrival.auto;

            let main_course_heading =
              angle_between_points(departure.pos, arrival.pos);
            let runways =
              arrival.airports.first().map(|a| a.runways.iter()).unwrap();

            let mut smallest_angle = f32::MAX;
            let mut closest = None;
            for runway in runways {
              let diff = delta_angle(runway.heading, main_course_heading).abs();
              if diff < smallest_angle {
                smallest_angle = diff;
                closest = Some(runway);
              }
            }

            // If an airport doesn't have a runway, we have other problems.
            let runway = closest.unwrap();

            let transition_sid = departure
              .pos
              .move_towards(arrival.pos, NAUTICALMILES_TO_FEET * 30.0);
            let transition_star = arrival
              .pos
              .move_towards(departure.pos, NAUTICALMILES_TO_FEET * 30.0);
            let transition_iaf = move_point(
              runway.start,
              inverse_degrees(runway.heading),
              NAUTICALMILES_TO_FEET * 15.0,
            );
            let transition_vctr = transition_star.lerp(transition_iaf, 0.5);

            let cruise_alt = if (0.0..180.0).contains(&main_course_heading) {
              EAST_CRUISE_ALTITUDE
            } else {
              WEST_CRUISE_ALTITUDE
            };
            let wp_sid = new_vor(Intern::from_ref("SID"), transition_sid)
              .with_actions(vec![
                EventKind::SpeedAtOrAbove(AircraftKind::A21N.stats().max_speed),
                EventKind::AltitudeAtOrAbove(cruise_alt),
                EventKind::Frequency(
                  departure.airports.first().unwrap().frequencies.center,
                ),
              ]);

            let wp_star = new_vor(Intern::from_ref("STAR"), transition_star)
              .with_limits(
                VORLimits::new()
                  .with_altitude(VORLimit::AtOrBelow(ARRIVAL_ALTITUDE))
                  .with_speed(VORLimit::AtOrBelow(250.0)),
              );
            let wp_vctr = new_vor(Intern::from_ref("VCTR"), transition_vctr)
              .with_actions(vec![EventKind::Land(runway.id)])
              .with_limits(
                VORLimits::new()
                  .with_altitude(VORLimit::AtOrBelow(APPROACH_ALTITUDE))
                  .with_speed(VORLimit::AtOrBelow(180.0)),
              );

            // Generate track waypoints.
            let min_wp_distance = NAUTICALMILES_TO_FEET * 90.0;
            let mut cmp = departure.pos;

            let mut waypoints = Vec::new();
            while let Some(closest) = bundle
              .world
              .waypoints
              .iter()
              .filter(|w| {
                w.data != cmp
                  // Ensure the waypoint keeps us on course.
                  && delta_angle(
                    angle_between_points(cmp, w.data),
                    main_course_heading,
                  )
                  .abs()
                    <= 45.0
                  // Ensure the waypoint doesn't take us too far.
                  && delta_angle(
                    angle_between_points(w.data, arrival.pos),
                    main_course_heading,
                  )
                  .abs()
                    <= 45.0
                  // Ensure the waypoint is within minimum distance.
                  && cmp.distance_squared(w.data) <= min_wp_distance.powf(2.0)
              })
              .min_by(|a, b| {
                let a = angle_between_points(cmp, a.data);
                let b = angle_between_points(cmp, b.data);

                let a = delta_angle(a, main_course_heading).abs();
                let b = delta_angle(b, main_course_heading).abs();

                a.partial_cmp(&b).unwrap()
              })
            {
              cmp = closest.data;
              waypoints.push(new_vor(closest.name, closest.data));
            }

            waypoints.push(wp_star);
            // Only add auto-vectors if we are arriving at an auto airspace.
            if auto_approach {
              waypoints.push(wp_vctr);
            }

            if !diversion {
              waypoints.insert(0, wp_sid);
            } else {
              for event in wp_sid.data.events.iter() {
                bundle.events.push(Event::Aircraft(AircraftEvent::new(
                  aircraft.id,
                  event.clone(),
                )));
              }
            }

            aircraft.flight_plan.clear_waypoints();
            aircraft.flight_plan.waypoints = waypoints;
          }
        }
      }
      EventKind::Direct(wp) => {
        if let Some((index, _)) = aircraft
          .flight_plan
          .waypoints
          .iter()
          .find_position(|w| w.name == *wp)
        {
          aircraft.flight_plan.set_index(index);
        }
      }

      // Transitions
      EventKind::Land(runway) => handle_land_event(aircraft, bundle, *runway),
      EventKind::GoAround => {
        if let AircraftState::Landing { .. } = aircraft.state {
          aircraft.state = AircraftState::Flying;
          aircraft.flight_plan.stop_following();
          aircraft.sync_targets_to_vals();

          bundle.events.push(
            AircraftEvent {
              id: aircraft.id,
              kind: EventKind::AltitudeAtOrAbove(3000.0),
            }
            .into(),
          );
          bundle.events.push(
            AircraftEvent {
              id: aircraft.id,
              kind: EventKind::SpeedAtOrAbove(250.0),
            }
            .into(),
          );
        }
      }
      EventKind::Touchdown => {
        if let AircraftState::Landing { .. } = aircraft.state {
          handle_touchdown_event(aircraft, bundle);
        }
      }
      EventKind::Takeoff(runway) => {
        if let AircraftState::Taxiing { .. } = aircraft.state {
          handle_takeoff_event(aircraft, bundle, *runway);
        }
      }

      // Taxiing
      EventKind::Taxi(waypoints) => {
        if let AircraftState::Taxiing { .. } | AircraftState::Parked { .. } =
          aircraft.state
        {
          if let Some(airport) =
            closest_airport(&bundle.world.airspaces, aircraft.pos)
          {
            handle_taxi_event(aircraft, bundle, waypoints, &airport.pathfinder);
          }
        }
      }
      EventKind::TaxiContinue => {
        if let AircraftState::Taxiing { state, .. } = &mut aircraft.state {
          match state {
            TaxiingState::Armed | TaxiingState::Override => {}
            TaxiingState::Holding => {
              *state = TaxiingState::Armed;
            }
            TaxiingState::Stopped => {
              *state = TaxiingState::Override;
            }
          }

          aircraft.target.speed = 20.0;
        }
      }
      EventKind::TaxiHold { and_state: force } => {
        if let AircraftState::Taxiing { state, .. } = &mut aircraft.state {
          aircraft.target.speed = 0.0;
          aircraft.speed = 0.0;

          if *force {
            *state = TaxiingState::Holding;
          }
        } else if let AircraftState::Parked { .. } = aircraft.state {
          aircraft.target.speed = 0.0;
          aircraft.speed = 0.0;
        }
      }
      EventKind::LineUp(runway) => {
        if let AircraftState::Taxiing { waypoints, .. } = &mut aircraft.state {
          // If we were told to hold short, line up instead
          if let Some(wp) = waypoints.first_mut() {
            if wp.kind == NodeKind::Runway && wp.name == *runway {
              wp.behavior = NodeBehavior::LineUp;
            }
          }

          bundle.events.push(Event::Aircraft(AircraftEvent::new(
            aircraft.id,
            EventKind::TaxiContinue,
          )));
        }
      }

      // Requests
      EventKind::Ident => {
        bundle.events.push(
          AircraftEvent::new(
            aircraft.id,
            EventKind::Callout(CommandWithFreq::new(
              aircraft.id.to_string(),
              aircraft.frequency,
              CommandReply::Empty,
              Vec::new(),
            )),
          )
          .into(),
        );
      }

      // Generic callouts are handled outside of the engine.
      EventKind::Callout(..) => {}
      EventKind::CalloutTARA => {
        handle_callout_tara(aircraft, bundle);
      }

      // State
      EventKind::Segment(segment) => {
        // TODO: Remove this once we don't need the vis.
        // tracing::info!(
        //   "Setting segment for {} from {:?} to {:?}",
        //   aircraft.id,
        //   aircraft.segment,
        //   segment
        // );

        aircraft.segment = *segment;

        match segment {
          FlightSegment::Unknown => {}
          FlightSegment::Dormant => {
            aircraft.flight_time = None;
          }
          FlightSegment::Boarding => {}
          FlightSegment::Parked => handle_parked_segment(aircraft, bundle),
          FlightSegment::TaxiDep => {}
          FlightSegment::Takeoff => {}
          FlightSegment::Departure => {}
          FlightSegment::Climb => {}
          FlightSegment::Cruise => {}
          FlightSegment::Arrival => {}
          FlightSegment::Approach => handle_approach_segment(aircraft, bundle),
          FlightSegment::Landing => {}
          FlightSegment::TaxiArr => {}
        }
      }

      // Configuration and Automation
      EventKind::QuickDepart => {
        if let AircraftState::Parked { .. } = &aircraft.state {
          let departure = bundle
            .world
            .airspaces
            .iter()
            .find(|a| a.id == aircraft.flight_plan.departing);
          let arrival = bundle
            .world
            .airspaces
            .iter()
            .find(|a| a.id == aircraft.flight_plan.arriving);
          if let Some((departure, arrival)) = departure.zip(arrival) {
            let departure_angle =
              angle_between_points(departure.pos, arrival.pos);
            let runways = departure
              .airports
              .first()
              .map(|a| a.runways.iter())
              .unwrap();

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

            aircraft.pos = runway.start;
            aircraft.heading = runway.heading;
            aircraft.target.heading = runway.heading;

            aircraft.state = AircraftState::Taxiing {
              current: Node::new(
                runway.id,
                NodeKind::Runway,
                NodeBehavior::Takeoff,
                runway.start,
              ),
              waypoints: Vec::new(),
              state: TaxiingState::default(),
            };

            bundle.events.push(Event::Aircraft(AircraftEvent::new(
              aircraft.id,
              EventKind::Takeoff(runway.id),
            )));
          } else {
            tracing::error!("No arrival airspace found for {:?}", aircraft.id);
          }
        }
      }
      EventKind::QuickArrive => {
        let arrival = bundle
          .world
          .airspaces
          .iter()
          .find(|a| a.id == aircraft.flight_plan.arriving)
          .and_then(|a| {
            a.airports
              .iter()
              .find(|a| a.id == aircraft.flight_plan.arriving)
          });
        if let Some(arrival) = arrival {
          let available_gate = arrival
            .terminals
            .iter()
            .flat_map(|t| t.gates.iter())
            .find(|g| g.available);
          if let Some(gate) = available_gate {
            aircraft.state = AircraftState::Parked {
              at: Node::new(
                gate.id,
                NodeKind::Gate,
                NodeBehavior::Park,
                gate.pos,
              ),
            };

            aircraft.pos = gate.pos;

            aircraft.speed = 0.0;
            aircraft.heading = gate.heading;
            aircraft.altitude = 0.0;
            aircraft.sync_targets_to_vals();

            aircraft.flip_flight_plan();
          } else {
            tracing::error!(
              "No available gates for {} at {}",
              aircraft.id,
              aircraft.flight_plan.arriving
            );
          }
        }
      }

      // External
      EventKind::Delete => {
        tracing::info!("Deleting aircraft: {}", aircraft.id);
        // This is handled outside of the engine.
        bundle
          .events
          .push(AircraftEvent::new(aircraft.id, EventKind::Delete).into());
      }
    }
  }
}

pub fn handle_land_event(
  aircraft: &mut Aircraft,
  bundle: &mut Bundle,
  runway_id: Intern<String>,
) {
  if let AircraftState::Flying = aircraft.state {
    if let Some(runway) = closest_airport(&bundle.world.airspaces, aircraft.pos)
      .and_then(|x| x.runways.iter().find(|r| r.id == runway_id))
    {
      aircraft.state = AircraftState::Landing {
        runway: runway.clone(),
        state: LandingState::default(),
      };
    }
  }
}

pub fn handle_touchdown_event(aircraft: &mut Aircraft, bundle: &mut Bundle) {
  let AircraftState::Landing { runway, .. } = &mut aircraft.state else {
    unreachable!("outer function asserts that aircraft is landing")
  };

  let airspace = closest_airspace(&bundle.world.airspaces, aircraft.pos);
  if let Some(airspace) = airspace {
    if airspace.auto {
      bundle.events.push(Event::Aircraft(AircraftEvent::new(
        aircraft.id,
        EventKind::QuickArrive,
      )));
    }
  }

  aircraft.target.altitude = 0.0;
  aircraft.altitude = 0.0;
  aircraft.target.heading = runway.heading;
  aircraft.heading = runway.heading;

  aircraft.target.speed = 0.0;

  aircraft.state = AircraftState::Taxiing {
    current: Node {
      name: runway.id,
      kind: NodeKind::Runway,
      behavior: NodeBehavior::GoTo,
      data: aircraft.pos,
    },
    waypoints: Vec::new(),
    state: TaxiingState::Override,
  };
}

pub fn handle_taxi_event(
  aircraft: &mut Aircraft,
  bundle: &mut Bundle,
  waypoint_strings: &[Node<()>],
  pathfinder: &Pathfinder,
) {
  if let AircraftState::Taxiing { current, .. }
  | AircraftState::Parked { at: current, .. } = &aircraft.state
  {
    let mut destinations = waypoint_strings.iter().peekable();
    let mut all_waypoints: Vec<Node<Vec2>> = Vec::new();

    if destinations.peek().map(|d| d.name_and_kind_eq(current)) == Some(true) {
      tracing::info!(
        "Skipping {} as we are there.",
        display_node_vec2(current)
      );
      destinations.next();
    }

    let mut pos = aircraft.pos;
    let mut heading = aircraft.heading;
    let mut curr: Node<Vec2> = current.clone();
    for destination in destinations {
      let path = pathfinder.path_to(
        Node {
          name: curr.name,
          kind: curr.kind,
          behavior: curr.behavior,
          data: (),
        },
        destination.clone(),
        pos,
        heading,
      );

      if let Some(path) = path {
        pos = path.final_pos;
        heading = path.final_heading;
        curr = path.path.last().unwrap().clone();

        all_waypoints.extend(path.path);
      } else {
        tracing::error!(
          "Failed to find path for destination: {:?}, from: {:?}",
          destination,
          curr
        );
        return;
      }
    }

    // If our destination is a gate, set our destination to that gate
    // (otherwise it will be the enterance on the apron but not the gate)
    if let Some(last) = all_waypoints.last() {
      if last.kind == NodeKind::Gate {
        if let Some(airport) =
          closest_airport(&bundle.world.airspaces, aircraft.pos)
        {
          if let Some(gate) = airport
            .terminals
            .iter()
            .flat_map(|t| t.gates.iter())
            .find(|g| g.id == last.name)
          {
            all_waypoints.push(Node::new(
              last.name,
              last.kind,
              NodeBehavior::Park,
              gate.pos,
            ));
          }
        }
      }
    }

    if all_waypoints.is_empty() {
      return;
    }

    all_waypoints.reverse();

    tracing::info!(
      "Initiating taxi for {}: {:?}",
      aircraft.id,
      display_vec_node_vec2(&all_waypoints)
    );

    let current = current.clone();
    if let AircraftState::Taxiing { waypoints, .. } = &mut aircraft.state {
      *waypoints = all_waypoints;
    } else {
      aircraft.state = AircraftState::Taxiing {
        current: current.clone(),
        waypoints: all_waypoints,
        state: TaxiingState::default(),
      };
    }
  }

  bundle.events.push(
    AircraftEvent {
      id: aircraft.id,
      kind: EventKind::TaxiContinue,
    }
    .into(),
  );
}

pub fn handle_takeoff_event(
  aircraft: &mut Aircraft,
  bundle: &mut Bundle,
  runway_id: Intern<String>,
) {
  if let AircraftState::Taxiing {
    current, waypoints, ..
  } = &mut aircraft.state
  {
    // If we are at the runway
    if let Some(runway) = closest_airport(&bundle.world.airspaces, aircraft.pos)
      .and_then(|x| x.runways.iter().find(|r| r.id == runway_id))
    {
      if NodeKind::Runway == current.kind && current.name == runway_id {
        aircraft.target.speed = aircraft.flight_plan.speed;
        aircraft.target.altitude = aircraft.flight_plan.altitude;
        aircraft.heading = runway.heading;
        aircraft.target.heading = runway.heading;

        aircraft.state = AircraftState::Flying;

        bundle.events.push(
          AircraftEvent {
            id: aircraft.id,
            kind: EventKind::ResumeOwnNavigation { diversion: false },
          }
          .into(),
        );
      } else if let Some(runway) = waypoints.first_mut() {
        if runway.kind == NodeKind::Runway && runway.name == runway_id {
          runway.behavior = NodeBehavior::Takeoff;

          bundle.events.push(
            AircraftEvent::new(aircraft.id, EventKind::TaxiContinue).into(),
          );
        }
      }
    }
  }
}

pub fn handle_parked_segment(aircraft: &mut Aircraft, bundle: &mut Bundle) {
  if let AircraftState::Parked { at } = &aircraft.state {
    if let Some(airspace) =
      closest_airspace(&bundle.world.airspaces, aircraft.pos)
    {
      if let Some(airport) = airspace
        .airports
        .iter()
        .find(|a| a.id == aircraft.flight_plan.departing)
      {
        aircraft.frequency = airport.frequencies.ground;

        bundle.events.push(
          AircraftEvent {
            id: aircraft.id,
            kind: EventKind::Callout(CommandWithFreq::new(
              aircraft.id.to_string(),
              aircraft.frequency,
              CommandReply::ReadyForTaxi {
                gate: at.name.to_string(),
              },
              Vec::new(),
            )),
          }
          .into(),
        );
      }
    }
  }
}

pub fn handle_approach_segment(aircraft: &mut Aircraft, bundle: &mut Bundle) {
  if let Some(airspace) =
    closest_airspace(&bundle.world.airspaces, aircraft.pos)
  {
    // If we are not arriving at the right airspace, ignore this event.
    if airspace.id != aircraft.flight_plan.arriving {
      return;
    }

    // TODO: This ensures that the flight is in the right segment, though
    // we might be better off using more concrete logic in the effect
    // instead of relying on this in case that logic fails.
    aircraft.segment = FlightSegment::Approach;
    aircraft.frequency =
      airspace.airports.first().unwrap().frequencies.approach;

    if !airspace.auto {
      if matches!(
        bundle.world.airspace_statuses.get(&airspace.id),
        Some(AirspaceStatus {
          // If airspace is accepting inbounds.
          arrival: ArrivalStatus::Normal,
          ..
        })
      ) {
        // TODO: This clears all waypoints to force the player to deal
        // with the approach rather than use its automated routing.
        // This might break future implementations of routing and
        // waypoints so please check this TODO when that happens.
        aircraft.flight_plan.clear_waypoints();

        aircraft.target.heading =
          angle_between_points(aircraft.pos, airspace.pos);

        let direction = heading_to_direction(angle_between_points(
          airspace.pos,
          aircraft.pos,
        ))
        .to_owned();
        let command = CommandWithFreq::new(
          Intern::to_string(&aircraft.id),
          aircraft.frequency,
          CommandReply::ArriveInAirspace {
            direction,
            altitude: aircraft.altitude,
          },
          Vec::new(),
        );

        bundle.events.push(Event::Aircraft(AircraftEvent::new(
          aircraft.id,
          EventKind::Callout(command),
        )));
      } else {
        // If not accepted, go to a random airspace.
        let arrival = bundle
          .rng
          .sample_iter(bundle.world.airspaces.iter().filter(|a| a.auto))
          .map(|a| a.id);
        if let Some(arrival) = arrival {
          // Use our old arrival as our departure.
          aircraft.flip_flight_plan();
          // Set our new arrival.
          aircraft.flight_plan.arriving = arrival;

          // Recompute waypoints.
          bundle.events.push(Event::Aircraft(AircraftEvent::new(
            aircraft.id,
            EventKind::ResumeOwnNavigation { diversion: true },
          )));
        }
      }
    }
  }
}

pub fn handle_callout_tara(aircraft: &mut Aircraft, bundle: &mut Bundle) {
  let command = CommandWithFreq::new(
    Intern::to_string(&aircraft.id),
    aircraft.frequency,
    CommandReply::TARAResolved {
      assigned_alt: aircraft.target.altitude,
    },
    Vec::new(),
  );

  bundle.events.push(Event::Aircraft(AircraftEvent::new(
    aircraft.id,
    EventKind::Callout(command),
  )));
}
