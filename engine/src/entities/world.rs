use glam::Vec2;
use internment::Intern;
use serde::{Deserialize, Serialize};

use super::{airport::Airport, airspace::Airspace};

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
