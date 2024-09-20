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
  Ident,

  // Flying
  DirectTo(Vec<Intern<String>>),
  FollowApproach(Intern<String>),
  FollowArrival(Intern<String>),
  FollowDeparture(Intern<String>),

  // Transitions
  Land(Intern<String>),
  GoAround,
  Touchdown,

  // Taxiing
  Taxi(Vec<Node<()>>),
  TaxiContinue,
  TaxiHold,
}

impl From<Task> for EventKind {
  fn from(value: Task) -> Self {
    match value {
      Task::Altitude(x) => EventKind::Altitude(x),
      Task::Approach(x) => EventKind::FollowApproach(Intern::from(x)),
      Task::Arrival(x) => EventKind::FollowArrival(Intern::from(x)),
      Task::Clearance { .. } => todo!(),
      Task::Depart(x) => EventKind::FollowDeparture(Intern::from(x)),
      Task::Direct(x) => {
        EventKind::DirectTo(x.iter().cloned().map(Intern::from).collect())
      }
      Task::DirectionOfTravel => todo!(),
      Task::Frequency(_) => todo!(),
      Task::GoAround => EventKind::GoAround,
      Task::Heading(x) => EventKind::Heading(x),
      Task::Ident => EventKind::Ident,
      Task::Land(x) => EventKind::Land(Intern::from(x)),
      Task::NamedFrequency(_) => todo!(),
      Task::ResumeOwnNavigation => todo!(),
      Task::Speed(x) => EventKind::Speed(x),
      Task::Takeoff(_) => todo!(),
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

      // Transitions
      EventKind::Land(runway) => handle_land_event(aircraft, bundle, *runway),
      EventKind::Touchdown => {
        if let AircraftState::Landing(runway) = &aircraft.state {
          handle_touchdown_event(aircraft, bundle, runway);
        }
      }
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
            kind: ActionKind::Speed(20.0),
          });
        }
      }
      EventKind::TaxiHold => {
        if let AircraftState::Taxiing { .. } = aircraft.state {
          bundle.actions.push(Action {
            id: aircraft.id,
            kind: ActionKind::Speed(0.0),
          });
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
  if let AircraftState::Taxiing {
    waypoints: wps,
    current,
    ..
  } = &aircraft.state
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
      kind: ActionKind::TaxiWaypoints(all_waypoints),
    });

    tracing::info!(
      "Initiating taxi for {}: {:?}",
      aircraft.id,
      wps.iter().map(|w| w.name).collect::<Vec<_>>()
    );

    if wps.is_empty() {
      return;
    }
  }

  bundle.events.push(Event {
    id: aircraft.id,
    kind: EventKind::TaxiContinue,
  });
}
