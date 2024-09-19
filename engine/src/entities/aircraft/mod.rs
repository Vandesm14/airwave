pub mod actions;
pub mod effects;
pub mod events;

use actions::Action;
use glam::Vec2;
use internment::Intern;
use serde::{Deserialize, Serialize};

use crate::{
  deserialize_vec2,
  pathfinder::{Node, WaypointNodeData},
  serialize_vec2,
};

use super::airport::Runway;

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

// TODO: use internment
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FlightPlan {
  pub departing: String,
  pub arriving: String,
  pub altitude: f32,
  pub speed: f32,
  pub waypoints: Vec<Node<WaypointNodeData>>,
}

impl FlightPlan {
  // TODO: use internment
  pub fn new(departing: String, arriving: String) -> Self {
    Self {
      departing,
      arriving,
      ..Default::default()
    }
  }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Aircraft {
  pub id: Intern<String>,

  #[serde(serialize_with = "serialize_vec2")]
  #[serde(deserialize_with = "deserialize_vec2")]
  pub pos: Vec2,
  pub speed: f32,
  pub heading: f32,
  pub altitude: f32,

  pub target: AircraftTargets,
  pub state: AircraftState,
  pub flight_plan: FlightPlan,

  pub airspace: Intern<String>,
}

// Helper methods
impl Aircraft {
  pub fn sync_targets_to_vals(&mut self) {
    self.target.heading = self.heading;
    self.target.speed = self.speed;
    self.target.altitude = self.altitude;
  }
}

// Performance stats
impl Aircraft {
  pub fn dt_climb_speed(&self, dt: f32) -> f32 {
    (2000.0_f32 / 60.0_f32).round() * dt
  }

  pub fn dt_turn_speed(&self, dt: f32) -> f32 {
    2.0 * dt
  }

  pub fn dt_speed_speed(&self, dt: f32) -> f32 {
    2.0 * dt
  }
}
