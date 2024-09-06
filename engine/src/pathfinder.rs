use glam::Vec2;
use petgraph::{
  algo::simple_paths, graph::DiGraph, visit::IntoNodeReferences, Directed,
  Graph,
};

use crate::{
  angle_between_points, find_line_intersection, inverse_degrees,
  structs::{Gate, Line, Runway, Taxiway},
};

#[derive(Debug, Clone, PartialEq)]
pub enum WaypointNode {
  Taxiway { name: String, pos: Vec2 },
  Runway { name: String, pos: Vec2 },
  Gate { name: String, pos: Vec2 },
}

impl PartialEq<Node> for WaypointNode {
  fn eq(&self, other: &Node) -> bool {
    match (self, other) {
      (
        WaypointNode::Taxiway { name, .. },
        Node::Taxiway {
          name: other_name, ..
        },
      ) => name == other_name,
      (
        WaypointNode::Runway { name, .. },
        Node::Runway {
          name: other_name, ..
        },
      ) => name == other_name,
      (
        WaypointNode::Gate { name, .. },
        Node::Gate {
          name: other_name, ..
        },
      ) => name == other_name,
      _ => false,
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
  Taxiway { name: String, line: Line },
  Runway { name: String, line: Line },
  Gate { name: String, line: Line },
}

impl From<Taxiway> for Node {
  fn from(value: Taxiway) -> Self {
    Node::Taxiway {
      name: value.id,
      line: Line::new(value.a, value.b),
    }
  }
}

impl From<Runway> for Node {
  fn from(value: Runway) -> Self {
    Node::Runway {
      name: value.id.clone(),
      line: Line::new(value.start(), value.end()),
    }
  }
}

impl From<Gate> for Node {
  fn from(value: Gate) -> Self {
    Node::Gate {
      name: value.id.clone(),
      line: Line::new(value.pos, value.pos),
    }
  }
}

impl Node {
  pub fn name(&self) -> &String {
    match self {
      Node::Taxiway { name, .. } => name,
      Node::Runway { name, .. } => name,
      Node::Gate { name, .. } => name,
    }
  }

  pub fn line(&self) -> &Line {
    match self {
      Node::Taxiway { line, .. } => line,
      Node::Runway { line, .. } => line,
      Node::Gate { line, .. } => line,
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
  pub pos: Vec2,
  pub heading: f32,
}

type WaypointGraph = Graph<Node, Edge, Directed>;

#[derive(Debug, Clone, Default)]
pub struct Pathfinder {
  pub graph: WaypointGraph,
}

impl Pathfinder {
  pub fn new() -> Self {
    Self {
      graph: WaypointGraph::new(),
    }
  }

  pub fn calculate(&mut self, mut segments: Vec<Node>) {
    let mut graph = WaypointGraph::new();
    if segments.is_empty() || segments.len() < 2 {
      tracing::error!("No segments to calculate path for");
      return;
    }

    while !segments.is_empty() {
      let current = segments.pop();
      if let Some(current) = current {
        let current_node = graph
          .node_references()
          .find(|(_, n)| *n == &current)
          .map(|(i, _)| i)
          .unwrap_or_else(|| graph.add_node(current.clone()));
        for segment in segments.iter() {
          let line = segment.line();

          let intersection = find_line_intersection(*line, *current.line());
          if let Some(intersection) = intersection {
            let segment_node = graph
              .node_references()
              .find(|(_, n)| *n == segment)
              .map(|(i, _)| i)
              .unwrap_or_else(|| graph.add_node(segment.clone()));

            let heading = angle_between_points(current.line().0, intersection);
            let edge = Edge {
              pos: intersection,
              heading,
            };
            graph.add_edge(current_node, segment_node, edge);

            let reverse_edge = Edge {
              pos: intersection,
              heading: inverse_degrees(heading),
            };
            graph.add_edge(segment_node, current_node, reverse_edge);
          }
        }
      }
    }

    self.graph = graph;
  }

  pub fn path_to(
    &self,
    from: &WaypointNode,
    to: &WaypointNode,
  ) -> Option<Vec<WaypointNode>> {
    let from_node = self.graph.node_references().find(|(_, n)| from.eq(*n));
    let to_node = self.graph.node_references().find(|(_, n)| to.eq(*n));

    if let Some((from_node, to_node)) = from_node.zip(to_node) {
      let ways = simple_paths::all_simple_paths::<Vec<_>, _>(
        &self.graph,
        from_node.0,
        to_node.0,
        0,
        None,
      );
      let mut ways = ways.collect::<Vec<_>>();
      ways.sort_by_key(|a| a.len());

      let ways = ways
        .into_iter()
        .map(|way| {
          way
            .into_iter()
            .map(|w| self.graph.node_weight(w).map(|n| n.name()))
            .collect::<Vec<_>>()
        })
        .take(5)
        .collect::<Vec<_>>();

      dbg!(ways);
    }

    todo!()
  }
}
