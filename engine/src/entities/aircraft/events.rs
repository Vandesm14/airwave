use super::{Action, Aircraft, Bundle, Event};

pub trait AircraftEventHandler {
  fn run(aircraft: &Aircraft, event: &Event, bundle: &mut Bundle);
}

pub struct HandleAircraftEvent;
impl AircraftEventHandler for HandleAircraftEvent {
  fn run(a: &Aircraft, event: &Event, bundle: &mut Bundle) {
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
      Event::Land(runway) => handle_land_event(a, bundle, runway),
    }
  }
}

pub fn handle_land_event<R: AsRef<str>>(
  aircraft: &Aircraft,
  bundle: &mut Bundle,
  runway_name: R,
) {
  let name = runway_name.as_ref();
  println!("Cleared to land on runway: {name}");
}
