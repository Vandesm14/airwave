use std::time::Duration;

use glam::Vec2;
use internment::Intern;
use serde::{Deserialize, Serialize};
use turborand::TurboRand;

use crate::{
  angle_between_points,
  command::{CommandReply, CommandWithFreq, Task, TaskWaypoint},
  engine::Bundle,
  entities::{
    airport::Runway,
    world::{closest_airport, find_random_airspace},
  },
  heading_to_direction,
  pathfinder::{Node, NodeBehavior, NodeKind, NodeVORData, Pathfinder},
};

use super::{
  actions::ActionKind, Action, Aircraft, AircraftState, DEPARTURE_WAIT_RANGE,
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
  DirectTo(Vec<TaskWaypoint>),
  ResumeOwnNavigation,

  // Transitions
  Land(Intern<String>),
  GoAround,
  Touchdown,
  Takeoff(Intern<String>),
  DepartureFromArrival,

  // Taxiing
  Taxi(Vec<Node<()>>),
  TaxiContinue,
  TaxiHold,
  Clearance {
    speed: Option<f32>,
    altitude: Option<f32>,
    departure: Option<Vec<TaskWaypoint>>,
  },

  // Requests
  DirectionOfTravel,
  Ident,

  // Callouts
  Callout(CommandWithFreq),

  // Internal
  Delete,
}

impl From<Task> for EventKind {
  fn from(value: Task) -> Self {
    match value {
      Task::Altitude(x) => EventKind::Altitude(x),
      Task::Clearance {
        departure,
        altitude,
        speed,
      } => EventKind::Clearance {
        speed,
        altitude,
        departure,
      },
      Task::Direct(x) => EventKind::DirectTo(x),
      Task::DirectionOfTravel => EventKind::DirectionOfTravel,
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
pub struct Event {
  pub id: Intern<String>,
  pub kind: EventKind,
}

impl Event {
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

          // Cancel waypoints
          bundle
            .actions
            .push(Action::new(aircraft.id, ActionKind::Flying(Vec::new())));
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
        let airspace_id =
          aircraft.airspace.unwrap_or(aircraft.flight_plan.arriving);
        if let Some(airspace) =
          bundle.airspaces.iter().find(|a| a.id == airspace_id)
        {
          if let Some(frequency) = airspace.frequencies.try_from_string(frq) {
            bundle
              .actions
              .push(Action::new(aircraft.id, ActionKind::Frequency(frequency)));
          }
        }
      }

      // Flying
      EventKind::DirectTo(waypoints) => {
        if let AircraftState::Flying { .. } = aircraft.state {
          handle_direct_to_event(aircraft, bundle, waypoints);
        }
      }
      EventKind::ResumeOwnNavigation => {
        if let AircraftState::Flying { .. } = aircraft.state {
          let arrival = bundle
            .airspaces
            .iter()
            .find(|a| a.id == aircraft.flight_plan.arriving);

          if let Some(arrival) = arrival {
            bundle.actions.push(Action {
              id: aircraft.id,
              kind: ActionKind::TargetSpeed(400.0),
            });
            bundle.actions.push(Action {
              id: aircraft.id,
              kind: ActionKind::TargetAltitude(13000.0),
            });
            bundle.actions.push(Action {
              id: aircraft.id,
              kind: ActionKind::Flying(vec![Node {
                name: arrival.id,
                kind: NodeKind::Runway,
                behavior: NodeBehavior::GoTo,
                value: NodeVORData {
                  to: arrival.pos,
                  then: Vec::new(),
                },
              }]),
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
      EventKind::DepartureFromArrival => {
        if let AircraftState::Taxiing { .. } = aircraft.state {
          let departure = bundle
            .airspaces
            .iter()
            .find(|a| a.id == aircraft.flight_plan.arriving);
          let arrival = find_random_airspace(bundle.airspaces, bundle.rng);
          if let Some((departure, destination)) = departure.zip(arrival) {
            let frequency = departure.frequencies.clearance;
            let wait_time = Duration::from_secs(
              bundle.rng.sample_iter(DEPARTURE_WAIT_RANGE).unwrap(),
            );
            bundle.actions.push(Action::new(
              aircraft.id,
              ActionKind::DepartureFromArrival {
                departure: departure.id,
                destination: destination.id,
                wait_time,
              },
            ));
            bundle
              .actions
              .push(Action::new(aircraft.id, ActionKind::Frequency(frequency)))
          }
        }
      }

      // Taxiing
      EventKind::Taxi(waypoints) => {
        if let AircraftState::Taxiing { .. } = aircraft.state {
          if let Some(airport) = closest_airport(bundle.airspaces, aircraft.pos)
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
      EventKind::Clearance {
        speed,
        altitude,
        departure,
      } => {
        if let AircraftState::Taxiing { .. } = aircraft.state {
          handle_clearance_event(
            aircraft,
            bundle,
            *speed,
            *altitude,
            departure.as_deref(),
          );
        }
      }

      // Requests
      EventKind::DirectionOfTravel => {
        if let Some(arrival) = bundle
          .airspaces
          .iter()
          .find(|a| a.id == aircraft.flight_plan.arriving)
        {
          let heading = angle_between_points(aircraft.pos, arrival.pos);
          let direction: String = heading_to_direction(heading).into();

          bundle.events.push(Event::new(
            aircraft.id,
            EventKind::Callout(CommandWithFreq {
              id: aircraft.id.to_string(),
              frequency: aircraft.frequency,
              reply: CommandReply::DirectionOfDeparture { direction },
              tasks: Vec::new(),
            }),
          ));
        }
      }
      EventKind::Ident => {
        bundle.events.push(Event::new(
          aircraft.id,
          EventKind::Callout(CommandWithFreq {
            id: aircraft.id.to_string(),
            frequency: aircraft.frequency,
            reply: CommandReply::Empty,
            tasks: Vec::new(),
          }),
        ));
      }

      // Callouts are handled outside of the engine.
      EventKind::Callout(..) => {}

      // Internal
      EventKind::Delete => {
        // This is handled outside of the engine
        bundle
          .events
          .push(Event::new(aircraft.id, EventKind::Delete));
      }
    }
  }
}

pub fn handle_land_event(
  aircraft: &Aircraft,
  bundle: &mut Bundle,
  runway_id: Intern<String>,
) {
  if let AircraftState::Flying { .. } = aircraft.state {
    if let Some(airspace) = aircraft
      .airspace
      .and_then(|airspace| bundle.airspaces.iter().find(|a| a.id == airspace))
    {
      if let Some(runway) = airspace
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
}

pub fn parse_waypoint_strings(
  bundle: &Bundle,
  waypoint_strings: &[Intern<String>],
) -> Option<Vec<Node<NodeVORData>>> {
  waypoint_strings
    .iter()
    .map(|w| bundle.waypoints.iter().find(|n| &n.name == w).cloned())
    .rev()
    .try_fold(Vec::new(), |mut vec, item| {
      vec.push(item?);

      Some(vec)
    })
}

pub fn parse_arrival(
  bundle: &Bundle,
  waypoints: &mut Vec<Node<NodeVORData>>,
  arrival_string: Intern<String>,
) {
  if let Some(wps) = bundle.waypoint_sets.arrival.get(&arrival_string) {
    waypoints.extend(wps.to_vec());
  }
}

pub fn parse_approach(
  bundle: &Bundle,
  waypoints: &mut Vec<Node<NodeVORData>>,
  approach_string: Intern<String>,
) {
  if let Some(wps) = bundle.waypoint_sets.approach.get(&approach_string) {
    waypoints.extend(wps.to_vec());
  }
}

pub fn parse_departure(
  bundle: &Bundle,
  waypoints: &mut Vec<Node<NodeVORData>>,
  departure_string: Intern<String>,
) {
  if let Some(wps) = bundle.waypoint_sets.departure.get(&departure_string) {
    waypoints.extend(wps.to_vec());
  }
}

pub fn parse_direct(
  bundle: &Bundle,
  waypoints: &mut Vec<Node<NodeVORData>>,
  name: Intern<String>,
) {
  if let Some(wp) = bundle.waypoints.iter().find(|wp| wp.name == name) {
    waypoints.push(wp.clone());
  }
}

pub fn parse_task_waypoints(
  aircraft: &Aircraft,
  bundle: &mut Bundle,
  task_waypoints: &[TaskWaypoint],
) -> Vec<Node<NodeVORData>> {
  let mut waypoints: Vec<Node<NodeVORData>> = Vec::new();
  for wp in task_waypoints.iter() {
    match wp {
      TaskWaypoint::Approach(id) => parse_approach(bundle, &mut waypoints, *id),
      TaskWaypoint::Arrival(id) => parse_arrival(bundle, &mut waypoints, *id),
      TaskWaypoint::Departure(id) => {
        parse_departure(bundle, &mut waypoints, *id)
      }
      TaskWaypoint::Direct(id) => {
        parse_direct(bundle, &mut waypoints, *id);
      }
      TaskWaypoint::Destination => {
        let id = aircraft.flight_plan.arriving;
        parse_direct(bundle, &mut waypoints, id);
      }
    }
  }

  waypoints
}

pub fn handle_direct_to_event(
  aircraft: &Aircraft,
  bundle: &mut Bundle,
  waypoints: &[TaskWaypoint],
) {
  let mut waypoints = parse_task_waypoints(aircraft, bundle, waypoints);
  waypoints.reverse();

  bundle.actions.push(Action {
    id: aircraft.id,
    kind: ActionKind::Flying(waypoints),
  });
}

pub fn handle_taxi_event(
  aircraft: &Aircraft,
  bundle: &mut Bundle,
  waypoint_strings: &[Node<()>],
  pathfinder: &Pathfinder,
) {
  if let AircraftState::Taxiing { current, .. } = &aircraft.state {
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

  bundle.events.push(Event {
    id: aircraft.id,
    kind: EventKind::TaxiContinue,
  });
}

pub fn handle_takeoff_event(
  aircraft: &Aircraft,
  bundle: &mut Bundle,
  runway_id: Intern<String>,
) {
  if let AircraftState::Taxiing { current, .. } = &aircraft.state {
    // If we are at the runway
    if let Some(runway) = bundle
      .airspaces
      .iter()
      .flat_map(|a| a.airports.iter())
      .flat_map(|a| a.runways.iter())
      .find(|r| r.id == runway_id)
    {
      if NodeKind::Runway == current.kind && current.name == runway_id {
        bundle.actions.push(Action {
          id: aircraft.id,
          kind: ActionKind::Pos(runway.start()),
        });
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
          kind: ActionKind::Flying(aircraft.flight_plan.waypoints.clone()),
        })
      }
    }

    // TODO: handle if the waypoint is coming up, update the behavior
    // to take off (once we have waypoint behaviors for takeoff)
  }
}

pub fn handle_clearance_event(
  aircraft: &Aircraft,
  bundle: &mut Bundle,
  speed: Option<f32>,
  altitude: Option<f32>,
  departure: Option<&[TaskWaypoint]>,
) {
  let mut waypoints = departure
    .map(|departure| parse_task_waypoints(aircraft, bundle, departure))
    .unwrap_or_default();
  waypoints.reverse();

  bundle.actions.push(Action {
    id: aircraft.id,
    kind: ActionKind::Clearance {
      speed,
      altitude,
      waypoints,
    },
  });
}
