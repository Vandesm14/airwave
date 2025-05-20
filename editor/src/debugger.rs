use editor::draw::Draw;
use engine::{
  AIRSPACE_RADIUS, APPROACH_ALTITUDE,
  assets::load_assets,
  engine::Engine,
  entities::{
    aircraft::{
      Aircraft, AircraftState, FlightPlan, LandingState, TaxiingState,
      events::{AircraftEvent, EventKind},
    },
    world::AirportStatus,
  },
  geometry::move_point,
  pathfinder::{Node, NodeBehavior, NodeKind},
};
use internment::Intern;
use nannou::prelude::*;
use nannou_egui::{
  Egui,
  egui::{self, Id, Widget},
};

pub fn main() {
  nannou::app(model).update(update).run();
}

struct HeldKeys {
  ctrl: bool,
  shift: bool,
  alt: bool,
}

struct Model {
  engine: Engine,
  egui: Egui,

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

  let mut engine = Engine {
    airports: load_assets().airports,
    ..Default::default()
  };

  let airport = engine.airports.get("ksfo").unwrap().clone();

  // let aircraft = Aircraft {
  //   id: Intern::from_ref("AAL1234"),
  //   pos: move_point(glam::Vec2::ZERO, 45.0, AIRSPACE_RADIUS),
  //   speed: 250.0,
  //   heading: 270.0,
  //   altitude: APPROACH_ALTITUDE,
  //   state: AircraftState::Flying,
  //   flight_plan: FlightPlan::new(
  //     Intern::from_ref("KDEF"),
  //     Intern::from_ref("KSFO"),
  //   ),
  //   flight_time: Some(0),
  //   ..Default::default()
  // }
  // .with_synced_targets();

  let gate = airport
    .terminals
    .iter()
    .flat_map(|t| t.gates.iter())
    .next()
    .unwrap();
  let aircraft = Aircraft {
    id: Intern::from_ref("AAL1234"),
    pos: gate.pos,
    speed: 0.0,
    heading: gate.heading,
    altitude: 0.0,
    state: AircraftState::Parked {
      at: Node::default().with_name(gate.id).with_kind(NodeKind::Gate),
    },
    flight_plan: FlightPlan::new(
      Intern::from_ref("KDEF"),
      Intern::from_ref("KSFO"),
    ),
    flight_time: None,
    ..Default::default()
  }
  .with_synced_targets();

  engine.game.aircraft.push(aircraft);
  engine
    .world
    .airport_statuses
    .insert(airport.id, AirportStatus::all_auto());
  engine.world.airports.push(airport);

  let mut snapshots = Vec::new();
  for i in 0..engine.tick_rate_tps * 60 * 20 {
    if i == 15 {
      engine.events.push(
        AircraftEvent::new(
          Intern::from_ref("AAL1234"),
          EventKind::Taxi(vec![
            Node::new(
              Intern::from_ref("A"),
              NodeKind::Taxiway,
              NodeBehavior::GoTo,
              (),
            ),
            Node::new(
              Intern::from_ref("E"),
              NodeKind::Taxiway,
              NodeBehavior::GoTo,
              (),
            ),
            Node::new(
              Intern::from_ref("G"),
              NodeKind::Taxiway,
              NodeBehavior::GoTo,
              (),
            ),
          ]),
        )
        .into(),
      );
    }

    if i % engine.tick_rate_tps == 0 {
      snapshots.push(engine.game.aircraft.clone());
    }

    engine.tick();
  }

  println!("Ready.");

  Model {
    engine,
    egui,

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

  model.engine.world.draw(&draw, model.scale, model.shift_pos);
  if let Some(snapshot) = model.snapshots.get(model.snapshot_index) {
    for aircraft in snapshot.iter() {
      aircraft.draw(&draw, model.scale, model.shift_pos);
    }
  }

  draw.to_frame(app, &frame).unwrap();
  model.egui.draw_to_frame(&frame).unwrap();
}
