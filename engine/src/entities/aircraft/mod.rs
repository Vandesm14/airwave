pub mod effects;
pub mod events;

use glam::Vec2;

use crate::KNOT_TO_FEET_PER_SECOND;

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