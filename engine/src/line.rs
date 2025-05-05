use glam::Vec2;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
  entities::airport::{Runway, Taxiway, Terminal},
  geometry::Translate,
};

#[derive(
  Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize, TS,
)]
#[ts(export)]
pub struct Line(
  #[ts(as = "(f32, f32)")] pub Vec2,
  #[ts(as = "(f32, f32)")] pub Vec2,
);

impl Translate for Line {
  fn translate(&mut self, offset: Vec2) -> &mut Self {
    self.0 += offset;
    self.1 += offset;
    self
  }
}

impl Line {
  pub fn new(a: Vec2, b: Vec2) -> Self {
    Self(a, b)
  }

  pub fn midpoint(&self) -> Vec2 {
    self.0.midpoint(self.1)
  }

  pub fn extend(&self, padding: f32) -> Self {
    Self(
      self.0.move_towards(self.1, -padding),
      self.1.move_towards(self.0, -padding),
    )
  }

  pub fn length(&self) -> f32 {
    self.0.distance(self.1)
  }
}

impl From<Runway> for Line {
  fn from(value: Runway) -> Self {
    Line::new(value.start, value.end())
  }
}

impl From<Taxiway> for Line {
  fn from(value: Taxiway) -> Self {
    Line::new(value.a, value.b)
  }
}

impl From<Terminal> for Line {
  fn from(value: Terminal) -> Self {
    value.apron
  }
}
