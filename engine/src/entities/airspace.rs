use glam::Vec2;
use internment::Intern;
use serde::{Deserialize, Serialize};
use turborand::{rng::Rng, TurboRand};

use super::airport::Airport;

// TODO: Support non-circular (regional) airspaces
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Airspace {
  pub id: Intern<String>,
  pub pos: Vec2,
  pub radius: f32,
  pub airports: Vec<Airport>,

  pub auto: bool,
}

impl Airspace {
  pub fn contains_point(&self, point: Vec2) -> bool {
    let distance = point.distance_squared(self.pos);
    distance <= self.radius.powf(2.0)
  }

  pub fn find_random_airport(&self, rng: &mut Rng) -> Option<&Airport> {
    rng.sample_iter(self.airports.iter())
  }
}
