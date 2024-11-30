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
  angle_between_points, circle_circle_intersection,
  command::{CommandWithFreq, OutgoingCommandReply, Task},
  engine::{Engine, Event},
  entities::{
    aircraft::{
      events::{AircraftEvent, EventKind},
      Aircraft, AircraftState, FlightPlan,
    },
    world::{Connection, ConnectionState, Game, Points, World},
  },
  pathfinder::new_vor,
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
  Messages,
  World,
  Points,
  Aircraft,
  OneAircraft(Intern<String>),
  Pause,
}

#[derive(Debug, Clone)]
pub enum ArgReqKind {
  /// A command sent from ATC to an aircraft.
  CommandATC(CommandWithFreq),
  /// A reply from an aircraft to ATC.
  CommandReply(CommandWithFreq),
}

#[derive(Debug, Clone, Default)]
pub enum ResKind {
  #[default]
  Any,

  // GET
  Pong,
  Messages(Vec<OutgoingCommandReply>),
  World(World),
  Points(Points),
  Aircraft(Vec<Aircraft>),
  OneAircraft(Option<Aircraft>),
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

  pub fn spawn_inbound(&mut self) {
    let rng = &mut self.rng;

    let departing = rng.sample(&self.world.connections).unwrap();
    let mut aircraft = Aircraft::random_flying(
      self.world.airspace.frequencies.approach,
      FlightPlan::new(departing.id, self.world.airspace.id),
      rng,
    );

    aircraft.speed = 300.0;
    aircraft.pos = departing.pos;
    aircraft.altitude = 7000.0;
    aircraft.heading =
      angle_between_points(departing.pos, self.world.airspace.pos);
    aircraft.sync_targets_to_vals();

    aircraft.state = AircraftState::Flying {
      enroute: true,
      waypoints: vec![new_vor(departing.id, departing.transition)
        .with_name(Intern::from_ref("TRSN"))
        .with_behavior(vec![
          EventKind::EnRoute(false),
          EventKind::SpeedAtOrBelow(250.0),
        ])],
    };

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
        let mut first = true;
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

          if first {
            aircraft.set_parked_now();
            first = false;
          }

          aircrafts.push(aircraft);
        }
      }
    }

    for aircraft in aircrafts.drain(..) {
      self.add_aircraft(aircraft);
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
        TinyReqKind::Messages => incoming.reply(ResKind::Messages(
          self.messages.iter().cloned().map(|m| m.into()).collect(),
        )),
        TinyReqKind::World => {
          incoming.reply(ResKind::World(self.world.clone()))
        }
        TinyReqKind::Points => {
          incoming.reply(ResKind::Points(self.game.points.clone()));
        }
        TinyReqKind::Aircraft => {
          incoming.reply(ResKind::Aircraft(self.game.aircraft.clone()));
        }
        TinyReqKind::OneAircraft(id) => {
          let aircraft =
            self.game.aircraft.iter().find(|a| a.id == *id).cloned();
          incoming.reply(ResKind::OneAircraft(aircraft));
        }
        TinyReqKind::Pause => {
          self.game.paused = !self.game.paused;
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

    // if Instant::now() - self.last_spawn
    //   >= self.game.points.takeoff_rate.calc_rate()
    //   && self.game.points.takeoff_rate.count() > 0
    //   && self.game.aircraft.len() < SPAWN_LIMIT
    // {
    //   self.last_spawn = Instant::now();
    //   self.spawn_inbound();
    // }

    for command in commands {
      self.execute_command(command);
    }

    // for ui_command in ui_commands {
    //   self.engine.events.push(Event::UiEvent(ui_command.into()));
    // }

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

  pub fn prepare(&mut self) {
    self.spawn_inbound();

    let mut i = 0;
    let mut last_spawn = 0.0;
    loop {
      let realtime = i as f32 * 1.0 / self.rate as f32;
      if Duration::from_secs_f32(realtime - last_spawn) >= PREP_SPAWN_RATE
        && self.game.aircraft.len() < SPAWN_LIMIT
      {
        self.spawn_inbound();
        last_spawn = realtime;
      }

      let dt = 1.0 / self.rate as f32;
      let events =
        self
          .engine
          .tick(&self.world, &mut self.game, &mut self.rng, dt);

      self.cleanup(events.iter());

      if self.game.aircraft.iter().any(|aircraft| {
        aircraft.altitude != 0.0
          && aircraft.pos.distance_squared(self.world.airspace.pos)
            <= MANUAL_TOWER_AIRSPACE_RADIUS.powf(2.0)
      }) {
        tracing::info!("Done ({} simulated seconds).", realtime.round());
        return;
      }

      i += 1;
    }
  }
}
