use glam::Vec2;
use internment::Intern;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use turborand::{TurboRand, rng::Rng};

use super::airport::Airport;

// TODO: Support non-circular (regional) airspaces
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Airspace {
  #[ts(as = "String")]
  pub id: Intern<String>,
  #[ts(as = "(f32, f32)")]
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
