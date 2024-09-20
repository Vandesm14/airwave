
use actions::{Action, AircraftActionHandler};
use effects::{
  AircraftUpdateAirspaceEffect, AircraftUpdateFlyingEffect,
  AircraftUpdateLandingEffect, AircraftUpdateTaxiingEffect,
};
use events::Event;

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

  pub dt: f32,
}

impl<'a> From<&'a World> for Bundle<'a> {
  fn from(value: &'a World) -> Self {
    Self {
      prev: Aircraft::default(),
      events: Vec::new(),
      actions: Vec::new(),
      airspaces: &value.airspaces,
      waypoints: &value.waypoints,
      waypoint_sets: &value.waypoint_sets,
      dt: 0.0,
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
    dt: f32,
  ) -> Vec<Event> {
    // let id_set: HashSet<Intern<String>> =
    //   HashSet::from_iter(self.events.iter().map(|e| e.id));
    let mut bundle = Bundle::from(world);
    bundle.dt = dt;

    tracing::debug!("tick events: {:?}", self.events);
    for aircraft in aircraft.iter_mut() {
      // Capture the previous state
      bundle.prev = aircraft.clone();

      // Run through all events
      for event in self.events.iter() {
        if event.id == aircraft.id {
          HandleAircraftEvent::run(aircraft, &event.kind, &mut bundle);

          // Apply all actions
          if !bundle.actions.is_empty() {
            tracing::debug!("event: {event:?} {:?}", &bundle.actions);
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
      tracing::debug!("state effects: {:?}", &bundle.actions);
      for action in bundle.actions.iter() {
        if action.id == aircraft.id {
          AircraftAllActionHandler::run(aircraft, &action.kind);
        }
      }
      bundle.actions.clear();

      AircraftUpdateFromTargetsEffect::run(aircraft, &mut bundle);
      AircraftUpdatePositionEffect::run(aircraft, &mut bundle);
      AircraftUpdateAirspaceEffect::run(aircraft, &mut bundle);

      // Apply all actions
      for action in bundle.actions.iter() {
        if action.id == aircraft.id {
          AircraftAllActionHandler::run(aircraft, &action.kind);
        }
      }
      bundle.actions.clear();
    }

    // Capture the left over events and actions for next time
    tracing::debug!("new events: {:?}", bundle.events);
    self.events = core::mem::take(&mut bundle.events);

    self.events.clone()
  }
}
