use std::{
  sync::mpsc::{self},
  time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};

use crate::structs::{Aircraft, AircraftState, Command, Runway, Task};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type", content = "value")]
pub enum OutgoingReply {
  // Partial/Small Updates
  ATCReply(String),
  Reply(Command),

  // Full State Updates
  Aircraft(Vec<Aircraft>),
  Runways(Vec<Runway>),
  Size(f32),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IncomingUpdate {
  Command(Command),
  Connect,
}

#[derive(Debug)]
pub struct Engine {
  pub aircraft: Vec<Aircraft>,
  pub runways: Vec<Runway>,
  pub receiver: mpsc::Receiver<IncomingUpdate>,
  pub sender: mpsc::Sender<OutgoingReply>,

  last_tick: Instant,
  last_spawn: Instant,
  airspace_size: f32,
  rate: usize,
}

impl Engine {
  pub fn new(
    receiver: mpsc::Receiver<IncomingUpdate>,
    sender: mpsc::Sender<OutgoingReply>,
    airspace_size: f32,
  ) -> Self {
    Self {
      aircraft: Vec::new(),
      runways: Vec::new(),
      receiver,
      sender,

      last_tick: Instant::now(),
      last_spawn: Instant::now(),
      airspace_size,
      rate: 30,
    }
  }

  pub fn spawn_random_aircraft(&mut self) {
    let aircraft = Aircraft::random(self.airspace_size);
    self.aircraft.push(aircraft.clone());
    self
      .sender
      .send(OutgoingReply::Reply(Command {
        id: aircraft.callsign.clone(),
        reply: format!(
          "Tower, {} is at {} feet, with you.",
          aircraft.callsign, aircraft.altitude
        ),
        tasks: Vec::new(),
      }))
      .unwrap();
  }

  pub fn begin_loop(&mut self) {
    loop {
      if Instant::now() - self.last_tick
        >= Duration::from_secs_f32(1.0 / self.rate as f32)
      {
        self.last_tick = Instant::now();

        if self.aircraft.len() < 7
          && self.last_spawn.elapsed() >= Duration::from_secs(60)
        {
          self.last_spawn = Instant::now();
          self.spawn_random_aircraft();
        }

        let mut commands: Vec<Command> = Vec::new();
        for incoming in self.receiver.try_iter() {
          match incoming {
            IncomingUpdate::Command(command) => commands.push(command),
            IncomingUpdate::Connect => self.broadcast_for_new_client(),
          }
        }

        for command in commands {
          self.execute_command(command);
        }

        self.update();
        self.cleanup();
        self.broadcast_aircraft();
      }
    }
  }

  fn broadcast_aircraft(&self) {
    self
      .sender
      .send(OutgoingReply::Aircraft(self.aircraft.clone()))
      .unwrap();
  }

  fn broadcast_runways(&self) {
    self
      .sender
      .send(OutgoingReply::Runways(self.runways.clone()))
      .unwrap();
  }

  fn broadcast_size(&self) {
    self
      .sender
      .send(OutgoingReply::Size(self.airspace_size))
      .unwrap();
  }

  fn broadcast_for_new_client(&self) {
    self.broadcast_runways();
    self.broadcast_size();
  }

  pub fn cleanup(&mut self) {
    let mut indicies: Vec<usize> = Vec::new();
    for (i, aircraft) in self.aircraft.iter().enumerate() {
      if matches!(aircraft.state, AircraftState::Deleted) {
        indicies.push(i);
      }
    }

    for index in indicies {
      self.aircraft.swap_remove(index);
    }
  }

  pub fn update(&mut self) {
    let dt = 1.0 / self.rate as f32;
    for aircraft in self.aircraft.iter_mut() {
      aircraft.update(dt);
    }
  }

  pub fn execute_command(&mut self, command: Command) {
    let aircraft = self.aircraft.iter_mut().find(|a| a.callsign == command.id);
    if let Some(aircraft) = aircraft {
      // TODO: Do go-around first (then filter it out from the rest of the tasks)
      for task in command.tasks.iter() {
        match task {
          Task::Land(runway) => {
            let target = self.runways.iter().find(|r| &r.id == runway);
            if let Some(target) = target {
              aircraft.state = AircraftState::Landing(target.clone());
            }
          }
          Task::GoAround => aircraft.go_around(),
          Task::Altitude(alt) => aircraft.target.altitude = *alt,
          Task::Heading(hdg) => aircraft.target.heading = *hdg,
          Task::Speed(spd) => aircraft.target.speed = *spd,
        }
      }

      self.sender.send(OutgoingReply::Reply(command)).unwrap();
    }
  }
}
