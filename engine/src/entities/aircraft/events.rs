use glam::Vec2;
use internment::Intern;

use crate::{
  command::Task,
  engine::Bundle,
  entities::{airport::Runway, world::closest_airport},
  pathfinder::{Node, NodeBehavior, NodeKind, NodeVORData, Pathfinder},
};

use super::{actions::ActionKind, Action, Aircraft, AircraftState};

#[derive(Debug, Clone, PartialEq)]
pub enum EventKind {
  // Any
  Speed(f32),
  Heading(f32),
  Altitude(f32),
  Frequency(f32),
  NamedFrequency(String),
  Ident,

  // Flying
  DirectTo(Vec<Intern<String>>),
  FollowApproach(Intern<String>),
  FollowArrival(Intern<String>),
  FollowDeparture(Intern<String>),
  ResumeOwnNavigation,

  // Transitions
  Land(Intern<String>),
  GoAround,
  Touchdown,
  Takeoff(Intern<String>),

  // Taxiing
  Taxi(Vec<Node<()>>),
  TaxiContinue,
  TaxiHold,
  Clearance {
    speed: Option<f32>,
    altitude: Option<f32>,
    departure: Option<Intern<String>>,
  },
}

impl From<Task> for EventKind {
  fn from(value: Task) -> Self {
    match value {
      Task::Altitude(x) => EventKind::Altitude(x),
      Task::Approach(x) => EventKind::FollowApproach(Intern::from(x)),
      Task::Arrival(x) => EventKind::FollowArrival(Intern::from(x)),
      Task::Clearance {
        departure,
        altitude,
        speed,
      } => EventKind::Clearance {
        speed,
        altitude,
        departure: departure.map(Intern::from),
      },
      Task::Depart(x) => EventKind::FollowDeparture(Intern::from(x)),
      Task::Direct(x) => {
        EventKind::DirectTo(x.iter().cloned().map(Intern::from).collect())
      }
      Task::DirectionOfTravel => todo!(),
      Task::Frequency(x) => EventKind::Frequency(x),
      Task::GoAround => EventKind::GoAround,
      Task::Heading(x) => EventKind::Heading(x),
      Task::Ident => EventKind::Ident,
      Task::Land(x) => EventKind::Land(Intern::from(x)),
      Task::NamedFrequency(x) => EventKind::NamedFrequency(x),
      Task::ResumeOwnNavigation => EventKind::ResumeOwnNavigation,
      Task::Speed(x) => EventKind::Speed(x),
      Task::Takeoff(x) => EventKind::Takeoff(Intern::from(x)),
      Task::Taxi(x) => EventKind::Taxi(x),
      Task::TaxiContinue => EventKind::TaxiContinue,
      Task::TaxiHold => EventKind::TaxiHold,
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
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
      EventKind::Heading(heading) => {
        bundle.actions.push(Action::new(
          aircraft.id,
          ActionKind::TargetHeading(*heading),
        ));
      }
      EventKind::Altitude(altitude) => {
        bundle.actions.push(Action::new(
          aircraft.id,
          ActionKind::TargetAltitude(*altitude),
        ));
      }
      EventKind::Frequency(frequency) => {
        bundle
          .actions
          .push(Action::new(aircraft.id, ActionKind::Frequency(*frequency)));
      }
      EventKind::NamedFrequency(frq) => {
        if let Some(airspace) = aircraft.airspace.and_then(|airspace| {
          bundle.airspaces.iter().find(|a| a.id == airspace)
        }) {
          if let Some(frequency) = airspace.frequencies.try_from_string(frq) {
            bundle
              .actions
              .push(Action::new(aircraft.id, ActionKind::Frequency(frequency)));
          }
        }
      }
      EventKind::Ident => {
        todo!("TODO: Ident")
      }

      // Flying
      EventKind::DirectTo(waypoints) => {
        if let AircraftState::Flying { .. } = aircraft.state {
          handle_direct_to_event(aircraft, bundle, waypoints);
        }
      }
      EventKind::FollowArrival(waypoint) => {
        if let AircraftState::Flying { .. } = aircraft.state {
          handle_arrival_event(aircraft, bundle, *waypoint);
        }
      }
      EventKind::FollowApproach(waypoint) => {
        if let AircraftState::Flying { .. } = aircraft.state {
          handle_approach_event(aircraft, bundle, *waypoint);
        }
      }
      EventKind::FollowDeparture(waypoint) => {
        if let AircraftState::Flying { .. } = aircraft.state {
          handle_departure_event(aircraft, bundle, *waypoint);
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
            aircraft, bundle, *speed, *altitude, *departure,
          );
        }
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
    .push(Action::new(aircraft.id, ActionKind::Altitude(0.0)));
  bundle.actions.push(Action::new(
    aircraft.id,
    ActionKind::Heading(runway.heading),
  ));
  bundle
    .actions
    .push(Action::new(aircraft.id, ActionKind::SyncTargets));

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

pub fn handle_direct_to_event(
  aircraft: &Aircraft,
  bundle: &mut Bundle,
  waypoint_strings: &[Intern<String>],
) {
  if let Some(waypoints) = parse_waypoint_strings(bundle, waypoint_strings) {
    bundle.actions.push(Action {
      id: aircraft.id,
      kind: ActionKind::Flying(waypoints),
    });
  }
}

pub fn handle_arrival_event(
  aircraft: &Aircraft,
  bundle: &mut Bundle,
  arrival_string: Intern<String>,
) {
  if let Some(waypoint_strings) =
    bundle.waypoint_sets.arrival.get(&arrival_string)
  {
    if let Some(waypoints) = parse_waypoint_strings(bundle, waypoint_strings) {
      bundle.actions.push(Action {
        id: aircraft.id,
        kind: ActionKind::Flying(waypoints),
      });
    }
  }
}

pub fn handle_approach_event(
  aircraft: &Aircraft,
  bundle: &mut Bundle,
  approach_string: Intern<String>,
) {
  if let Some(waypoint_strings) =
    bundle.waypoint_sets.approach.get(&approach_string)
  {
    if let Some(waypoints) = parse_waypoint_strings(bundle, waypoint_strings) {
      bundle.actions.push(Action {
        id: aircraft.id,
        kind: ActionKind::Flying(waypoints),
      });
    }
  }
}

pub fn handle_departure_event(
  aircraft: &Aircraft,
  bundle: &mut Bundle,
  depart_string: Intern<String>,
) {
  if let Some(waypoint_strings) =
    bundle.waypoint_sets.departure.get(&depart_string)
  {
    if let Some(waypoints) = parse_waypoint_strings(bundle, waypoint_strings) {
      bundle.actions.push(Action {
        id: aircraft.id,
        kind: ActionKind::Flying(waypoints),
      });
    }
  }
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

        // TODO: Change this once we have clearances working again
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
  departure: Option<Intern<String>>,
) {
  let waypoints = departure
    .as_ref()
    .and_then(|d| bundle.waypoint_sets.departure.get(d))
    .map(|depart| {
      depart
        .iter()
        .map(|w| bundle.waypoints.iter().find(|n| &n.name == w).cloned())
        .rev()
        .try_fold(Vec::new(), |mut vec, item| {
          vec.push(item?);

          Some(vec)
        })
        .unwrap_or_default()
    })
    .unwrap_or_default();

  bundle.actions.push(Action {
    id: aircraft.id,
    kind: ActionKind::Clearance {
      speed,
      altitude,
      waypoints,
    },
  });
}
