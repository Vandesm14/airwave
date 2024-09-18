use glam::Vec2;

use crate::{
  delta_angle, move_point, normalize_angle, KNOT_TO_FEET_PER_SECOND,
};

#[derive(Debug, Clone, PartialEq)]

pub enum Event {
  TargetSpeed(f32),
  TargetHeading(f32),
  TargetAltitude(f32),
}

#[derive(Debug, Clone, PartialEq)]

pub enum Action {
  Pos(Vec2),

  Speed(f32),
  Heading(f32),
  Altitude(f32),

  TargetSpeed(f32),
  TargetHeading(f32),
  TargetAltitude(f32),
}

#[derive(Debug, Default)]
pub struct Bundle {
  pub events: Vec<Event>,
  pub actions: Vec<Action>,
  pub dt: f32,
}

pub trait AircraftEventHandler {
  fn run(aircraft: &Aircraft, event: &Event, bundle: &mut Bundle);
}
pub trait AircraftEffect {
  fn run(aircraft: &Aircraft, bundle: &mut Bundle);
}
pub trait AircraftActionHandler {
  fn run(aircraft: &mut Aircraft, action: &Action);
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct AircraftTargets {
  pub heading: f32,
  pub speed: f32,
  pub altitude: f32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Aircraft {
  pub pos: Vec2,
  pub speed: f32,
  pub heading: f32,
  pub altitude: f32,

  pub target: AircraftTargets,
}

// Consts
impl Aircraft {
  pub fn speed_in_feet(&self) -> f32 {
    self.speed * KNOT_TO_FEET_PER_SECOND
  }

  pub fn dt_climb_sp(&self, dt: f32) -> f32 {
    (2000.0_f32 / 60.0_f32).round() * dt
  }

  pub fn dt_turn_speed(&self, dt: f32) -> f32 {
    2.0 * dt
  }

  pub fn dt_speed_speed(&self, dt: f32) -> f32 {
    2.0 * dt
  }
}

// Event Handlers
pub struct HandleAircraftEvent;
impl AircraftEventHandler for HandleAircraftEvent {
  fn run(_: &Aircraft, event: &Event, bundle: &mut Bundle) {
    match event {
      Event::TargetSpeed(speed) => {
        bundle.actions.push(Action::TargetSpeed(*speed));
      }
      Event::TargetHeading(heading) => {
        bundle.actions.push(Action::TargetHeading(*heading));
      }
      Event::TargetAltitude(altitude) => {
        bundle.actions.push(Action::TargetAltitude(*altitude));
      }
    }
  }
}

// Effects
pub struct AircraftUpdateFromTargetsEffect;
impl AircraftEffect for AircraftUpdateFromTargetsEffect {
  fn run(aircraft: &Aircraft, bundle: &mut Bundle) {
    // In feet per second
    let climb_speed = aircraft.dt_climb_sp(bundle.dt);
    // In degrees per second
    let turn_speed = aircraft.dt_turn_speed(bundle.dt);
    // In knots per second
    let speed_speed = aircraft.dt_speed_speed(bundle.dt);

    if (aircraft.altitude - aircraft.target.altitude).abs() < climb_speed {
      bundle
        .actions
        .push(Action::Altitude(aircraft.target.altitude));
    }
    if (aircraft.heading - aircraft.target.heading).abs() < turn_speed {
      bundle
        .actions
        .push(Action::Heading(normalize_angle(aircraft.target.heading)));
    }
    if (aircraft.speed - aircraft.target.speed).abs() < speed_speed {
      bundle.actions.push(Action::Speed(aircraft.target.speed));
    }

    // Change based on speed if not equal
    if aircraft.altitude != aircraft.target.altitude {
      if aircraft.altitude < aircraft.target.altitude {
        bundle
          .actions
          .push(Action::Altitude(aircraft.altitude + climb_speed));
      } else {
        bundle
          .actions
          .push(Action::Altitude(aircraft.altitude - climb_speed));
      }
    }
    if aircraft.heading != aircraft.target.heading {
      let delta_angle = delta_angle(aircraft.heading, aircraft.target.heading);
      if delta_angle < 0.0 {
        // aircraft.heading -= turn_speed;
      } else {
        bundle.actions.push(Action::Heading(normalize_angle(
          aircraft.heading + turn_speed,
        )));
      }
    }
    if aircraft.speed != aircraft.target.speed {
      if aircraft.speed < aircraft.target.speed {
        bundle
          .actions
          .push(Action::Speed(aircraft.speed + speed_speed));
      } else {
        bundle
          .actions
          .push(Action::Speed(aircraft.speed - speed_speed));
      }
    }
  }
}

pub struct AircraftUpdatePositionEffect;
impl AircraftEffect for AircraftUpdatePositionEffect {
  fn run(aircraft: &Aircraft, bundle: &mut Bundle) {
    let pos = move_point(
      aircraft.pos,
      aircraft.heading,
      aircraft.speed_in_feet() * bundle.dt,
    );
    bundle.actions.push(Action::Pos(pos));
  }
}

// Action Handlers
pub struct AircraftAllActionHandler;
impl AircraftActionHandler for AircraftAllActionHandler {
  fn run(aircraft: &mut Aircraft, action: &Action) {
    match action {
      Action::TargetSpeed(speed) => aircraft.target.speed = *speed,
      Action::TargetHeading(heading) => aircraft.target.heading = *heading,
      Action::TargetAltitude(altitude) => aircraft.target.altitude = *altitude,

      Action::Speed(speed) => aircraft.speed = *speed,
      Action::Heading(heading) => aircraft.heading = *heading,
      Action::Altitude(altitude) => aircraft.altitude = *altitude,

      Action::Pos(pos) => aircraft.pos = *pos,
    }
  }
}
