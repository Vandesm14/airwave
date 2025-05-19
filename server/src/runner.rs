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
use turborand::{TurboRand, rng::Rng};

use engine::{
  AIRSPACE_PADDING_RADIUS, AIRSPACE_RADIUS, DEFAULT_TICK_RATE_TPS,
  NAUTICALMILES_TO_FEET, WORLD_RADIUS,
  command::{CommandWithFreq, OutgoingCommandReply, Task},
  engine::{Engine, EngineConfig, Event},
  entities::{
    aircraft::{
      Aircraft, AircraftKind, AircraftState, FlightSegment,
      events::{AircraftEvent, EventKind},
    },
    airport::Frequencies,
    airspace::Airspace,
    world::{AirspaceStatus, ArrivalStatus, DepartureStatus, World},
  },
  geometry::{Translate, circle_circle_intersection},
  pathfinder::{Node, NodeBehavior, NodeKind},
};

use crate::{
  job::{JobQueue, JobReq},
  merge_points,
  ring::RingBuffer,
  signal_gen::SignalGenerator,
};

pub const DEPARTURE_SPAWN_CHANCE: f64 = 0.8;
// TODO: Remove this since it's 100%?
pub const NON_AUTO_DEPARTURE_CHANCE: f64 = 1.0;
pub const ARRIVE_TO_NON_AUTO_CHANCE: f64 = 0.2;
pub const SPAWN_RATE_SECONDS: usize = 75;
pub const PERF_LOG_SECONDS: usize = 60;

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

  Pong(usize),

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
  pub engine: Engine,
  pub messages: RingBuffer<CommandWithFreq>,

  pub preparing: bool,

  pub get_queue: JobQueue<TinyReqKind, ResKind>,
  pub post_queue: JobQueue<ArgReqKind, ResKind>,

  pub save_to: Option<PathBuf>,

  spawns: SignalGenerator,
  perf_log: SignalGenerator,
}

impl Runner {
  pub fn new(
    get_rcv: tokio::sync::mpsc::UnboundedReceiver<JobReq<TinyReqKind, ResKind>>,
    post_rcv: tokio::sync::mpsc::UnboundedReceiver<JobReq<ArgReqKind, ResKind>>,
    save_to: Option<PathBuf>,
    rng: Rng,
  ) -> Self {
    let engine = Engine {
      rng,
      ..Default::default()
    };

    Self {
      engine,
      messages: RingBuffer::new(30),

      preparing: false,

      get_queue: JobQueue::new(get_rcv),
      post_queue: JobQueue::new(post_rcv),

      save_to,

      spawns: SignalGenerator::new(DEFAULT_TICK_RATE_TPS * SPAWN_RATE_SECONDS),
      perf_log: SignalGenerator::new(DEFAULT_TICK_RATE_TPS * PERF_LOG_SECONDS),
    }
  }

  pub fn reset_signal_gens(&mut self) {
    self.spawns.set_first();
    self.perf_log.set_first();
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
      .engine
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

        for airport in self.engine.world.airspaces.iter() {
          if circle_circle_intersection(
            position,
            airport.pos,
            AIRSPACE_RADIUS + AIRSPACE_PADDING_RADIUS,
            AIRSPACE_RADIUS + AIRSPACE_PADDING_RADIUS,
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
      airport.id = airspace.id;
      airport.translate(airspace.pos);
      airspace.airports.push(airport);

      self.engine.world.airspaces.push(airspace);
    }
  }

  pub fn generate_waypoints(&mut self) {
    let separation = NAUTICALMILES_TO_FEET * 30.0;
    let min_distance = NAUTICALMILES_TO_FEET * 15.0;

    let mut waypoints: Vec<Vec2> = Vec::new();
    for airspace in self.engine.world.airspaces.iter().combinations(2) {
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
        !self
          .engine
          .world
          .airspaces
          .iter()
          .any(|a| a.pos.distance_squared(*w) < AIRSPACE_RADIUS.powf(2.0))
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

    self.engine.world.waypoints = waypoints.collect();
  }

  pub fn fill_gates(&mut self) {
    let mut aircrafts: Vec<Aircraft> = Vec::new();
    for airspace in self.engine.world.airspaces.iter() {
      for airport in airspace.airports.iter() {
        for terminal in airport.terminals.iter() {
          for gate in terminal.gates.iter() {
            let mut aircraft = Aircraft::random_dormant(
              gate.clone(),
              &mut self.engine.rng,
              airport,
            );
            aircraft.flight_plan.departing = airspace.id;
            aircraft.flight_plan.arriving = self
              .engine
              .rng
              .sample(&self.engine.world.airspaces)
              .filter(|a| a.auto && a.id != airspace.id)
              .map(|a| a.id)
              .unwrap_or_default();

            aircrafts.push(aircraft);
          }
        }
      }
    }

    for aircraft in aircrafts.drain(..) {
      self.engine.add_aircraft(aircraft);
    }
  }

  fn do_spawns(&mut self) {
    // If spawn tick, do spawns.
    if self.spawns.tick(self.engine.tick_counter) {
      let airports = self
        .engine
        .world
        .airspaces
        .iter()
        .flat_map(|a| a.airports.iter().map(|ar| (a.auto, ar)));
      for (auto, airport) in airports {
        let do_spawn = self.engine.rng.chance(DEPARTURE_SPAWN_CHANCE);
        if !do_spawn {
          continue;
        }

        let gates = airport
          .terminals
          .iter()
          .flat_map(|t| t.gates.iter().filter(|g| !g.available));
        let random_gate = self.engine.rng.sample_iter(gates);
        if let Some(gate) = random_gate {
          let aircraft = self
            .engine
            .game
            .aircraft
            .iter_mut()
            .filter(|a| {
              a.flight_time.is_none() && a.segment == FlightSegment::Dormant
            })
            .find(|a| {
              // Find the aircraft that is parked at the gate.
              if let AircraftState::Parked { at } = &a.state {
                at.name == gate.id && a.pos == gate.pos
              } else {
                false
              }
            });

          if let Some(aircraft) = aircraft {
            // Chance for a flight to go to a non-auto airspace.
            let go_to_non_auto =
              self.engine.rng.chance(ARRIVE_TO_NON_AUTO_CHANCE);
            let destination = self.engine.rng.sample_iter(
              self.engine.world.airspaces.iter().filter(|a| {
                if a.id == aircraft.flight_plan.departing {
                  return false;
                }

                if go_to_non_auto { !a.auto } else { a.auto }
              }),
            );
            if let Some(destination) = destination {
              // If we are preparing, only schedule departures from auto
              // airspaces.
              if (auto || !self.preparing)
                && matches!(
                  self.engine.world.airspace_statuses.get(&airport.id),
                  Some(AirspaceStatus {
                    departure: DepartureStatus::Normal,
                    ..
                  })
                )
              {
                aircraft.flight_plan.departing = airport.id;
                aircraft.flight_plan.arriving = destination.id;

                let min_time_seconds = if self.preparing { 0 } else { 60 };
                let max_time_seconds = 60 * 5;
                let delay_seconds =
                  self.engine.rng.usize(min_time_seconds..=max_time_seconds);
                let delay = delay_seconds * self.engine.tick_rate_tps;

                aircraft.flight_time = Some(self.engine.tick_counter + delay);
              }
            }
          }
        }
      }
    }

    // QuickDepart based on Flight Time.
    if self.preparing {
      for aircraft in self.engine.game.aircraft.iter_mut().filter(|a| {
        a.flight_time.is_some_and(|t| self.engine.tick_counter >= t)
      }) {
        if self
          .engine
          .world
          .airspaces
          .iter()
          .any(|a| a.id == aircraft.flight_plan.departing && a.auto)
        {
          self.engine.events.push(
            AircraftEvent::new(aircraft.id, EventKind::QuickDepart).into(),
          );
        }
      }
    }
  }

  pub fn tick(&mut self) -> Vec<Event> {
    let mut commands: Vec<CommandWithFreq> = Vec::new();

    // GET
    loop {
      let incoming = match self.get_queue.recv() {
        Ok(incoming) => incoming,
        Err(TryRecvError::Disconnected) => return Vec::new(),
        Err(TryRecvError::Empty) => break,
      };

      match incoming.req() {
        TinyReqKind::Ping => {
          incoming.reply(ResKind::Pong(self.engine.tick_counter))
        }
        TinyReqKind::Pause => {
          self.engine.game.paused = !self.engine.game.paused;
        }

        // Aircraft
        TinyReqKind::Aircraft => {
          incoming.reply(ResKind::Aircraft(self.engine.game.aircraft.clone()));
        }
        TinyReqKind::OneAircraft(id) => {
          let aircraft = self
            .engine
            .game
            .aircraft
            .iter()
            .find(|a| a.id == *id)
            .cloned();
          incoming.reply(ResKind::OneAircraft(aircraft));
        }
        TinyReqKind::AirspaceStatus(id) => {
          let status = self.engine.world.airspace_statuses.get(id);
          if let Some(status) = status {
            incoming.reply(ResKind::AirspaceStatus(*status))
          } else {
            incoming.reply(ResKind::Err);
          }
        }
        TinyReqKind::ArrivalStatus(id, status) => {
          if let Some(airspace_status) =
            self.engine.world.airspace_statuses.get_mut(id)
          {
            airspace_status.arrival = *status;

            incoming.reply(ResKind::Any);
          } else {
            incoming.reply(ResKind::Err);
          }
        }
        TinyReqKind::DepartureStatus(id, status) => {
          if let Some(airspace_status) =
            self.engine.world.airspace_statuses.get_mut(id)
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
          incoming.reply(ResKind::World(self.engine.world.clone()))
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

    if self.engine.game.paused {
      return Vec::new();
    }

    for command in commands {
      self.execute_command(command);
    }

    let events = self.engine.tick();

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

    self.do_spawns();
    self.cleanup(events.iter());
    // TODO: self.save_world();

    // Log performance of engine.
    if !self.preparing && self.perf_log.tick(self.engine.tick_counter) {
      let diff = self.engine.last_tick.elapsed();
      let mills = diff.as_micros() as f32 / 1000.0;
      let max = 1.0 / self.engine.tick_rate_tps as f32 * 1000.0;
      let percent =
        diff.as_secs_f32() / (1.0 / self.engine.tick_rate_tps as f32);

      tracing::info!(
        "Using {:.2}ms of {:.0}ms total tick time ({:.2}%)",
        mills,
        max,
        percent * 100.0
      );
    }

    self.engine.tick_counter += 1;

    events.clone()
  }

  pub fn quick_start(&mut self) -> usize {
    self.preparing = true;

    self.engine.config = EngineConfig::Minimal;

    let size_nm = (WORLD_RADIUS) / NAUTICALMILES_TO_FEET;
    let base_speed_knots = AircraftKind::A21N.stats().max_speed;

    let max_time_hours = size_nm / base_speed_knots;
    let max_time_secs = max_time_hours * 60.0 * 60.0;
    let max_ticks =
      (max_time_secs * self.engine.tick_rate_tps as f32).ceil() as usize;

    for _ in 0..max_ticks {
      for event in self.tick().drain(..) {
        if let Event::Aircraft(AircraftEvent {
          id,
          kind: EventKind::Segment(FlightSegment::Approach),
        }) = event
        {
          if let Some(aircraft) =
            self.engine.game.aircraft.iter_mut().find(|a| a.id == id)
          {
            if let Some(airspace) = self
              .engine
              .world
              .airspaces
              .iter()
              .find(|a| a.id == aircraft.flight_plan.arriving)
            {
              if !airspace.auto {
                tracing::info!(
                  "Quick start interrupted by {}. Aircraft entered non-auto airspace.",
                  aircraft.id
                );

                self.preparing = false;

                return self.engine.tick_counter;
              }
            }
          }
        }
      }
    }

    self.preparing = false;

    self.engine.tick_counter
  }

  pub fn begin_loop(&mut self) {
    self.engine.config = EngineConfig::Full;

    loop {
      if Instant::now() - self.engine.last_tick
        >= Duration::from_secs_f32(1.0 / self.engine.tick_rate_tps as f32)
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
          .engine
          .game
          .aircraft
          .iter()
          .enumerate()
          .find_map(|(i, a)| (a.id == *id).then_some(i));
        if let Some(index) = index {
          self.engine.game.aircraft.swap_remove(index);
        }
      }
    }
  }

  fn execute_command(&mut self, command: CommandWithFreq) {
    let id = Intern::from_ref(&command.id);
    if self
      .engine
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
