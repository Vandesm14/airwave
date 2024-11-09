use glam::Vec2;
use internment::Intern;
use serde::{Deserialize, Serialize};

use crate::{
  command::{CommandReply, CommandWithFreq, Task},
  engine::Bundle,
  entities::{airport::Runway, world::closest_airport},
  pathfinder::{new_vor, Node, NodeBehavior, NodeKind, Pathfinder},
};

use super::{actions::ActionKind, Action, Aircraft, AircraftState};

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
  TaxiHold,

  // Requests
  Ident,

  // Callouts
  Callout(CommandWithFreq),

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
      Task::TaxiHold => EventKind::TaxiHold,
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
  fn run(aircraft: &Aircraft, event: &EventKind, bundle: &mut Bundle);
}

pub struct HandleAircraftEvent;
impl AircraftEventHandler for HandleAircraftEvent {
  fn run(aircraft: &Aircraft, event: &EventKind, bundle: &mut Bundle) {
    match event {
      // Any
      EventKind::Speed(speed) => {
        bundle
          .actions
          .push(Action::new(aircraft.id, ActionKind::TargetSpeed(*speed)));
      }
      EventKind::SpeedAtOrBelow(speed) => {
        if aircraft.target.speed > *speed {
          bundle
            .actions
            .push(Action::new(aircraft.id, ActionKind::TargetSpeed(*speed)));
        }
      }
      EventKind::SpeedAtOrAbove(speed) => {
        if aircraft.target.speed < *speed {
          bundle
            .actions
            .push(Action::new(aircraft.id, ActionKind::TargetSpeed(*speed)));
        }
      }
      EventKind::Heading(heading) => {
        if let AircraftState::Flying { .. } = aircraft.state {
          bundle.actions.push(Action::new(
            aircraft.id,
            ActionKind::TargetHeading(*heading),
          ));
        }
      }
      EventKind::Altitude(altitude) => {
        bundle.actions.push(Action::new(
          aircraft.id,
          ActionKind::TargetAltitude(*altitude),
        ));
      }
      EventKind::AltitudeAtOrBelow(altitude) => {
        if aircraft.target.altitude > *altitude {
          bundle.actions.push(Action::new(
            aircraft.id,
            ActionKind::TargetAltitude(*altitude),
          ));
        }
      }
      EventKind::AltitudeAtOrAbove(altitude) => {
        if aircraft.target.altitude < *altitude {
          bundle.actions.push(Action::new(
            aircraft.id,
            ActionKind::TargetAltitude(*altitude),
          ));
        }
      }
      EventKind::Frequency(frequency) => {
        bundle
          .actions
          .push(Action::new(aircraft.id, ActionKind::Frequency(*frequency)));
      }
      EventKind::NamedFrequency(frq) => {
        if let Some(frequency) =
          bundle.airspace.frequencies.try_from_string(frq)
        {
          bundle
            .actions
            .push(Action::new(aircraft.id, ActionKind::Frequency(frequency)));
        }
      }

      // Flying
      EventKind::ResumeOwnNavigation => {
        if let AircraftState::Flying { .. } = aircraft.state {
          let arrival = bundle
            .connections
            .iter()
            .find(|a| a.id == aircraft.flight_plan.arriving);

          if let Some(arrival) = arrival {
            bundle.actions.push(Action {
              id: aircraft.id,
              kind: ActionKind::TargetSpeed(300.0),
            });
            bundle.actions.push(Action {
              id: aircraft.id,
              kind: ActionKind::TargetAltitude(13000.0),
            });
            bundle.actions.push(Action {
              id: aircraft.id,
              kind: ActionKind::Flying(vec![
                new_vor(arrival.id, arrival.transition)
                  .with_name(Intern::from_ref("TRSN"))
                  .with_behavior(vec![
                    EventKind::EnRoute(false),
                    EventKind::SpeedAtOrBelow(250.0),
                  ]),
                new_vor(arrival.id, arrival.pos)
                  .with_name(Intern::from_ref("APRT"))
                  .with_behavior(vec![
                    EventKind::AltitudeAtOrBelow(7000.0),
                    EventKind::FlipFlightPlan,
                  ]),
                new_vor(arrival.id, arrival.transition)
                  .with_name(Intern::from_ref("TRSN"))
                  .with_behavior(vec![EventKind::EnRoute(true)]),
              ]),
            });
          }
        }
      }

      // Transitions
      EventKind::Land(runway) => handle_land_event(aircraft, bundle, *runway),
      EventKind::GoAround => {
        if let AircraftState::Landing(..) = aircraft.state {
          bundle
            .actions
            .push(Action::new(aircraft.id, ActionKind::Flying(Vec::new())));
          bundle
            .actions
            .push(Action::new(aircraft.id, ActionKind::SyncTargets));
        }
      }
      EventKind::Touchdown => {
        if let AircraftState::Landing(runway) = &aircraft.state {
          handle_touchdown_event(aircraft, bundle, runway);
        }
      }
      EventKind::Takeoff(runway) => {
        if let AircraftState::Taxiing { .. } = aircraft.state {
          handle_takeoff_event(aircraft, bundle, *runway);
        }
      }
      EventKind::EnRoute(bool) => {
        if let AircraftState::Flying { .. } = &aircraft.state {
          bundle.actions.push(Action {
            id: aircraft.id,
            kind: ActionKind::EnRoute(*bool),
          });
        }
      }
      EventKind::FlipFlightPlan => {
        bundle.actions.push(Action {
          id: aircraft.id,
          kind: ActionKind::FlipFlightPlan,
        });
      }

      // Taxiing
      EventKind::Taxi(waypoints) => {
        if let AircraftState::Taxiing { .. } | AircraftState::Parked(..) =
          aircraft.state
        {
          if let Some(airport) = closest_airport(bundle.airspace, aircraft.pos)
          {
            handle_taxi_event(aircraft, bundle, waypoints, &airport.pathfinder);
          }
        }
      }
      EventKind::TaxiContinue => {
        if let AircraftState::Taxiing { .. } = aircraft.state {
          bundle.actions.push(Action {
            id: aircraft.id,
            kind: ActionKind::TargetSpeed(20.0),
          });
        }
      }
      EventKind::TaxiHold => {
        if let AircraftState::Taxiing { .. } = aircraft.state {
          bundle.actions.push(Action {
            id: aircraft.id,
            kind: ActionKind::Speed(0.0),
          });
          bundle.actions.push(Action {
            id: aircraft.id,
            kind: ActionKind::TargetSpeed(0.0),
          });
        }
      }

      // Requests
      EventKind::Ident => {
        bundle.events.push(
          AircraftEvent::new(
            aircraft.id,
            EventKind::Callout(CommandWithFreq {
              id: aircraft.id.to_string(),
              frequency: aircraft.frequency,
              reply: CommandReply::Empty,
              tasks: Vec::new(),
            }),
          )
          .into(),
        );
      }

      // Callouts are handled outside of the engine.
      EventKind::Callout(..) => {}

      // Internal
      EventKind::Delete => {
        // This is handled outside of the engine
        bundle
          .events
          .push(AircraftEvent::new(aircraft.id, EventKind::Delete).into());
      }

      // Points
      // Points are handled outside of the engine
      EventKind::SuccessfulTakeoff => {}
      EventKind::SuccessfulLanding => {}
    }
  }
}

pub fn handle_land_event(
  aircraft: &Aircraft,
  bundle: &mut Bundle,
  runway_id: Intern<String>,
) {
  if let AircraftState::Flying { .. } = aircraft.state {
    if let Some(runway) = bundle
      .airspace
      .airports
      .iter()
      .flat_map(|a| a.runways.iter())
      .find(|r| r.id == runway_id)
    {
      bundle.actions.push(Action::new(
        aircraft.id,
        ActionKind::Landing(runway.clone()),
      ));
    }
  }
}

pub fn handle_touchdown_event(
  aircraft: &Aircraft,
  bundle: &mut Bundle,
  runway: &Runway,
) {
  bundle
    .actions
    .push(Action::new(aircraft.id, ActionKind::TargetAltitude(0.0)));
  bundle
    .actions
    .push(Action::new(aircraft.id, ActionKind::Altitude(0.0)));
  bundle.actions.push(Action::new(
    aircraft.id,
    ActionKind::TargetHeading(runway.heading),
  ));
  bundle.actions.push(Action::new(
    aircraft.id,
    ActionKind::Heading(runway.heading),
  ));

  bundle
    .actions
    .push(Action::new(aircraft.id, ActionKind::TargetSpeed(0.0)));

  bundle.actions.push(Action::new(
    aircraft.id,
    ActionKind::Taxi {
      current: Node {
        name: runway.id,
        kind: NodeKind::Runway,
        behavior: NodeBehavior::GoTo,
        value: aircraft.pos,
      },
      waypoints: Vec::new(),
    },
  ));

  bundle.events.push(
    AircraftEvent {
      id: aircraft.id,
      kind: EventKind::SuccessfulLanding,
    }
    .into(),
  );
}

pub fn handle_taxi_event(
  aircraft: &Aircraft,
  bundle: &mut Bundle,
  waypoint_strings: &[Node<()>],
  pathfinder: &Pathfinder,
) {
  if let AircraftState::Taxiing { current, .. }
  | AircraftState::Parked(current) = &aircraft.state
  {
    let destinations = waypoint_strings.iter();
    let mut all_waypoints: Vec<Node<Vec2>> = Vec::new();

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

    all_waypoints.reverse();
    bundle.actions.push(Action {
      id: aircraft.id,
      kind: ActionKind::TaxiWaypoints(all_waypoints.clone()),
    });

    tracing::info!(
      "Initiating taxi for {}: {:?}",
      aircraft.id,
      all_waypoints.iter().map(|w| w.name).collect::<Vec<_>>()
    );

    if all_waypoints.is_empty() {
      return;
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
  aircraft: &Aircraft,
  bundle: &mut Bundle,
  runway_id: Intern<String>,
) {
  if let AircraftState::Taxiing { current, .. } = &aircraft.state {
    // If we are at the runway
    if let Some(runway) = bundle
      .airspace
      .airports
      .iter()
      .flat_map(|a| a.runways.iter())
      .find(|r| r.id == runway_id)
    {
      if NodeKind::Runway == current.kind && current.name == runway_id {
        bundle.actions.push(Action {
          id: aircraft.id,
          kind: ActionKind::TargetSpeed(aircraft.flight_plan.speed),
        });
        bundle.actions.push(Action {
          id: aircraft.id,
          kind: ActionKind::TargetAltitude(aircraft.flight_plan.altitude),
        });

        bundle.actions.push(Action {
          id: aircraft.id,
          kind: ActionKind::Heading(runway.heading),
        });
        bundle.actions.push(Action {
          id: aircraft.id,
          kind: ActionKind::TargetHeading(runway.heading),
        });

        bundle.actions.push(Action {
          id: aircraft.id,
          kind: ActionKind::Flying(vec![]),
        });

        bundle.events.push(
          AircraftEvent {
            id: aircraft.id,
            kind: EventKind::SuccessfulTakeoff,
          }
          .into(),
        );
      }
    }
  }
}
