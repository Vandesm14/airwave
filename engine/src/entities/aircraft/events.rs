use glam::Vec2;
use internment::Intern;
use serde::{Deserialize, Serialize};

use crate::{
  angle_between_points,
  command::{CommandReply, CommandWithFreq, Task},
  engine::{Bundle, Event},
  entities::world::closest_airport,
  heading_to_direction,
  pathfinder::{
    display_node_vec2, display_vec_node_vec2, new_vor, Node, NodeBehavior,
    NodeKind, Pathfinder,
  },
};

use super::{Aircraft, AircraftState, LandingState, TaxiingState};

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
  ResumeOwnNavigation,

  // Transitions
  Land(Intern<String>),
  GoAround,
  Touchdown,
  Takeoff(Intern<String>),
  EnRoute(bool),
  FlipFlightPlan,

  // Taxiing
  Taxi(Vec<Node<()>>),
  TaxiContinue,
  TaxiHold { and_state: bool },

  // Requests
  Ident,

  // Callouts
  Callout(CommandWithFreq),
  CalloutInAirspace,

  // Internal
  Delete,

  // Points
  SuccessfulTakeoff,
  SuccessfulLanding,
}

impl From<Task> for EventKind {
  fn from(value: Task) -> Self {
    match value {
      Task::Altitude(x) => EventKind::Altitude(x),
      Task::Frequency(x) => EventKind::Frequency(x),
      Task::GoAround => EventKind::GoAround,
      Task::Heading(x) => EventKind::Heading(x),
      Task::Ident => EventKind::Ident,
      Task::Land(x) => EventKind::Land(x),
      Task::NamedFrequency(x) => EventKind::NamedFrequency(x),
      Task::ResumeOwnNavigation => EventKind::ResumeOwnNavigation,
      Task::Speed(x) => EventKind::Speed(x),
      Task::Takeoff(x) => EventKind::Takeoff(x),
      Task::Taxi(x) => EventKind::Taxi(x),
      Task::TaxiContinue => EventKind::TaxiContinue,
      Task::TaxiHold => EventKind::TaxiHold { and_state: true },
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
        if let AircraftState::Flying { enroute, .. } = aircraft.state {
          aircraft.target.heading = *heading;

          // Cancel waypoints of not enroute
          if !enroute {
            aircraft.state = AircraftState::Flying {
              enroute: false,
              waypoints: Vec::new(),
            };
          }
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
          bundle.world.airspace.frequencies.try_from_string(frq)
        {
          aircraft.frequency = frequency;
        }
      }

      // Flying
      EventKind::ResumeOwnNavigation => {
        if let AircraftState::Flying { enroute, .. } = aircraft.state {
          let arrival = bundle
            .world
            .connections
            .iter()
            .find(|a| a.id == aircraft.flight_plan.arriving);

          if let Some(arrival) = arrival {
            aircraft.target.speed = 300.0;
            aircraft.target.altitude = 13000.0;
            aircraft.state = AircraftState::Flying {
              enroute,
              waypoints: vec![
                new_vor(arrival.id, arrival.transition)
                  .with_name(Intern::from_ref("TRSN"))
                  .with_behavior(vec![
                    EventKind::EnRoute(false),
                    EventKind::CalloutInAirspace,
                    EventKind::SpeedAtOrBelow(250.0),
                  ]),
                // TODO: Add a waypoint between APRT and TRSN that decreases
                // their altitude and speed so they use cruise rules until
                // transition to airspace.
                new_vor(arrival.id, arrival.pos)
                  .with_name(Intern::from_ref("APRT"))
                  .with_behavior(vec![
                    EventKind::AltitudeAtOrBelow(7000.0),
                    EventKind::FlipFlightPlan,
                  ]),
                new_vor(arrival.id, arrival.transition)
                  .with_name(Intern::from_ref("TRSN"))
                  .with_behavior(vec![EventKind::EnRoute(true)]),
              ],
            }
          }
        }
      }

      // Transitions
      EventKind::Land(runway) => handle_land_event(aircraft, bundle, *runway),
      EventKind::GoAround => {
        if let AircraftState::Landing { .. } = aircraft.state {
          aircraft.state = AircraftState::Flying {
            waypoints: Vec::new(),
            enroute: false,
          };
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
              kind: EventKind::SpeedAtOrAbove(210.0),
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
      EventKind::EnRoute(bool) => {
        if let AircraftState::Flying { enroute, .. } = &mut aircraft.state {
          *enroute = *bool;
        }

        // TODO: Automatically tuning them to approach when they enter the
        // airspace might not be the best UX.
        if !bool {
          bundle.events.push(Event::Aircraft(AircraftEvent::new(
            aircraft.id,
            EventKind::Frequency(bundle.world.airspace.frequencies.approach),
          )))
        }
      }
      EventKind::FlipFlightPlan => {
        aircraft.flip_flight_plan();
      }

      // Taxiing
      EventKind::Taxi(waypoints) => {
        if let AircraftState::Taxiing { .. } | AircraftState::Parked { .. } =
          aircraft.state
        {
          if let Some(airport) =
            closest_airport(&bundle.world.airspace, aircraft.pos)
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

      // Callouts are handled outside of the engine.
      EventKind::Callout(..) => {}
      EventKind::CalloutInAirspace => {
        let direction = heading_to_direction(angle_between_points(
          bundle.world.airspace.pos,
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
      }

      // Internal
      EventKind::Delete => {
        // This is handled outside of the engine.
        bundle
          .events
          .push(AircraftEvent::new(aircraft.id, EventKind::Delete).into());
      }

      // Points
      // Points are handled within the engine itself.
      EventKind::SuccessfulTakeoff => {}
      EventKind::SuccessfulLanding => {}
    }
  }
}

pub fn handle_land_event(
  aircraft: &mut Aircraft,
  bundle: &mut Bundle,
  runway_id: Intern<String>,
) {
  if let AircraftState::Flying { .. } = aircraft.state {
    if let Some(runway) = bundle
      .world
      .airspace
      .airports
      .iter()
      .flat_map(|a| a.runways.iter())
      .find(|r| r.id == runway_id)
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
      value: aircraft.pos,
    },
    waypoints: Vec::new(),
    state: TaxiingState::default(),
  };

  bundle.events.push(
    AircraftEvent {
      id: aircraft.id,
      kind: EventKind::SuccessfulLanding,
    }
    .into(),
  );
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
    let mut current: Node<Vec2> = current.clone();
    for destination in destinations {
      let path = pathfinder.path_to(
        Node {
          name: current.name,
          kind: current.kind,
          behavior: current.behavior,
          value: (),
        },
        destination.clone(),
        pos,
        heading,
      );

      if let Some(path) = path {
        pos = path.final_pos;
        heading = path.final_heading;
        current = path.path.last().unwrap().clone();

        all_waypoints.extend(path.path);
      } else {
        tracing::error!(
          "Failed to find path for destination: {:?}, from: {:?}",
          destination,
          current
        );
        return;
      }
    }

    // If our destination is a gate, set our destination to that gate
    // (otherwise it will be the enterance on the apron but not the gate)
    if let Some(last) = all_waypoints.last() {
      if last.kind == NodeKind::Gate {
        if let Some(airport) =
          closest_airport(&bundle.world.airspace, aircraft.pos)
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

    all_waypoints.reverse();
    // bundle.actions.push(Action {
    //   id: aircraft.id,
    //   kind: ActionKind::TaxiWaypoints(all_waypoints.clone()),
    // });

    if let AircraftState::Taxiing { waypoints, .. } = &mut aircraft.state {
      if all_waypoints.is_empty() {
        return;
      }

      tracing::info!(
        "Initiating taxi for {}: {:?}",
        aircraft.id,
        display_vec_node_vec2(&all_waypoints)
      );

      *waypoints = all_waypoints;
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
  if let AircraftState::Taxiing { current, .. } = &aircraft.state {
    // If we are at the runway
    if let Some(runway) = bundle
      .world
      .airspace
      .airports
      .iter()
      .flat_map(|a| a.runways.iter())
      .find(|r| r.id == runway_id)
    {
      if NodeKind::Runway == current.kind && current.name == runway_id {
        aircraft.target.speed = aircraft.flight_plan.speed;
        aircraft.target.altitude = aircraft.flight_plan.altitude;
        aircraft.heading = runway.heading;
        aircraft.target.heading = runway.heading;

        aircraft.state = AircraftState::Flying {
          enroute: false,
          waypoints: Vec::new(),
        };

        bundle.events.push(
          AircraftEvent {
            id: aircraft.id,
            kind: EventKind::SuccessfulTakeoff,
          }
          .into(),
        );
        bundle.events.push(
          AircraftEvent {
            id: aircraft.id,
            kind: EventKind::ResumeOwnNavigation,
          }
          .into(),
        );
      }
    }
  }
}
