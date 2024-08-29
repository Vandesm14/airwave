use engine::structs::Runway;
use glam::Vec2;
use nannou::{
  color::*,
  prelude::{App, Frame, Update},
};

use map_viewer::{Airport, Draw};
use nannou_egui::{egui, Egui};

fn main() {
  nannou::app(model).update(update).run();
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Settings {
  pub scale: f32,
  pub pos: f32,
}

impl Settings {
  pub fn new() -> Self {
    Self {
      scale: 1.0,
      pos: 0.0,
    }
  }
}

pub struct Model {
  settings: Settings,
  airport: Airport,
  egui: Egui,
}

fn model(app: &App) -> Model {
  let airport = Airport {
    taxiways: vec![],
    runways: vec![Runway {
      id: "".into(),
      pos: Vec2::new(0.0, 0.0),
      heading: 0.0,
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

  Model {
    settings: Settings::new(),
    airport,
    egui,
  }
}

fn update(_app: &App, model: &mut Model, update: Update) {
  let egui = &mut model.egui;

  egui.set_elapsed_time(update.since_start);
  let ctx = egui.begin_frame();

  egui::SidePanel::new(egui::panel::Side::Left, "Settings").show(&ctx, |ui| {
    ui.label("Scale:");
    ui.add(egui::widgets::DragValue::new(&mut model.settings.scale));

    ui.collapsing("Objects", |ui| {
      ui.collapsing("Runways", |ui| {
        model.airport.runways.iter_mut().for_each(|runway| {
          ui.label("Runway:");
          ui.add(egui::widgets::TextEdit::singleline(&mut runway.id));
          ui.label("X:");
          ui.add(egui::widgets::DragValue::new(&mut runway.pos.x));
          ui.label("Y:");
          ui.add(egui::widgets::DragValue::new(&mut runway.pos.y));
          ui.label("Heading:");
          ui.add(egui::widgets::DragValue::new(&mut runway.heading));
        });
      });
    });
  });
}

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
  let scale = model.settings.scale;

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
  model.egui.draw_to_frame(&frame).unwrap();
}
