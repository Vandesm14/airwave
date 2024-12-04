use std::{
  path::PathBuf,
  time::{Duration, Instant},
};

use glam::Vec2;
use internment::Intern;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::error::TryRecvError;
use turborand::{rng::Rng, TurboRand};

use engine::{
  circle_circle_intersection,
  command::{CommandReply, CommandWithFreq, OutgoingCommandReply, Task},
  duration_now,
  engine::{Engine, Event},
  entities::{
    aircraft::{
      events::{AircraftEvent, EventKind},
      Aircraft, AircraftState,
    },
    flight::{Flight, FlightKind, FlightStatus},
    world::{Connection, ConnectionState, Game, Points, World},
  },
};

use crate::{
  job::{JobQueue, JobReq},
  ring::RingBuffer,
  AUTO_TOWER_AIRSPACE_RADIUS, MANUAL_TOWER_AIRSPACE_RADIUS,
  TOWER_AIRSPACE_PADDING_RADIUS, WORLD_RADIUS,
};

pub const SPAWN_RATE: Duration = Duration::from_secs(210);
pub const PREP_SPAWN_RATE: Duration = Duration::from_secs(120);
pub const SPAWN_LIMIT: usize = 34;

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
  Points(Points),
  Funds(usize),
}

#[derive(Debug, Clone)]
pub enum TinyReqKind {
  Ping,
  Pause,

  // Aircraft
  Aircraft,
  OneAircraft(Intern<String>),

  // Flights
  Flights,
  GetFlight(usize),
  CreateFlight {
    kind: FlightKind,
    spawn_at: Duration,
  },
  DeleteFlight(usize),

  // Other State
  Messages,
  World,
  Points,
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
  Pong,

  // Aircraft
  Aircraft(Vec<Aircraft>),
  OneAircraft(Option<Aircraft>),

  // Flights
  Flights(Vec<Flight>),
  OneFlight(Option<Flight>),

  // Other State
  Messages(Vec<OutgoingCommandReply>),
  World(World),
  Points(Points),
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
  rate: usize,
}

impl Runner {
  pub fn new(
    get_rcv: tokio::sync::mpsc::UnboundedReceiver<JobReq<TinyReqKind, ResKind>>,
    post_rcv: tokio::sync::mpsc::UnboundedReceiver<JobReq<ArgReqKind, ResKind>>,
    save_to: Option<PathBuf>,
    rng: Rng,
  ) -> Self {
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
      rate: 15,
    }
  }

  pub fn add_aircraft(&mut self, mut aircraft: Aircraft) {
    while self.game.aircraft.iter().any(|a| a.id == aircraft.id) {
      aircraft.id = Intern::from(Aircraft::random_callsign(&mut self.rng));
    }

    if aircraft.flight_plan.departing == aircraft.flight_plan.arriving {
      tracing::warn!(
        "deleted a flight departing and arriving at the same airspace"
      );
      return;
    }

    self.game.aircraft.push(aircraft);
  }

  pub fn generate_airspaces(&mut self, world_rng: &mut Rng) {
    let airspace_names = [
      "KLAX", "KPHL", "KJFK", "KMGM", "KCLT", "KDFW", "KATL", "KMCO", "EGLL",
      "EGLC", "EGNV", "EGNT", "EGGP", "EGCC", "EGKK", "EGHI",
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

        for airport in self.world.connections.iter() {
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

      let connection = Connection {
        id: Intern::from_ref(airspace_name),
        state: ConnectionState::Active,
        pos: airspace_position,
        transition: self
          .world
          .airspace
          .pos
          .move_towards(airspace_position, MANUAL_TOWER_AIRSPACE_RADIUS),
      };

      self.world.connections.push(connection);
    }
  }

  pub fn fill_gates(&mut self) {
    let mut aircrafts: Vec<Aircraft> = Vec::new();
    for airport in self.world.airspace.airports.iter() {
      for terminal in airport.terminals.iter() {
        for gate in terminal.gates.iter() {
          let mut aircraft = Aircraft::random_parked(
            gate.clone(),
            &mut self.rng,
            &self.world.airspace,
          );
          aircraft.flight_plan.departing = self.world.airspace.id;
          aircraft.flight_plan.arriving = self
            .rng
            .sample(&self.world.connections)
            .map(|c| c.id)
            .unwrap_or_default();

          aircrafts.push(aircraft);
        }
      }
    }

    for aircraft in aircrafts.drain(..) {
      self.add_aircraft(aircraft);
    }
  }

  pub fn handle_flights(&mut self) {
    let now = duration_now();
    let mut to_mark: Vec<(usize, Intern<String>)> = Vec::new();
    for flight in self.game.flights.iter() {
      if flight.spawn_at <= now
        && matches!(flight.status, FlightStatus::Scheduled)
      {
        match flight.kind {
          FlightKind::Inbound => {
            let aircraft = Aircraft::random_inbound(
              self.world.airspace.frequencies.approach,
              self.rng.sample(&self.world.connections).unwrap(),
              &self.world.airspace,
              &mut self.rng,
            );

            to_mark.push((flight.id, aircraft.id));

            self.game.aircraft.push(aircraft);
          }
          FlightKind::Outbound => {
            let aircraft =
              self
                .rng
                .sample_iter(self.game.aircraft.iter_mut().filter(|a| {
                  matches!(a.state, AircraftState::Parked { active: false, .. })
                }));

            if let Some(aircraft) = aircraft {
              aircraft.flight_plan.departing = self.world.airspace.id;
              aircraft.flight_plan.arriving =
                self.rng.sample(&self.world.connections).unwrap().id;
              aircraft.set_active(true);
              aircraft.sync_targets_to_vals();

              to_mark.push((flight.id, aircraft.id));

              self.messages.push(CommandWithFreq::new(
                aircraft.id.to_string(),
                aircraft.frequency,
                CommandReply::ReadyForDeparture {
                  airport: aircraft.flight_plan.arriving.to_string(),
                },
                Vec::new(),
              ));
            } else {
              tracing::warn!("No aircraft available for outbound flight.");
            }
          }
        }
      }
    }

    for (flight, aircraft) in to_mark {
      tracing::info!("Spawned flight #{}", flight);
      self.game.flights.get_mut(flight).unwrap().status =
        FlightStatus::Ongoing(aircraft);
    }
  }

  pub fn tick(&mut self) {
    self.last_tick = Instant::now();

    let mut commands: Vec<CommandWithFreq> = Vec::new();

    // GET
    loop {
      let incoming = match self.get_queue.recv() {
        Ok(incoming) => incoming,
        Err(TryRecvError::Disconnected) => return,
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

        // Flights
        TinyReqKind::Flights => {
          incoming
            .reply(ResKind::Flights(self.game.flights.flights().to_vec()));
        }
        TinyReqKind::GetFlight(id) => {
          let flight = self.game.flights.get(*id).cloned();
          incoming.reply(ResKind::OneFlight(flight));
        }
        TinyReqKind::CreateFlight { kind, spawn_at } => {
          let id = self.game.flights.add(kind.clone(), *spawn_at);
          incoming
            .reply(ResKind::OneFlight(self.game.flights.get(id).cloned()));
        }
        TinyReqKind::DeleteFlight(id) => {
          let flight = self.game.flights.remove(*id);
          incoming.reply(ResKind::OneFlight(flight));
        }

        // Other State
        TinyReqKind::Messages => incoming.reply(ResKind::Messages(
          self.messages.iter().cloned().map(|m| m.into()).collect(),
        )),
        TinyReqKind::World => {
          incoming.reply(ResKind::World(self.world.clone()))
        }
        TinyReqKind::Points => {
          incoming.reply(ResKind::Points(self.game.points.clone()));
        }
      }
    }

    // POST
    loop {
      let incoming = match self.post_queue.recv() {
        Ok(incoming) => incoming,
        Err(TryRecvError::Disconnected) => return,
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
      return;
    }

    for command in commands {
      self.execute_command(command);
    }

    let dt = 1.0 / self.rate as f32;
    let events =
      self
        .engine
        .tick(&self.world, &mut self.game, &mut self.rng, dt);

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

    self.handle_flights();
    self.cleanup(events.iter());
    // TODO: self.save_world();
  }

  pub fn begin_loop(&mut self) {
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
      match event {
        AircraftEvent {
          id,
          kind: EventKind::Delete,
        } => {
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
        AircraftEvent {
          id,
          kind: EventKind::CompleteFlight,
        } => {
          if let Some(flight) = self.game.flights.get_by_aircraft_id(*id) {
            self.game.flights.get_mut(flight).unwrap().status =
              FlightStatus::Completed(*id, duration_now());
          }
        }
        _ => {}
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
