use glam::Vec2;
use internment::Intern;
use serde::{Deserialize, Serialize};
use turborand::{rng::Rng, TurboRand};

use crate::{deserialize_vec2, serialize_vec2};

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
  pub fn from_string(&self, s: &str) -> f32 {
    match s {
      "clearance" => self.clearance,
      "approach" => self.approach,
      "departure" => self.departure,
      "tower" => self.tower,
      "ground" => self.ground,
      "center" => self.center,
      _ => self.center,
    }
  }
}

// TODO: Support non-circular (regional) airspaces
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Airspace {
  pub id: Intern<String>,
  #[serde(serialize_with = "serialize_vec2")]
  #[serde(deserialize_with = "deserialize_vec2")]
  pub pos: Vec2,
  pub size: f32,
  pub airports: Vec<Airport>,

  /// Determines whether the airspace is automatically controlled.
  pub auto: bool,
  pub frequencies: Frequencies,
}

impl Airspace {
  pub fn contains_point(&self, point: Vec2) -> bool {
    let distance = point.distance_squared(self.pos);
    distance <= self.size.powf(2.0)
  }

  pub fn find_random_airport(&self, rng: &mut Rng) -> Option<&Airport> {
    rng.sample_iter(self.airports.iter())
  }
}
