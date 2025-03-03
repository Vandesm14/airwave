use engine::{
  angle_between_points,
  entities::airport::{Airport, Gate, Runway, Taxiway, Terminal},
};
use glam::Vec2;
use internment::Intern;
use nannou::{color, geom};
use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, SlotMap};

pub fn scale_point(point: Vec2, offset: Vec2, scale: f32) -> Vec2 {
  (point + offset) / scale
}

pub fn unscale_point(point: Vec2, offset: Vec2, scale: f32) -> Vec2 {
  point * scale - offset
}

new_key_type! { pub struct PointKey; }

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorldFile {
  pub points: SlotMap<PointKey, Vec2>,
  pub meta_airport: MetaAirport,
  pub airport: Airport,
}

impl WorldFile {
  pub fn find_closest_point(
    &self,
    test_point: Vec2,
    threshold: f32,
  ) -> Option<(PointKey, Vec2)> {
    let mut smallest_distance = threshold;
    let mut point: Option<(PointKey, Vec2)> = None;
    for p in self.points.iter() {
      let distance = p.1.distance_squared(test_point);
      if distance < smallest_distance {
        smallest_distance = distance;
        point = Some((p.0, *p.1));
      }
    }

    point
  }

  pub fn regenerate_airport(&mut self) {
    self.airport = self.meta_airport.clone().into_airport(&self.points);
  }

  // Placeholder for if we need more functionality
  pub fn trigger_update(&mut self) {
    self.regenerate_airport();
  }
}

pub fn glam_to_geom(v: Vec2) -> geom::Vec2 {
  geom::Vec2::new(v.x, v.y)
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct MetaTaxiway {
  pub name: String,
  pub a: PointKey,
  pub b: PointKey,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct MetaRunway {
  pub name: String,
  pub a: PointKey,
  pub b: PointKey,
}

pub trait Draw {
  fn draw(&self, draw: &nannou::Draw, scale: f32, offset: Vec2);
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct MetaAirport {
  pub taxiways: Vec<MetaTaxiway>,
  pub runways: Vec<MetaRunway>,
}

impl MetaAirport {
  pub fn into_airport(self, points: &SlotMap<PointKey, Vec2>) -> Airport {
    let mut airport = Airport::default();
    for t in self.taxiways.into_iter() {
      let a = points.get(t.a).unwrap();
      let b = points.get(t.b).unwrap();
      airport.add_taxiway(Taxiway::new(Intern::from(t.name), *a, *b));
    }

    for r in self.runways.into_iter() {
      let a = points.get(r.a).unwrap();
      let b = points.get(r.b).unwrap();
      airport.add_runway(Runway {
        id: Intern::from(r.name),
        heading: angle_between_points(*a, *b),
        length: a.distance(*b),
        pos: a.midpoint(*b),
      });
    }

    airport
  }
}

impl Draw for Taxiway {
  fn draw(&self, draw: &nannou::Draw, scale: f32, offset: Vec2) {
    draw
      .line()
      .start(glam_to_geom(self.a * scale))
      .end(glam_to_geom(self.b * scale))
      .weight(200.0 * scale)
      .color(color::rgb::<u8>(0x99, 0x99, 0x99));
  }
}

impl Draw for Runway {
  fn draw(&self, draw: &nannou::Draw, scale: f32, offset: Vec2) {
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
  fn draw(&self, draw: &nannou::Draw, scale: f32, offset: Vec2) {
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
      gate.draw(draw, scale, offset);
    }
  }
}

impl Draw for Gate {
  fn draw(&self, draw: &nannou::Draw, scale: f32, offset: Vec2) {
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
