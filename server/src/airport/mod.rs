use engine::{
  entities::{airport::Airport, world::WaypointSet},
  pathfinder::{Node, NodeVORData},
};

pub mod new_v_pattern;
pub mod parallel;

pub type AirportSetupFn = fn(
  airport: &mut Airport,
  waypoints: &mut Vec<Node<NodeVORData>>,
  waypoint_set: &mut WaypointSet,
);
