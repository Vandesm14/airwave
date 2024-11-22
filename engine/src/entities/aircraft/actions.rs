use std::time::Duration;

use glam::Vec2;
use internment::Intern;

use crate::{
  entities::airport::Runway,
  pathfinder::{Node, NodeBehavior, NodeVORData},
};

use super::{Aircraft, AircraftState, LandingState, TaxiingState};

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

  Frequency(f32),

  // Substate
  PopWaypoint,
  TaxiWaypoints(Vec<Node<Vec2>>),
  TaxiCurrent(Node<Vec2>),
  TaxiLastAsGoto,
  EnRoute(bool),
  FlipFlightPlan,
  LandingState(LandingState),
  TaxiingState(TaxiingState),

  // State
  Landing(Runway),
  Taxi {
    current: Node<Vec2>,
    waypoints: Vec<Node<Vec2>>,
  },
  Flying(Vec<Node<NodeVORData>>),
  Parked {
    at: Node<Vec2>,
    ready_at: Duration,
  },
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

      ActionKind::Frequency(frequency) => aircraft.frequency = *frequency,

      // Substate
      ActionKind::PopWaypoint => {
        if let AircraftState::Flying { waypoints, .. } = &mut aircraft.state {
          waypoints.pop();
        } else if let AircraftState::Taxiing {
          current, waypoints, ..
        } = &mut aircraft.state
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
        } else if let AircraftState::Parked { at, .. } = &mut aircraft.state {
          aircraft.state = AircraftState::Taxiing {
            current: at.clone(),
            waypoints: w.clone(),
            state: TaxiingState::default(),
          };
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
      ActionKind::EnRoute(bool) => {
        if let AircraftState::Flying { enroute, .. } = &mut aircraft.state {
          *enroute = *bool;
        }
      }
      ActionKind::FlipFlightPlan => {
        aircraft.flip_flight_plan();
      }
      ActionKind::LandingState(s) => {
        if let AircraftState::Landing { state, .. } = &mut aircraft.state {
          *state = *s;
        }
      }
      ActionKind::TaxiingState(s) => {
        if let AircraftState::Taxiing { state, .. } = &mut aircraft.state {
          *state = *s;
        }
      }

      // State
      ActionKind::Landing(runway) => {
        aircraft.state = AircraftState::Landing {
          runway: runway.clone(),
          state: LandingState::default(),
        }
      }
      ActionKind::Flying(waypoints) => {
        aircraft.state = AircraftState::Flying {
          waypoints: waypoints.clone(),
          enroute: false,
        }
      }
      ActionKind::Taxi { current, waypoints } => {
        aircraft.state = AircraftState::Taxiing {
          current: current.clone(),
          waypoints: waypoints.clone(),
          state: TaxiingState::default(),
        }
      }
      ActionKind::Parked {
        at,
        ready_at: ready_in,
      } => {
        aircraft.state = AircraftState::Parked {
          at: at.clone(),
          ready_at: *ready_in,
        };
        aircraft.speed = 0.0;
        aircraft.target.speed = 0.0;
      }
    }
  }
}

// fn prune_waypoints(
//   aircraft: &Aircraft,
//   waypoints: &[Node<NodeVORData>],
// ) -> Vec<Node<NodeVORData>> {
//   if waypoints.len() < 2 {
//     return waypoints.to_vec();
//   }

//   let waypoints = waypoints.iter().rev().cloned().collect::<Vec<_>>();
//   let mut skip_amount = 0;
//   for (i, wp) in waypoints.windows(2).enumerate() {
//     let a = wp.first().unwrap();
//     let b = wp.last().unwrap();

//     let wp_distance = a.value.to.distance_squared(b.value.to);
//     let distance = aircraft.pos.distance_squared(b.value.to);

//     if distance < wp_distance {
//       println!("was: {:?}", a.name);
//       skip_amount = i + 1;
//     }
//   }

//   waypoints.iter().skip(skip_amount).rev().cloned().collect()
// }
