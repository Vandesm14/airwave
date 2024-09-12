use glam::Vec2;
use petgraph::{
  algo::simple_paths,
  visit::{IntoNodeReferences, NodeRef},
  Graph, Undirected,
};
use serde::{Deserialize, Serialize};

use crate::{
  angle_between_points, closest_point_on_line, delta_angle,
  find_line_intersection,
  objects::airport::{Gate, Runway, Taxiway, Terminal},
  Line,
};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeKind {
  Taxiway,
  Runway,
  Gate,
  Apron,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeBehavior {
  GoTo,
  HoldShort,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node<T> {
  pub name: String,
  pub kind: NodeKind,
  pub behavior: NodeBehavior,

  #[serde(default)]
  pub value: T,
}

impl<T> Node<T> {
  pub fn new(
    name: String,
    kind: NodeKind,
    behavior: NodeBehavior,
    value: T,
  ) -> Self {
    Self {
      name,
      kind,
      behavior,
      value,
    }
  }
}

impl<T> Node<T> {
  pub fn name_and_kind_eq<U>(&self, other: &Node<U>) -> bool {
    self.name == other.name && self.kind == other.kind
  }
}

impl From<Gate> for Node<Vec2> {
  fn from(value: Gate) -> Self {
    Self {
      name: value.id,
      kind: NodeKind::Gate,
      behavior: NodeBehavior::GoTo,
      value: value.pos,
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
        behavior: NodeBehavior::GoTo,
        value: value.into(),
      },
      Object::Runway(value) => Node {
        name: value.id.clone(),
        kind: NodeKind::Runway,
        behavior: NodeBehavior::HoldShort,
        value: value.into(),
      },
      Object::Terminal(value) => Node {
        name: value.id.to_string(),
        kind: NodeKind::Apron,
        behavior: NodeBehavior::GoTo,
        value: value.into(),
      },
    }
  }
}

pub fn total_distance_squared(path: &[Node<Vec2>], current_pos: Vec2) -> f32 {
  let mut distance = 0.0;
  let mut first = current_pos;
  for next in path.iter() {
    distance += first.distance_squared(next.value);
    first = next.value;
  }

  distance
}

pub fn display_vec_node_vec2(path: &[Node<Vec2>]) -> String {
  path
    .iter()
    .enumerate()
    .fold(String::new(), |mut acc, (i, n)| {
      if i > 0 {
        acc.push_str(", ");
      }

      acc.push_str(&format!("{:?}: {}", n.kind, n.name));

      acc
    })
}

type WaypointGraph = Graph<Node<Line>, Vec2, Undirected>;
type WaypointString = Node<()>;

#[derive(Debug, Clone, Default)]
pub struct PathfinderPath {
  pub path: Vec<Node<Vec2>>,
  pub final_heading: f32,
  pub final_pos: Vec2,
}

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
          let gate_node = graph.add_node(Node {
            name: gate.id.clone(),
            kind: NodeKind::Gate,
            behavior: NodeBehavior::GoTo,
            value: Line::new(gate.pos, gate.pos),
          });
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
    from: WaypointString,
    to: WaypointString,
    pos: Vec2,
    heading: f32,
  ) -> Option<PathfinderPath> {
    let from_node = self
      .graph
      .node_references()
      .find(|(_, n)| from.name_and_kind_eq(*n));
    let to_node = self
      .graph
      .node_references()
      .find(|(_, n)| to.name_and_kind_eq(*n));

    if let Some((from_node, to_node)) = from_node.zip(to_node) {
      let paths = simple_paths::all_simple_paths::<Vec<_>, _>(
        &self.graph,
        from_node.0,
        to_node.0,
        0,
        None,
      );

      let mut paths: Vec<PathfinderPath> = paths
        .map(|path| {
          path
            .into_iter()
            .map(|wp| (wp, self.graph.node_weight(wp).unwrap()))
            .collect::<Vec<_>>()
        })
        // Generate a list of waypoints for each path
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
              next.1.name.clone(),
              next.1.kind,
              next.1.behavior,
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
            waypoints.push(Node::new(
              to.name.clone(),
              to.kind,
              to.behavior,
              value.0,
            ));
          }

          waypoints
        })
        // Turn the Vec<Node<Vec2>> paths into PathfinderPaths
        .map(|path| {
          let mut pos = pos;
          let mut heading = heading;

          let mut first = &Node {
            name: from.name.clone(),
            kind: from.kind,
            behavior: NodeBehavior::GoTo,
            value: pos,
          };
          for wp in path.iter() {
            let angle = angle_between_points(pos, wp.value);

            pos = first.value;
            heading = angle;
            first = wp;
          }

          PathfinderPath {
            path,
            final_heading: heading,
            final_pos: pos,
          }
        })
        // Filter out paths that don't fulfill our requirements
        .filter(|path| {
          let mut pos = pos;
          let mut heading = heading;

          let mut first = &Node {
            name: from.name.clone(),
            kind: from.kind,
            behavior: NodeBehavior::GoTo,
            value: pos,
          };
          for wp in path.path.iter() {
            let angle = angle_between_points(pos, wp.value);
            // If our waypoint is not a gate and we are not heading towards it,
            // don't use this path.
            //
            // Inverse: If this is a gate, ignore the heading check.
            if first.kind != NodeKind::Gate
              && delta_angle(heading, angle).abs() >= 175.0
            {
              return false;
            }

            // If the waypoint is a runway and we haven't instructed to go to
            // it, don't use this path.
            if wp.kind == NodeKind::Runway && !to.name_and_kind_eq(wp) {
              return false;
            }

            pos = first.value;
            heading = angle;

            first = wp;
          }

          true
        })
        .collect();

      // TODO: The distance function is broken for some reason so we won't
      // sort by it for now until its fixed.
      //
      // paths.sort_by(|a, b| {
      //   total_distance(a, pos)
      //     .partial_cmp(&total_distance(b, pos))
      //     .unwrap()
      // });
      paths.sort_by_key(|p| p.path.len());

      for path in paths.iter() {
        println!(
          "path: {:?} ({} ft)",
          path
            .path
            .iter()
            .map(|n| n.name.clone())
            .collect::<Vec<_>>()
            .join(", "),
          total_distance_squared(&path.path, pos).sqrt()
        );
      }

      let first_path = paths.first();
      if let Some(first_path) = first_path {
        println!("chosen path: {:?}", display_vec_node_vec2(&first_path.path));
      }

      first_path.cloned()
    } else {
      None
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn total_distance_two_points() {
    let a = Vec2::new(0.0, 0.0);
    let b = Vec2::new(1.0, 1.0);

    assert_eq!(
      total_distance_squared(
        &[Node {
          name: "B".into(),
          kind: NodeKind::Apron,
          behavior: NodeBehavior::GoTo,
          value: b
        }],
        a
      ),
      a.distance_squared(b)
    );
  }

  #[test]
  fn total_distance_multiple_points() {
    let a = Vec2::new(0.0, 0.0);
    let b = Vec2::new(1.0, 0.0);
    let c = Vec2::new(1.0, 1.0);
    let d = Vec2::new(0.0, 1.0);

    let ab = a.distance_squared(b);
    let bc = b.distance_squared(c);
    let cd = c.distance_squared(d);

    let distance = ab + bc + cd;

    assert_eq!(
      total_distance_squared(
        &[
          Node {
            name: "B".into(),
            kind: NodeKind::Apron,
            behavior: NodeBehavior::GoTo,
            value: b
          },
          Node {
            name: "C".into(),
            kind: NodeKind::Apron,
            behavior: NodeBehavior::GoTo,
            value: c
          },
          Node {
            name: "D".into(),
            kind: NodeKind::Apron,
            behavior: NodeBehavior::GoTo,
            value: d
          }
        ],
        a
      ),
      distance
    );
  }
}
