use actions::AircraftActionHandler;

use crate::entities::aircraft::{
  actions::AircraftAllActionHandler,
  effects::{
    AircraftEffect, AircraftUpdateFromTargetsEffect,
    AircraftUpdatePositionEffect,
  },
  events::{AircraftEventHandler, HandleAircraftEvent},
  *,
};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Engine {
  pub aircraft: Vec<Aircraft>,
  pub events: Vec<Event>,
  pub actions: Vec<Action>,
}

impl Engine {
  pub fn tick(&mut self) {
    let mut bundle = Bundle {
      dt: 0.5,
      ..Default::default()
    };

    for aircraft in self.aircraft.iter_mut() {
      for event in self.events.iter() {
        HandleAircraftEvent::run(aircraft, event, &mut bundle);

        // Apply all actions after each event
        for action in bundle.actions.drain(..) {
          AircraftAllActionHandler::run(aircraft, &action);
        }
      }

      AircraftUpdateFromTargetsEffect::run(aircraft, &mut bundle);
      for action in bundle.actions.drain(..) {
        AircraftAllActionHandler::run(aircraft, &action);
      }

      AircraftUpdatePositionEffect::run(aircraft, &mut bundle);
      for action in bundle.actions.drain(..) {
        AircraftAllActionHandler::run(aircraft, &action);
      }
    }

    self.events.clear();
    self.actions.clear();
  }
}
