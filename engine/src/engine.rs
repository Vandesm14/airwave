use actions::AircraftActionHandler;
use effects::{AircraftIsPast205Effect, AircraftUpdateAirspaceEffect};

use crate::entities::{
  aircraft::{
    actions::AircraftAllActionHandler,
    effects::{
      AircraftEffect, AircraftUpdateFromTargetsEffect,
      AircraftUpdatePositionEffect,
    },
    events::{AircraftEventHandler, HandleAircraftEvent},
    *,
  },
  world::World,
};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Engine {
  pub events: Vec<Event>,
  pub actions: Vec<Action>,
}

impl Engine {
  pub fn tick(&mut self, world: &World, aircraft: &mut [Aircraft]) {
    let mut bundle = Bundle {
      dt: 0.5,
      airspaces: &world.airspaces,
      ..Default::default()
    };

    for aircraft in aircraft.iter_mut() {
      // Capture the previous state
      bundle.prev = aircraft.clone();

      // Run through all events
      for event in self.events.iter() {
        HandleAircraftEvent::run(aircraft, event, &mut bundle);
      }

      // Run through all effects
      AircraftUpdateFromTargetsEffect::run(aircraft, &mut bundle);
      AircraftUpdatePositionEffect::run(aircraft, &mut bundle);
      AircraftUpdateAirspaceEffect::run(aircraft, &mut bundle);
      AircraftIsPast205Effect::run(aircraft, &mut bundle);

      // Run through all actions
      for action in bundle.actions.drain(..) {
        AircraftAllActionHandler::run(aircraft, &action);
      }
    }

    // Capture the left over events and actions for next time
    self.events = core::mem::take(&mut bundle.events);
    self.actions = core::mem::take(&mut bundle.actions);
  }
}
