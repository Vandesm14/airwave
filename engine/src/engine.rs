use std::collections::HashSet;

use internment::Intern;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use turborand::rng::Rng;

use crate::{
  angle_between_points, delta_angle,
  entities::{
    aircraft::{
      effects::{
        AircraftEffect, AircraftUpdateFlyingEffect,
        AircraftUpdateFromTargetsEffect, AircraftUpdateLandingEffect,
        AircraftUpdatePositionEffect, AircraftUpdateTaxiingEffect,
      },
      events::{
        AircraftEvent, AircraftEventHandler, EventKind, HandleAircraftEvent,
      },
      Aircraft, AircraftState, TaxiingState,
    },
    world::{Game, World},
  },
  pathfinder::Node,
};

#[derive(Debug)]
pub struct Bundle<'a> {
  pub prev: Aircraft,

  pub events: Vec<Event>,
  pub world: &'a World,

  pub rng: &'a mut Rng,
  pub dt: f32,
}

impl<'a> Bundle<'a> {
  pub fn from_world(world: &'a World, rng: &'a mut Rng, dt: f32) -> Self {
    let prev = Aircraft::default();
    Self {
      prev,
      events: Vec::new(),
      world,
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

  Pause,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// UI Events are sent from the engine to the frontend.
pub enum UIEvent {
  // Inbound
  Purchase(usize),

  // Outbound
  Funds(usize),

  Pause,
}

impl From<UICommand> for UIEvent {
  fn from(value: UICommand) -> Self {
    match value {
      UICommand::Purchase(aircraft_id) => Self::Purchase(aircraft_id),
      UICommand::Pause => Self::Pause,
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
pub enum EngineConfig {
  /// Runs only flight layers.
  Minimal,

  /// Runs all layers for aircraft movement.
  Base,

  #[default]
  /// Runs all collision checks.
  Full,
}

impl EngineConfig {
  pub fn run_collisions(&self) -> bool {
    matches!(self, EngineConfig::Full)
  }

  pub fn run_all_layers(&self) -> bool {
    matches!(self, EngineConfig::Base) || matches!(self, EngineConfig::Full)
  }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Engine {
  pub events: Vec<Event>,
  pub config: EngineConfig,
}

impl Engine {
  pub fn tick(
    &mut self,
    world: &mut World,
    game: &mut Game,
    rng: &mut Rng,
    dt: f32,
  ) -> Vec<Event> {
    self.compute_available_gates(&game.aircraft, world);
    if self.config.run_collisions() {
      self.handle_collisions(&mut game.aircraft);
    }

    if !self.events.is_empty() {
      tracing::trace!("tick events: {:?}", self.events);
    }

    let mut bundle = Bundle::from_world(world, rng, dt);
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
        }
      }

      // Run through all effects
      if self.config.run_all_layers() {
        AircraftUpdateLandingEffect::run(aircraft, &mut bundle);
        AircraftUpdateTaxiingEffect::run(aircraft, &mut bundle);
      }
      AircraftUpdateFlyingEffect::run(aircraft, &mut bundle);
      AircraftUpdateFromTargetsEffect::run(aircraft, &mut bundle);
      AircraftUpdatePositionEffect::run(aircraft, &mut bundle);
    }

    if self.config.run_collisions() {
      self.taxi_collisions(&mut game.aircraft, &mut bundle);
    }

    // Capture the left over events and actions for next time
    if !bundle.events.is_empty() {
      tracing::info!("new events: {:?}", bundle.events);
    }

    self.events = core::mem::take(&mut bundle.events);
    self.events.clone()
  }

  pub fn compute_available_gates(
    &mut self,
    aircrafts: &[Aircraft],
    world: &mut World,
  ) {
    for gate in world
      .airspaces
      .iter_mut()
      .flat_map(|a| a.airports.iter_mut())
      .flat_map(|a| a.terminals.iter_mut())
      .flat_map(|t| t.gates.iter_mut())
    {
      let aircraft = aircrafts
        .iter()
        .filter_map(|a| {
          if let AircraftState::Parked { at, .. } = &a.state {
            Some(at)
          } else {
            None
          }
        })
        .find(|a| a.name == gate.id && a.value == gate.pos);

      gate.available = aircraft.is_some();
    }
  }

  pub fn handle_collisions(&mut self, aircrafts: &mut [Aircraft]) {
    // TODO: Collisions are disabled for now
    // let mut collisions: HashSet<Intern<String>> = HashSet::new();
    // for pair in aircrafts.iter().combinations(2) {
    //   let aircraft = pair.first().unwrap();
    //   let other_aircraft = pair.last().unwrap();

    //   let distance = aircraft.pos.distance_squared(other_aircraft.pos);
    //   let vertical_distance =
    //     (aircraft.altitude - other_aircraft.altitude).abs();

    //   if matches!(aircraft.state, AircraftState::Flying { enroute: false, .. })
    //     && matches!(
    //       other_aircraft.state,
    //       AircraftState::Flying { enroute: false, .. }
    //     )
    //     && aircraft.altitude > 1000.0
    //     && distance <= (NAUTICALMILES_TO_FEET * 4.0).powf(2.0)
    //     && vertical_distance < 1000.0
    //   {
    //     collisions.insert(aircraft.id);
    //     collisions.insert(other_aircraft.id);
    //   }
    // }

    // aircrafts.iter_mut().for_each(|aircraft| {
    //   let is_colliding = collisions.contains(&aircraft.id);

    //   // TODO: Fire collision events
    //   // if is_colliding && aircraft.is_colliding != is_colliding {
    //   //   self.events.push();
    //   // }

    //   aircraft.is_colliding = is_colliding;
    // });
  }

  pub fn taxi_collisions(
    &mut self,
    aircrafts: &mut [Aircraft],
    bundle: &mut Bundle,
  ) {
    let mut collisions: HashSet<Intern<String>> = HashSet::new();
    for pair in aircrafts
      .iter()
      .filter(|a| {
        matches!(
          a.state,
          AircraftState::Taxiing { .. } | AircraftState::Parked { .. }
        )
      })
      .combinations(2)
    {
      let aircraft = pair.first().unwrap();
      let other_aircraft = pair.last().unwrap();

      // This allows us to ignore non-moving aircraft included parked.
      if aircraft.speed == 0.0 && other_aircraft.speed == 0.0 {
        continue;
      }

      let distance = aircraft.pos.distance_squared(other_aircraft.pos);

      if distance <= 250.0_f32.powf(2.0) * 2.0 {
        if delta_angle(
          aircraft.heading,
          angle_between_points(aircraft.pos, other_aircraft.pos),
        )
        .abs()
          <= 45.0
        {
          collisions.insert(aircraft.id);
        }

        if delta_angle(
          other_aircraft.heading,
          angle_between_points(other_aircraft.pos, aircraft.pos),
        )
        .abs()
          <= 45.0
        {
          collisions.insert(other_aircraft.id);
        }
      }
    }

    for aircraft in aircrafts.iter_mut() {
      if let AircraftState::Taxiing { state, .. } = &mut aircraft.state {
        if collisions.contains(&aircraft.id) && state == &TaxiingState::Armed {
          *state = TaxiingState::Stopped;
          bundle.events.push(Event::Aircraft(AircraftEvent::new(
            aircraft.id,
            EventKind::TaxiHold { and_state: false },
          )));
        } else if !collisions.contains(&aircraft.id)
          && matches!(state, TaxiingState::Override | TaxiingState::Stopped)
        {
          if matches!(state, TaxiingState::Stopped) {
            bundle.events.push(Event::Aircraft(AircraftEvent::new(
              aircraft.id,
              EventKind::TaxiContinue,
            )));
          }

          *state = TaxiingState::Armed;
        }
      }
    }
  }
}
