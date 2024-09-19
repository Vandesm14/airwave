pub mod actions;
pub mod effects;
pub mod events;

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::KNOT_TO_FEET_PER_SECOND;

use super::airspace::Airspace;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "type", content = "value")]
pub enum Event {
  TargetSpeed(f32),
  TargetHeading(f32),
  TargetAltitude(f32),

  Land(String),
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

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Aircraft {
  pub pos: Vec2,
  pub speed: f32,
  pub heading: f32,
  pub altitude: f32,

  pub target: AircraftTargets,
}

// Consts
impl Aircraft {
  pub fn speed_in_feet(&self) -> f32 {
    self.speed * KNOT_TO_FEET_PER_SECOND
  }

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
