use std::{
  sync::mpsc::{self},
  time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};

use crate::structs::{Aircraft, Command, Runway, Task};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StateUpdate {
  Aircraft(Aircraft),
  Runway(Runway),
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
  pub sender: mpsc::Sender<StateUpdate>,

  last_tick: Instant,
}

impl Engine {
  pub fn new(
    receiver: mpsc::Receiver<IncomingUpdate>,
    sender: mpsc::Sender<StateUpdate>,
  ) -> Self {
    Self {
      aircraft: Vec::new(),
      runways: Vec::new(),
      receiver,
      sender,

      last_tick: Instant::now(),
    }
  }

  pub fn begin_loop(&mut self) {
    loop {
      if Instant::now() - self.last_tick >= Duration::from_secs_f32(1.0 / 5.0) {
        self.last_tick = Instant::now();

        for incoming in self.receiver.iter() {
          match incoming {
            IncomingUpdate::Command(_) => todo!("command"),
            IncomingUpdate::Connect => self.broadcast_all(),
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
    for aircraft in self.aircraft.iter() {
      self
        .sender
        .send(StateUpdate::Aircraft(aircraft.clone()))
        .unwrap();
    }
  }

  fn broadcast_runways(&self) {
    for runway in self.runways.iter() {
      self
        .sender
        .send(StateUpdate::Runway(runway.clone()))
        .unwrap();
    }
  }

  pub fn update(&mut self) {
    let dt = 1000.0 / 30.0;
    for aircraft in self.aircraft.iter_mut() {
      aircraft.update(dt);

      self
        .sender
        .send(StateUpdate::Aircraft(aircraft.clone()))
        .unwrap();
    }
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
              aircraft.target.runway = Some(target.clone());
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
