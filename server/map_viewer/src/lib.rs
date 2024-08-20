use glam::Vec2;
use nannou::{color::*, geom, prelude::App};
use serde::{Deserialize, Serialize};

use shared::{
  structs::{Runway, Taxiway, Terminal},
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
  T: Clone,
{
  Action(Box<Action>),
  Value(T),
  Ref(RefType<String>),
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[repr(transparent)]
pub struct Degrees(f32);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]

pub enum Action {
  Move(RefOrValue<Vec2>, RefOrValue<Degrees>, RefOrValue<f32>),
  AddVec2(RefOrValue<Vec2>, RefOrValue<Vec2>),
  AddDegrees(RefOrValue<Degrees>, RefOrValue<Degrees>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EntityData {
  Taxiway {
    a: RefOrValue<Vec2>,
    b: RefOrValue<Vec2>,
  },
  Runway {
    pos: RefOrValue<Vec2>,
    heading: RefOrValue<f32>,
    length: RefOrValue<f32>,
  },
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
  fn draw(&self, draw: &nannou::Draw, scale: f32) {
    draw
      .line()
      .start(glam_to_geom(self.a * scale))
      .end(glam_to_geom(self.b * scale))
      .weight(200.0 * scale)
      .color(WHITE);
  }
}

impl Draw for Runway {
  fn draw(&self, draw: &nannou::Draw, scale: f32) {}
}

impl Draw for Terminal {
  fn draw(&self, draw: &nannou::Draw, scale: f32) {}
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
