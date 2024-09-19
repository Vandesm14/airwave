use super::{Action, Aircraft, AircraftState};

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
