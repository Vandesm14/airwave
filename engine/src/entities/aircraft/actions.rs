use glam::Vec2;
use internment::Intern;

use crate::{
  entities::airport::Runway,
  pathfinder::{Node, NodeBehavior, NodeVORData},
};

use super::{Aircraft, AircraftState};

#[derive(Debug, Clone, PartialEq)]

pub enum ActionKind {
  Pos(Vec2),

  Speed(f32),
  Heading(f32),
  Altitude(f32),

  TargetSpeed(f32),
  TargetHeading(f32),
  TargetAltitude(f32),
  SyncTargets,

  Clearance {
    speed: Option<f32>,
    altitude: Option<f32>,
    waypoints: Vec<Node<NodeVORData>>,
  },

  Frequency(f32),
  Created(u128),
  Airspace(Option<Intern<String>>),

  // Substate
  PopWaypoint,
  TaxiWaypoints(Vec<Node<Vec2>>),
  TaxiCurrent(Node<Vec2>),
  TaxiLastAsGoto,

  // State
  Landing(Runway),
  Taxi {
    current: Node<Vec2>,
    waypoints: Vec<Node<Vec2>>,
  },
  Flying(Vec<Node<NodeVORData>>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Action {
  pub id: Intern<String>,
  pub kind: ActionKind,
}

impl Action {
  pub fn new(id: Intern<String>, kind: ActionKind) -> Self {
    Self { id, kind }
  }
}

pub trait AircraftActionHandler {
  fn run(aircraft: &mut Aircraft, action: &ActionKind);
}

pub struct AircraftAllActionHandler;
impl AircraftActionHandler for AircraftAllActionHandler {
  fn run(aircraft: &mut Aircraft, action: &ActionKind) {
    match action {
      ActionKind::Pos(pos) => aircraft.pos = *pos,

      ActionKind::Speed(speed) => aircraft.speed = *speed,
      ActionKind::Heading(heading) => aircraft.heading = *heading,
      ActionKind::Altitude(altitude) => aircraft.altitude = *altitude,

      ActionKind::TargetSpeed(speed) => aircraft.target.speed = *speed,
      ActionKind::TargetHeading(heading) => aircraft.target.heading = *heading,
      ActionKind::TargetAltitude(altitude) => {
        aircraft.target.altitude = *altitude
      }
      ActionKind::SyncTargets => {
        aircraft.sync_targets_to_vals();
      }

      ActionKind::Clearance {
        speed,
        altitude,
        waypoints,
      } => {
        if let Some(speed) = speed {
          aircraft.flight_plan.speed = *speed;
        }
        if let Some(altitude) = altitude {
          aircraft.flight_plan.altitude = *altitude;
        }
        aircraft.flight_plan.waypoints = waypoints.clone();
      }

      ActionKind::Frequency(frequency) => aircraft.frequency = *frequency,
      ActionKind::Created(created) => aircraft.created = *created,
      ActionKind::Airspace(spur) => aircraft.airspace = *spur,

      ActionKind::PopWaypoint => {
        if let AircraftState::Flying { waypoints } = &mut aircraft.state {
          waypoints.pop();
        } else if let AircraftState::Taxiing { current, waypoints } =
          &mut aircraft.state
        {
          if let Some(last) = waypoints.pop() {
            aircraft.pos = last.value;
            *current = last;
          }
        }
      }
      ActionKind::TaxiWaypoints(w) => {
        if let AircraftState::Taxiing { waypoints, .. } = &mut aircraft.state {
          *waypoints = w.clone();
        }
      }
      ActionKind::TaxiCurrent(w) => {
        if let AircraftState::Taxiing { current, .. } = &mut aircraft.state {
          *current = w.clone();
        }
      }
      ActionKind::TaxiLastAsGoto => {
        if let AircraftState::Taxiing { waypoints, .. } = &mut aircraft.state {
          if let Some(last) = waypoints.last_mut() {
            last.behavior = NodeBehavior::GoTo;
          }
        }
      }

      ActionKind::Landing(runway) => {
        aircraft.state = AircraftState::Landing(runway.clone())
      }
      ActionKind::Flying(waypoints) => {
        aircraft.state = AircraftState::Flying {
          waypoints: waypoints.clone(),
        }
      }
      ActionKind::Taxi { current, waypoints } => {
        aircraft.state = AircraftState::Taxiing {
          current: current.clone(),
          waypoints: waypoints.clone(),
        }
      }
    }
  }
}
