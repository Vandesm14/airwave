use glam::Vec2;
use internment::Intern;

use crate::entities::airport::Runway;

use super::{Aircraft, AircraftState};

#[derive(Debug, Clone, PartialEq)]

pub enum ActionKind {
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
  Flying,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Action {
  pub id: Intern<String>,
  pub kind: ActionKind,
}

impl Action {
  pub fn new(id: Intern<String>, kind: ActionKind) -> Self {
    Self { id, kind }
  }
}

pub trait AircraftActionHandler {
  fn run(aircraft: &mut Aircraft, action: &ActionKind);
}

pub struct AircraftAllActionHandler;
impl AircraftActionHandler for AircraftAllActionHandler {
  fn run(aircraft: &mut Aircraft, action: &ActionKind) {
    match action {
      ActionKind::Pos(pos) => aircraft.pos = *pos,

      ActionKind::TargetSpeed(speed) => aircraft.target.speed = *speed,
      ActionKind::TargetHeading(heading) => aircraft.target.heading = *heading,
      ActionKind::TargetAltitude(altitude) => {
        aircraft.target.altitude = *altitude
      }

      ActionKind::Speed(speed) => aircraft.speed = *speed,
      ActionKind::Heading(heading) => aircraft.heading = *heading,
      ActionKind::Altitude(altitude) => aircraft.altitude = *altitude,

      ActionKind::Airspace(spur) => aircraft.airspace = *spur,

      ActionKind::Land(runway) => {
        aircraft.state = AircraftState::Landing(runway.clone())
      }
      ActionKind::Flying => {
        aircraft.state = AircraftState::Flying {
          waypoints: Vec::new(),
        }
      }
    }
  }
}
