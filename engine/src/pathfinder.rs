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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeKind {
  Taxiway,
  Runway,
  Gate,
  Apron,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node<T> {
  pub name: String,
  pub kind: NodeKind,
  pub value: T,
}

impl<T> Node<T> {
  pub fn new(name: String, kind: NodeKind, value: T) -> Self {
    Self { name, kind, value }
  }
}

impl<T> Node<T> {
  pub fn name_and_kind_eq<U>(&self, other: &Node<U>) -> bool {
    self.name == other.name && self.kind == other.kind
  }
}

impl From<Taxiway> for Node<Line> {
  fn from(value: Taxiway) -> Self {
    Node {
      name: value.id,
      kind: NodeKind::Taxiway,
      value: Line::new(value.a, value.b),
    }
  }
}

impl From<Runway> for Node<Line> {
  fn from(value: Runway) -> Self {
    Node {
      name: value.id.clone(),
      kind: NodeKind::Runway,
      value: Line::new(value.start(), value.end()),
    }
  }
}

impl From<Gate> for Node<Line> {
  fn from(value: Gate) -> Self {
    Node {
      name: value.id,
      kind: NodeKind::Gate,
      value: Line::new(value.pos, value.pos),
    }
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
      Object::Taxiway(value) => Node {
        name: value.id.clone(),
        kind: NodeKind::Taxiway,
        value: value.into(),
      },
      Object::Runway(value) => Node {
        name: value.id.clone(),
        kind: NodeKind::Runway,
        value: value.into(),
      },
      Object::Terminal(value) => Node {
        name: value.id.to_string(),
        kind: NodeKind::Apron,
        value: value.into(),
      },
    }
  }
}

pub fn total_distance(path: &[Node<Vec2>]) -> f32 {
  let mut distance = 0.0;
  let mut first = path.first().unwrap();
  for next in path.iter().skip(1) {
    distance += first.value.distance_squared(next.value);
    first = next;
  }

  distance
}

// impl<T> PartialEq<Node<T>> for WaypointString {
//   fn eq(&self, other: &Node<T>) -> bool {
//     match self {
//       WaypointString::Taxiway(n) => match other {
//         Node::Taxiway { name, .. } => n == name,
//         _ => false,
//       },
//       WaypointString::Runway(n) => match other {
//         Node::Runway { name, .. } => n == name,
//         _ => false,
//       },
//       WaypointString::Gate(n) => match other {
//         Node::Gate { name, .. } => n == name,
//         _ => false,
//       },
//     }
//   }
// }

type WaypointGraph = Graph<Node<Line>, Vec2, Undirected>;
type WaypointString = Node<()>;

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
    let from_node = self
      .graph
      .node_references()
      .find(|(_, n)| from.name_and_kind_eq(*n));
    let to_node = self
      .graph
      .node_references()
      .find(|(_, n)| to.name_and_kind_eq(*n));

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

            waypoints.push(Node::new(
              first.1.name.clone(),
              first.1.kind,
              *edge,
            ));

            first = next;
          }

          let wp = to_node.weight();

          // If our destination is a gate, set our destination to that gate
          // (otherwise it will be the enterance on the apron but not the gate)
          if let Node {
            kind: NodeKind::Gate,
            value,
            ..
          } = wp
          {
            waypoints.push(Node::new(to.name.clone(), to.kind, value.0));
          }

          waypoints
        })
        .filter(|path| {
          let mut via = via.iter().peekable();

          let mut pos = pos;
          let mut heading = heading;

          let mut first = path.first().unwrap();
          for next in path.iter().skip(1) {
            let angle = angle_between_points(pos, first.value);
            if first.kind != NodeKind::Gate
              && delta_angle(heading, angle).abs() >= 175.0
            {
              return false;
            }

            pos = first.value;
            heading = angle;

            // if Some(first) == via.peek().copied().copied() {
            //   dbg!("YES");
            //   via.next();
            // }
            if let Some(v) = via.peek().copied().copied() {
              if v.name_and_kind_eq(first) {
                via.next();
              }
            }

            first = next;
          }

          if let Some(v) = via.peek().copied().copied() {
            if v.name_and_kind_eq(first) {
              via.next();
            }
          }

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
