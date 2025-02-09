use std::{
  collections::VecDeque,
  time::{Duration, SystemTime},
};

use glam::Vec2;
use internment::Intern;
use serde::{Deserialize, Serialize};

use super::{
  aircraft::Aircraft, airport::Airport, airspace::Airspace, flight::Flights,
};

pub fn closest_airport(airspace: &Airspace, point: Vec2) -> Option<&Airport> {
  let mut closest: Option<&Airport> = None;
  let mut distance = f32::MAX;
  for airport in airspace.airports.iter() {
    if airport.center.distance_squared(point) < distance {
      distance = airport.center.distance_squared(point);
      closest = Some(airport);
    }
  }

  closest
}

pub fn calculate_airport_waypoints(airspaces: &mut [Airspace]) {
  for airspace in airspaces.iter_mut() {
    for airport in airspace.airports.iter_mut() {
      airport.calculate_waypoints();
    }
  }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionState {
  #[default]
  Inactive,
  Active,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Connection {
  pub id: Intern<String>,
  pub state: ConnectionState,
  pub pos: Vec2,
  pub transition: Vec2,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct World {
  pub airspace: Airspace,
  pub connections: Vec<Connection>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Points {
  pub landings: usize,
  pub landing_rate: Marker,

  pub takeoffs: usize,
  pub takeoff_rate: Marker,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Game {
  pub aircraft: Vec<Aircraft>,
  pub funds: usize,
  pub flights: Flights,
  pub points: Points,
  pub paused: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Marker {
  window: Duration,
  marks: VecDeque<SystemTime>,
  rate: Duration,
}

impl Default for Marker {
  fn default() -> Self {
    Self::new(Duration::from_secs(10 * 60))
  }
}

impl Marker {
  pub fn new(window: Duration) -> Self {
    Self {
      window,
      marks: VecDeque::new(),
      rate: Duration::from_secs(0),
    }
  }

  pub fn mark(&mut self) {
    self.marks.push_back(SystemTime::now());
  }

  pub fn trim(&mut self) {
    while let Some(front) = self.marks.front() {
      if front.elapsed().unwrap() > self.window {
        self.marks.pop_front();
      } else {
        break;
      }
    }
  }

  pub fn calc_rate(&mut self) -> Duration {
    self.trim();

    let count = self.marks.len();
    if count > 0 {
      self.rate =
        Duration::from_secs_f32(self.window.as_secs_f32() / count as f32);
    }

    self.rate
  }

  pub fn count(&self) -> usize {
    self.marks.len()
  }
}
