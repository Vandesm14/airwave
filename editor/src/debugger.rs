use std::collections::HashMap;

use editor::draw::Draw;
use engine::{
  AIRSPACE_RADIUS, APPROACH_ALTITUDE,
  assets::load_assets,
  engine::Engine,
  entities::{
    aircraft::{Aircraft, AircraftState, FlightPlan, LandingState},
    airport::Airport,
    airspace::Airspace,
    world::{Game, World},
  },
  geometry::move_point,
};
use internment::Intern;
use nannou::prelude::*;
use nannou_egui::{
  Egui,
  egui::{self, Id, Widget},
};
use turborand::rng::Rng;

pub fn main() {
  nannou::app(model).update(update).run();
}

struct Runner {
  tick_counter: usize,
  rate: usize,

  airports: HashMap<String, Airport>,
  world: World,
  game: Game,
  engine: Engine,

  rng: Rng,
}

impl Default for Runner {
  fn default() -> Self {
    let mut airports = HashMap::new();
    airports.insert("KSEA".to_string(), Airport::default());

    Self {
      tick_counter: 0,
      rate: 15,

      airports,
      world: World::default(),
      game: Game::default(),
      engine: Engine::default(),

      rng: Rng::new(),
    }
  }
}

struct HeldKeys {
  ctrl: bool,
  shift: bool,
  alt: bool,
}

struct Model {
  egui: Egui,
  runner: Runner,

  snapshots: Vec<Vec<Aircraft>>,
  snapshot_index: usize,

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

  let mut runner = Runner {
    airports: load_assets().airports,
    ..Default::default()
  };

  let mut airspace = Airspace {
    id: Intern::from_ref("KSFO"),
    pos: glam::Vec2::ZERO,
    radius: AIRSPACE_RADIUS,
    airports: vec![],
    auto: false,
  };

  let airport = runner.airports.get("ksfo").unwrap().clone();

  let aircraft = Aircraft {
    id: Intern::from_ref("AAL1234"),
    pos: move_point(glam::Vec2::ZERO, 45.0, AIRSPACE_RADIUS),
    speed: 250.0,
    heading: 270.0,
    altitude: APPROACH_ALTITUDE,
    state: AircraftState::Landing {
      runway: airport
        .runways
        .iter()
        .find(|r| r.id == Intern::from_ref("19L"))
        .unwrap()
        .clone(),
      state: LandingState::default(),
    },
    flight_plan: FlightPlan::new(
      Intern::from_ref("KSFO"),
      Intern::from_ref("KSFO"),
    ),
    flight_time: Some(0),
    ..Default::default()
  }
  .with_synced_targets();

  runner.game.aircraft.push(aircraft);
  airspace.airports.push(airport);
  runner.world.airspaces.push(airspace);

  let mut snapshots = Vec::new();
  for i in 0..runner.rate * 60 * 20 {
    if i % runner.rate == 0 {
      snapshots.push(runner.game.aircraft.clone());
    }

    let dt = 1.0 / runner.rate as f32;
    runner.engine.tick(
      &mut runner.world,
      &mut runner.game,
      &mut runner.rng,
      dt,
      runner.tick_counter,
    );
  }

  Model {
    egui,
    runner,

    snapshots,
    snapshot_index: 0,

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
  let ctx = model.egui.begin_frame();

  let side_panel =
    egui::panel::SidePanel::new(egui::panel::Side::Left, Id::new("side_panel"))
      .show(&ctx, |ui| {
        ui.label(format!("{:#?}", model.snapshots.get(model.snapshot_index)));
      });

  let bottom_panel = egui::panel::TopBottomPanel::new(
    egui::panel::TopBottomSide::Bottom,
    Id::new("bottom_panel"),
  )
  .show(&ctx, |ui| {
    ui.spacing_mut().slider_width = ui.available_width() - 60.0;
    egui::Slider::new(&mut model.snapshot_index, 0..=model.snapshots.len() - 1)
      .ui(ui);
  });

  model.is_over_ui =
    side_panel.response.hovered() || bottom_panel.response.hovered();
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

  model.runner.world.draw(&draw, model.scale, model.shift_pos);
  if let Some(snapshot) = model.snapshots.get(model.snapshot_index) {
    for aircraft in snapshot.iter() {
      aircraft.draw(&draw, model.scale, model.shift_pos);
    }
  }

  draw.to_frame(app, &frame).unwrap();
  model.egui.draw_to_frame(&frame).unwrap();
}
