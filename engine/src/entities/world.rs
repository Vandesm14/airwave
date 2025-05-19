use std::collections::HashMap;

use glam::Vec2;
use internment::Intern;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::pathfinder::Node;

use super::{aircraft::Aircraft, airport::Airport};

pub fn calculate_airport_waypoints(airports: &mut [Airport]) {
  for airport in airports.iter_mut() {
    airport.calculate_waypoints();
  }
}

#[derive(
  Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize, TS,
)]
#[ts(export)]
#[serde(rename_all = "kebab-case")]
pub enum ArrivalStatus {
  #[default]
  Normal,
  Divert,
}

#[derive(
  Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize, TS,
)]
#[ts(export)]
#[serde(rename_all = "kebab-case")]
pub enum DepartureStatus {
  #[default]
  Normal,
  Delay,
}

#[derive(
  Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize, TS,
)]
#[ts(export)]
pub struct AirportStatus {
  pub arrival: ArrivalStatus,
  pub departure: DepartureStatus,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct World {
  pub airports: Vec<Airport>,
  #[ts(as = "Vec<Node<(f32, f32)>>")]
  pub waypoints: Vec<Node<Vec2>>,
  #[ts(as = "HashMap<String, AirportStatus>")]
  pub airport_statuses: HashMap<Intern<String>, AirportStatus>,
}

impl World {
  pub fn reset_statuses(&mut self) {
    self.airport_statuses.clear();
    for airport in self.airports.iter() {
      self
        .airport_statuses
        .insert(airport.id, AirportStatus::default());
    }
  }

  pub fn closest_airport(&self, point: Vec2) -> Option<&Airport> {
    let mut closest: Option<&Airport> = None;
    let mut distance = f32::MAX;
    for airport in self.airports.iter().filter(|a| a.contains_point(point)) {
      if airport.center.distance_squared(point) < distance {
        distance = airport.center.distance_squared(point);
        closest = Some(airport);
      }
    }

    closest
  }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Game {
  pub aircraft: Vec<Aircraft>,
  pub paused: bool,
}
