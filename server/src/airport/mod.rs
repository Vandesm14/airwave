use engine::{
  objects::{airport::Airport, world::WaypointSet},
  pathfinder::Node,
};
use glam::Vec2;

pub mod cross_roads;
pub mod new_v_pattern;
pub mod parallel;
pub mod v_pattern;

pub type AirportSetupFn = fn(
  airport: &mut Airport,
  waypoints: &mut Vec<Node<Vec2>>,
  waypoint_set: &mut WaypointSet,
);
