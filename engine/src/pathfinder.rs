use glam::Vec2;
use petgraph::{
  algo::simple_paths,
  visit::{IntoNodeReferences, NodeRef},
  Graph, Undirected,
};

use crate::{
  angle_between_points, closest_point_on_line, delta_angle,
  find_line_intersection,
  structs::{Gate, Line, Runway, Taxiway, Terminal},
};

#[derive(Debug, Clone, PartialEq)]
pub enum WaypointNode {
  Taxiway { name: String, pos: Vec2 },
  Runway { name: String, pos: Vec2 },
  Gate { name: String, pos: Vec2 },
  Apron { name: String, pos: Vec2 },
}

impl WaypointNode {
  pub fn from_node(node: Node, pos: Vec2) -> Self {
    match node {
      Node::Taxiway { name, .. } => WaypointNode::Taxiway { name, pos },
      Node::Runway { name, .. } => WaypointNode::Runway { name, pos },
      Node::Gate { name, .. } => WaypointNode::Gate { name, pos },
      Node::Apron { name, .. } => WaypointNode::Apron { name, pos },
    }
  }

  pub fn name(&self) -> &String {
    match self {
      WaypointNode::Taxiway { name, .. } => name,
      WaypointNode::Runway { name, .. } => name,
      WaypointNode::Gate { name, .. } => name,
      WaypointNode::Apron { name, .. } => name,
    }
  }

  pub fn set_name(&mut self, name: String) {
    let n = match self {
      WaypointNode::Taxiway { name, .. } => name,
      WaypointNode::Runway { name, .. } => name,
      WaypointNode::Gate { name, .. } => name,
      WaypointNode::Apron { name, .. } => name,
    };

    *n = name;
  }

  pub fn pos(&self) -> &Vec2 {
    match self {
      WaypointNode::Taxiway { pos, .. } => pos,
      WaypointNode::Runway { pos, .. } => pos,
      WaypointNode::Gate { pos, .. } => pos,
      WaypointNode::Apron { pos, .. } => pos,
    }
  }

  pub fn set_pos(&mut self, pos: Vec2) {
    let p = match self {
      WaypointNode::Taxiway { pos, .. } => pos,
      WaypointNode::Runway { pos, .. } => pos,
      WaypointNode::Gate { pos, .. } => pos,
      WaypointNode::Apron { pos, .. } => pos,
    };

    *p = pos;
  }
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
  Apron { name: String, line: Line },
}

impl Node {
  fn into_waypoint(self, pos: Vec2) -> WaypointNode {
    WaypointNode::from_node(self, pos)
  }
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
      Node::Apron { name, .. } => name,
    }
  }

  pub fn line(&self) -> &Line {
    match self {
      Node::Taxiway { line, .. } => line,
      Node::Runway { line, .. } => line,
      Node::Gate { line, .. } => line,
      Node::Apron { line, .. } => line,
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Segment {
  Taxiway(Taxiway),
  Runway(Runway),
  Terminal(Terminal),
}

impl From<Taxiway> for Segment {
  fn from(value: Taxiway) -> Self {
    Segment::Taxiway(value)
  }
}

impl From<Runway> for Segment {
  fn from(value: Runway) -> Self {
    Segment::Runway(value)
  }
}

impl From<Terminal> for Segment {
  fn from(value: Terminal) -> Self {
    Segment::Terminal(value)
  }
}

impl From<&Segment> for Line {
  fn from(value: &Segment) -> Self {
    match value {
      Segment::Taxiway(value) => Line::new(value.a, value.b),
      Segment::Runway(value) => Line::new(value.start(), value.end()),
      Segment::Terminal(value) => value.apron,
    }
  }
}

impl From<Segment> for Line {
  fn from(value: Segment) -> Self {
    (&value).into()
  }
}

impl From<Segment> for Node {
  fn from(value: Segment) -> Self {
    match value {
      Segment::Taxiway(value) => Node::Taxiway {
        name: value.id.clone(),
        line: value.into(),
      },
      Segment::Runway(value) => Node::Runway {
        name: value.id.clone(),
        line: value.into(),
      },
      Segment::Terminal(value) => Node::Apron {
        name: value.id.to_string(),
        line: value.into(),
      },
    }
  }
}

pub fn total_distance(path: &[WaypointNode]) -> f32 {
  dbg!(path.len());
  let mut distance = 0.0;
  let mut first = path.first().unwrap();
  for next in path.iter().skip(1) {
    distance += first.pos().distance_squared(*next.pos());
    first = next;
  }

  distance
}

type WaypointGraph = Graph<Node, Vec2, Undirected>;

#[derive(Debug, Clone, Default)]
pub struct Pathfinder {
  pub graph: WaypointGraph,
}

impl Pathfinder {
  pub fn new() -> Self {
    Self {
      graph: WaypointGraph::new_undirected(),
    }
  }

  pub fn calculate(&mut self, mut segments: Vec<Segment>) {
    let mut graph = WaypointGraph::new_undirected();
    if segments.is_empty() || segments.len() < 2 {
      tracing::error!("No segments to calculate path for");
      return;
    }

    while let Some(current) = segments.pop() {
      let current_node = graph
        .node_references()
        .find(|(_, n)| **n == current.clone().into())
        .map(|(i, _)| i)
        .unwrap_or_else(|| graph.add_node(current.clone().into()));

      for segment in segments.iter() {
        let line: Line = segment.into();

        let intersection = find_line_intersection(line, current.clone().into());
        if let Some(intersection) = intersection {
          let segment_node = graph
            .node_references()
            .find(|(_, n)| **n == segment.clone().into())
            .map(|(i, _)| i)
            .unwrap_or_else(|| graph.add_node(segment.clone().into()));

          graph.add_edge(current_node, segment_node, intersection);
        }
      }

      if let Segment::Terminal(terminal) = current {
        for gate in terminal.gates.iter() {
          let gate_node = graph.add_node(gate.clone().into());
          let intersection =
            closest_point_on_line(gate.pos, terminal.apron.0, terminal.apron.1);

          graph.add_edge(current_node, gate_node, intersection);
        }
      }
    }

    self.graph = graph;
  }

  pub fn path_to(
    &self,
    from: &WaypointNode,
    to: &WaypointNode,
    pos: Vec2,
    heading: f32,
  ) -> Option<Vec<WaypointNode>> {
    let from_node = self.graph.node_references().find(|(_, n)| from.eq(*n));
    let to_node = self.graph.node_references().find(|(_, n)| to.eq(*n));

    if let Some((from_node, to_node)) = from_node.zip(to_node) {
      let mut paths = simple_paths::all_simple_paths::<Vec<_>, _>(
        &self.graph,
        from_node.0,
        to_node.0,
        0,
        None,
      );

      paths.next()?;

      let mut paths: Vec<Vec<WaypointNode>> = paths
        .map(|path| {
          path
            .into_iter()
            .map(|wp| (wp, self.graph.node_weight(wp).unwrap()))
            .collect::<Vec<_>>()
        })
        .map(|path| {
          let mut waypoints: Vec<WaypointNode> = Vec::new();

          let mut first = path.first().unwrap();
          for next in path.iter().skip(1) {
            let edge =
              self.graph.edges_connecting(first.0, next.0).next().unwrap();
            let wp = first.1.clone().into_waypoint(*edge.weight());

            waypoints.push(wp);

            first = next;
          }

          let wp = to_node.weight();
          let point = match wp {
            Node::Taxiway { line, .. } => line.midpoint(),
            Node::Runway { line, .. } => line.0,
            Node::Gate { line, .. } => line.0,
            Node::Apron { .. } => {
              unreachable!("Apron should not be a waypoint")
            }
          };
          let mut last_wp = to.clone();
          last_wp.set_pos(point);

          waypoints.push(last_wp);

          waypoints
        })
        .filter(|path| {
          let mut pos = pos;
          let mut heading = heading;

          let mut first = path.first().unwrap();
          for next in path.iter().skip(1) {
            let angle = angle_between_points(pos, *first.pos());
            if delta_angle(heading, angle).abs() >= 175.0 {
              return false;
            }

            pos = *first.pos();
            heading = angle;

            first = next;
          }

          true
        })
        .collect();

      // paths.sort_by(|a, b| {
      //   // TODO: unwrapping might cause errors with NaN's and Infinity's
      //   total_distance(a).partial_cmp(&total_distance(b)).unwrap()
      // });
      paths.sort_by_key(|p| p.len());

      let path = paths.first().unwrap();
      Some(path.clone())
    } else {
      None
    }
  }
}
