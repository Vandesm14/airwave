use glam::Vec2;
use rand::{seq::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};

use crate::{objects::aircraft::Aircraft, pathfinder::Node};

use super::{airport::Airport, airspace::Airspace};

pub fn find_random_airspace(
  airspaces: &[Airspace],
  auto: bool,
  with_airports: bool,
) -> Option<&Airspace> {
  let mut rng = thread_rng();
  let filtered_airspaces: Vec<&Airspace> = airspaces
    .iter()
    .filter(|a| {
      if auto != a.auto {
        return false;
      }
      if with_airports {
        return !a.airports.is_empty();
      }
      true
    })
    .collect();

  filtered_airspaces.choose(&mut rng).copied()
}

pub fn find_random_departure(airspaces: &[Airspace]) -> Option<&Airspace> {
  // TODO: We should probably do `true` for the second bool, which specifies
  // that a departure airspace needs an airport. This just saves us time
  // when testing and messing about with single airspaces instead of those
  // plus an airport.
  find_random_airspace(airspaces, true, false)
}

pub fn find_random_arrival(airspaces: &[Airspace]) -> Option<&Airspace> {
  find_random_airspace(airspaces, false, true)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct World {
  pub airspaces: Vec<Airspace>,
  pub aircraft: Vec<Aircraft>,
  pub waypoints: Vec<Node<Vec2>>,
}

impl World {
  pub fn closest_airport(&self, point: Vec2) -> Option<&Airport> {
    let mut closest: Option<&Airport> = None;
    let mut distance = f32::MAX;
    for airspace in self.airspaces.iter().filter(|a| a.contains_point(point)) {
      for airport in airspace.airports.iter() {
        if airport.center.distance_squared(point) < distance {
          distance = airport.center.distance_squared(point);
          closest = Some(airport);
        }
      }
    }

    closest
  }

  pub fn closest_airspace(&self, point: Vec2) -> Option<&Airspace> {
    let mut closest: Option<&Airspace> = None;
    let mut distance = f32::MAX;
    for airspace in self.airspaces.iter() {
      if airspace.pos.distance_squared(point) < distance {
        distance = airspace.pos.distance_squared(point);
        closest = Some(airspace);
      }
    }

    closest
  }
}
