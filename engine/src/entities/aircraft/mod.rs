pub mod actions;
pub mod effects;
pub mod events;

use glam::Vec2;
use internment::Intern;
use serde::{Deserialize, Serialize};

use crate::pathfinder::{Node, WaypointNodeData};

use super::{airport::Runway, airspace::Airspace};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "type", content = "value")]
pub enum Event {
  TargetSpeed(f32),
  TargetHeading(f32),
  TargetAltitude(f32),

  Land(Intern<String>),
}

#[derive(Debug, Clone, PartialEq)]

pub enum Action {
  Pos(Vec2),

  Speed(f32),
  Heading(f32),
  Altitude(f32),

  TargetSpeed(f32),
  TargetHeading(f32),
  TargetAltitude(f32),

  Airspace(Intern<String>),

  // Substate
  Land(Runway),
}

#[derive(Debug, Default)]
pub struct Bundle<'a> {
  pub prev: Aircraft,

  pub events: Vec<Event>,
  pub actions: Vec<Action>,

  pub airspaces: &'a [Airspace],

  pub dt: f32,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AircraftTargets {
  pub heading: f32,
  pub speed: f32,
  pub altitude: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AircraftState {
  Flying {
    waypoints: Vec<Node<WaypointNodeData>>,
  },
  Landing(Runway),
  Taxiing {
    current: Node<Vec2>,
    waypoints: Vec<Node<Vec2>>,
  },
}

impl Default for AircraftState {
  fn default() -> Self {
    Self::Flying {
      waypoints: Vec::new(),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Aircraft {
  pub id: Intern<String>,

  pub pos: Vec2,
  pub speed: f32,
  pub heading: f32,
  pub altitude: f32,

  pub target: AircraftTargets,
  pub state: AircraftState,

  pub airspace: Intern<String>,
}

impl Aircraft {
  pub fn dt_climb_sp(&self, dt: f32) -> f32 {
    (2000.0_f32 / 60.0_f32).round() * dt
  }

  pub fn dt_turn_speed(&self, dt: f32) -> f32 {
    2.0 * dt
  }

  pub fn dt_speed_speed(&self, dt: f32) -> f32 {
    2.0 * dt
  }
}
