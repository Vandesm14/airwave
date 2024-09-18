use glam::Vec2;

use crate::{delta_angle, move_point, KNOT_TO_FEET_PER_SECOND, TIME_SCALE};

#[derive(Debug, Clone, PartialEq)]

pub enum Event {
  TargetSpeed(f32),
  TargetHeading(f32),
  TargetAltitude(f32),
}

#[derive(Debug, Clone, PartialEq)]

pub enum Action {
  Pos(Vec2),

  Speed(f32),
  Heading(f32),
  Altitude(f32),

  TargetSpeed(f32),
  TargetHeading(f32),
  TargetAltitude(f32),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct AircraftTargets {
  pub heading: f32,
  pub speed: f32,
  pub altitude: f32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Aircraft {
  pub pos: Vec2,
  pub speed: f32,
  pub heading: f32,
  pub altitude: f32,

  pub target: AircraftTargets,
}

// Consts
impl Aircraft {
  pub fn speed_in_feet(&self) -> f32 {
    self.speed * KNOT_TO_FEET_PER_SECOND
  }

  pub fn dt_climb_sp(&self, dt: f32) -> f32 {
    (2000.0_f32 / 60.0_f32).round() * dt
  }

  pub fn dt_turn_speed(&self, dt: f32) -> f32 {
    2.0 * dt
  }

  pub fn dt_speed_speed(&self, dt: f32) -> f32 {
    2.0 * dt
  }
}

// Handlers
impl Aircraft {
  pub fn handle_target_events(
    &self,
    events: &[Event],
    actions: &mut Vec<Action>,
  ) {
    for event in events {
      match event {
        Event::TargetSpeed(speed) => {
          actions.push(Action::TargetSpeed(*speed));
        }
        Event::TargetHeading(heading) => {
          actions.push(Action::TargetHeading(*heading));
        }
        Event::TargetAltitude(altitude) => {
          actions.push(Action::TargetAltitude(*altitude));
        }

        _ => {}
      }
    }
  }

  pub fn handle_all_events(&self, events: &[Event], actions: &mut Vec<Action>) {
    self.handle_target_events(events, actions);
  }
}

// Updaters
impl Aircraft {
  pub fn update_mains(&mut self, actions: &mut Vec<Action>, dt: f32) {
    // TODO: change speeds for takeoff and taxi (turn and speed speeds)

    // In feet per second
    let climb_speed = self.dt_climb_sp(dt);
    // In degrees per second
    let turn_speed = self.dt_turn_speed(dt);
    // In knots per second
    let speed_speed = self.dt_speed_speed(dt);

    if (self.altitude - self.target.altitude).abs() < climb_speed {
      self.altitude = self.target.altitude;
    }
    if (self.heading - self.target.heading).abs() < turn_speed {
      self.heading = self.target.heading;
    }
    if (self.speed - self.target.speed).abs() < speed_speed {
      self.speed = self.target.speed;
    }

    // Change based on speed if not equal
    if self.altitude != self.target.altitude {
      if self.altitude < self.target.altitude {
        self.altitude += climb_speed;
      } else {
        self.altitude -= climb_speed;
      }
    }
    if self.heading != self.target.heading {
      let delta_angle = delta_angle(self.heading, self.target.heading);
      if delta_angle < 0.0 {
        self.heading -= turn_speed;
      } else {
        self.heading += turn_speed;
      }
    }
    if self.speed != self.target.speed {
      if self.speed < self.target.speed {
        self.speed += speed_speed;
      } else {
        self.speed -= speed_speed;
      }
    }

    self.heading = (360.0 + self.heading) % 360.0;
  }

  pub fn update_position(&mut self, dt: f32) {
    let pos = move_point(self.pos, self.heading, self.speed_in_feet() * dt);
    self.pos = pos;
  }

  pub fn update_all(&mut self, actions: &mut Vec<Action>, dt: f32) {
    self.update_mains(actions, dt);
    self.update_position(dt);
  }
}

// Appliers
impl Aircraft {
  pub fn apply_targets(&mut self, actions: &[Action]) {
    for action in actions {
      match action {
        Action::TargetSpeed(speed) => self.target.speed = *speed,
        Action::TargetHeading(heading) => self.target.heading = *heading,
        Action::TargetAltitude(altitude) => self.target.altitude = *altitude,

        _ => {}
      }
    }
  }
  pub fn apply_mains(&mut self, actions: &[Action]) {
    for action in actions {
      match action {
        Action::Speed(speed) => self.speed = *speed,
        Action::Heading(heading) => self.heading = *heading,
        Action::Altitude(altitude) => self.altitude = *altitude,

        _ => {}
      }
    }
  }

  pub fn apply_actions(&mut self, actions: &[Action]) {
    self.apply_targets(actions);
    self.apply_mains(actions);
  }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Engine {
  pub aircraft: Vec<Aircraft>,
  pub events: Vec<Event>,
  pub actions: Vec<Action>,
}

impl Engine {
  pub fn tick(&mut self) {
    for aircraft in self.aircraft.iter_mut() {
      aircraft.handle_all_events(&self.events, &mut self.actions);
      aircraft.apply_actions(&self.actions);
      aircraft.update_all(&mut self.actions, 0.5);
    }

    self.events.clear();
    self.actions.clear();
  }
}
