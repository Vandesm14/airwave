use std::fs;

use glam::Vec2;
use nannou::{
  color::*,
  geom,
  prelude::{App, Frame, Update},
};
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};

use shared::{
  structs::{Runway, Taxiway, Terminal},
  FEET_PER_UNIT,
};

fn main() {
  // Open the "airport.ron" file on `../` and deserialize it into a vec of entities

  let entities: Vec<Entity> =
    ron::de::from_bytes(include_bytes!("../../airport.ron")).unwrap();

  let runway = Entity {
    id: "20".into(),
    data: EntityData::Runway {
      pos: TypeOrRef::Type(Vec2::new(0.0, 0.0)),
      heading: TypeOrRef::Type(200.0),
      length: TypeOrRef::Type(7000.0),
    },
  };

  let taxiway = Entity {
    id: "A".into(),
    data: EntityData::Taxiway {
      a: TypeOrRef::Action(Box::new(Action::Move(
        TypeOrRef::Ref(RefType::A("20".into())),
        TypeOrRef::Ref(RefType::R("20".into())),
        TypeOrRef::Type(500.0),
      ))),
      b: TypeOrRef::Ref(RefType::B("20".into())),
    },
  };

  let rust_entities = vec![runway, taxiway];

  println!("{:?}", entities);
  println!("{:?}", rust_entities);

  // save the rust entities to airport_rust.ron
  fs::write(
    "airport_rust.ron",
    ron::ser::to_string_pretty(
      &rust_entities,
      PrettyConfig::new().struct_names(true).indentor("  ".into()),
    )
    .unwrap(),
  )
  .unwrap();

  // nannou::app(model).update(update).simple_window(view).run();
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

#[derive(Debug, Clone, PartialEq, Default)]
struct Model {
  entities: Vec<Entity>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum RefType<T> {
  A(T),
  B(T),
  R(T),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum TypeOrRef<T> {
  Action(Box<Action>),
  Type(T),
  Ref(RefType<String>),
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[repr(transparent)]
struct Degrees {
  degrees: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]

enum Action {
  Move(TypeOrRef<Vec2>, TypeOrRef<Degrees>, TypeOrRef<f32>),
  AddVec2(TypeOrRef<Vec2>, TypeOrRef<Vec2>),
  AddDegrees(TypeOrRef<Degrees>, TypeOrRef<Degrees>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum EntityData {
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
struct Entity {
  id: String,
  data: EntityData,
}

fn model(_app: &App) -> Model {
  let runway = Entity {
    id: "20".into(),
    data: EntityData::Runway {
      pos: TypeOrRef::Type(Vec2::new(0.0, 0.0)),
      heading: TypeOrRef::Type(200.0),
      length: TypeOrRef::Type(7000.0),
    },
  };

  let taxiway = Entity {
    id: "A".into(),
    data: EntityData::Taxiway {
      a: TypeOrRef::Action(Box::new(Action::Move(
        TypeOrRef::Ref(RefType::A("20".into())),
        TypeOrRef::Ref(RefType::R("20".into())),
        TypeOrRef::Type(500.0),
      ))),
      b: TypeOrRef::Ref(RefType::B("20".into())),
    },
  };
  Model {
    entities: vec![runway, taxiway],
  }
}

fn glam_to_geom(v: Vec2) -> geom::Vec2 {
  geom::Vec2::new(v.x, v.y)
}

trait Draw {
  fn draw(&self, app: &App);
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

fn update(_app: &App, _model: &mut Model, _update: Update) {}

fn view(app: &App, model: &Model, frame: Frame) {
  frame.clear(BLACK);
}
