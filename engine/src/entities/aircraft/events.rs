use super::{Action, Aircraft, Bundle, Event};

pub trait AircraftEventHandler {
  fn run(aircraft: &Aircraft, event: &Event, bundle: &mut Bundle);
}

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
      Event::Land(runway) => todo!("land event"),
    }
  }
}
