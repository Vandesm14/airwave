use std::{
  sync::mpsc::{self},
  time::{Duration, Instant},
};

use glam::Vec2;
use rand::{seq::SliceRandom, thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::{
  angle_between_points, degrees_to_heading, heading_to_direction,
  structs::{
    Aircraft, AircraftState, CommandWithFreq, Runway, Task, Taxiway, Terminal,
  },
};

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
  Taxiways(Vec<Taxiway>),
  Terminals(Vec<Terminal>),
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
  pub taxiways: Vec<Taxiway>,
  pub terminals: Vec<Terminal>,

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
      taxiways: Vec::new(),
      terminals: Vec::new(),

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
      let mut rng = thread_rng();
      let heading: f32 = rng.gen_range(1.0..36.0);
      let heading: f32 = heading.round() * 10.0;

      aircraft.state = AircraftState::WillDepart {
        runway: self.runways.choose(&mut rng).unwrap().clone(),
        heading,
      };
      aircraft.frequency = 118.6;
    }

    self.aircraft.push(aircraft.clone());
    let reply =
      if let AircraftState::WillDepart { runway, heading } = aircraft.state {
        format!(
          "Tower, {} is holding short of runway {}, departure to the {}.",
          aircraft.callsign,
          runway.id,
          heading_to_direction(heading)
        )
      } else if let AircraftState::Approach = aircraft.state {
        let center = Vec2::splat(self.airspace_size * 0.5);
        let heading =
          degrees_to_heading(angle_between_points(center, aircraft.pos));
        let direction = heading_to_direction(heading);

        format!(
          "Tower, {} is {} of the airport, with you.",
          aircraft.callsign, direction
        )
      } else {
        "Error generating reply for spawned aircraft".to_owned()
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

        if self.aircraft.len() < 1
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
    let _ = self
      .sender
      .send(OutgoingReply::Aircraft(self.aircraft.clone()))
      .inspect_err(|e| eprintln!("failed to broadcast aircraft: {}", e));
  }

  fn broadcast_runways(&self) {
    let _ = self
      .sender
      .send(OutgoingReply::Runways(self.runways.clone()))
      .inspect_err(|e| eprintln!("failed to broadcast runways: {}", e));
  }

  fn broadcast_taxiways(&self) {
    let _ = self
      .sender
      .send(OutgoingReply::Taxiways(self.taxiways.clone()))
      .inspect_err(|e| eprintln!("failed to broadcast taxiways: {}", e));
  }

  fn broadcast_terminals(&self) {
    let _ = self
      .sender
      .send(OutgoingReply::Terminals(self.terminals.clone()))
      .inspect_err(|e| eprintln!("failed to broadcast terminals: {}", e));
  }

  fn broadcast_size(&self) {
    let _ = self
      .sender
      .send(OutgoingReply::Size(self.airspace_size))
      .inspect_err(|e| eprintln!("failed to broadcast size: {}", e));
  }

  fn broadcast_for_new_client(&self) {
    self.broadcast_terminals();
    self.broadcast_taxiways();
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
      let went_around = aircraft.update(self.airspace_size, dt);
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
            Task::ResumeOwnNavigation => aircraft.resume_own_navigation(),
          }
        }

        self.sender.send(OutgoingReply::Reply(command)).unwrap();
      }
    }
  }
}
