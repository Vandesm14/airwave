use glam::Vec2;
use nannou::{color::*, geom, prelude::App};
use serde::{Deserialize, Serialize};

use shared::{
  structs::{Runway, Taxiway, Terminal},
  FEET_PER_UNIT,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RefType<T> {
  A(T),
  B(T),
  R(T),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeOrRef<T> {
  Action(Box<Action>),
  Type(T),
  Ref(RefType<String>),
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[repr(transparent)]
pub struct Degrees {
  degrees: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]

pub enum Action {
  Move(TypeOrRef<Vec2>, TypeOrRef<Degrees>, TypeOrRef<f32>),
  AddVec2(TypeOrRef<Vec2>, TypeOrRef<Vec2>),
  AddDegrees(TypeOrRef<Degrees>, TypeOrRef<Degrees>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EntityData {
  Taxiway {
    a: TypeOrRef<Vec2>,
    b: TypeOrRef<Vec2>,
  },
  Runway {
    pos: TypeOrRef<Vec2>,
    heading: TypeOrRef<f32>,
    length: TypeOrRef<f32>,
  },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParsedEntity {
  id: String,
  data: EntityData,
}

fn glam_to_geom(v: Vec2) -> geom::Vec2 {
  geom::Vec2::new(v.x, v.y)
}

trait Draw {
  fn draw(&self, app: &App);
}

#[derive(Debug, Clone, PartialEq, Default)]
struct Airport {
  taxiways: Vec<Taxiway>,
  runways: Vec<Runway>,
  terminals: Vec<Terminal>,
}

impl Airport {
  pub fn add_taxiway(&mut self, taxiway: Taxiway) {
    let taxiway = taxiway.extend_ends_by(FEET_PER_UNIT * 100.0);
    self.taxiways.push(taxiway);
  }
}

impl Draw for Taxiway {
  fn draw(&self, app: &App) {
    app
      .draw()
      .line()
      .start(glam_to_geom(self.a))
      .end(glam_to_geom(self.b))
      .weight(FEET_PER_UNIT * 200.0)
      .color(WHITE);
  }
}

impl Draw for Runway {
  fn draw(&self, app: &App) {}
}

impl Draw for Terminal {
  fn draw(&self, app: &App) {}
}

impl Draw for Airport {
  fn draw(&self, app: &App) {
    for taxiway in self.taxiways.iter() {
      taxiway.draw(app);
    }
    for runway in self.runways.iter() {
      runway.draw(app);
    }
    for terminal in self.terminals.iter() {
      terminal.draw(app);
    }
  }
}
