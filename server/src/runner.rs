use std::{
  collections::HashMap,
  fs,
  ops::Div,
  path::PathBuf,
  time::{Duration, Instant, SystemTime},
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
    world::{AirspaceStatus, ArrivalStatus, DepartureStatus, Game, World},
  },
  pathfinder::{Node, NodeBehavior, NodeKind},
  Translate, NAUTICALMILES_TO_FEET,
};

use crate::{
  job::{JobQueue, JobReq},
  merge_points,
  ring::RingBuffer,
  signal_gen::SignalGenerator,
  AUTO_TOWER_AIRSPACE_RADIUS, TOWER_AIRSPACE_PADDING_RADIUS, WORLD_RADIUS,
};

pub const AIRPORT_SPAWN_CHANCE: f64 = 0.8;
// TODO: Remove this since it's 100%?
pub const NON_AUTO_DEPARTURE_CHANCE: f64 = 1.0;
pub const ARRIVE_TO_NON_AUTO_CHANCE: f64 = 0.2;
pub const SPAWN_RATE_SECONDS: usize = 75;

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

  // Other State
  Messages,
  World,
  AirspaceStatus(Intern<String>),
  DepartureStatus(Intern<String>, DepartureStatus),
  ArrivalStatus(Intern<String>, ArrivalStatus),
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
  AirspaceStatus(AirspaceStatus),
}

#[derive(Debug)]
pub struct Runner {
  pub airports: HashMap<String, Airport>,

  pub world: World,
  pub game: Game,
  pub engine: Engine,
  pub messages: RingBuffer<CommandWithFreq>,

  pub preparing: bool,

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
      airports: HashMap::new(),

      world: World::default(),
      game: Game::default(),
      engine: Engine::default(),
      messages: RingBuffer::new(30),

      preparing: false,

      get_queue: JobQueue::new(get_rcv),
      post_queue: JobQueue::new(post_rcv),

      save_to,
      rng,

      last_tick: Instant::now(),
      rate,
      tick_counter: 0,

      // Spawn rate is 60 + 15 seconds to make it less robotic.
      spawns: SignalGenerator::new(rate * SPAWN_RATE_SECONDS),
    }
  }

  pub fn load_assets(&mut self) {
    if let Ok(dir) = fs::read_dir("assets/airports") {
      for path in dir
        .flatten()
        .filter(|f| f.file_name().to_str().unwrap().ends_with(".json"))
      {
        match fs::read_to_string(path.path()) {
          Ok(content) => {
            match serde_json::from_str::<Airport>(&content) {
              Ok(mut airport) => {
                airport.translate(airport.center * -1.0);
                airport.extend_all();
                airport.calculate_waypoints();

                let name = path.file_name();
                let name = name.to_str().unwrap().replace(".json", "");
                tracing::info!(
                  "Loaded airport \"{}\" from {}",
                  airport.id,
                  path.file_name().to_str().unwrap()
                );
                self.airports.insert(name.to_owned(), airport);
              }
              Err(e) => {
                tracing::error!(
                  "Failed to read {:?}: {:?}",
                  path.file_name(),
                  e
                );
              }
            };
          }
          Err(e) => {
            tracing::error!(
              "Failed to read airport file {:?}: {:?}",
              path.file_name(),
              e
            );
          }
        }
      }
    } else {
      tracing::error!("Failed to read assets directory.");
      std::process::exit(1);
    }
  }

  pub fn airport(&self, id: impl AsRef<str>) -> Option<&Airport> {
    self.airports.get(id.as_ref())
  }

  pub fn default_airport(&self) -> Option<&Airport> {
    self.airport("default")
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

    let frequencies = Frequencies {
      approach: 0.0,
      departure: 0.0,
      tower: 0.0,
      ground: 0.0,
      center: config_frequencies.center,
    };

    let mut airport = self
      .default_airport()
      .expect("Could not find default airport.")
      .clone();
    airport.frequencies = frequencies;

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

      let mut airport = airport.clone();
      airport.translate(airspace.pos);
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
        TinyReqKind::AirspaceStatus(id) => {
          let status = self.world.airspace_statuses.get(id);
          if let Some(status) = status {
            incoming.reply(ResKind::AirspaceStatus(*status))
          } else {
            incoming.reply(ResKind::Err);
          }
        }
        TinyReqKind::ArrivalStatus(id, status) => {
          if let Some(airspace_status) =
            self.world.airspace_statuses.get_mut(id)
          {
            airspace_status.arrival = *status;

            incoming.reply(ResKind::Any);
          } else {
            incoming.reply(ResKind::Err);
          }
        }
        TinyReqKind::DepartureStatus(id, status) => {
          if let Some(airspace_status) =
            self.world.airspace_statuses.get_mut(id)
          {
            airspace_status.departure = *status;

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

    // If spawn tick, do spawns.
    if self.spawns.tick(self.tick_counter) {
      let airports = self
        .world
        .airspaces
        .iter()
        .flat_map(|a| a.airports.iter().map(|ar| (a.auto, ar)));
      for (auto, airport) in airports {
        let do_spawn = self.rng.chance(AIRPORT_SPAWN_CHANCE);
        if !do_spawn {
          continue;
        }

        let do_non_auto_spawn = self.rng.chance(NON_AUTO_DEPARTURE_CHANCE);
        // Only use dedicated spawn rate for manual airspaces if we are
        // preparing via `self.quick_start()`.
        if !auto && self.preparing && !do_non_auto_spawn {
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
            let go_to_non_auto = self.rng.chance(ARRIVE_TO_NON_AUTO_CHANCE);
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
              if !auto
                && matches!(
                  self.world.airspace_statuses.get(&airport.id),
                  Some(AirspaceStatus {
                    departure: DepartureStatus::Normal,
                    ..
                  })
                )
              {
                aircraft.flight_plan.arriving = destination.id;

                let min_time_seconds = if self.preparing { 0 } else { 60 };
                let max_time_seconds = 60 * 5;
                let delay = self.rng.u64(min_time_seconds..=max_time_seconds);
                aircraft.timer = Some(
                  SystemTime::now()
                    .duration_since(
                      // Set the timer a few minutes into the future.
                      SystemTime::UNIX_EPOCH - Duration::from_secs(delay),
                    )
                    .unwrap(),
                );
              } else if auto {
                aircraft.flight_plan.arriving = destination.id;

                aircraft.timer = Some(
                  SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap(),
                );
              }
            } else {
              tracing::warn!("No destination available for {:?}", aircraft.id);
            }

            if auto {
              self.engine.events.push(Event::Aircraft(AircraftEvent::new(
                aircraft.id,
                EventKind::QuickDepart,
              )));
            }
          }
        } else {
          // TODO: Do we want to keep this for vis or remove?
          // tracing::warn!(
          //   "No departures available for {:?} (gates are empty)",
          //   airport.id
          // );
        }
      }
    }

    self.cleanup(events.iter());
    // TODO: self.save_world();

    self.tick_counter += 1;

    events.clone()
  }

  pub fn quick_start(&mut self) -> usize {
    self.preparing = true;

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
                tracing::info!("Quick start interrupted by {}. Aircraft entered non-auto airspace.", aircraft.id);
                return self.tick_counter;
              }
            }
          }
        }
      }
    }

    self.preparing = false;

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
}
