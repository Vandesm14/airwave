use engine::{structs::Runway, FEET_PER_UNIT, NAUTICALMILES_TO_FEET};
use glam::Vec2;
use nannou::{
  color::*,
  prelude::{App, Frame, Update},
};

use map_viewer::{Airport, Draw};

fn main() {
  nannou::app(model).update(update).simple_window(view).run();
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Model {
  airport: Airport,
}

fn model(_app: &App) -> Model {
  let padding = FEET_PER_UNIT * 7000.0;

  let airport = Airport {
    taxiways: vec![],
    runways: vec![Runway {
      id: "".into(),
      pos: Vec2::new(0.0, 0.0),
      heading: 090.0,
      length: 7000.0,
    }],
    terminals: vec![],
  };

  Model { airport }
}
fn update(app: &App, _model: &mut Model, _update: Update) {
  // find the cursor position and print it out
  let cursor = app.mouse.position();
}

fn view(app: &App, model: &Model, frame: Frame) {
  let draw = app.draw();

  draw.background().color(BLACK);
  let scale = 10.0;

  model.airport.taxiways.iter().for_each(|taxiway| {
    taxiway.draw(&draw, scale);
  });

  model.airport.terminals.iter().for_each(|terminal| {
    terminal.draw(&draw, scale);
  });

  model.airport.runways.iter().for_each(|taxiway| {
    taxiway.draw(&draw, scale);
  });

  // TODO: draw a scale for 1, 10, 100, and 1000 feet

  draw.to_frame(app, &frame).unwrap();
}
