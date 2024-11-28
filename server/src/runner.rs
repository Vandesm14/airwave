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
  command::{CommandReply, CommandWithFreq, OutgoingCommandReply, Task},
  engine::{Engine, Event, UICommand},
  entities::{
    aircraft::{
      events::{AircraftEvent, EventKind},
      Aircraft, AircraftState, FlightPlan,
    },
    world::{Connection, ConnectionState, Game, Points, World},
  },
  heading_to_direction,
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
pub enum JobReqKind {
  Ping,

  // GET
  Messages,
  World,
  Game,
  Aircraft,

  // POST
  Command {
    atc: CommandWithFreq,
    reply: CommandWithFreq,
  },
}

#[derive(Debug, Clone)]
pub enum JobResKind {
  Pong,

  // GET
  Messages(Vec<OutgoingCommandReply>),
  World(World),
  Game(Game),
  Aircraft(Vec<Aircraft>),
}

#[derive(Debug)]
pub struct Runner {
  pub world: World,
  pub game: Game,
  pub engine: Engine,
  pub messages: RingBuffer<CommandWithFreq>,

  pub job_queue: JobQueue<JobReqKind, JobResKind>,

  pub save_to: Option<PathBuf>,
  pub rng: Rng,

  last_tick: Instant,
  rate: usize,
  paused: bool,
}

impl Runner {
  pub fn new(
    receiver: tokio::sync::mpsc::UnboundedReceiver<
      JobReq<JobReqKind, JobResKind>,
    >,
    save_to: Option<PathBuf>,
    rng: Rng,
  ) -> Self {
    Self {
      world: World::default(),
      game: Game::default(),
      engine: Engine::default(),
      messages: RingBuffer::new(3),

      job_queue: JobQueue::new(receiver),

      save_to,
      rng,

      last_tick: Instant::now(),
      rate: 15,
      paused: false,
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
    let mut ui_commands: Vec<UICommand> = Vec::new();

    loop {
      let incoming = match self.job_queue.recv() {
        Ok(incoming) => incoming,
        Err(TryRecvError::Disconnected) => return,
        Err(TryRecvError::Empty) => break,
      };

      match incoming.req() {
        JobReqKind::Ping => incoming.reply(JobResKind::Pong),

        // GET
        JobReqKind::Messages => incoming.reply(JobResKind::Messages(
          self.messages.iter().cloned().map(|m| m.into()).collect(),
        )),
        JobReqKind::World => {
          incoming.reply(JobResKind::World(self.world.clone()))
        }
        JobReqKind::Game => {
          incoming.reply(JobResKind::Game(self.game.clone()));
        }
        JobReqKind::Aircraft => {
          incoming.reply(JobResKind::Aircraft(self.game.aircraft.clone()));
        }

        // POST
        JobReqKind::Command { atc, reply } => {
          self.messages.push(atc.clone());
          commands.push(reply.clone());
        }
      }
    }

    if self.paused {
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

    for ui_command in ui_commands {
      self.engine.events.push(Event::UiEvent(ui_command.into()));
    }

    let dt = 1.0 / self.rate as f32;
    let events =
      self
        .engine
        .tick(&self.world, &mut self.game, &mut self.rng, dt);

    // Run through all callout events and broadcast them
    for event in events.iter().filter_map(|e| match e {
      Event::Aircraft(aircraft_event) => Some(aircraft_event),
      Event::UiEvent(_) => None,
    }) {
      match &event.kind {
        EventKind::Callout(command) => {
          self.messages.push(command.clone());
        }

        EventKind::EnRoute(false) => {
          if let Some(aircraft) =
            self.game.aircraft.iter().find(|a| a.id == event.id)
          {
            let direction = heading_to_direction(angle_between_points(
              self.world.airspace.pos,
              aircraft.pos,
            ))
            .to_owned();
            let command = CommandWithFreq::new(
              Intern::to_string(&aircraft.id),
              aircraft.frequency,
              CommandReply::ArriveInAirspace {
                direction,
                altitude: aircraft.altitude,
              },
              Vec::new(),
            );

            self.messages.push(command.clone());
          }
        }
        _ => {}
      }
    }

    self.cleanup(events.iter().filter_map(|e| match e {
      Event::Aircraft(aircraft_event) => Some(aircraft_event),
      Event::UiEvent(_) => None,
    }));
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
    T: Iterator<Item = &'a AircraftEvent>,
  {
    for event in events {
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

      self.cleanup(events.iter().filter_map(|e| match e {
        Event::Aircraft(aircraft_event) => Some(aircraft_event),
        Event::UiEvent(_) => None,
      }));

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
