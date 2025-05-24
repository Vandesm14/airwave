use std::collections::HashMap;

use glam::Vec2;
use internment::Intern;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{AIRSPACE_RADIUS, pathfinder::Node};

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
pub struct AirportStatus {
  pub divert_arrivals: bool,
  pub delay_departures: bool,
  pub automate_air: bool,
  pub automate_ground: bool,
}

impl AirportStatus {
  pub fn all_auto() -> Self {
    Self {
      divert_arrivals: false,
      delay_departures: false,
      automate_air: true,
      automate_ground: true,
    }
  }

  pub fn all_normal() -> Self {
    Self {
      divert_arrivals: false,
      delay_departures: false,
      automate_air: false,
      automate_ground: false,
    }
  }

  pub fn all_diverted() -> Self {
    Self {
      divert_arrivals: true,
      delay_departures: true,
      automate_air: false,
      automate_ground: false,
    }
  }
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

  pub fn detect_airspace(&self, point: Vec2) -> Option<&Airport> {
    self
      .closest_airport(point)
      .filter(|a| point.distance_squared(a.center) <= AIRSPACE_RADIUS.powf(2.0))
  }

  pub fn airport_status(&self, airport_id: Intern<String>) -> AirportStatus {
    self
      .airport_statuses
      .get(&airport_id)
      .copied()
      .unwrap_or_default()
  }

  pub fn airport(&self, airport_id: Intern<String>) -> Option<&Airport> {
    self.airports.iter().find(|a| a.id == airport_id)
  }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Game {
  pub aircraft: Vec<Aircraft>,
  pub paused: bool,
}
