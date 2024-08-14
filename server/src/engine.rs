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
  Reply(String),

  // Full State Updates
  Aircraft(Vec<Aircraft>),
  Runways(Vec<Runway>),
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
  rate: usize,
}

impl Engine {
  pub fn new(
    receiver: mpsc::Receiver<IncomingUpdate>,
    sender: mpsc::Sender<OutgoingReply>,
  ) -> Self {
    Self {
      aircraft: Vec::new(),
      runways: Vec::new(),
      receiver,
      sender,

      last_tick: Instant::now(),
      rate: 5,
    }
  }

  pub fn begin_loop(&mut self) {
    loop {
      if Instant::now() - self.last_tick
        >= Duration::from_secs_f32(1.0 / self.rate as f32)
      {
        self.last_tick = Instant::now();

        for incoming in self.receiver.try_iter() {
          match incoming {
            IncomingUpdate::Command(_) => todo!("command"),
            IncomingUpdate::Connect => self.broadcast_runways(),
          }
        }

        self.update();
      }
    }
  }

  fn broadcast_all(&self) {
    self.broadcast_runways();
    self.broadcast_aircraft();
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

  pub fn update(&mut self) {
    let dt = 1.0 / self.rate as f32;
    for aircraft in self.aircraft.iter_mut() {
      aircraft.update(dt);
    }

    self.broadcast_aircraft();
  }

  pub fn execute_command(&mut self, command: Command) {
    let aircraft = self.aircraft.iter_mut().find(|a| a.callsign == command.id);
    if let Some(aircraft) = aircraft {
      // TODO: Do go-around first (then filter it out from the rest of the tasks)
      for task in command.tasks {
        match task {
          Task::Land(runway) => {
            let target = self.runways.iter().find(|r| r.id == runway);
            if let Some(target) = target {
              aircraft.state = AircraftState::Landing(target.clone());
            }
          }
          Task::GoAround => aircraft.go_around(),
          Task::Altitude(alt) => aircraft.target.altitude = alt,
          Task::Heading(hdg) => aircraft.target.heading = hdg,
          Task::Speed(spd) => aircraft.target.speed = spd,
        }
      }
    }
  }
}
