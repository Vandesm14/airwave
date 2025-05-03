use engine::entities::airport::{Airport, Gate, Runway, Taxiway, Terminal};
use glam::Vec2;
use nannou::{color, geom};

const TAXIWAY_COLOR: u8 = 0x55;
const RUNWAY_COLOR: u8 = 0x22;

pub fn scale_point(point: Vec2, offset: Vec2, scale: f32) -> Vec2 {
  (point + offset) * scale
}

pub fn unscale_point(point: Vec2, offset: Vec2, scale: f32) -> Vec2 {
  (point / scale) - offset
}

pub fn glam_to_geom(v: Vec2) -> geom::Vec2 {
  geom::Vec2::new(v.x, v.y)
}

pub trait Draw {
  fn draw(&self, draw: &nannou::Draw, scale: f32, offset: Vec2);
}

impl Draw for Taxiway {
  fn draw(&self, draw: &nannou::Draw, scale: f32, offset: Vec2) {
    draw
      .line()
      .start(glam_to_geom(scale_point(self.a, offset, scale)))
      .end(glam_to_geom(scale_point(self.b, offset, scale)))
      .weight(200.0 * scale)
      .color(color::rgb::<u8>(
        TAXIWAY_COLOR,
        TAXIWAY_COLOR,
        TAXIWAY_COLOR,
      ));
  }
}

impl Draw for Runway {
  fn draw(&self, draw: &nannou::Draw, scale: f32, offset: Vec2) {
    // let scaled_start = glam_to_geom(self.start());
    // let scaled_end = glam_to_geom(self.end());

    draw
      .line()
      .start(glam_to_geom(scale_point(self.start(), offset, scale)))
      .end(glam_to_geom(scale_point(self.end(), offset, scale)))
      .weight(250.0 * scale)
      .color(color::rgb::<u8>(RUNWAY_COLOR, RUNWAY_COLOR, RUNWAY_COLOR));
  }
}

impl Draw for Terminal {
  fn draw(&self, draw: &nannou::Draw, scale: f32, offset: Vec2) {
    draw
      .quad()
      .points(
        glam_to_geom(scale_point(self.a, offset, scale)),
        glam_to_geom(scale_point(self.b, offset, scale)),
        glam_to_geom(scale_point(self.c, offset, scale)),
        glam_to_geom(scale_point(self.d, offset, scale)),
      )
      .color(color::rgb::<u8>(
        TAXIWAY_COLOR,
        TAXIWAY_COLOR,
        TAXIWAY_COLOR,
      ));

    for gate in self.gates.iter() {
      gate.draw(draw, scale, offset);
    }
  }
}

impl Draw for Gate {
  fn draw(&self, draw: &nannou::Draw, scale: f32, offset: Vec2) {
    let pos = scale_point(self.pos, offset, scale);
    draw
      .ellipse()
      .x_y(pos.x, pos.y)
      .width(200.0 * scale)
      .height(200.0 * scale)
      .color(color::rgb::<u8>(0xff, 0x00, 0x00));
  }
}

impl Draw for Airport {
  fn draw(&self, draw: &nannou::Draw, scale: f32, offset: Vec2) {
    for taxiway in self.taxiways.iter() {
      taxiway.draw(draw, scale, offset);
    }
    for runway in self.runways.iter() {
      runway.draw(draw, scale, offset);
    }
    for terminal in self.terminals.iter() {
      terminal.draw(draw, scale, offset);
    }
  }
}
