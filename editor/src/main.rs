use std::{ops::Div, path::PathBuf};

use clap::Parser;
use editor::{geom_to_glam, WorldFile};
use nannou::{event::KeyboardInput, prelude::*};
use nannou_egui::{
  egui::{self, Id},
  Egui,
};

/// View and edit an Airwave world file
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
  /// The file to load
  file: PathBuf,
}

fn main() {
  nannou::app(model).update(update).run();
}

enum PointMode {
  Add,
  Remove,
  Select,
}

struct Model {
  egui: Egui,

  path: PathBuf,
  world_file: WorldFile,

  mode: PointMode,
  selected: Option<usize>,
  is_mouse_down: bool,
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

  let args = Cli::parse();
  let world_file = if let Ok(world_file) = std::fs::read_to_string(&args.file) {
    ron::from_str(&world_file).unwrap()
  } else {
    WorldFile::default()
  };

  Model {
    egui,

    path: args.file,
    world_file,

    mode: PointMode::Add,
    selected: None,
    is_mouse_down: false,
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

fn real_mouse_pos(app: &App, model: &Model) -> glam::Vec2 {
  let size = app.main_window().inner_size_points();
  let size = Vec2::new(size.0, size.1);
  let half_size = size / 2.0;

  let pos = model.egui.input().pointer_pos;

  glam::Vec2::new(pos.x - half_size.x, -pos.y + half_size.y)
}

fn raw_window_event(
  app: &App,
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
    let pos = real_mouse_pos(app, model);
    let closest = model.world_file.find_closest_point(pos, 100.0);
    match model.mode {
      PointMode::Add => {
        model.world_file.points.push(pos);
      }
      PointMode::Remove => {
        if let Some(closest) = closest {
          model.world_file.points.remove(closest);
        }
      }
      PointMode::Select => {
        model.selected = closest;

        if closest.is_some() {
          model.is_mouse_down = true;
        }
      }
    }
  } else if let nannou::winit::event::WindowEvent::MouseInput {
    state: nannou::winit::event::ElementState::Released,
    button: nannou::winit::event::MouseButton::Left,
    ..
  } = event
  {
    model.is_mouse_down = false;
  }

  // Detect mouse move
  if let nannou::winit::event::WindowEvent::CursorMoved { .. } = event {
    let pos = real_mouse_pos(app, model);

    if model.is_mouse_down {
      if let Some(point) = model
        .selected
        .and_then(|s| model.world_file.points.get_mut(s))
      {
        *point = pos;
      }
    }
  }

  // Detect Keyboard input
  if let nannou::winit::event::WindowEvent::KeyboardInput {
    input:
      KeyboardInput {
        state,
        virtual_keycode,
        modifiers,
        ..
      },
    ..
  } = event
  {
    // If Ctrl+S is pressed, save the world file
    if let (
      Some(nannou::winit::event::VirtualKeyCode::S),
      nannou::winit::event::ElementState::Pressed,
    ) = (virtual_keycode, state)
    {
      if modifiers.ctrl() {
        if let Ok(world_file) = ron::to_string(&model.world_file) {
          std::fs::write(model.path.clone(), world_file).unwrap();
        }
      }
    }

    match virtual_keycode {
      Some(nannou::winit::event::VirtualKeyCode::A) => {
        model.mode = PointMode::Add;
      }
      Some(nannou::winit::event::VirtualKeyCode::D) => {
        model.mode = PointMode::Remove;
      }
      Some(nannou::winit::event::VirtualKeyCode::S) => {
        model.mode = PointMode::Select;
      }

      _ => {}
    }
  }
}

fn view(app: &App, model: &Model, frame: Frame) {
  let world_file = &model.world_file;

  let draw = app.draw();
  draw.background().color(BLACK);

  let pos = real_mouse_pos(app, model);
  let closest = world_file.find_closest_point(pos, 100.0);

  for (i, point) in world_file.points.iter().enumerate() {
    let color = if Some(i) == closest { RED } else { WHITE };
    let color = if Some(i) == model.selected {
      GREEN
    } else {
      color
    };

    draw
      .ellipse()
      .x_y(point.x, point.y)
      .w_h(10.0, 10.0)
      .color(color);
  }

  // Draw mode at the bottom of the screen
  let mode = match model.mode {
    PointMode::Add => "Add",
    PointMode::Remove => "Remove",
    PointMode::Select => "Select",
  };
  let size = app.main_window().inner_size_points();
  draw
    .text(mode)
    .x_y(-size.0.div(4.0), -size.1.div(4.0))
    .color(WHITE)
    .font_size(20)
    .left_justify()
    .align_text_bottom();

  draw.to_frame(app, &frame).unwrap();
  model.egui.draw_to_frame(&frame).unwrap();
}
