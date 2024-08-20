use glam::Vec2;
use nannou::{
  color::*,
  geom::pt2,
  prelude::{App, Frame, Update},
};

use map_viewer::{
  entity_constructor::EntityConstructor, glam_to_geom, Draw, Entity,
};
use shared::FEET_PER_UNIT;

fn main() {
  nannou::app(model).update(update).simple_window(view).run();
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Model {
  entity_constructor: EntityConstructor,
}

fn model(_app: &App) -> Model {
  let parsed_entities: Vec<Entity> =
    ron::de::from_bytes(include_bytes!("../../airport.ron")).unwrap();

  let mut entity_constructor = EntityConstructor::new();
  for entity in parsed_entities.into_iter() {
    entity_constructor.add_entity(entity)
  }

  println!("{:#?}", entity_constructor);
  Model { entity_constructor }
}
fn update(_app: &App, _model: &mut Model, _update: Update) {}

fn view(app: &App, model: &Model, frame: Frame) {
  let draw = app.draw();

  draw.background().color(BLACK);

  model
    .entity_constructor
    .taxiways
    .iter()
    .for_each(|taxiway| {
      taxiway.draw(&draw, 0.05);
    });

  draw.to_frame(app, &frame).unwrap();
}
