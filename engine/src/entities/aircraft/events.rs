use internment::Intern;

use super::{Action, Aircraft, AircraftState, Bundle, Event};

pub trait AircraftEventHandler {
  fn run(aircraft: &Aircraft, event: &Event, bundle: &mut Bundle);
}

pub struct HandleAircraftEvent;
impl AircraftEventHandler for HandleAircraftEvent {
  fn run(aircraft: &Aircraft, event: &Event, bundle: &mut Bundle) {
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
      Event::Land(runway) => handle_land_event(aircraft, bundle, *runway),
    }
  }
}

pub fn handle_land_event(
  aircraft: &Aircraft,
  bundle: &mut Bundle,
  runway_id: Intern<String>,
) {
  if let AircraftState::Flying { .. } = aircraft.state {
    if let Some(airspace) =
      bundle.airspaces.iter().find(|a| a.id == aircraft.airspace)
    {
      if let Some(runway) = airspace
        .airports
        .iter()
        .flat_map(|a| a.runways.iter())
        .find(|r| r.id == runway_id)
      {
        println!("Landing on runway {}", runway.id);
        bundle.actions.push(Action::Land(runway.clone()));
      } else {
        eprintln!("No runway: {}", runway_id)
      }
    } else {
      eprintln!("No airspace: {}", aircraft.airspace)
    }
  } else {
    eprintln!("Not flying")
  }
}
