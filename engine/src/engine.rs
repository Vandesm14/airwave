use actions::{Action, AircraftActionHandler};
use effects::{
  AircraftContactCenterEffect, AircraftContactClearanceEffect,
  AircraftIsNowParkedEffect, AircraftUpdateAirspaceEffect,
  AircraftUpdateFlyingEffect, AircraftUpdateLandingEffect,
  AircraftUpdateTaxiingEffect,
};
use events::Event;
use turborand::rng::Rng;

use crate::{
  entities::{
    aircraft::{
      actions::AircraftAllActionHandler,
      effects::{
        AircraftEffect, AircraftUpdateFromTargetsEffect,
        AircraftUpdatePositionEffect,
      },
      events::{AircraftEventHandler, HandleAircraftEvent},
      *,
    },
    airspace::Airspace,
    world::{WaypointSet, World},
  },
  pathfinder::{Node, NodeVORData},
};

#[derive(Debug)]
pub struct Bundle<'a> {
  pub prev: Aircraft,

  pub events: Vec<Event>,
  pub actions: Vec<Action>,

  pub airspaces: &'a [Airspace],
  pub waypoints: &'a [Node<NodeVORData>],
  pub waypoint_sets: &'a WaypointSet,

  pub rng: &'a mut Rng,
  pub dt: f32,
}

impl<'a> Bundle<'a> {
  pub fn from_world(world: &'a World, rng: &'a mut Rng, dt: f32) -> Self {
    Self {
      prev: Aircraft::default(),
      events: Vec::new(),
      actions: Vec::new(),
      airspaces: &world.airspaces,
      waypoints: &world.waypoints,
      waypoint_sets: &world.waypoint_sets,
      rng,
      dt,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Engine {
  pub events: Vec<Event>,
}

impl Engine {
  pub fn tick(
    &mut self,
    world: &World,
    aircraft: &mut [Aircraft],
    rng: &mut Rng,
    dt: f32,
  ) -> Vec<Event> {
    // let id_set: HashSet<Intern<String>> =
    //   HashSet::from_iter(self.events.iter().map(|e| e.id));
    let mut bundle = Bundle::from_world(world, rng, dt);

    if !self.events.is_empty() {
      tracing::trace!("tick events: {:?}", self.events);
    }
    for aircraft in aircraft.iter_mut() {
      // Capture the previous state
      bundle.prev = aircraft.clone();

      // Run through all events
      for event in self.events.iter() {
        if event.id == aircraft.id {
          HandleAircraftEvent::run(aircraft, &event.kind, &mut bundle);

          // Apply all actions
          if !bundle.actions.is_empty() {
            tracing::trace!("event: {event:?} {:?}", &bundle.actions);
          }
          for action in bundle.actions.iter() {
            if action.id == aircraft.id {
              AircraftAllActionHandler::run(aircraft, &action.kind);
            }
          }
          bundle.actions.clear();
        }
      }

      // Run through all effects
      AircraftUpdateLandingEffect::run(aircraft, &mut bundle);
      AircraftUpdateFlyingEffect::run(aircraft, &mut bundle);
      AircraftUpdateTaxiingEffect::run(aircraft, &mut bundle);

      // Apply all actions
      if !bundle.actions.is_empty() {
        tracing::trace!("state actions: {:?}", &bundle.actions);
      }
      for action in bundle.actions.iter() {
        if action.id == aircraft.id {
          AircraftAllActionHandler::run(aircraft, &action.kind);
        }
      }
      bundle.actions.clear();

      AircraftUpdateFromTargetsEffect::run(aircraft, &mut bundle);

      // Apply all actions
      if !bundle.actions.is_empty() {
        tracing::trace!("target actions: {:?}", &bundle.actions);
      }
      for action in bundle.actions.iter() {
        if action.id == aircraft.id {
          AircraftAllActionHandler::run(aircraft, &action.kind);
        }
      }
      bundle.actions.clear();

      AircraftUpdatePositionEffect::run(aircraft, &mut bundle);

      // Apply all actions
      if !bundle.actions.is_empty() {
        tracing::trace!("position actions: {:?}", &bundle.actions);
      }
      for action in bundle.actions.iter() {
        if action.id == aircraft.id {
          AircraftAllActionHandler::run(aircraft, &action.kind);
        }
      }
      bundle.actions.clear();

      AircraftUpdateAirspaceEffect::run(aircraft, &mut bundle);

      // Apply all actions
      if !bundle.actions.is_empty() {
        tracing::trace!("airspace actions: {:?}", &bundle.actions);
      }
      for action in bundle.actions.iter() {
        if action.id == aircraft.id {
          AircraftAllActionHandler::run(aircraft, &action.kind);
        }
      }
      bundle.actions.clear();

      AircraftIsNowParkedEffect::run(aircraft, &mut bundle);
      AircraftContactCenterEffect::run(aircraft, &mut bundle);
      AircraftContactClearanceEffect::run(aircraft, &mut bundle);

      // Apply all actions
      if !bundle.actions.is_empty() {
        tracing::trace!("other actions: {:?}", &bundle.actions);
      }
      for action in bundle.actions.iter() {
        if action.id == aircraft.id {
          AircraftAllActionHandler::run(aircraft, &action.kind);
        }
      }
      bundle.actions.clear();
    }

    // Capture the left over events and actions for next time
    if !bundle.events.is_empty() {
      tracing::trace!("new events: {:?}", bundle.events);
    }
    self.events = core::mem::take(&mut bundle.events);

    self.events.clone()
  }
}
