use std::ops::RangeInclusive;

use glam::Vec2;
use internment::Intern;
use serde::{Deserialize, Serialize};
use turborand::{rng::Rng, TurboRand};

use super::airport::Airport;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frequencies {
  pub clearance: f32,
  pub approach: f32,
  pub departure: f32,
  pub tower: f32,
  pub ground: f32,
  pub center: f32,
}

impl Default for Frequencies {
  fn default() -> Self {
    Self {
      clearance: 118.5,
      approach: 118.5,
      departure: 118.5,
      tower: 118.5,
      ground: 118.5,
      center: 118.5,
    }
  }
}

impl Frequencies {
  pub fn try_from_string(&self, s: &str) -> Option<f32> {
    match s {
      "clearance" => Some(self.clearance),
      "approach" => Some(self.approach),
      "departure" => Some(self.departure),
      "tower" => Some(self.tower),
      "ground" => Some(self.ground),
      "center" => Some(self.center),

      _ => None,
    }
  }
}

// TODO: Support non-circular (regional) airspaces
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Airspace {
  pub id: Intern<String>,
  pub pos: Vec2,
  pub size: f32,
  pub airports: Vec<Airport>,
  pub altitude: RangeInclusive<f32>,

  /// Determines whether the airspace is automatically controlled.
  pub auto: bool,
  pub frequencies: Frequencies,
}

impl Airspace {
  pub fn contains_point(&self, point: Vec2, altitude: Option<f32>) -> bool {
    let distance = point.distance_squared(self.pos);
    distance <= self.size.powf(2.0)
      && altitude
        .map(|altitude| self.altitude.contains(&altitude))
        .unwrap_or(true)
  }

  pub fn find_random_airport(&self, rng: &mut Rng) -> Option<&Airport> {
    rng.sample_iter(self.airports.iter())
  }
}
