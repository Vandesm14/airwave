use std::{
  ops::Div,
  path::PathBuf,
  time::{Duration, Instant},
};

use glam::Vec2;
use internment::Intern;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::error::TryRecvError;
use turborand::{rng::Rng, TurboRand};

use engine::{
  circle_circle_intersection,
  command::{CommandWithFreq, OutgoingCommandReply, Task},
  engine::{Engine, EngineConfig, Event},
  entities::{
    aircraft::{
      events::{AircraftEvent, EventKind},
      Aircraft, AircraftKind, AircraftState,
    },
    airport::{Airport, Frequencies},
    airspace::Airspace,
    world::{Game, World},
  },
  pathfinder::{Node, NodeBehavior, NodeKind},
  Translate, NAUTICALMILES_TO_FEET,
};

use crate::{
  airport::new_v_pattern,
  job::{JobQueue, JobReq},
  merge_points,
  ring::RingBuffer,
  signal_gen::SignalGenerator,
  AUTO_TOWER_AIRSPACE_RADIUS, TOWER_AIRSPACE_PADDING_RADIUS, WORLD_RADIUS,
};

pub const AIRPORT_SPAWN_CHANCE: f64 = 0.8;
pub const NON_AUTO_SPAWN_CHANCE: f64 = 0.2;
pub const SPAWN_RATE: usize = 75;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type", content = "value")]
pub enum OutgoingReply {
  // Partial/Small Updates
  ATCReply(OutgoingCommandReply),
  Reply(OutgoingCommandReply),

  // Full State Updates
  Aircraft(Vec<Aircraft>),
  World(World),
  Size(f32),
}

#[derive(Debug, Clone)]
pub enum TinyReqKind {
  Ping,
  Pause,

  // Aircraft
  Aircraft,
  OneAircraft(Intern<String>),
  AcceptFlight(Intern<String>),
  RejectFlight(Intern<String>),

  // Other State
  Messages,
  World,
}

#[derive(Debug, Clone)]
pub enum ArgReqKind {
  // Comms
  /// A command sent from ATC to an aircraft.
  CommandATC(CommandWithFreq),
  /// A reply from an aircraft to ATC.
  CommandReply(CommandWithFreq),
}

#[derive(Debug, Clone, Default)]
pub enum ResKind {
  #[default]
  Any,
  Err,

  Pong,

  // Aircraft
  Aircraft(Vec<Aircraft>),
  OneAircraft(Option<Aircraft>),

  // Other State
  Messages(Vec<OutgoingCommandReply>),
  World(World),
}

#[derive(Debug)]
pub struct Runner {
  pub world: World,
  pub game: Game,
  pub engine: Engine,
  pub messages: RingBuffer<CommandWithFreq>,

  pub get_queue: JobQueue<TinyReqKind, ResKind>,
  pub post_queue: JobQueue<ArgReqKind, ResKind>,

  pub save_to: Option<PathBuf>,
  pub rng: Rng,

  last_tick: Instant,
  pub rate: usize,
  pub tick_counter: usize,

  spawns: SignalGenerator,
}

impl Runner {
  pub fn new(
    get_rcv: tokio::sync::mpsc::UnboundedReceiver<JobReq<TinyReqKind, ResKind>>,
    post_rcv: tokio::sync::mpsc::UnboundedReceiver<JobReq<ArgReqKind, ResKind>>,
    save_to: Option<PathBuf>,
    rng: Rng,
  ) -> Self {
    let rate = 15;

    Self {
      world: World::default(),
      game: Game::default(),
      engine: Engine::default(),
      messages: RingBuffer::new(30),

      get_queue: JobQueue::new(get_rcv),
      post_queue: JobQueue::new(post_rcv),

      save_to,
      rng,

      last_tick: Instant::now(),
      rate,
      tick_counter: 0,

      // Spawn rate is 60 + 15 seconds to make it less robotic.
      spawns: SignalGenerator::new(rate * SPAWN_RATE),
    }
  }

  pub fn add_aircraft(&mut self, mut aircraft: Aircraft) {
    while self.game.aircraft.iter().any(|a| a.id == aircraft.id) {
      aircraft.id = Intern::from(Aircraft::random_callsign(&mut self.rng));
    }

    self.game.aircraft.push(aircraft);
  }

  pub fn generate_airspaces(
    &mut self,
    world_rng: &mut Rng,
    config_frequencies: &Frequencies,
  ) {
    let airspace_names = [
      // "KLAX", "KPHL", "KJFK", "KMGM", "KCLT", "KDFW", "KATL", "KMCO", "EGLL",
      // "EGLC", "EGNV", "EGNT", "EGGP", "EGCC", "EGKK", "EGHI",
      "KLAX", "KPHL", "KJFK", "KMGM", "KCLT", "KDFW", "KATL", "KMCO", "EGLL",
      "EGKK", "EGHI",
    ];

    // Generate randomly positioned uncontrolled airspaces.
    for airspace_name in airspace_names {
      // TODO: This is a brute-force approach. A better solution would be to use
      //       some form of jitter or other, potentially, less infinite-loop-prone
      //       solution.

      let mut i = 0;

      let airspace_position = 'outer: loop {
        if i >= 1000 {
          tracing::error!(
            "Unable to find a place for airspace '{airspace_name}'"
          );
          std::process::exit(1);
        }

        i += 1;

        let position = Vec2::new(
          (world_rng.f32() - 0.5) * WORLD_RADIUS,
          (world_rng.f32() - 0.5) * WORLD_RADIUS,
        );

        for airport in self.world.airspaces.iter() {
          if circle_circle_intersection(
            position,
            airport.pos,
            AUTO_TOWER_AIRSPACE_RADIUS + TOWER_AIRSPACE_PADDING_RADIUS,
            AUTO_TOWER_AIRSPACE_RADIUS + TOWER_AIRSPACE_PADDING_RADIUS,
          ) {
            continue 'outer;
          }
        }

        break position;
      };

      let mut airspace = Airspace {
        id: Intern::from_ref(airspace_name),
        pos: airspace_position,
        radius: NAUTICALMILES_TO_FEET * 30.0,
        airports: Vec::with_capacity(1),
        auto: true,
      };

      let frequencies = Frequencies {
        approach: 0.0,
        departure: 0.0,
        tower: 0.0,
        ground: 0.0,
        center: config_frequencies.center,
      };

      let mut airport = Airport {
        id: Intern::from_ref(airspace_name),
        frequencies,
        ..Default::default()
      };

      new_v_pattern::setup(&mut airport);

      airport.translate(airspace.pos);
      airport.calculate_waypoints();

      airspace.airports.push(airport);

      self.world.airspaces.push(airspace);
    }
  }

  pub fn generate_waypoints(&mut self) {
    let separation = NAUTICALMILES_TO_FEET * 30.0;
    let min_distance = NAUTICALMILES_TO_FEET * 15.0;

    let mut waypoints: Vec<Vec2> = Vec::new();
    for airspace in self.world.airspaces.iter().combinations(2) {
      let first = airspace.first().unwrap();
      let second = airspace.last().unwrap();
      let count =
        first.pos.distance(second.pos).div(separation).ceil() as usize - 1;
      for i in 1..count {
        waypoints
          .push(first.pos.move_towards(second.pos, separation * i as f32));
      }
    }

    let mut waypoints = merge_points(&waypoints, min_distance);
    let waypoints = waypoints
      .drain(..)
      .filter(|w| {
        !self.world.airspaces.iter().any(|a| {
          a.pos.distance_squared(*w) < AUTO_TOWER_AIRSPACE_RADIUS.powf(2.0)
        })
      })
      .enumerate()
      .map(|(i, w)| {
        Node::new(
          Intern::from(i.to_string()),
          NodeKind::VOR,
          NodeBehavior::GoTo,
          w,
        )
      });

    self.world.waypoints = waypoints.collect();
  }

  pub fn fill_gates(&mut self) {
    let mut aircrafts: Vec<Aircraft> = Vec::new();
    for airspace in self.world.airspaces.iter() {
      for airport in airspace.airports.iter() {
        for terminal in airport.terminals.iter() {
          for gate in terminal.gates.iter() {
            let mut aircraft =
              Aircraft::random_parked(gate.clone(), &mut self.rng, airport);
            aircraft.flight_plan.departing = airspace.id;
            aircraft.flight_plan.arriving = self
              .rng
              .sample(&self.world.airspaces)
              .filter(|a| a.auto && a.id != airspace.id)
              .map(|a| a.id)
              .unwrap_or_default();

            aircrafts.push(aircraft);
          }
        }
      }
    }

    for aircraft in aircrafts.drain(..) {
      self.add_aircraft(aircraft);
    }
  }

  pub fn tick(&mut self) -> Vec<Event> {
    self.last_tick = Instant::now();

    let mut commands: Vec<CommandWithFreq> = Vec::new();

    // GET
    loop {
      let incoming = match self.get_queue.recv() {
        Ok(incoming) => incoming,
        Err(TryRecvError::Disconnected) => return Vec::new(),
        Err(TryRecvError::Empty) => break,
      };

      match incoming.req() {
        TinyReqKind::Ping => incoming.reply(ResKind::Pong),
        TinyReqKind::Pause => {
          self.game.paused = !self.game.paused;
        }

        // Aircraft
        TinyReqKind::Aircraft => {
          incoming.reply(ResKind::Aircraft(self.game.aircraft.clone()));
        }
        TinyReqKind::OneAircraft(id) => {
          let aircraft =
            self.game.aircraft.iter().find(|a| a.id == *id).cloned();
          incoming.reply(ResKind::OneAircraft(aircraft));
        }
        TinyReqKind::AcceptFlight(id) => {
          let aircraft = self.game.aircraft.iter_mut().find(|a| a.id == *id);
          if let Some(aircraft) = aircraft {
            aircraft.accepted = true;

            incoming.reply(ResKind::Any);
          } else {
            incoming.reply(ResKind::Err);
          }
        }
        TinyReqKind::RejectFlight(id) => {
          let aircraft = self.game.aircraft.iter_mut().find(|a| a.id == *id);
          if let Some(aircraft) = aircraft {
            aircraft.accepted = false;

            incoming.reply(ResKind::Any);
          } else {
            incoming.reply(ResKind::Err);
          }
        }

        // Other State
        TinyReqKind::Messages => incoming.reply(ResKind::Messages(
          self.messages.iter().cloned().map(|m| m.into()).collect(),
        )),
        TinyReqKind::World => {
          incoming.reply(ResKind::World(self.world.clone()))
        }
      }
    }

    // POST
    loop {
      let incoming = match self.post_queue.recv() {
        Ok(incoming) => incoming,
        Err(TryRecvError::Disconnected) => return Vec::new(),
        Err(TryRecvError::Empty) => break,
      };

      match incoming.req() {
        ArgReqKind::CommandATC(command) => {
          self.messages.push(command.clone());
          incoming.reply(ResKind::Any);
        }
        ArgReqKind::CommandReply(command) => {
          commands.push(command.clone());
          incoming.reply(ResKind::Any);
        }
      }
    }

    if self.game.paused {
      return Vec::new();
    }

    for command in commands {
      self.execute_command(command);
    }

    let dt = 1.0 / self.rate as f32;
    let events =
      self
        .engine
        .tick(&mut self.world, &mut self.game, &mut self.rng, dt);

    // Run through all callout events and broadcast them
    self.messages.extend(
      events
        .iter()
        .filter_map(|e| match e {
          Event::Aircraft(AircraftEvent {
            kind: EventKind::Callout(command),
            ..
          }) => Some(command),
          _ => None,
        })
        .cloned(),
    );

    if self.spawns.tick(self.tick_counter) {
      let airports = self
        .world
        .airspaces
        .iter()
        .filter(|a| a.auto)
        .flat_map(|a| a.airports.iter());
      for airport in airports {
        let chance = self.rng.chance(AIRPORT_SPAWN_CHANCE);
        if !chance {
          continue;
        }

        let gates = airport
          .terminals
          .iter()
          .flat_map(|t| t.gates.iter().filter(|g| !g.available));
        let gate = self.rng.sample_iter(gates);
        if let Some(gate) = gate {
          let aircraft = self.game.aircraft.iter_mut().find(|a| {
            if let AircraftState::Parked { at } = &a.state {
              at.name == gate.id && a.pos == gate.pos
            } else {
              false
            }
          });

          if let Some(aircraft) = aircraft {
            // Chance for a flight to go to a non-auto airspace.
            let go_to_non_auto = self.rng.chance(NON_AUTO_SPAWN_CHANCE);
            let destination =
              self
                .rng
                .sample_iter(self.world.airspaces.iter().filter(|a| {
                  if a.id == aircraft.flight_plan.departing {
                    return false;
                  }

                  if go_to_non_auto {
                    !a.auto
                  } else {
                    a.auto
                  }
                }));
            if let Some(destination) = destination {
              aircraft.flight_plan.arriving = destination.id;
            } else {
              tracing::warn!("No destination available for {:?}", aircraft.id);
            }

            self.engine.events.push(Event::Aircraft(AircraftEvent::new(
              aircraft.id,
              EventKind::QuickDepart,
            )));
          }
        } else {
          tracing::warn!(
            "No departures available for {:?} (gates are empty)",
            airport.id
          );
        }
      }
    }

    self.cleanup(events.iter());
    // TODO: self.save_world();

    self.tick_counter += 1;

    events.clone()
  }

  pub fn quick_start(&mut self) -> usize {
    self.engine.config = EngineConfig::Minimal;

    let size_nm = (WORLD_RADIUS) / NAUTICALMILES_TO_FEET;
    let base_speed_knots = AircraftKind::A21N.stats().max_speed;

    let max_time_hours = size_nm / base_speed_knots;
    let max_time_secs = max_time_hours * 60.0 * 60.0;
    let max_ticks = (max_time_secs * self.rate as f32).ceil() as usize;

    for _ in 0..max_ticks {
      for event in self.tick().drain(..) {
        if let Event::Aircraft(AircraftEvent {
          id,
          kind: EventKind::CalloutInAirspace,
        }) = event
        {
          if let Some(aircraft) =
            self.game.aircraft.iter_mut().find(|a| a.id == id)
          {
            if let Some(airspace) = self
              .world
              .airspaces
              .iter()
              .find(|a| a.id == aircraft.flight_plan.arriving)
            {
              if !airspace.auto {
                tracing::info!("Quick start interrupted. Aircraft entered non-auto airspace.");

                aircraft.accepted = true;

                return self.tick_counter;
              }
            }
          }
        }
      }
    }

    self.tick_counter
  }

  pub fn begin_loop(&mut self) {
    self.engine.config = EngineConfig::Full;

    loop {
      if Instant::now() - self.last_tick
        >= Duration::from_secs_f32(1.0 / self.rate as f32)
      {
        self.tick();
      }
    }
  }

  fn cleanup<'a, T>(&mut self, events: T)
  where
    T: Iterator<Item = &'a Event>,
  {
    for event in events.filter_map(|e| match e {
      Event::Aircraft(aircraft_event) => Some(aircraft_event),
      Event::UiEvent(_) => None,
    }) {
      if let AircraftEvent {
        id,
        kind: EventKind::Delete,
      } = event
      {
        let index = self
          .game
          .aircraft
          .iter()
          .enumerate()
          .find_map(|(i, a)| (a.id == *id).then_some(i));
        if let Some(index) = index {
          self.game.aircraft.swap_remove(index);
        }
      }
    }
  }

  fn execute_command(&mut self, command: CommandWithFreq) {
    let id = Intern::from_ref(&command.id);
    if self
      .game
      .aircraft
      .iter()
      .any(|a| a.id == id && a.frequency == command.frequency)
    {
      self.engine.events.extend(
        command
          .tasks
          .iter()
          .cloned()
          .map(|t| AircraftEvent { id, kind: t.into() }.into()),
      );

      let mut callout = true;
      for task in command.tasks.iter() {
        match task {
          Task::Ident => {
            // Don't generate a callout for these commands
            callout = command.tasks.len() > 1;
          }

          _ => {
            // Generate a callout from the command
            callout = true;
          }
        }
      }

      if callout {
        self.messages.push(command.clone());
      }
    }
  }

  // pub fn prepare(&mut self) {
  //   self.spawn_inbound();

  //   let mut i = 0;
  //   let mut last_spawn = 0.0;
  //   loop {
  //     let realtime = i as f32 * 1.0 / self.rate as f32;
  //     if Duration::from_secs_f32(realtime - last_spawn) >= PREP_SPAWN_RATE
  //       && self.game.aircraft.len() < SPAWN_LIMIT
  //     {
  //       self.spawn_inbound();
  //       last_spawn = realtime;
  //     }

  //     let dt = 1.0 / self.rate as f32;
  //     let events =
  //       self
  //         .engine
  //         .tick(&self.world, &mut self.game, &mut self.rng, dt);

  //     self.cleanup(events.iter());

  //     if self.game.aircraft.iter().any(|aircraft| {
  //       aircraft.altitude != 0.0
  //         && aircraft.pos.distance_squared(self.world.airspace.pos)
  //           <= MANUAL_TOWER_AIRSPACE_RADIUS.powf(2.0)
  //     }) {
  //       tracing::info!("Done ({} simulated seconds).", realtime.round());
  //       return;
  //     }

  //     i += 1;
  //   }
  // }
}
