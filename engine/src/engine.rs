use std::collections::HashSet;

use actions::{Action, AircraftActionHandler};
use effects::{
  AircraftUpdateFlyingEffect, AircraftUpdateLandingEffect,
  AircraftUpdateTaxiingEffect,
};
use events::AircraftEvent;
use internment::Intern;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
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
    world::{Connection, Game, World},
  },
  NAUTICALMILES_TO_FEET,
};

#[derive(Debug)]
pub struct Bundle<'a> {
  pub prev: Aircraft,

  pub events: Vec<Event>,
  pub actions: Vec<Action>,

  pub airspace: &'a Airspace,
  pub connections: &'a Vec<Connection>,

  pub rng: &'a mut Rng,
  pub dt: f32,
}

impl<'a> Bundle<'a> {
  pub fn from_world(world: &'a World, rng: &'a mut Rng, dt: f32) -> Self {
    Self {
      prev: Aircraft::default(),
      events: Vec::new(),
      actions: Vec::new(),
      airspace: &world.airspace,
      connections: &world.connections,
      rng,
      dt,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type", content = "value")]
/// UI Commands come from the frontend and are handled within the engine.
pub enum UICommand {
  Purchase(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// UI Events are sent from the engine to the frontend.
pub enum UIEvent {
  // Inbound
  Purchase(usize),

  // Outbound
  Funds(usize),
}

impl From<UICommand> for UIEvent {
  fn from(value: UICommand) -> Self {
    match value {
      UICommand::Purchase(aircraft_id) => Self::Purchase(aircraft_id),
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
  Aircraft(AircraftEvent),
  UiEvent(UIEvent),
}

impl From<AircraftEvent> for Event {
  fn from(value: AircraftEvent) -> Self {
    Self::Aircraft(value)
  }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Engine {
  pub events: Vec<Event>,
}

impl Engine {
  pub fn handle_collisions(&mut self, aircrafts: &mut [Aircraft]) {
    let mut collisions: HashSet<Intern<String>> = HashSet::new();
    for pair in aircrafts.iter().combinations(2) {
      let aircraft = pair.first().unwrap();
      let other_aircraft = pair.last().unwrap();

      let distance = aircraft.pos.distance_squared(other_aircraft.pos);
      let vertical_distance =
        (aircraft.altitude - other_aircraft.altitude).abs();

      if aircraft.altitude > 1000.0
        && distance <= (NAUTICALMILES_TO_FEET * 2.0).powf(2.0)
        && vertical_distance < 1000.0
      {
        collisions.insert(aircraft.id);
        collisions.insert(other_aircraft.id);
      }
    }

    aircrafts.iter_mut().for_each(|aircraft| {
      let is_colliding = collisions.contains(&aircraft.id);

      // TODO: Fire collision events
      // if is_colliding && aircraft.is_colliding != is_colliding {
      //   self.events.push();
      // }

      aircraft.is_colliding = is_colliding;
    });
  }

  pub fn apply_actions(
    &self,
    bundle: &mut Bundle,
    aircraft: &mut Aircraft,
    name: Option<&str>,
  ) {
    if !bundle.actions.is_empty() {
      if let Some(name) = name {
        tracing::trace!("{name}: {:?}", &bundle.actions);
      }
    }
    for action in bundle.actions.iter() {
      if action.id == aircraft.id {
        AircraftAllActionHandler::run(aircraft, &action.kind);
      }
    }
    bundle.actions.clear();
  }

  pub fn tick(
    &mut self,
    world: &World,
    game: &mut Game,
    rng: &mut Rng,
    dt: f32,
  ) -> Vec<Event> {
    let mut bundle = Bundle::from_world(world, rng, dt);
    self.handle_collisions(&mut game.aircraft);

    if !self.events.is_empty() {
      tracing::trace!("tick events: {:?}", self.events);
    }
    for aircraft in game.aircraft.iter_mut() {
      // Capture the previous state
      bundle.prev = aircraft.clone();

      // Run through all events
      for event in self.events.iter().filter_map(|e| match e {
        Event::Aircraft(aircraft_event) => Some(aircraft_event),
        Event::UiEvent(_) => None,
      }) {
        if event.id == aircraft.id {
          HandleAircraftEvent::run(aircraft, &event.kind, &mut bundle);

          // Apply all actions
          if !bundle.actions.is_empty() {
            tracing::trace!("event: {event:?} {:?}", &bundle.actions);
          }
          self.apply_actions(&mut bundle, aircraft, None);
        }
      }

      // Run through all effects
      AircraftUpdateLandingEffect::run(aircraft, &mut bundle);
      AircraftUpdateFlyingEffect::run(aircraft, &mut bundle);
      AircraftUpdateTaxiingEffect::run(aircraft, &mut bundle);
      self.apply_actions(&mut bundle, aircraft, Some("state actions"));

      // Apply all actions
      AircraftUpdateFromTargetsEffect::run(aircraft, &mut bundle);
      self.apply_actions(&mut bundle, aircraft, Some("target actions"));

      AircraftUpdatePositionEffect::run(aircraft, &mut bundle);
      self.apply_actions(&mut bundle, aircraft, Some("position actions"));
    }

    // Capture the left over events and actions for next time
    if !bundle.events.is_empty() {
      tracing::info!("new events: {:?}", bundle.events);
    }
    self.events = core::mem::take(&mut bundle.events);

    self.events.clone()
  }
}
