use std::{
  sync::mpsc::{self},
  time::{Duration, Instant},
};

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::{
  angle_between_points, degrees_to_heading, heading_to_direction,
  structs::{
    Aircraft, AircraftIntention, AircraftState, CommandWithFreq, Runway, Task,
    TaxiPoint, TaxiWaypoint, TaxiWaypointBehavior, Taxiway, TaxiwayKind,
    Terminal,
  },
  FEET_PER_UNIT,
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

  pub fn add_taxiway(&mut self, taxiway: Taxiway) {
    let taxiway = taxiway.extend_ends_by(FEET_PER_UNIT * 100.0);
    self.taxiways.push(taxiway);
  }

  pub fn spawn_random_aircraft(&mut self) {
    let aircraft =
      Aircraft::random_to_land(self.airspace_size, self.default_frequency);
    self.aircraft.push(aircraft.clone());

    // TODO: update replies
    let reply = if let AircraftIntention::Land = aircraft.intention {
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

        if self.aircraft.len() < 6
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
      aircraft.update(self.airspace_size, dt, &self.sender);
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
            Task::GoAround => aircraft
              .do_go_around(&self.sender, crate::structs::GoAroundReason::None),
            Task::Takeoff(runway) => {
              let target = self.runways.iter().find(|r| &r.id == runway);
              if let Some(target) = target {
                aircraft.do_takeoff(target);
              }
            }
            Task::ResumeOwnNavigation => aircraft.resume_own_navigation(),
            Task::TaxiRunway {
              runway: runway_str,
              waypoints: waypoints_str,
            } => {
              if let AircraftState::Taxiing { .. } = &mut aircraft.state {
                let runway = self.runways.iter().find(|r| r.id == *runway_str);
                let hold_short_taxiway = runway.and_then(|r| {
                  self.taxiways.iter().find(|t| {
                    if let TaxiwayKind::HoldShort(rw) = &t.kind {
                      rw == &r.id
                    } else {
                      false
                    }
                  })
                });
                if let Some((runway, hold_short_taxiway)) =
                  runway.zip(hold_short_taxiway)
                {
                  let mut taxi_instructions: Vec<TaxiWaypoint> = vec![
                    TaxiWaypoint {
                      pos: Vec2::default(),
                      wp: TaxiPoint::Runway(runway.clone()),
                      behavior: TaxiWaypointBehavior::HoldShort,
                    },
                    TaxiWaypoint {
                      pos: Vec2::default(),
                      wp: TaxiPoint::Taxiway(hold_short_taxiway.clone()),
                      behavior: TaxiWaypointBehavior::GoTo,
                    },
                  ];

                  for (instruction, hold) in waypoints_str.iter().rev() {
                    let taxiway =
                      self.taxiways.iter().find(|t| t.id == *instruction);
                    if let Some(taxiway) = taxiway {
                      taxi_instructions.push(TaxiWaypoint {
                        pos: Vec2::default(),
                        wp: TaxiPoint::Taxiway(taxiway.clone()),
                        behavior: if *hold {
                          TaxiWaypointBehavior::HoldShort
                        } else {
                          TaxiWaypointBehavior::GoTo
                        },
                      });
                    }
                  }

                  taxi_instructions.reverse();
                  aircraft.do_taxi(taxi_instructions);
                }
              }
            }
            Task::TaxiGate {
              gate: gate_str,
              waypoints: waypoints_str,
            } => {
              let terminal = self
                .terminals
                .iter()
                .find(|t| t.id == gate_str.chars().next().unwrap());
              let gate = terminal
                .and_then(|t| t.gates.iter().find(|g| g.id == *gate_str));

              if let Some((terminal, gate)) = terminal.zip(gate) {
                let mut taxi_instructions: Vec<TaxiWaypoint> =
                  vec![TaxiWaypoint {
                    pos: Vec2::default(),
                    wp: TaxiPoint::Gate(terminal.clone(), gate.clone()),
                    behavior: TaxiWaypointBehavior::GoTo,
                  }];

                for (instruction, hold) in waypoints_str.iter().rev() {
                  let taxiway =
                    self.taxiways.iter().find(|t| t.id == *instruction);
                  if let Some(taxiway) = taxiway {
                    taxi_instructions.push(TaxiWaypoint {
                      pos: Vec2::default(),
                      wp: TaxiPoint::Taxiway(taxiway.clone()),
                      behavior: if *hold {
                        TaxiWaypointBehavior::HoldShort
                      } else {
                        TaxiWaypointBehavior::GoTo
                      },
                    });
                  }
                }

                taxi_instructions.reverse();
                aircraft.do_taxi(taxi_instructions);
              }
            }
            Task::TaxiHold => aircraft.do_hold_taxi(false),
            Task::TaxiContinue => aircraft.do_continue_taxi(),
          }
        }

        self.sender.send(OutgoingReply::Reply(command)).unwrap();
      }
    }
  }
}
