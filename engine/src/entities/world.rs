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
#[serde(rename_all = "kebab-case")]
pub enum ArrivalStatus {
  #[default]
  Automated,
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
  Automated,
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

impl AirportStatus {
  pub fn all_auto() -> Self {
    Self {
      arrival: ArrivalStatus::Automated,
      departure: DepartureStatus::Automated,
    }
  }

  pub fn all_normal() -> Self {
    Self {
      arrival: ArrivalStatus::Normal,
      departure: DepartureStatus::Normal,
    }
  }

  pub fn auto_arrivals(&self) -> bool {
    matches!(self.arrival, ArrivalStatus::Automated)
  }

  pub fn auto_departures(&self) -> bool {
    matches!(self.departure, DepartureStatus::Automated)
  }

  pub fn normal_arrivals(&self) -> bool {
    matches!(self.arrival, ArrivalStatus::Normal)
  }

  pub fn normal_departures(&self) -> bool {
    matches!(self.departure, DepartureStatus::Normal)
  }

  pub fn divert_arrivals(&self) -> bool {
    matches!(self.arrival, ArrivalStatus::Divert)
  }

  pub fn delay_departures(&self) -> bool {
    matches!(self.departure, DepartureStatus::Delay)
  }

  pub fn nominal_arrivals(&self) -> bool {
    matches!(
      self.arrival,
      ArrivalStatus::Normal | ArrivalStatus::Automated
    )
  }

  pub fn nominal_departures(&self) -> bool {
    matches!(
      self.departure,
      DepartureStatus::Normal | DepartureStatus::Automated
    )
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

  pub fn detect_airspace(&self, point: Vec2) -> Option<&Airport> {
    self
      .closest_airport(point)
      .filter(|a| point.distance_squared(a.center) <= AIRSPACE_RADIUS.powf(2.0))
  }

  pub fn automated_arrivals(&self, airport_id: Intern<String>) -> bool {
    self
      .airport_statuses
      .get(&airport_id)
      .map(|s| s.auto_arrivals())
      .unwrap_or(true)
  }

  pub fn automated_departures(&self, airport_id: Intern<String>) -> bool {
    self
      .airport_statuses
      .get(&airport_id)
      .map(|s| s.auto_departures())
      .unwrap_or(true)
  }

  pub fn nominal_arrivals(&self, airport_id: Intern<String>) -> bool {
    self
      .airport_statuses
      .get(&airport_id)
      .map(|s| s.nominal_arrivals())
      .unwrap_or(false)
  }

  pub fn nominal_departures(&self, airport_id: Intern<String>) -> bool {
    self
      .airport_statuses
      .get(&airport_id)
      .map(|s| s.nominal_departures())
      .unwrap_or(false)
  }

  pub fn divert_arrivals(&self, airport_id: Intern<String>) -> bool {
    self
      .airport_statuses
      .get(&airport_id)
      .map(|s| s.divert_arrivals())
      .unwrap_or(false)
  }

  pub fn delay_departures(&self, airport_id: Intern<String>) -> bool {
    self
      .airport_statuses
      .get(&airport_id)
      .map(|s| s.delay_departures())
      .unwrap_or(false)
  }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Game {
  pub aircraft: Vec<Aircraft>,
  pub paused: bool,
}
