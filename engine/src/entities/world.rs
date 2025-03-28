use glam::Vec2;
use serde::{Deserialize, Serialize};
use turborand::{rng::Rng, TurboRand};

use crate::pathfinder::Node;

use super::{aircraft::Aircraft, airport::Airport, airspace::Airspace};

pub fn find_random_airspace_with<'a>(
  airspaces: &'a [Airspace],
  auto: Option<bool>,
  require_airports: bool,
  rng: &mut Rng,
) -> Option<&'a Airspace> {
  let filtered_airspaces = airspaces.iter().filter(|a| {
    if let Some(auto) = auto {
      if auto != a.auto {
        return false;
      }
    }

    if require_airports && a.airports.is_empty() {
      return false;
    }

    true
  });

  rng.sample_iter(filtered_airspaces)
}

pub fn find_random_airspace<'a>(
  airspaces: &'a [Airspace],
  rng: &mut Rng,
) -> Option<&'a Airspace> {
  rng.sample(airspaces)
}

pub fn find_random_departure<'a>(
  airspaces: &'a [Airspace],
  rng: &mut Rng,
) -> Option<&'a Airspace> {
  find_random_airspace_with(airspaces, Some(true), false, rng)
}

pub fn find_random_arrival<'a>(
  airspaces: &'a [Airspace],
  rng: &mut Rng,
) -> Option<&'a Airspace> {
  find_random_airspace_with(airspaces, Some(false), true, rng)
}

pub fn closest_airport(
  airspaces: &[Airspace],
  point: Vec2,
) -> Option<&Airport> {
  let mut closest: Option<&Airport> = None;
  let mut distance = f32::MAX;
  for airspace in airspaces.iter().filter(|a| a.contains_point(point)) {
    for airport in airspace.airports.iter() {
      if airport.center.distance_squared(point) < distance {
        distance = airport.center.distance_squared(point);
        closest = Some(airport);
      }
    }
  }

  closest
}

pub fn closest_airspace(
  airspaces: &[Airspace],
  point: Vec2,
) -> Option<&Airspace> {
  let mut closest: Option<&Airspace> = None;
  let mut distance = f32::MAX;
  for airspace in airspaces.iter() {
    if airspace.pos.distance_squared(point) < distance {
      distance = airspace.pos.distance_squared(point);
      closest = Some(airspace);
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
pub struct World {
  pub airspaces: Vec<Airspace>,
  pub waypoints: Vec<Node<Vec2>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Game {
  pub aircraft: Vec<Aircraft>,
  pub paused: bool,
}
