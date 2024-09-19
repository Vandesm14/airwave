use internment::Intern;

use crate::{command::Task, engine::Bundle};

use super::{actions::ActionKind, Action, Aircraft, AircraftState};

#[derive(Debug, Clone, PartialEq)]
pub enum EventKind {
  TargetSpeed(f32),
  TargetHeading(f32),
  TargetAltitude(f32),

  Land(Intern<String>),
  GoAround,
}

impl From<Task> for EventKind {
  fn from(value: Task) -> Self {
    match value {
      Task::Altitude(x) => EventKind::TargetAltitude(x),
      Task::Approach(_) => todo!(),
      Task::Arrival(_) => todo!(),
      Task::Clearance { .. } => todo!(),
      Task::Depart(_) => todo!(),
      Task::Direct(_) => todo!(),
      Task::DirectionOfTravel => todo!(),
      Task::Frequency(_) => todo!(),
      Task::GoAround => EventKind::GoAround,
      Task::Heading(x) => EventKind::TargetHeading(x),
      Task::Ident => todo!(),
      Task::Land(x) => EventKind::Land(Intern::from(x)),
      Task::NamedFrequency(_) => todo!(),
      Task::ResumeOwnNavigation => todo!(),
      Task::Speed(x) => EventKind::TargetSpeed(x),
      Task::Takeoff(_) => todo!(),
      Task::Taxi(_) => todo!(),
      Task::TaxiContinue => todo!(),
      Task::TaxiHold => todo!(),
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Event {
  pub id: Intern<String>,
  pub kind: EventKind,
}

impl Event {
  pub fn new(id: Intern<String>, kind: EventKind) -> Self {
    Self { id, kind }
  }
}

pub trait AircraftEventHandler {
  fn run(aircraft: &Aircraft, event: &EventKind, bundle: &mut Bundle);
}

pub struct HandleAircraftEvent;
impl AircraftEventHandler for HandleAircraftEvent {
  fn run(aircraft: &Aircraft, event: &EventKind, bundle: &mut Bundle) {
    match event {
      EventKind::TargetSpeed(speed) => {
        bundle
          .actions
          .push(Action::new(aircraft.id, ActionKind::TargetSpeed(*speed)));
      }
      EventKind::TargetHeading(heading) => {
        bundle.actions.push(Action::new(
          aircraft.id,
          ActionKind::TargetHeading(*heading),
        ));
      }
      EventKind::TargetAltitude(altitude) => {
        bundle.actions.push(Action::new(
          aircraft.id,
          ActionKind::TargetAltitude(*altitude),
        ));
      }

      EventKind::Land(runway) => handle_land_event(aircraft, bundle, *runway),
      EventKind::GoAround => {
        if let AircraftState::Landing(..) = aircraft.state {
          bundle
            .actions
            .push(Action::new(aircraft.id, ActionKind::Flying))
        }
      }
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
        bundle
          .actions
          .push(Action::new(aircraft.id, ActionKind::Land(runway.clone())));
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
