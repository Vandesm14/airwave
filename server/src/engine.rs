use std::{
  sync::mpsc::{self},
  time::{Duration, Instant},
};

use rand::{seq::SliceRandom, thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::structs::{Aircraft, AircraftState, CommandWithFreq, Runway, Task};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type", content = "value")]
pub enum OutgoingReply {
  // Partial/Small Updates
  ATCReply(CommandWithFreq),
  Reply(CommandWithFreq),

  // Full State Updates
  Aircraft(Vec<Aircraft>),
  Runways(Vec<Runway>),
  Size(f32),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IncomingUpdate {
  Command(CommandWithFreq),
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
  default_frequency: f32,
  rate: usize,
}

impl Engine {
  pub fn new(
    receiver: mpsc::Receiver<IncomingUpdate>,
    sender: mpsc::Sender<OutgoingReply>,
    airspace_size: f32,
    default_frequency: f32,
  ) -> Self {
    Self {
      aircraft: Vec::new(),
      runways: Vec::new(),
      receiver,
      sender,

      last_tick: Instant::now(),
      last_spawn: Instant::now(),
      airspace_size,
      default_frequency,
      rate: 30,
    }
  }

  pub fn spawn_random_aircraft(&mut self) {
    let mut rng = thread_rng();
    let should_be_takeoff = rng.gen_ratio(1, 1);

    let mut aircraft =
      Aircraft::random(self.airspace_size, self.default_frequency);

    if should_be_takeoff {
      aircraft.state = AircraftState::WillDepart(
        self.runways.choose(&mut rng).unwrap().clone(),
      )
    }

    self.aircraft.push(aircraft.clone());
    let reply = if let AircraftState::WillDepart(runway) = aircraft.state {
      format!(
        "Tower, {} is holding short of runway {}.",
        aircraft.callsign, runway.id
      )
    } else {
      format!(
        "Tower, {} is at {} feet, with you.",
        aircraft.callsign, aircraft.altitude
      )
    };
    self
      .sender
      .send(OutgoingReply::Reply(CommandWithFreq {
        id: aircraft.callsign.clone(),
        frequency: aircraft.frequency,
        reply,
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

        let mut commands: Vec<CommandWithFreq> = Vec::new();
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
      let went_around = aircraft.update(dt);
      if went_around {
        self
          .sender
          .send(OutgoingReply::Reply(CommandWithFreq {
            id: aircraft.callsign.clone(),
            frequency: aircraft.frequency,
            reply: format!("Tower, {} is going around.", aircraft.callsign),
            tasks: Vec::new(),
          }))
          .unwrap();
      }
    }
  }

  pub fn execute_command(&mut self, command: CommandWithFreq) {
    let aircraft = self.aircraft.iter_mut().find(|a| a.callsign == command.id);
    if let Some(aircraft) = aircraft {
      if aircraft.frequency == command.frequency {
        // TODO: Do go-around first (then filter it out from the rest of the tasks)
        for task in command.tasks.iter() {
          match task {
            Task::Altitude(alt) => aircraft.target.altitude = *alt,
            Task::Heading(hdg) => aircraft.target.heading = *hdg,
            Task::Speed(spd) => aircraft.target.speed = *spd,
            Task::Frequency(frq) => aircraft.frequency = *frq,
            Task::Land(runway) => {
              let target = self.runways.iter().find(|r| &r.id == runway);
              if let Some(target) = target {
                aircraft.state = AircraftState::Landing(target.clone());
              }
            }
            Task::GoAround => aircraft.do_go_around(),
            Task::Takeoff => aircraft.do_takeoff(),
          }
        }

        self.sender.send(OutgoingReply::Reply(command)).unwrap();
      }
    }
  }
}
