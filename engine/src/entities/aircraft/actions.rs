use glam::Vec2;
use internment::Intern;

use crate::entities::airport::Runway;

use super::{Aircraft, AircraftState};

#[derive(Debug, Clone, PartialEq)]

pub enum Action {
  Pos(Vec2),

  Speed(f32),
  Heading(f32),
  Altitude(f32),

  TargetSpeed(f32),
  TargetHeading(f32),
  TargetAltitude(f32),

  Airspace(Intern<String>),

  // Substate
  Land(Runway),
}

pub trait AircraftActionHandler {
  fn run(aircraft: &mut Aircraft, action: &Action);
}

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

      Action::Airspace(spur) => aircraft.airspace = *spur,
      Action::Land(runway) => {
        aircraft.state = AircraftState::Landing(runway.clone())
      }
    }
  }
}
