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
pub enum Node<T> {
  Taxiway { name: String, value: T },
  Runway { name: String, value: T },
  Gate { name: String, value: T },
  Apron { name: String, value: T },
}

impl From<Taxiway> for Node<Line> {
  fn from(value: Taxiway) -> Self {
    Node::Taxiway {
      name: value.id,
      value: Line::new(value.a, value.b),
    }
  }
}

impl From<Runway> for Node<Line> {
  fn from(value: Runway) -> Self {
    Node::Runway {
      name: value.id.clone(),
      value: Line::new(value.start(), value.end()),
    }
  }
}

impl From<Gate> for Node<Line> {
  fn from(value: Gate) -> Self {
    Node::Gate {
      name: value.id.clone(),
      value: Line::new(value.pos, value.pos),
    }
  }
}

impl<T> Node<T> {
  pub fn name(&self) -> &String {
    match self {
      Node::Taxiway { name, .. } => name,
      Node::Runway { name, .. } => name,
      Node::Gate { name, .. } => name,
      Node::Apron { name, .. } => name,
    }
  }

  pub fn value(&self) -> &T {
    match self {
      Node::Taxiway { value, .. } => value,
      Node::Runway { value, .. } => value,
      Node::Gate { value, .. } => value,
      Node::Apron { value, .. } => value,
    }
  }
}

impl Node<Line> {
  pub fn line(&self) -> &Line {
    self.value()
  }
}

impl Node<Vec2> {
  pub fn pos(&self) -> &Vec2 {
    self.value()
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
  Taxiway(Taxiway),
  Runway(Runway),
  Terminal(Terminal),
}

impl From<Taxiway> for Object {
  fn from(value: Taxiway) -> Self {
    Object::Taxiway(value)
  }
}

impl From<Runway> for Object {
  fn from(value: Runway) -> Self {
    Object::Runway(value)
  }
}

impl From<Terminal> for Object {
  fn from(value: Terminal) -> Self {
    Object::Terminal(value)
  }
}

impl From<&Object> for Line {
  fn from(value: &Object) -> Self {
    match value {
      Object::Taxiway(value) => Line::new(value.a, value.b),
      Object::Runway(value) => Line::new(value.start(), value.end()),
      Object::Terminal(value) => value.apron,
    }
  }
}

impl From<Object> for Line {
  fn from(value: Object) -> Self {
    (&value).into()
  }
}

impl From<Object> for Node<Line> {
  fn from(value: Object) -> Self {
    match value {
      Object::Taxiway(value) => Node::Taxiway {
        name: value.id.clone(),
        value: value.into(),
      },
      Object::Runway(value) => Node::Runway {
        name: value.id.clone(),
        value: value.into(),
      },
      Object::Terminal(value) => Node::Apron {
        name: value.id.to_string(),
        value: value.into(),
      },
    }
  }
}

pub fn total_distance(path: &[Node<Vec2>]) -> f32 {
  let mut distance = 0.0;
  let mut first = path.first().unwrap();
  for next in path.iter().skip(1) {
    distance += first.pos().distance_squared(*next.pos());
    first = next;
  }

  distance
}

#[derive(Debug, Clone, PartialEq)]
pub enum WaypointString {
  Taxiway(String),
  Runway(String),
  Gate(String),
}

impl WaypointString {
  fn name(&self) -> &String {
    match self {
      WaypointString::Taxiway(name) => name,
      WaypointString::Runway(name) => name,
      WaypointString::Gate(name) => name,
    }
  }
}

impl<T> PartialEq<Node<T>> for WaypointString {
  fn eq(&self, other: &Node<T>) -> bool {
    match self {
      WaypointString::Taxiway(n) => match other {
        Node::Taxiway { name, .. } => n == name,
        _ => false,
      },
      WaypointString::Runway(n) => match other {
        Node::Runway { name, .. } => n == name,
        _ => false,
      },
      WaypointString::Gate(n) => match other {
        Node::Gate { name, .. } => n == name,
        _ => false,
      },
    }
  }
}

type WaypointGraph = Graph<Node<Line>, Vec2, Undirected>;

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

  pub fn calculate(&mut self, mut segments: Vec<Object>) {
    let mut graph = WaypointGraph::new_undirected();
    if segments.is_empty() || segments.len() < 2 {
      tracing::error!("No segments to calculate path for");
      return;
    }

    while let Some(current) = segments.pop() {
      let current_node = graph
        .node_references()
        .find(|(_, n)| **n == Node::from(current.clone()))
        .map(|(i, _)| i)
        .unwrap_or_else(|| graph.add_node(current.clone().into()));

      for segment in segments.iter() {
        let line: Line = segment.into();

        let intersection = find_line_intersection(line, current.clone().into());
        if let Some(intersection) = intersection {
          let segment_node = graph
            .node_references()
            .find(|(_, n)| **n == Node::from(segment.clone()))
            .map(|(i, _)| i)
            .unwrap_or_else(|| graph.add_node(segment.clone().into()));

          graph.add_edge(current_node, segment_node, intersection);
        }
      }

      if let Object::Terminal(terminal) = current {
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
    from: &WaypointString,
    to: &WaypointString,
    via: Vec<&WaypointString>,
    pos: Vec2,
    heading: f32,
  ) -> Option<Vec<Node<Vec2>>> {
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

      let mut paths: Vec<Vec<Node<Vec2>>> = paths
        .map(|path| {
          path
            .into_iter()
            .map(|wp| (wp, self.graph.node_weight(wp).unwrap()))
            .collect::<Vec<_>>()
        })
        .map(|path| {
          let mut waypoints: Vec<Node<Vec2>> = Vec::new();

          let mut first = path.first().unwrap();
          for next in path.iter().skip(1) {
            let edge = self
              .graph
              .edges_connecting(first.0, next.0)
              .next()
              .unwrap()
              .weight();

            waypoints.push(wp);

            first = next;
          }

          let wp = to_node.weight();
          let point = match wp {
            Node::Taxiway { value, .. } => value.midpoint(),
            Node::Runway { value, .. } => value.0,
            Node::Gate { value, .. } => value.0,
            Node::Apron { .. } => {
              unreachable!("Apron should not be a waypoint")
            }
          };

          waypoints.push(Waypoint::from_waypoint_string(to.clone(), point));

          waypoints
        })
        .filter(|path| {
          let mut via = via.iter().peekable();

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

            // if Some(first) == via.peek().copied().copied() {
            //   dbg!("YES");
            //   via.next();
            // }
            if let Some(v) = via.peek().copied().copied() {
              if v.clone().eq(first) {
                via.next();
              }
            }

            first = next;
          }

          if let Some(v) = via.peek().copied().copied() {
            if v.clone().eq(first) {
              via.next();
            }
          }

          dbg!(path);
          dbg!(&via);

          // If we didn't fulfill our via's
          if via.peek().is_some() {
            println!("doesn't fulfill vias");
            return false;
          }

          true
        })
        .collect();

      // paths.sort_by(|a, b| {
      //   // TODO: unwrapping might cause errors with NaN's and Infinity's
      //   total_distance(a).partial_cmp(&total_distance(b)).unwrap()
      // });
      paths.sort_by_key(|p| p.len());

      paths.first().cloned()
    } else {
      None
    }
  }
}
