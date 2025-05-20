use std::time::Instant;

use glam::Vec2;
use internment::Intern;
use petgraph::{
  Graph, Undirected, algo::simple_paths, visit::IntoNodeReferences,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
  entities::airport::{Gate, Runway, Taxiway, Terminal},
  geometry::{
    angle_between_points, closest_point_on_line, delta_angle,
    find_line_intersection,
  },
  line::Line,
};

#[derive(
  Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize, TS,
)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum NodeKind {
  #[default]
  Taxiway,
  Runway,
  Gate,
  Apron,

  VOR,
}

#[derive(
  Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize, TS,
)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum NodeBehavior {
  #[default]
  GoTo,
  Park,
  HoldShort,

  // Runway specific
  Takeoff,
  LineUp,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Node<T> {
  #[ts(as = "String")]
  pub name: Intern<String>,
  pub kind: NodeKind,
  pub behavior: NodeBehavior,

  #[serde(default)]
  pub data: T,
}

impl<T> Node<T> {
  pub fn new(
    name: Intern<String>,
    kind: NodeKind,
    behavior: NodeBehavior,
    value: T,
  ) -> Self {
    Self {
      name,
      kind,
      behavior,
      data: value,
    }
  }

  pub fn build(data: T) -> Self {
    Self {
      name: Intern::from_ref(""),
      kind: NodeKind::default(),
      behavior: NodeBehavior::default(),
      data,
    }
  }

  pub fn with_name(mut self, name: Intern<String>) -> Self {
    self.name = name;
    self
  }

  pub fn with_kind(mut self, kind: NodeKind) -> Self {
    self.kind = kind;
    self
  }

  pub fn with_behavior(mut self, behavior: NodeBehavior) -> Self {
    self.behavior = behavior;
    self
  }

  pub fn with_data(mut self, data: T) -> Self {
    self.data = data;
    self
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
      behavior: NodeBehavior::Park,
      data: value.pos,
    }
  }
}

impl From<Gate> for Node<Line> {
  fn from(value: Gate) -> Self {
    Self {
      name: value.id,
      kind: NodeKind::Gate,
      behavior: NodeBehavior::Park,
      data: Line::new(value.pos, value.pos),
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
      Object::Runway(value) => Line::new(value.start, value.end()),
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
        name: value.id,
        kind: NodeKind::Taxiway,
        behavior: NodeBehavior::GoTo,
        data: value.into(),
      },
      Object::Runway(value) => Node {
        name: value.id,
        kind: NodeKind::Runway,
        behavior: NodeBehavior::GoTo,
        data: value.into(),
      },
      Object::Terminal(value) => Node {
        name: value.id,
        kind: NodeKind::Apron,
        behavior: NodeBehavior::GoTo,
        data: value.into(),
      },
    }
  }
}

pub fn total_distance_squared(path: &[Node<Vec2>], current_pos: Vec2) -> f32 {
  let mut distance = 0.0;
  let mut first = current_pos;
  for next in path.iter() {
    distance += first.distance_squared(next.data);
    first = next.data;
  }

  distance
}

pub fn display_node_vec2<T>(n: &Node<T>) -> String {
  let exclamation = if n.behavior == NodeBehavior::Park
    || n.behavior == NodeBehavior::HoldShort
  {
    "!"
  } else {
    ""
  };
  format!("{:?}: {}{}", n.kind, exclamation, n.name)
}

pub fn display_vec_node_vec2(path: &[Node<Vec2>]) -> String {
  path
    .iter()
    .enumerate()
    .fold(String::new(), |mut acc, (i, n)| {
      if i > 0 {
        acc.push_str(", ");
      }

      acc.push_str(&display_node_vec2(n));

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
      // This limits the number of intermediate nodes to greatly reduce
      // enumeration. It's technically a "magic number" because we still want
      // the pathfinder to try its best to find a path, but we don't want it to
      // take forever to do so.
      //
      // Setting this to 8 reduced the enumeration from 600k paths to 420.
      let max_intermediates_magic_number = 8;
      let paths =
        simple_paths::all_simple_paths::<Vec<_>, _, std::hash::RandomState>(
          &self.graph,
          from_node.0,
          to_node.0,
          0,
          Some(max_intermediates_magic_number),
        );

      let mut count = 0;

      let main_start = Instant::now();
      let mut paths: Vec<PathfinderPath> = paths
        .map(|path| {
          path
            .into_iter()
            .map(|wp| (wp, self.graph.node_weight(wp).unwrap()))
            .collect::<Vec<_>>()
        })
        // Generate a list of waypoints for each path
        .map(|path| {
          let mut waypoints: Vec<Node<Vec2>> = Vec::with_capacity(path.len());

          let mut first = path.first().unwrap();
          for next in path.iter().skip(1) {
            let edge = self
              .graph
              .edges_connecting(first.0, next.0)
              .next()
              .unwrap()
              .weight();

            waypoints.push(Node::new(
              next.1.name,
              next.1.kind,
              next.1.behavior,
              *edge,
            ));

            first = next;
          }

          waypoints
        })
        // Turn the Vec<Node<Vec2>> paths into PathfinderPaths
        .map(|path| {
          let mut pos = pos;
          let mut heading = heading;

          let mut first = &Node {
            name: from.name,
            kind: from.kind,
            behavior: from.behavior,
            data: pos,
          };
          for wp in path.iter() {
            let angle = angle_between_points(pos, wp.data);

            pos = first.data;
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
          count += 1;
          let mut pos = pos;
          let mut heading = heading;

          let mut first = &Node {
            name: from.name,
            kind: from.kind,
            behavior: from.behavior,
            data: pos,
          };
          for wp in path.path.iter() {
            let angle = angle_between_points(pos, wp.data);
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

            pos = first.data;
            heading = angle;

            first = wp;
          }

          true
        })
        .collect();

      let main_start = main_start.elapsed();
      tracing::info!(
        "filtered results to {} paths (out of {} total) in {:.2}ms",
        paths.len(),
        count,
        main_start.as_secs_f32() * 1000.0
      );

      // TODO: The distance function is broken for some reason so we won't
      // sort by it for now until its fixed.
      //
      // paths.sort_by(|a, b| {
      //   total_distance(a, pos)
      //     .partial_cmp(&total_distance(b, pos))
      //     .unwrap()
      // });
      paths.sort_by_key(|p| p.path.len());

      // for path in paths.iter() {
      //   println!(
      //     "path: {:?} ({} ft)",
      //     path
      //       .path
      //       .iter()
      //       .map(|n| n.name.clone())
      //       .collect::<Vec<_>>()
      //       .join(", "),
      //     total_distance_squared(&path.path, pos).sqrt()
      //   );
      // }

      paths.first().map(|p| {
        let mut p = p.clone();
        p.path = p
          .path
          .into_iter()
          .rev()
          .enumerate()
          .map(|(i, wp)| {
            let mut wp = wp.clone();
            if i == 0 {
              wp.behavior = to.behavior;
            }

            wp
          })
          .rev()
          .collect();

        p
      })
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
          name: Intern::from_ref("B"),
          kind: NodeKind::Apron,
          behavior: NodeBehavior::GoTo,
          data: b
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
            name: Intern::from_ref("B"),
            kind: NodeKind::Apron,
            behavior: NodeBehavior::GoTo,
            data: b
          },
          Node {
            name: Intern::from_ref("C"),
            kind: NodeKind::Apron,
            behavior: NodeBehavior::GoTo,
            data: c
          },
          Node {
            name: Intern::from_ref("D"),
            kind: NodeKind::Apron,
            behavior: NodeBehavior::GoTo,
            data: d
          }
        ],
        a
      ),
      distance
    );
  }

  mod pathfinder {
    use crate::entities::airport::Taxiway;

    use super::*;

    #[test]
    fn calculate_two_taxiways() {
      let mut pathfinder = Pathfinder::new();

      let mut segments = Vec::new();
      let taxiway_a = Taxiway::new(
        Intern::from_ref("A"),
        Vec2::new(0.0, 0.0),
        Vec2::new(10.0, 0.0),
      );

      let taxiway_b = Taxiway::new(
        Intern::from_ref("B"),
        Vec2::new(5.0, -5.0),
        Vec2::new(5.0, 5.0),
      );

      segments.push(Object::Taxiway(taxiway_a));
      segments.push(Object::Taxiway(taxiway_b));
      pathfinder.calculate(segments);

      let path = pathfinder.path_to(
        Node {
          name: Intern::from_ref("A"),
          kind: NodeKind::Taxiway,
          behavior: NodeBehavior::GoTo,
          data: (),
        },
        Node {
          name: Intern::from_ref("B"),
          kind: NodeKind::Taxiway,
          behavior: NodeBehavior::GoTo,
          data: (),
        },
        Vec2::new(0.0, 0.0),
        90.0,
      );

      assert!(path.is_some());
      if let Some(path) = path {
        assert_eq!(path.path.len(), 1);
        assert_eq!(path.path[0].name, Intern::from_ref("B"));
        assert_eq!(path.path[0].data, Vec2::new(5.0, 0.0));
      }
    }

    #[test]
    fn calculate_two_taxiways_2() {
      let mut pathfinder = Pathfinder::new();

      let mut segments = Vec::new();
      let taxiway_a = Taxiway::new(
        Intern::from_ref("A"),
        Vec2::new(0.0, 0.0),
        Vec2::new(10.0, 0.0),
      );

      let taxiway_b = Taxiway::new(
        Intern::from_ref("B"),
        Vec2::new(5.0, -5.0),
        Vec2::new(5.0, 5.0),
      );

      segments.push(Object::Taxiway(taxiway_a));
      segments.push(Object::Taxiway(taxiway_b));
      pathfinder.calculate(segments);

      let path = pathfinder.path_to(
        Node {
          name: Intern::from_ref("A"),
          kind: NodeKind::Taxiway,
          behavior: NodeBehavior::GoTo,
          data: (),
        },
        Node {
          name: Intern::from_ref("B"),
          kind: NodeKind::Taxiway,
          behavior: NodeBehavior::GoTo,
          data: (),
        },
        Vec2::new(2.0, 0.0),
        90.0,
      );

      assert!(path.is_some());
      if let Some(path) = path {
        assert_eq!(path.path.len(), 1);
        assert_eq!(path.path[0].name, Intern::from_ref("B"));
        assert_eq!(path.path[0].data, Vec2::new(5.0, 0.0));
      }
    }

    #[test]
    fn taxiway_before_runway() {
      let mut pathfinder = Pathfinder::new();

      let mut segments = Vec::new();
      let taxiway_a = Taxiway::new(
        Intern::from_ref("A"),
        Vec2::new(0.0, 0.0),
        Vec2::new(10.0, 0.0),
      );

      let runway_36 = Runway {
        id: Intern::from_ref("36"),
        start: Vec2::new(5.0, 0.0),
        heading: 360.0,
        length: 500.0,
      };

      segments.push(Object::Taxiway(taxiway_a));
      segments.push(Object::Runway(runway_36));
      pathfinder.calculate(segments);

      let path = pathfinder.path_to(
        Node {
          name: Intern::from_ref("A"),
          kind: NodeKind::Taxiway,
          behavior: NodeBehavior::GoTo,
          data: (),
        },
        Node {
          name: Intern::from_ref("36"),
          kind: NodeKind::Runway,
          behavior: NodeBehavior::GoTo,
          data: (),
        },
        Vec2::new(2.0, 0.0),
        90.0,
      );

      assert!(path.is_some());
      if let Some(path) = path {
        assert_eq!(path.path.len(), 1);
        assert_eq!(path.path[0].name, Intern::from_ref("36"));
        assert_eq!(path.path[0].behavior, NodeBehavior::GoTo);
        // This is slightly off due to floating-point math, so we can't
        // assert the coordinates of the intersection.
        //
        // TODO: use intersection math to assert
        // assert_eq!(path.path[0].value, Vec2::new(5.0, 0.0));
      }
    }

    #[test]
    fn taxiway_before_runway_hold_short() {
      let mut pathfinder = Pathfinder::new();

      let mut segments = Vec::new();
      let taxiway_a = Taxiway::new(
        Intern::from_ref("A"),
        Vec2::new(0.0, 0.0),
        Vec2::new(10.0, 0.0),
      );

      let runway_36 = Runway {
        id: Intern::from_ref("36"),
        start: Vec2::new(5.0, 0.0),
        heading: 360.0,
        length: 500.0,
      };

      segments.push(Object::Taxiway(taxiway_a));
      segments.push(Object::Runway(runway_36));
      pathfinder.calculate(segments);

      let path = pathfinder.path_to(
        Node {
          name: Intern::from_ref("A"),
          kind: NodeKind::Taxiway,
          behavior: NodeBehavior::GoTo,
          data: (),
        },
        Node {
          name: Intern::from_ref("36"),
          kind: NodeKind::Runway,
          behavior: NodeBehavior::HoldShort,
          data: (),
        },
        Vec2::new(2.0, 0.0),
        90.0,
      );

      assert!(path.is_some());
      if let Some(path) = path {
        assert_eq!(path.path.len(), 1);
        assert_eq!(path.path[0].name, Intern::from_ref("36"));
        assert_eq!(path.path[0].behavior, NodeBehavior::HoldShort);
        // This is slightly off due to floating-point math, so we can't
        // assert the coordinates of the intersection.
        //
        // TODO: use intersection math to assert
        // assert_eq!(path.path[0].value, Vec2::new(5.0, 0.0));
      }
    }
  }
}
