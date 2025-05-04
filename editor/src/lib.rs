pub mod viewer;

use engine::entities::airport::{Airport, Gate, Runway, Taxiway, Terminal};
use glam::Vec2;
use nannou::{color, geom};

const TAXIWAY_COLOR: u8 = 0x55;
const RUNWAY_COLOR: u8 = 0x22;
const FONT_SIZE: f32 = 150.0;

pub fn scale_point(point: Vec2, offset: Vec2, scale: f32) -> Vec2 {
  (point + offset) * scale
}

pub fn unscale_point(point: Vec2, offset: Vec2, scale: f32) -> Vec2 {
  (point / scale) - offset
}

pub fn glam_to_geom(v: Vec2) -> geom::Vec2 {
  geom::Vec2::new(v.x, v.y)
}

// Helper function to get midpoint between two points
pub fn midpoint(a: Vec2, b: Vec2) -> Vec2 {
  (a + b) * 0.5
}

pub trait Draw {
  fn draw(&self, draw: &nannou::Draw, scale: f32, offset: Vec2);

  fn draw_label(&self, _draw: &nannou::Draw, _scale: f32, _offset: Vec2) {}
}

fn draw_label(
  text: String,
  pos: Vec2,
  draw: &nannou::Draw,
  scale: f32,
  offset: Vec2,
) {
  let point = scale_point(pos, offset, scale);

  let wh = Vec2::new(30.0, 22.0);

  // Background rectangle for the label
  draw
    .rect()
    .xy(glam_to_geom(point))
    .wh(glam_to_geom(wh))
    .color(color::rgba(0.0, 0.0, 0.0, 0.8));

  // Draw the label text
  draw
    .text(&text)
    .xy(glam_to_geom(point + Vec2::new(0.0, 4.0)))
    .wh(glam_to_geom(wh))
    .font_size(16)
    .color(color::ORANGE);
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

  fn draw_label(&self, draw: &nannou::Draw, scale: f32, offset: Vec2) {
    let middle = midpoint(self.a, self.b);
    draw_label(self.id.to_string(), middle, draw, scale, offset);
  }
}

impl Draw for Runway {
  fn draw(&self, draw: &nannou::Draw, scale: f32, offset: Vec2) {
    // Draw the runway line
    draw
      .line()
      .start(glam_to_geom(scale_point(self.start, offset, scale)))
      .end(glam_to_geom(scale_point(self.end(), offset, scale)))
      .weight(200.0 * scale)
      .color(color::rgb::<u8>(RUNWAY_COLOR, RUNWAY_COLOR, RUNWAY_COLOR));
  }

  fn draw_label(&self, draw: &nannou::Draw, scale: f32, offset: Vec2) {
    draw_label(self.id.to_string(), self.start, draw, scale, offset);
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

    draw
      .line()
      .start(glam_to_geom(scale_point(self.apron.0, offset, scale)))
      .end(glam_to_geom(scale_point(self.apron.1, offset, scale)))
      .weight(20.0 * scale)
      .color(color::GREENYELLOW);
  }
}

impl Draw for Gate {
  fn draw(&self, draw: &nannou::Draw, scale: f32, offset: Vec2) {
    // Draw the gate dot
    let pos = scale_point(self.pos, offset, scale);
    draw
      .ellipse()
      .x_y(pos.x, pos.y)
      .width(100.0 * scale)
      .height(100.0 * scale)
      .color(color::RED);
  }

  fn draw_label(&self, draw: &nannou::Draw, scale: f32, offset: Vec2) {
    draw_label(
      self.id.to_string(),
      self.pos + Vec2::new(0.0, 0.0),
      draw,
      scale,
      offset,
    );
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
    for gate in self.terminals.iter().flat_map(|t| t.gates.iter()) {
      gate.draw(draw, scale, offset);
    }

    for point in self.pathfinder.graph.edge_weights() {
      let point = scale_point(*point, offset, scale);
      draw
        .ellipse()
        .x_y(point.x, point.y)
        .width(100.0 * scale)
        .height(100.0 * scale)
        .color(color::YELLOW);
    }

    let center = scale_point(self.center, offset, scale);
    draw
      .ellipse()
      .x_y(center.x, center.y)
      .width(150.0 * scale)
      .height(150.0 * scale)
      .color(color::BLUE);

    for taxiway in self.taxiways.iter() {
      taxiway.draw_label(draw, scale, offset);
    }
    for runway in self.runways.iter() {
      runway.draw_label(draw, scale, offset);
    }
    for gate in self.terminals.iter().flat_map(|t| t.gates.iter()) {
      gate.draw_label(draw, scale, offset);
    }
  }
}
