use glam::Vec2;
use internment::Intern;
use serde::{Deserialize, Serialize};

use crate::{
  deserialize_vec2, inverse_degrees, move_point,
  pathfinder::{Object, Pathfinder},
  serialize_vec2, Line,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Airport {
  pub id: Intern<String>,
  #[serde(serialize_with = "serialize_vec2")]
  #[serde(deserialize_with = "deserialize_vec2")]
  pub center: Vec2,
  pub runways: Vec<Runway>,
  pub taxiways: Vec<Taxiway>,
  pub terminals: Vec<Terminal>,

  #[serde(skip)]
  pub pathfinder: Pathfinder,
}

impl Airport {
  pub fn new(id: Intern<String>, center: Vec2) -> Self {
    Self {
      id,
      center,
      runways: Vec::new(),
      taxiways: Vec::new(),
      terminals: Vec::new(),

      pathfinder: Pathfinder::new(),
    }
  }

  pub fn add_taxiway(&mut self, taxiway: Taxiway) {
    let taxiway = taxiway.extend_ends_by(100.0);
    self.taxiways.push(taxiway);
  }

  pub fn add_runway(&mut self, mut runway: Runway) {
    runway.length += 200.0;
    self.runways.push(runway);
  }

  pub fn calculate_waypoints(&mut self) {
    let mut nodes: Vec<Object> = Vec::new();
    nodes.extend(self.runways.iter().map(|r| r.clone().into()));
    nodes.extend(self.taxiways.iter().map(|t| t.clone().into()));
    nodes.extend(self.terminals.iter().map(|g| g.clone().into()));

    self.pathfinder.calculate(nodes);
  }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Runway {
  pub id: Intern<String>,
  #[serde(flatten)]
  #[serde(serialize_with = "serialize_vec2")]
  #[serde(deserialize_with = "deserialize_vec2")]
  pub pos: Vec2,
  pub heading: f32,
  pub length: f32,
}

impl Runway {
  pub fn start(&self) -> Vec2 {
    move_point(self.pos, inverse_degrees(self.heading), self.length * 0.5)
  }

  pub fn end(&self) -> Vec2 {
    move_point(self.pos, self.heading, self.length * 0.5)
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Taxiway {
  pub id: String,
  #[serde(serialize_with = "serialize_vec2")]
  #[serde(deserialize_with = "deserialize_vec2")]
  pub a: Vec2,
  #[serde(serialize_with = "serialize_vec2")]
  #[serde(deserialize_with = "deserialize_vec2")]
  pub b: Vec2,
}

impl Taxiway {
  pub fn new(id: String, a: Vec2, b: Vec2) -> Self {
    Self { id, a, b }
  }

  pub fn extend_ends_by(mut self, padding: f32) -> Self {
    self.a = self.a.move_towards(self.b, -padding);
    self.b = self.b.move_towards(self.a, -padding);

    self
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Terminal {
  pub id: char,
  #[serde(serialize_with = "serialize_vec2")]
  #[serde(deserialize_with = "deserialize_vec2")]
  pub a: Vec2,
  #[serde(serialize_with = "serialize_vec2")]
  #[serde(deserialize_with = "deserialize_vec2")]
  pub b: Vec2,
  #[serde(serialize_with = "serialize_vec2")]
  #[serde(deserialize_with = "deserialize_vec2")]
  pub c: Vec2,
  #[serde(serialize_with = "serialize_vec2")]
  #[serde(deserialize_with = "deserialize_vec2")]
  pub d: Vec2,

  pub gates: Vec<Gate>,
  pub apron: Line,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Gate {
  pub id: String,
  #[serde(serialize_with = "serialize_vec2")]
  #[serde(deserialize_with = "deserialize_vec2")]
  pub pos: Vec2,
  pub heading: f32,
}
