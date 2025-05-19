use engine::{
  AIRSPACE_RADIUS, NAUTICALMILES_TO_FEET,
  entities::{
    aircraft::Aircraft,
    airport::{Airport, Gate, Runway, Taxiway, Terminal},
    world::World,
  },
  geometry::move_point,
};
use glam::Vec2;
use nannou::color;

use crate::{glam_to_geom, midpoint, scale_point};

const TAXIWAY_COLOR: u8 = 0x55;
const RUNWAY_COLOR: u8 = 0x22;

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
    .xy(glam_to_geom(point + Vec2::new(0.0, 2.0)))
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

    // Draw the ILS line
    draw
      .line()
      .start(glam_to_geom(scale_point(self.start, offset, scale)))
      .end(glam_to_geom(scale_point(
        move_point(self.start, self.heading, -NAUTICALMILES_TO_FEET * 20.0),
        offset,
        scale,
      )))
      .weight(1.0)
      .color(color::rgb::<u8>(0x30, 0x87, 0xf2));
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
    let gate_size = 175.0;

    // Draw the gate dot
    let pos = scale_point(self.pos, offset, scale);
    draw
      .rect()
      .x_y(pos.x, pos.y)
      .rotate(-self.heading.to_radians())
      .wh(glam_to_geom(Vec2::splat(gate_size) * scale))
      .stroke_weight(2.0)
      .stroke(color::rgb::<u8>(0xaa, 0x22, 0x22))
      .color(color::rgb::<u8>(0x22, 0x22, 0x22));

    let point = scale_point(
      move_point(self.pos, self.heading, gate_size * 0.5),
      offset,
      scale,
    );
    draw
      .ellipse()
      .x_y(point.x, point.y)
      .width(30.0 * scale)
      .height(30.0 * scale)
      .color(color::YELLOW);
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

    let radius = AIRSPACE_RADIUS * scale * 2.0;

    draw
      .ellipse()
      .x_y(center.x, center.y)
      .width(radius)
      .height(radius)
      .color(color::rgba::<u8>(0, 0, 0, 0))
      .stroke_weight(2.0)
      .stroke(color::rgb::<u8>(0x22, 0x22, 0x22));
  }
}

impl Draw for Aircraft {
  fn draw(&self, draw: &nannou::Draw, scale: f32, offset: Vec2) {
    let pos = scale_point(self.pos, offset, scale);
    let point_scale = (6.0_f32).max(3000.0 * scale).min(20.0);

    draw
      .rect()
      .x_y(pos.x, pos.y)
      .width(point_scale)
      .height(point_scale)
      .color(color::GREEN);

    draw
      .line()
      .start(glam_to_geom(pos))
      .end(glam_to_geom(scale_point(
        move_point(
          self.pos,
          self.heading,
          (self.speed / 60.0) * NAUTICALMILES_TO_FEET,
        ),
        offset,
        scale,
      )))
      .weight(2.0)
      .color(color::GREEN);
  }
}

impl Draw for World {
  fn draw(&self, draw: &nannou::Draw, scale: f32, offset: Vec2) {
    for airport in self.airports.iter() {
      airport.draw(draw, scale, offset);
    }
  }
}
