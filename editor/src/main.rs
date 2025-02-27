use nannou::prelude::*;
use nannou_egui::{
  self,
  egui::{self, Id},
  Egui,
};
use serde::{Deserialize, Serialize};

fn main() {
  nannou::app(model).update(update).run();
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
struct WorldFile {
  points: Vec<Vec2>,
}

struct Model {
  world_file: WorldFile,
  egui: Egui,
}

fn model(app: &App) -> Model {
  // Create window
  let window_id = app
    .new_window()
    .view(view)
    .raw_event(raw_window_event)
    .build()
    .unwrap();
  let window = app.window(window_id).unwrap();

  let egui = Egui::from_window(&window);

  Model {
    egui,
    world_file: WorldFile::default(),
  }
}

fn update(_app: &App, model: &mut Model, update: Update) {
  let egui = &mut model.egui;
  let world_file = &mut model.world_file;

  egui.set_elapsed_time(update.since_start);
  let ctx = egui.begin_frame();

  egui::SidePanel::new(egui::panel::Side::Left, Id::new("side_panel"))
    .show(&ctx, |ui| {});
}

fn raw_window_event(
  _app: &App,
  model: &mut Model,
  event: &nannou::winit::event::WindowEvent,
) {
  // Let egui handle things like keyboard and mouse input.
  model.egui.handle_raw_event(event);

  // Detect mouse click
  if let nannou::winit::event::WindowEvent::MouseInput {
    state: nannou::winit::event::ElementState::Pressed,
    button: nannou::winit::event::MouseButton::Left,
    ..
  } = event
  {
    let pos = model.egui.input().pointer_pos;
    let size = _app.main_window().inner_size_points();
    let size = Vec2::new(size.0 as f32, size.1 as f32);
    let half_size = size / 2.0;

    model
      .world_file
      .points
      .push(Vec2::new(pos.x - half_size.x, -pos.y + half_size.y));
  }
}

fn view(app: &App, model: &Model, frame: Frame) {
  let world_file = &model.world_file;

  let draw = app.draw();
  draw.background().color(BLACK);

  for point in &world_file.points {
    draw
      .ellipse()
      .x_y(point.x, point.y)
      .w_h(10.0, 10.0)
      .rgba(0.0, 1.0, 0.0, 0.5);
  }

  draw.to_frame(app, &frame).unwrap();
  model.egui.draw_to_frame(&frame).unwrap();
}
