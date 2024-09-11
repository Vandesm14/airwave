use glam::Vec2;
use nannou::{
  color::{self},
  geom,
};

use engine::{
  objects::{
    airport::{Gate, Runway, Taxiway, Terminal},
    airspace::Airspace,
  },
  pathfinder::{Node, NodeBehavior, NodeKind},
};

fn glam_to_geom(v: Vec2) -> geom::Vec2 {
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
    let scaled_start = glam_to_geom(self.start() * scale);
    let scaled_end = glam_to_geom(self.end() * scale);

    draw
      .line()
      .start(scaled_start)
      .end(scaled_end)
      .weight(250.0 * scale)
      .color(color::rgb::<u8>(0x66, 0x66, 0x66));

    draw
      .ellipse()
      .x_y(scaled_start.x, scaled_start.y)
      .width(200.0 * scale)
      .height(200.0 * scale)
      .color(color::rgb::<u8>(0xff, 0x00, 0x00));
  }
}

impl Draw for Terminal {
  fn draw(&self, draw: &nannou::Draw, scale: f32) {
    draw
      .quad()
      .points(
        glam_to_geom(self.a * scale),
        glam_to_geom(self.b * scale),
        glam_to_geom(self.c * scale),
        glam_to_geom(self.d * scale),
      )
      .color(color::rgb::<u8>(0x99, 0x99, 0x99));

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

impl Draw for Node<Vec2> {
  fn draw(&self, draw: &nannou::Draw, scale: f32) {
    let pos = self.value * scale;
    draw
      .ellipse()
      .x_y(pos.x, pos.y)
      .width(100.0 * scale)
      .height(100.0 * scale)
      .color(color::rgb::<u8>(0xff, 0xff, 0x00));
  }
}

impl Draw for Vec<Node<Vec2>> {
  fn draw(&self, draw: &nannou::Draw, scale: f32) {
    let yellow = color::rgb::<u8>(0xff, 0xff, 0x00);
    draw.polyline().weight(20.0 * scale).points_colored(
      self.iter().map(|w| (glam_to_geom(w.value * scale), yellow)),
    );

    for point in self.iter() {
      point.draw(draw, scale);
    }
  }
}
