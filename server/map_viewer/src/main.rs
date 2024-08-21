use std::fs;

use nannou::{
  color::*,
  prelude::{App, Frame, Update},
};

use map_viewer::{
  entity_constructor::EntityConstructor, Airport, Draw, Entity,
};

fn main() {
  nannou::app(model).update(update).simple_window(view).run();
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Model {
  airport: Airport,
}

fn model(_app: &App) -> Model {
  let parsed_entities: Vec<Entity> =
    ron::de::from_bytes(include_bytes!("../../airport.ron")).unwrap();

  fs::write(
    "../../airport.ron",
    ron::ser::to_string_pretty(
      &parsed_entities,
      ron::ser::PrettyConfig::default().struct_names(true),
    )
    .unwrap(),
  )
  .unwrap();

  let mut entity_constructor = EntityConstructor::new();
  for entity in parsed_entities.into_iter() {
    entity_constructor.add_entity(entity)
  }

  let mut airport = Airport::default();
  for taxiway in entity_constructor.taxiways.drain(..) {
    airport.add_taxiway(taxiway)
  }
  for runway in entity_constructor.runways.drain(..) {
    airport.add_runway(runway)
  }
  for terminal in entity_constructor.terminals.drain(..) {
    airport.add_terminal(terminal)
  }

  println!("{:#?}", entity_constructor);
  Model { airport }
}
fn update(_app: &App, _model: &mut Model, _update: Update) {}

fn view(app: &App, model: &Model, frame: Frame) {
  let draw = app.draw();

  draw.background().color(BLACK);

  model.airport.taxiways.iter().for_each(|taxiway| {
    taxiway.draw(&draw, 0.05);
  });

  model.airport.runways.iter().for_each(|taxiway| {
    taxiway.draw(&draw, 0.05);
  });

  // TODO: draw a scale for 1, 10, 100, and 1000 feet

  draw.to_frame(app, &frame).unwrap();
}
