use engine::structs::Runway;
use glam::Vec2;
use nannou::{
  color::*,
  prelude::{App, Frame, Update},
};

use map_viewer::{Airport, Draw};
use nannou_egui::Egui;

fn main() {
  nannou::app(model).update(update).simple_window(view).run();
}

pub struct Model {
  airport: Airport,
  egui: Egui,
}

fn model(app: &App) -> Model {
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

  let window_id = app
    .new_window()
    .view(view)
    .raw_event(raw_window_event)
    .build()
    .unwrap();
  let window = app.window(window_id).unwrap();

  let egui = Egui::from_window(&window);

  Model { airport, egui }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {}

fn raw_window_event(
  _app: &App,
  model: &mut Model,
  event: &nannou::winit::event::WindowEvent,
) {
  // Let egui handle things like keyboard and mouse input.
  model.egui.handle_raw_event(event);
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
