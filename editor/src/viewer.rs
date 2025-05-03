use std::sync::{mpsc, Mutex, OnceLock};

use engine::entities::airport::Airport;
use nannou::prelude::*;
use nannou_egui::Egui;

use crate::Draw;

static GLOBAL_CHANNEL: OnceLock<Mutex<mpsc::Receiver<Airport>>> =
  OnceLock::new();

pub fn start_app(channel: mpsc::Receiver<Airport>) {
  GLOBAL_CHANNEL.set(Mutex::new(channel)).unwrap();

  nannou::app(model).update(update).run();
}

struct HeldKeys {
  ctrl: bool,
  shift: bool,
  alt: bool,
}

struct Model {
  egui: Egui,
  airport: Airport,

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

  Model {
    egui,
    airport: Airport::default(),

    is_mouse_down: false,
    is_over_ui: false,

    drag_anchor: None,
    old_shift_pos: glam::Vec2::default(),
    shift_pos: glam::Vec2::default(),
    scale: 0.1,

    held_keys: HeldKeys {
      ctrl: false,
      shift: false,
      alt: false,
    },
  }
}

fn update(_app: &App, model: &mut Model, update: Update) {
  model.egui.set_elapsed_time(update.since_start);
  let _ = model.egui.begin_frame();

  if let Some(mutex) = GLOBAL_CHANNEL.get() {
    if let Ok(chan) = mutex.lock() {
      while let Ok(airport) = chan.try_recv() {
        model.airport = airport;
        model.airport.calculate_waypoints();
      }
    }
  }
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
    if *y < 0.0 {
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
      model.drag_anchor = Some(pos);
      model.old_shift_pos = model.shift_pos;
    }
  } else if let nannou::winit::event::WindowEvent::MouseInput {
    state: nannou::winit::event::ElementState::Released,
    button: nannou::winit::event::MouseButton::Left,
    ..
  } = event
  {
    model.is_mouse_down = false;
    model.drag_anchor = None;
  }

  // Detect mouse move
  if let nannou::winit::event::WindowEvent::CursorMoved { .. } = event {
    let pos = real_mouse_pos(app, model);
    if model.is_mouse_down {
      if let Some(drag_anchor) = model.drag_anchor {
        model.shift_pos =
          model.old_shift_pos + (pos - drag_anchor) / model.scale;
      }
    }
  }
}

fn view(app: &App, model: &Model, frame: Frame) {
  let draw = app.draw();
  draw.background().color(BLACK);

  model.airport.draw(&draw, model.scale, model.shift_pos);

  draw.to_frame(app, &frame).unwrap();
  model.egui.draw_to_frame(&frame).unwrap();
}
