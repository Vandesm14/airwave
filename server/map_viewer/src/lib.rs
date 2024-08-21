use glam::Vec2;
use nannou::{
  color::{self},
  geom,
};
use serde::{Deserialize, Serialize};

use shared::{
  structs::{Gate, Runway, Taxiway, Terminal},
  FEET_PER_UNIT,
};

pub mod entity_constructor;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RefType<T> {
  A(T),
  B(T),
  R(T),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RefOrValue<T>
where
  T: Clone + Serialize,
{
  Action(Box<Action<T>>),
  Value(T),
  Ref(RefType<String>),
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[repr(transparent)]
pub struct Degrees(f32);

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[repr(transparent)]
pub struct Feet(f32);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]

pub enum Action<T>
where
  T: Clone + Serialize,
{
  Move(RefOrValue<Vec2>, RefOrValue<Degrees>, RefOrValue<Feet>),
  Add(RefOrValue<T>, RefOrValue<T>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Var {
  Position(RefOrValue<Vec2>),
  Degrees(RefOrValue<Degrees>),
  Feet(RefOrValue<Feet>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EntityData {
  Taxiway {
    a: RefOrValue<Vec2>,
    b: RefOrValue<Vec2>,
  },
  Runway {
    a: RefOrValue<Vec2>,
    b: RefOrValue<Vec2>,
  },
  Terminal {
    a: RefOrValue<Vec2>,
    b: RefOrValue<Vec2>,
    c: RefOrValue<Vec2>,
    d: RefOrValue<Vec2>,

    gates: Vec<Entity>,
  },
  Gate {
    a: RefOrValue<Vec2>,
  },
  Var(Var),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entity {
  pub id: String,
  pub data: EntityData,
}

pub fn glam_to_geom(v: Vec2) -> geom::Vec2 {
  geom::Vec2::new(v.x, v.y)
}

pub trait Draw {
  fn draw(&self, draw: &nannou::Draw, scale: f32);
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Airport {
  pub taxiways: Vec<Taxiway>,
  pub runways: Vec<Runway>,
  pub terminals: Vec<Terminal>,
}

impl Airport {
  pub fn add_taxiway(&mut self, taxiway: Taxiway) {
    let taxiway = taxiway.extend_ends_by(FEET_PER_UNIT * 100.0);
    self.taxiways.push(taxiway);
  }

  pub fn add_runway(&mut self, runway: Runway) {
    self.runways.push(runway);
  }

  pub fn add_terminal(&mut self, terminal: Terminal) {
    self.terminals.push(terminal);
  }
}

impl Draw for Taxiway {
  fn draw(&self, draw: &nannou::Draw, scale: f32) {
    draw
      .line()
      .start(glam_to_geom(self.a * scale))
      .end(glam_to_geom(self.b * scale))
      .weight(200.0 * scale)
      .color(color::rgb::<u8>(0x99, 0x99, 0x99));
  }
}

impl Draw for Runway {
  fn draw(&self, draw: &nannou::Draw, scale: f32) {
    draw
      .line()
      .start(glam_to_geom(self.a() * scale))
      .end(glam_to_geom(self.b() * scale))
      .weight(250.0 * scale)
      .color(color::rgb::<u8>(0x66, 0x66, 0x66));
  }
}

impl Draw for Terminal {
  fn draw(&self, draw: &nannou::Draw, scale: f32) {
    for gate in self.gates.iter() {
      gate.draw(draw, scale);
    }
  }
}

impl Draw for Gate {
  fn draw(&self, draw: &nannou::Draw, scale: f32) {
    let pos = self.pos * scale;
    draw
      .ellipse()
      .x_y(pos.x, pos.y)
      .width(200.0 * scale)
      .height(200.0 * scale)
      .color(color::rgb::<u8>(0xff, 0x00, 0x00));
  }
}

impl Draw for Airport {
  fn draw(&self, draw: &nannou::Draw, scale: f32) {
    for taxiway in self.taxiways.iter() {
      taxiway.draw(draw, scale);
    }
    for runway in self.runways.iter() {
      runway.draw(draw, scale);
    }
    for terminal in self.terminals.iter() {
      terminal.draw(draw, scale);
    }
  }
}
