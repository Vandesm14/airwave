use std::{ops::Div, path::PathBuf};

use clap::Parser;
use editor::{
  scale_point, unscale_point, Draw, MetaTaxiway, PointKey, WorldFile,
};
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

struct HeldKeys {
  ctrl: bool,
  shift: bool,
  alt: bool,
}

enum PointMode {
  Add,
  Remove,
  Select,
}

struct Model {
  egui: Egui,

  path: PathBuf,
  world_data: WorldFile,

  mode: PointMode,
  selected: Vec<PointKey>,
  is_mouse_down: bool,
  is_over_ui: bool,

  drag_anchor: Option<glam::Vec2>,
  old_shift_pos: glam::Vec2,
  shift_pos: glam::Vec2,
  scale: f32,

  held_keys: HeldKeys,
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
    world_data: world_file,

    mode: PointMode::Add,
    selected: Vec::new(),
    is_mouse_down: false,
    is_over_ui: false,

    drag_anchor: None,
    old_shift_pos: glam::Vec2::default(),
    shift_pos: glam::Vec2::default(),
    scale: 1.0,

    held_keys: HeldKeys {
      ctrl: false,
      shift: false,
      alt: false,
    },
  }
}

fn update(_app: &App, model: &mut Model, update: Update) {
  model.egui.set_elapsed_time(update.since_start);
  let ctx = model.egui.begin_frame();

  let side_panel = egui::SidePanel::new(
    egui::panel::Side::Left,
    Id::new("side_panel"),
  )
  .show(&ctx, |ui| {
    if ui.button("Add Taxiway").clicked() {
      if model.selected.len() == 2 {
        model.world_data.meta_airport.taxiways.push(MetaTaxiway {
          name: "New Taxiway".to_string(),
          a: model.selected[0],
          b: model.selected[1],
        });
        model.world_data.trigger_update();
      } else {
        // TODO: show toast
      }
    }
  });

  model.is_over_ui = side_panel.response.hovered();
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

  // Update Modifiers
  if let nannou::winit::event::WindowEvent::ModifiersChanged(modifiers) = event
  {
    model.held_keys.ctrl = modifiers.ctrl();
    model.held_keys.shift = modifiers.shift();
    model.held_keys.alt = modifiers.alt();
  }

  // Detect mouse wheel
  if let nannou::winit::event::WindowEvent::MouseWheel {
    delta: MouseScrollDelta::LineDelta(_, y),
    ..
  } = event
  {
    if *y > 0.0 {
      model.scale *= 0.9;
    } else {
      model.scale *= 1.1;
    }
  }

  // Detect mouse click
  if let nannou::winit::event::WindowEvent::MouseInput {
    state: nannou::winit::event::ElementState::Pressed,
    button: nannou::winit::event::MouseButton::Left,
    ..
  } = event
  {
    if !model.is_over_ui {
      model.is_mouse_down = true;

      let pos = real_mouse_pos(app, model);
      let scaled_pos = unscale_point(pos, model.shift_pos, model.scale);
      let closest = model.world_data.find_closest_point(scaled_pos, 100.0);
      match model.mode {
        PointMode::Add => {
          model.world_data.points.insert(scaled_pos);
          model.world_data.trigger_update();
        }
        PointMode::Remove => {
          if let Some(closest) = closest {
            model.world_data.points.remove(closest.0);
            model.world_data.trigger_update();
          }
        }
        PointMode::Select => {
          if let Some(closest) = closest {
            if !model.held_keys.shift {
              model.selected.clear();
            }
            model.selected.push(closest.0);
          } else {
            model.selected.clear();
            model.drag_anchor = Some(pos);
            model.old_shift_pos = model.shift_pos;
          }
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

    if model.drag_anchor.is_some() {
      model.drag_anchor = None;
    }
  }

  // Detect mouse move
  if let nannou::winit::event::WindowEvent::CursorMoved { .. } = event {
    let pos = real_mouse_pos(app, model);
    let scaled_pos = unscale_point(pos, model.shift_pos, model.scale);
    if model.is_mouse_down {
      if let Some(point) = model
        .selected
        .first()
        .and_then(|s| model.world_data.points.get_mut(*s))
      {
        *point = scaled_pos;
        model.world_data.trigger_update();
      } else if let Some(drag_anchor) = model.drag_anchor {
        model.shift_pos =
          model.old_shift_pos + (pos - drag_anchor) * model.scale;
      }
    }
  }

  // Detect Keyboard input
  if let nannou::winit::event::WindowEvent::KeyboardInput {
    input: KeyboardInput {
      state,
      virtual_keycode,
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
      if model.held_keys.ctrl {
        if let Ok(world_file) = ron::to_string(&model.world_data) {
          // This ensures that the airport is up-to-date before saving.
          model.world_data.trigger_update();
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
  let world_file = &model.world_data;

  let draw = app.draw();
  draw.background().color(BLACK);

  model.world_data.airport.draw(&draw, 1.0, model.shift_pos);

  let pos =
    unscale_point(real_mouse_pos(app, model), model.shift_pos, model.scale);
  let closest = world_file.find_closest_point(pos, 100.0);

  for point in world_file.points.iter() {
    let color = if Some(point.0) == closest.map(|c| c.0) {
      RED
    } else {
      WHITE
    };
    let color = if model.selected.contains(&point.0) {
      GREEN
    } else {
      color
    };

    let pos = scale_point(*point.1, model.shift_pos, model.scale);
    draw
      .ellipse()
      .x_y(pos.x, pos.y)
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
