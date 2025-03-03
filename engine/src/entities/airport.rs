use glam::Vec2;
use internment::Intern;
use serde::{Deserialize, Serialize};

use crate::{
  inverse_degrees, move_point,
  pathfinder::{Object, Pathfinder},
  Line, Translate,
};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Frequencies {
  pub approach: f32,
  pub departure: f32,
  pub tower: f32,
  pub ground: f32,
  pub center: f32,
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Airport {
  pub id: Intern<String>,
  pub frequencies: Frequencies,

  pub center: Vec2,
  pub runways: Vec<Runway>,
  pub taxiways: Vec<Taxiway>,
  pub terminals: Vec<Terminal>,

  #[serde(skip)]
  pub pathfinder: Pathfinder,
}

impl Translate for Airport {
  fn translate(&mut self, offset: Vec2) -> &mut Self {
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

    self
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

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Runway {
  pub id: Intern<String>,
  pub pos: Vec2,
  pub heading: f32,
  pub length: f32,
}

impl Translate for Runway {
  fn translate(&mut self, offset: Vec2) -> &mut Self {
    self.pos += offset;
    self
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Taxiway {
  pub id: Intern<String>,
  pub a: Vec2,
  pub b: Vec2,
}

impl Translate for Taxiway {
  fn translate(&mut self, offset: Vec2) -> &mut Self {
    self.a += offset;
    self.b += offset;
    self
  }
}

impl Taxiway {
  pub fn new(id: Intern<String>, a: Vec2, b: Vec2) -> Self {
    Self { id, a, b }
  }

  pub fn extend_ends_by(mut self, padding: f32) -> Self {
    self.a = self.a.move_towards(self.b, -padding);
    self.b = self.b.move_towards(self.a, -padding);

    self
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Gate {
  pub id: Intern<String>,
  pub pos: Vec2,
  pub heading: f32,
  pub available: bool,
}

impl Translate for Gate {
  fn translate(&mut self, offset: Vec2) -> &mut Self {
    self.pos += offset;
    self
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Terminal {
  pub id: Intern<String>,
  pub a: Vec2,
  pub b: Vec2,
  pub c: Vec2,
  pub d: Vec2,

  pub gates: Vec<Gate>,
  pub apron: Line,
}

impl Translate for Terminal {
  fn translate(&mut self, offset: Vec2) -> &mut Self {
    self.a += offset;
    self.b += offset;
    self.c += offset;
    self.d += offset;

    for gate in self.gates.iter_mut() {
      gate.translate(offset);
    }

    self
  }
}
