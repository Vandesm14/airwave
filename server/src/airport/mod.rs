use engine::{
  entities::{airport::Airport, world::WaypointSet},
  pathfinder::{Node, WaypointNodeData},
};

pub mod cross_roads;
pub mod new_v_pattern;
pub mod parallel;
pub mod v_pattern;

pub type AirportSetupFn = fn(
  airport: &mut Airport,
  waypoints: &mut Vec<Node<WaypointNodeData>>,
  waypoint_set: &mut WaypointSet,
);
