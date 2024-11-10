use std::{
  path::PathBuf,
  time::{Duration, Instant},
};

use async_channel::TryRecvError;
use internment::Intern;
use serde::{Deserialize, Serialize};
use turborand::rng::Rng;

use engine::{
  command::{CommandWithFreq, OutgoingCommandReply, Task},
  engine::{Engine, Event, UICommand, UIEvent},
  entities::{
    aircraft::{
      events::{AircraftEvent, EventKind},
      Aircraft,
    },
    world::{Game, Points, World},
  },
};

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IncomingUpdate {
  Command(CommandWithFreq),
  UICommand(UICommand),
  Connect,
}

#[derive(Debug, Clone)]
pub struct Runner {
  pub world: World,
  pub game: Game,
  pub engine: Engine,

  pub receiver: async_channel::Receiver<IncomingUpdate>,
  pub outgoing_sender: async_broadcast::Sender<OutgoingReply>,
  pub incoming_sender: async_channel::Sender<IncomingUpdate>,

  pub save_to: Option<PathBuf>,
  pub rng: Rng,

  last_tick: Instant,
  rate: usize,
}

impl Runner {
  pub fn new(
    receiver: async_channel::Receiver<IncomingUpdate>,
    outgoing_sender: async_broadcast::Sender<OutgoingReply>,
    incoming_sender: async_channel::Sender<IncomingUpdate>,
    save_to: Option<PathBuf>,
    rng: Rng,
  ) -> Self {
    Self {
      world: World::default(),
      game: Game::default(),
      engine: Engine::default(),

      receiver,
      outgoing_sender,
      incoming_sender,

      save_to,
      rng,

      last_tick: Instant::now(),
      rate: 10,
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

  pub fn begin_loop(&mut self) {
    'main_loop: loop {
      if Instant::now() - self.last_tick
        >= Duration::from_secs_f32(1.0 / self.rate as f32)
      {
        self.last_tick = Instant::now();

        let mut commands: Vec<CommandWithFreq> = Vec::new();
        let mut ui_commands: Vec<UICommand> = Vec::new();

        loop {
          let incoming = match self.receiver.try_recv() {
            Ok(incoming) => incoming,
            Err(TryRecvError::Closed) => break 'main_loop,
            Err(TryRecvError::Empty) => break,
          };

          match incoming {
            IncomingUpdate::Command(command) => commands.push(command),
            IncomingUpdate::UICommand(ui_command) => {
              ui_commands.push(ui_command)
            }
            IncomingUpdate::Connect => self.broadcast_for_new_client(),
          }
        }

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
          if let EventKind::Callout(command) = &event.kind {
            if let Err(e) = self
              .outgoing_sender
              .try_broadcast(OutgoingReply::Reply(command.clone().into()))
            {
              tracing::error!("error sending outgoing reply: {e}")
            }
          }
        }

        for event in events.iter().filter_map(|e| match e {
          Event::Aircraft(_) => None,
          Event::UiEvent(ui_event) => Some(ui_event),
        }) {
          if let UIEvent::Funds(_) = &event {
            self.broadcast_funds();
          }
        }

        self.cleanup(events.iter().filter_map(|e| match e {
          Event::Aircraft(aircraft_event) => Some(aircraft_event),
          Event::UiEvent(_) => None,
        }));
        self.broadcast_aircraft();
        // TODO: self.save_world();
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
        self
          .outgoing_sender
          .try_broadcast(OutgoingReply::Reply(command.clone().into()))
          .unwrap();
      }
    }
  }

  fn broadcast_aircraft(&self) {
    let _ = self
      .outgoing_sender
      .try_broadcast(OutgoingReply::Aircraft(self.game.aircraft.clone()))
      .inspect_err(|e| tracing::warn!("failed to broadcast aircraft: {}", e));
  }

  fn broadcast_points(&self) {
    let _ = self
      .outgoing_sender
      .try_broadcast(OutgoingReply::Points(self.game.points.clone()))
      .inspect_err(|e| tracing::warn!("failed to broadcast points: {}", e));
  }

  fn broadcast_funds(&self) {
    let _ = self
      .outgoing_sender
      .try_broadcast(OutgoingReply::Funds(self.game.funds))
      .inspect_err(|e| tracing::warn!("failed to broadcast funds: {}", e));
  }

  fn broadcast_world(&self) {
    let _ = self
      .outgoing_sender
      .try_broadcast(OutgoingReply::World(self.world.clone()))
      .inspect_err(|e| tracing::warn!("failed to broadcast world: {}", e));
  }

  fn broadcast_for_new_client(&self) {
    self.broadcast_world();
    self.broadcast_points();
  }
}
