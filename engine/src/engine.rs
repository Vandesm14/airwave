use std::{
  io::Write,
  path::PathBuf,
  time::{Duration, Instant},
};

use async_channel::TryRecvError;
use glam::Vec2;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{
  angle_between_points, delta_angle, heading_to_direction,
  pathfinder::Node,
  structs::{
    Aircraft, AircraftIntention, AircraftState, CommandWithFreq, Task, World,
  },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type", content = "value")]
pub enum OutgoingReply {
  // Partial/Small Updates
  ATCReply(CommandWithFreq),
  Reply(CommandWithFreq),

  // Full State Updates
  Aircraft(Vec<Aircraft>),
  World(World),
  Size(f32),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IncomingUpdate {
  Command(CommandWithFreq),
  Connect,
}

#[derive(Debug)]
pub struct Engine {
  pub world: World,

  pub receiver: async_channel::Receiver<IncomingUpdate>,
  pub sender: async_broadcast::Sender<OutgoingReply>,

  pub save_to: Option<PathBuf>,

  last_tick: Instant,
  last_spawn: Instant,
  rate: usize,
}

impl Engine {
  pub fn new(
    receiver: async_channel::Receiver<IncomingUpdate>,
    sender: async_broadcast::Sender<OutgoingReply>,
    save_to: Option<PathBuf>,
  ) -> Self {
    Self {
      world: World::default(),

      receiver,
      sender,

      save_to,

      last_tick: Instant::now(),
      last_spawn: Instant::now(),
      rate: 10,
    }
  }

  pub fn spawn_random_aircraft(&mut self) {
    // TODO: spawn aircraft
    let airspace = self.world.airspaces.first().unwrap();

    // TODO: don't hard-code this frequency
    let aircraft = Aircraft::random_to_land(airspace, 118.5);
    self.world.aircraft.push(aircraft.clone());

    // TODO: update replies
    let reply = if let AircraftIntention::Land = aircraft.intention {
      let center = Vec2::splat(airspace.size);
      let heading = angle_between_points(center, aircraft.pos);
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
      .try_broadcast(OutgoingReply::Reply(CommandWithFreq {
        id: aircraft.callsign.clone(),

        frequency: aircraft.frequency,
        reply,
        tasks: Vec::new(),
      }))
      .unwrap();
  }

  pub fn begin_loop(&mut self) {
    'main_loop: loop {
      if Instant::now() - self.last_tick
        >= Duration::from_secs_f32(1.0 / self.rate as f32)
      {
        self.last_tick = Instant::now();

        if self.world.aircraft.len() < 10
          && self.last_spawn.elapsed() >= Duration::from_secs(150)
        {
          self.last_spawn = Instant::now();
          self.spawn_random_aircraft();
        }

        let mut commands: Vec<CommandWithFreq> = Vec::new();

        loop {
          let incoming = match self.receiver.try_recv() {
            Ok(incoming) => incoming,
            Err(TryRecvError::Closed) => break 'main_loop,
            Err(TryRecvError::Empty) => break,
          };

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
        self.save_world();
      }
    }
  }

  pub fn save_world(&self) {
    if let Some(path) = &self.save_to {
      let world: World = self.world.clone();

      let string = serde_json::ser::to_string(&world);
      match string {
        Ok(string) => {
          // Make the directory if it doesn't exist
          let _ = std::fs::create_dir_all(path.parent().unwrap());
          let mut file = std::fs::File::create(path).unwrap();
          file.write_all(string.as_bytes()).unwrap();
        }
        Err(e) => {
          error!("failed to save world: {}", e);
        }
      }
    }
  }

  fn broadcast_aircraft(&self) {
    let _ = self
      .sender
      .try_broadcast(OutgoingReply::Aircraft(self.world.aircraft.clone()))
      .inspect_err(|e| tracing::warn!("failed to broadcast aircraft: {}", e));
  }

  fn broadcast_world(&self) {
    let _ = self
      .sender
      .try_broadcast(OutgoingReply::World(self.world.clone()))
      .inspect_err(|e| tracing::warn!("failed to broadcast world: {}", e));
  }

  fn broadcast_for_new_client(&self) {
    self.broadcast_world();
  }

  pub fn cleanup(&mut self) {
    let mut indicies: Vec<usize> = Vec::new();
    for (i, aircraft) in self.world.aircraft.iter().enumerate() {
      if matches!(aircraft.state, AircraftState::Deleted) {
        indicies.push(i);
      }
    }

    for index in indicies {
      self.world.aircraft.swap_remove(index);
    }
  }

  pub fn update(&mut self) {
    let dt = 1.0 / self.rate as f32;
    for aircraft in self.world.aircraft.iter_mut() {
      aircraft.update(dt, &self.sender);

      for airspace in &self.world.airspaces {
        if !airspace.contains_point(aircraft.pos) {
          aircraft.state = AircraftState::Deleted;
        }
      }
    }
  }

  pub fn execute_command(&mut self, command: CommandWithFreq) {
    let aircraft = self
      .world
      .aircraft
      .iter()
      .find(|a| a.callsign == command.id);
    // TODO: Cloning isn't great but yet again this is a "you can't reference
    // the thing you're referencing twice even though you're accessing different
    // fields".
    let airport = aircraft
      .and_then(|a| self.world.closest_airport(a.pos))
      .cloned();

    let aircraft = self
      .world
      .aircraft
      .iter_mut()
      .find(|a| a.callsign == command.id);
    if let Some(aircraft) = aircraft {
      if aircraft.frequency == command.frequency {
        // TODO: Do go-around first (then filter it out from the rest of the tasks)
        for task in command.tasks.iter() {
          match task {
            Task::Altitude(alt) => aircraft.target.altitude = *alt,
            Task::Heading(hdg) => {
              aircraft.target.heading = *hdg;

              if let AircraftState::HoldingPattern(_) = &aircraft.state {
                aircraft.state = AircraftState::Flying {
                  waypoints: Vec::new(),
                };
              }
            }
            Task::Speed(spd) => aircraft.target.speed = *spd,
            Task::Frequency(frq) => aircraft.frequency = *frq,
            Task::Land(runway) => {
              if let Some(ref airport) = airport {
                let target = airport.runways.iter().find(|r| &r.id == runway);
                if let Some(target) = target {
                  aircraft.state = AircraftState::Landing(target.clone());
                }
              } else {
                // TODO: broadcast reply for no airport
                tracing::warn!("no airport found for {}", aircraft.callsign);
              }
            }
            Task::GoAround => aircraft
              .do_go_around(&self.sender, crate::structs::GoAroundReason::None),
            Task::Takeoff(runway) => {
              if let Some(ref airport) = airport {
                let target = airport.runways.iter().find(|r| &r.id == runway);
                if let Some(target) = target {
                  aircraft.do_takeoff(target);
                }
              }
            }
            Task::ResumeOwnNavigation => aircraft.resume_own_navigation(),
            Task::Taxi(waypoints) => {
              if let Some(ref airport) = airport {
                aircraft.do_taxi(waypoints.clone(), &airport.pathfinder);
              }
            }
            Task::TaxiHold => aircraft.do_hold_taxi(false),
            Task::TaxiContinue => aircraft.do_continue_taxi(),
            Task::Direct(waypoint) => {
              if let Some(ref airport) = airport {
                if let Some(node) =
                  airport.waypoints.iter().find(|w| &w.name == waypoint)
                {
                  aircraft.target.heading =
                    angle_between_points(aircraft.pos, node.value);
                }
              }
            }
          }
        }

        self
          .sender
          .try_broadcast(OutgoingReply::Reply(command))
          .unwrap();
      }
    }
  }
}
