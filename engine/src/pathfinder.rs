use core::fmt;

use glam::Vec2;
use petgraph::{Graph, Undirected};

type WaypointGraph = Graph<Vec2, u16, Undirected>;

pub trait Segment {
  fn name(&self) -> String;
  fn lines(&self) -> Vec<Vec2>;
}

#[derive(Debug, Clone, Default)]
pub struct Pathfinder<T>
where
  T: Segment + fmt::Debug + Clone,
{
  pub names: Vec<String>,
  pub graph: WaypointGraph,
  pub segments: Vec<T>,
}

impl<T> Pathfinder<T>
where
  T: Segment + fmt::Debug + Clone,
{
  pub fn new() -> Self {
    Self {
      names: Vec::new(),
      segments: Vec::new(),
      graph: WaypointGraph::new_undirected(),
    }
  }

  pub fn add_segment(&mut self, segment: T) {}
}
