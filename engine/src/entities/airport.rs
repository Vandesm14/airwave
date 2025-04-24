use glam::Vec2;
use internment::Intern;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
  inverse_degrees, move_point,
  pathfinder::{Object, Pathfinder},
  Line, Translate,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Frequencies {
  pub approach: f32,
  pub departure: f32,
  pub tower: f32,
  pub ground: f32,
  pub center: f32,
}

impl Default for Frequencies {
  fn default() -> Self {
    Self {
      approach: 118.5,
      departure: 118.5,
      tower: 118.5,
      ground: 118.5,
      center: 118.5,
    }
  }
}

impl Frequencies {
  pub fn try_from_string(&self, s: &str) -> Option<f32> {
    match s {
      "approach" => Some(self.approach),
      "departure" => Some(self.departure),
      "tower" => Some(self.tower),
      "ground" => Some(self.ground),
      "center" => Some(self.center),

      _ => None,
    }
  }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Airport {
  #[ts(as = "String")]
  pub id: Intern<String>,
  pub frequencies: Frequencies,

  #[ts(as = "(f32, f32)")]
  pub center: Vec2,
  pub runways: Vec<Runway>,
  pub taxiways: Vec<Taxiway>,
  pub terminals: Vec<Terminal>,

  #[serde(skip)]
  pub pathfinder: Pathfinder,
}

impl Translate for Airport {
  fn translate(&mut self, offset: Vec2) {
    self.center += offset;

    for runway in self.runways.iter_mut() {
      runway.translate(offset);
    }

    for taxiway in self.taxiways.iter_mut() {
      taxiway.translate(offset);
    }

    for terminal in self.terminals.iter_mut() {
      terminal.translate(offset);
    }
  }
}

impl Airport {
  pub fn new(id: Intern<String>, center: Vec2) -> Self {
    Self {
      id,
      center,
      runways: Vec::new(),
      taxiways: Vec::new(),
      terminals: Vec::new(),
      frequencies: Frequencies::default(),

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

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS)]
pub struct Runway {
  #[ts(as = "String")]
  pub id: Intern<String>,
  #[ts(as = "(f32, f32)")]
  pub pos: Vec2,
  pub heading: f32,
  pub length: f32,
}

impl Translate for Runway {
  fn translate(&mut self, offset: Vec2) {
    self.pos += offset;
  }
}

impl Runway {
  pub fn start(&self) -> Vec2 {
    move_point(self.pos, inverse_degrees(self.heading), self.length * 0.5)
  }

  pub fn end(&self) -> Vec2 {
    move_point(self.pos, self.heading, self.length * 0.5)
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Taxiway {
  #[ts(as = "String")]
  pub id: Intern<String>,
  pub segments: Vec<Line>,
}

impl Translate for Taxiway {
  fn translate(&mut self, offset: Vec2) {
    self.segments.iter_mut().for_each(|s| {
      s.translate(offset);
    })
  }
}

impl Taxiway {
  pub fn new(id: Intern<String>, line: Line) -> Self {
    Self {
      id,
      segments: vec![line],
    }
  }

  pub fn extend_ends_by(mut self, padding: f32) -> Self {
    for segment in self.segments.iter_mut() {
      segment.0 = segment.0.move_towards(segment.1, -padding);
      segment.1 = segment.1.move_towards(segment.0, -padding);
    }

    self
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Gate {
  #[ts(as = "String")]
  pub id: Intern<String>,
  #[ts(as = "(f32, f32)")]
  pub pos: Vec2,
  pub heading: f32,
  pub available: bool,
}

impl Translate for Gate {
  fn translate(&mut self, offset: Vec2) {
    self.pos += offset;
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Terminal {
  #[ts(as = "String")]
  pub id: Intern<String>,
  #[ts(as = "(f32, f32)")]
  pub a: Vec2,
  #[ts(as = "(f32, f32)")]
  pub b: Vec2,
  #[ts(as = "(f32, f32)")]
  pub c: Vec2,
  #[ts(as = "(f32, f32)")]
  pub d: Vec2,

  pub gates: Vec<Gate>,
  pub apron: Line,
}

impl Translate for Terminal {
  fn translate(&mut self, offset: Vec2) {
    self.a += offset;
    self.b += offset;
    self.c += offset;
    self.d += offset;

    for gate in self.gates.iter_mut() {
      gate.translate(offset);
    }
  }
}
