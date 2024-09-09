use std::{
  io::Write,
  path::PathBuf,
  time::{Duration, Instant},
};

use async_channel::TryRecvError;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::structs::{
  Aircraft, AircraftState, AircraftUpdate, CommandWithFreq, Task, World,
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
    if let Some(aircraft) = Aircraft::random_to_arrive(&self.world) {
      self.world.aircraft.push(aircraft);
    }

    // TODO: update replies
    // let reply = if let AircraftIntention::Land = aircraft.intention {
    //   let heading = angle_between_points(airspace.pos, aircraft.pos);
    //   let direction = heading_to_direction(heading);

    //   Some(CommandReply {
    //     callsign: aircraft.callsign.clone(),
    //     kind: CommandReplyKind::AircraftArrivedInTowerAirspace {
    //       direction: direction.to_owned(),
    //     },
    //   })
    // } else {
    //   None
    // };
    // self
    //   .sender
    //   .try_broadcast(OutgoingReply::Reply(CommandWithFreq {
    //     id: aircraft.callsign.clone(),
    //     frequency: aircraft.frequency,
    //     reply: reply.map(|s| s.to_string()).unwrap_or_else(|| {
    //       "Error generating reply for spawned aircraft".to_owned()
    //     }),
    //     tasks: Vec::new(),
    //   }))
    //   .unwrap();
  }

  pub fn begin_loop(&mut self) {
    'main_loop: loop {
      if Instant::now() - self.last_tick
        >= Duration::from_secs_f32(1.0 / self.rate as f32)
      {
        self.last_tick = Instant::now();

        if self.world.aircraft.len() < 20
          && self.last_spawn.elapsed() >= Duration::from_secs(180)
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
      let result = aircraft.update(dt, &self.sender);
      match result {
        AircraftUpdate::None => {}
        AircraftUpdate::NewDeparture => {
          aircraft.departure_from_arrival(&self.world.airspaces)
        }
      }

      // TODO: switch this to find an airport with the name when we switch
      // automated flights to use real airports instead of empty airspaces
      if let Some(airspace) = self
        .world
        .airspaces
        .iter()
        .find(|a| a.id == aircraft.flight_plan.1)
      {
        if Some(airspace.id.clone()) == aircraft.airspace && airspace.auto {
          aircraft.state = AircraftState::Deleted;
        }
      }

      let airspace = self
        .world
        .airspaces
        .iter()
        .find(|a| a.contains_point(aircraft.pos));
      aircraft.airspace = airspace.map(|a| a.id.clone());
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
              aircraft.clear_waypoints();
              aircraft.target.heading = *hdg;
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
            Task::ResumeOwnNavigation => {
              let arrival = self
                .world
                .airspaces
                .iter()
                .find(|a| a.id == aircraft.flight_plan.1)
                .unwrap();
              aircraft.resume_own_navigation(arrival.pos);
              aircraft.clear_waypoints();
            }
            Task::Taxi(waypoints) => {
              if let Some(ref airport) = airport {
                aircraft.do_taxi(waypoints.clone(), &airport.pathfinder);
              }
            }
            Task::TaxiHold => aircraft.do_hold_taxi(false),
            Task::TaxiContinue => aircraft.do_continue_taxi(),
            Task::Direct(waypoint_str) => {
              if let Some(waypoints) = waypoint_str
                .iter()
                .map(|w| {
                  self.world.waypoints.iter().find(|n| &n.name == w).cloned()
                })
                .rev()
                .try_fold(Vec::new(), |mut vec, item| {
                  vec.push(item?);

                  Some(vec)
                })
              {
                aircraft.state = AircraftState::Flying { waypoints };
              } else {
                tracing::warn!("Bad waypoints: {:?}", waypoint_str);
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
