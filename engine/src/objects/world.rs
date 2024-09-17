use std::collections::HashMap;

use glam::Vec2;
use rand::{rngs::StdRng, seq::SliceRandom};
use serde::{Deserialize, Serialize};

use crate::pathfinder::{Node, WaypointNodeData};

use super::{aircraft::Aircraft, airport::Airport, airspace::Airspace};

pub fn find_random_airspace_with<'a>(
  airspaces: &'a [Airspace],
  auto: bool,
  require_airports: bool,
  rng: &mut StdRng,
) -> Option<&'a Airspace> {
  let filtered_airspaces: Vec<&Airspace> = airspaces
    .iter()
    .filter(|a| {
      if auto != a.auto {
        return false;
      }
      if require_airports {
        return !a.airports.is_empty();
      }
      true
    })
    .collect();

  filtered_airspaces.choose(rng).copied()
}

pub fn find_random_airspace<'a>(
  airspaces: &'a [Airspace],
  rng: &mut StdRng,
) -> Option<&'a Airspace> {
  airspaces.choose(rng)
}

pub fn find_random_departure<'a>(
  airspaces: &'a [Airspace],
  rng: &mut StdRng,
) -> Option<&'a Airspace> {
  // TODO: We should probably do `true` for the second bool, which specifies
  // that a departure airspace needs an airport. This just saves us time
  // when testing and messing about with single airspaces instead of those
  // plus an airport.
  find_random_airspace_with(airspaces, true, false, rng)
}

pub fn find_random_arrival<'a>(
  airspaces: &'a [Airspace],
  rng: &mut StdRng,
) -> Option<&'a Airspace> {
  find_random_airspace_with(airspaces, false, true, rng)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WaypointSet {
  pub approach: HashMap<String, Vec<String>>,
  pub departure: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct World {
  pub airspaces: Vec<Airspace>,
  pub aircraft: Vec<Aircraft>,
  pub waypoints: Vec<Node<WaypointNodeData>>,
  pub waypoint_sets: WaypointSet,
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

  pub fn calculate_airport_waypoints(&mut self) {
    for airspace in self.airspaces.iter_mut() {
      for airport in airspace.airports.iter_mut() {
        airport.calculate_waypoints();
      }
    }
  }
}
