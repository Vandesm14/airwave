use engine::{
  entities::{airport::Airport, world::WaypointSet},
  pathfinder::{Node, WaypointNodeData},
};

pub mod new_v_pattern;
pub mod parallel;

pub type AirportSetupFn = fn(
  airport: &mut Airport,
  waypoints: &mut Vec<Node<WaypointNodeData>>,
  waypoint_set: &mut WaypointSet,
);
