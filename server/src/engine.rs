use std::time::Instant;

use crate::structs::{Aircraft, Command, Runway, Task};

#[derive(Debug, Clone, PartialEq)]
pub struct Engine {
  pub aircraft: Vec<Aircraft>,
  pub runways: Vec<Runway>,

  last_tick: Instant,
}

impl Engine {
  pub fn new() -> Self {
    Self {
      aircraft: Vec::new(),
      runways: Vec::new(),
      last_tick: Instant::now(),
    }
  }

  pub fn update(&mut self) {
    let dt = 1000.0 / 30.0;
    for aircraft in self.aircraft.iter_mut() {
      aircraft.update(dt);
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
