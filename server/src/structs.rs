use std::time::{Duration, SystemTime};

use glam::Vec2;
use rand::{seq::SliceRandom, thread_rng, Rng};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{
  angle_between_points, degrees_to_heading, delta_angle,
  get_random_point_on_circle, heading_to_degrees, inverse_degrees, move_point,
  FEET_PER_UNIT, KNOT_TO_FEET_PER_SECOND, NAUTICALMILES_TO_FEET, TIME_SCALE,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type", content = "value")]
pub enum Task {
  Land(String),
  #[serde(rename = "go-around")]
  GoAround,
  Altitude(f32),
  Heading(f32),
  Speed(f32),
  Frequency(f32),
  Takeoff,
  #[serde(rename = "resume")]
  ResumeOwnNavigation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Command {
  pub id: String,
  pub reply: String,
  pub tasks: Vec<Task>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandWithFreq {
  pub id: String,
  pub frequency: f32,
  pub reply: String,
  pub tasks: Vec<Task>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct XY {
  pub x: f32,
  pub y: f32,
}

fn serialize_vec2<S>(pos: &Vec2, serializer: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  XY { x: pos.x, y: pos.y }.serialize(serializer)
}

fn deserialize_vec2<'de, D>(deserializer: D) -> Result<Vec2, D::Error>
where
  D: Deserializer<'de>,
{
  let xy = XY::deserialize(deserializer)?;

  Ok(Vec2::new(xy.x, xy.y))
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Runway {
  pub id: String,
  #[serde(flatten)]
  #[serde(serialize_with = "serialize_vec2")]
  #[serde(deserialize_with = "deserialize_vec2")]
  pub pos: Vec2,
  pub heading: f32,
  pub length: f32,
}

impl Runway {
  pub fn start(&self) -> Vec2 {
    move_point(
      self.pos,
      inverse_degrees(heading_to_degrees(self.heading)),
      self.length * FEET_PER_UNIT * 0.5,
    )
  }

  pub fn end(&self) -> Vec2 {
    move_point(
      self.pos,
      heading_to_degrees(self.heading),
      self.length * FEET_PER_UNIT * 0.5,
    )
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type", content = "value")]
pub enum AircraftState {
  Approach,
  Landing(Runway),
  WillDepart { runway: Runway, heading: f32 },
  Departing(f32),

  Deleted,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AircraftTargets {
  pub heading: f32,
  pub speed: f32,
  pub altitude: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Aircraft {
  pub callsign: String,

  pub is_colliding: bool,
  pub state: AircraftState,

  #[serde(flatten)]
  #[serde(serialize_with = "serialize_vec2")]
  #[serde(deserialize_with = "deserialize_vec2")]
  pub pos: Vec2,
  pub heading: f32,
  pub speed: f32,
  pub altitude: f32,
  pub frequency: f32,

  pub target: AircraftTargets,
  pub created: u128,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Taxiway {
  pub id: String,
  #[serde(serialize_with = "serialize_vec2")]
  #[serde(deserialize_with = "deserialize_vec2")]
  pub a: Vec2,
  #[serde(serialize_with = "serialize_vec2")]
  #[serde(deserialize_with = "deserialize_vec2")]
  pub b: Vec2,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type", content = "value")]
pub enum TaxiwayKind {
  Normal(Taxiway),
  HoldShort(Taxiway),
  Apron(Taxiway),
}

impl Aircraft {
  pub fn random(airspace_size: f32, frequency: f32) -> Self {
    let airspace_center = Vec2::splat(airspace_size * 0.5);
    let point =
      get_random_point_on_circle(airspace_center, airspace_size * 0.5);

    Self {
      callsign: Self::random_callsign(),
      is_colliding: false,
      state: AircraftState::Approach,
      pos: point.position,
      heading: degrees_to_heading(angle_between_points(
        point.position,
        airspace_center,
      )),
      speed: 250.0,
      altitude: 7000.0,
      frequency,
      target: AircraftTargets {
        heading: degrees_to_heading(angle_between_points(
          point.position,
          airspace_center,
        )),
        speed: 250.0,
        altitude: 7000.0,
      },
      created: SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::from_millis(0))
        .as_millis(),
    }
  }

  pub fn random_callsign() -> String {
    let mut string = String::new();
    let airlines = ["AAL", "SKW", "JBL"];

    let mut rng = thread_rng();
    let airline = airlines.choose(&mut rng).unwrap();

    string.push_str(airline);

    string.push_str(&rng.gen_range(0..=9).to_string());
    string.push_str(&rng.gen_range(0..=9).to_string());
    string.push_str(&rng.gen_range(0..=9).to_string());
    string.push_str(&rng.gen_range(0..=9).to_string());

    string
  }

  pub fn speed_in_pixels(&self) -> f32 {
    self.speed * KNOT_TO_FEET_PER_SECOND * FEET_PER_UNIT
  }

  pub fn do_go_around(&mut self) {
    if let AircraftState::Landing(_) = &self.state {
      if self.target.speed < 250.0 {
        self.target.speed = 250.0;
      }

      if self.target.altitude < 3000.0 {
        self.target.altitude = 3000.0;
      }
    }

    self.state = AircraftState::Approach;
  }

  pub fn do_takeoff(&mut self) {
    if let AircraftState::WillDepart { heading, .. } = &self.state {
      self.state = AircraftState::Departing(*heading);
    }
  }

  pub fn resume_own_navigation(&mut self) {
    if let AircraftState::Departing(heading) = &self.state {
      self.target.heading = *heading;
      self.target.speed = 400.0;
      self.target.altitude = 13000.;
    }
  }

  fn update_takeoff(&mut self) {
    if let AircraftState::WillDepart { runway, .. } = &self.state {
      self.pos = runway.start();

      self.heading = runway.heading;
      self.target.heading = runway.heading;

      self.speed = 170.0;
      self.target.speed = 250.0;

      self.altitude = 500.0;
      self.target.altitude = 5000.0;
    }
  }

  fn update_position(&mut self, dt: f32) {
    let pos = move_point(
      self.pos,
      heading_to_degrees(self.heading),
      self.speed_in_pixels() * dt,
    );
    self.pos = pos;
  }

  fn update_ils(&mut self) -> bool {
    if let AircraftState::Landing(runway) = &self.state {
      let delta_angle = delta_angle(
        angle_between_points(runway.start(), self.pos),
        inverse_degrees(heading_to_degrees(runway.heading)),
      );

      let distance_to_runway = self.pos.distance_squared(runway.start());
      let start_decrease_altitude = NAUTICALMILES_TO_FEET * FEET_PER_UNIT * 6.0;
      let start_decrease_speed = NAUTICALMILES_TO_FEET * FEET_PER_UNIT * 6.0;

      // If we are on approach to the runway
      if delta_angle.abs() <= 5.0 {
        let turn_amount = 30.0_f32.min(delta_angle.abs() * 6.0);

        // If we have passed the threshold for 4000 feet, go around
        if self.altitude > 4000.0
          && distance_to_runway <= start_decrease_altitude.powf(2.0)
        {
          self.do_go_around();
          return true;
        } else if distance_to_runway <= start_decrease_altitude.powf(2.0) {
          self.target.altitude = 0.0;
        }

        // If we are inline with the runway, decrease speed
        if delta_angle.abs().round() == 0.0
          && distance_to_runway <= start_decrease_speed.powf(2.0)
        {
          self.target.speed = 170.0;
        }
        if delta_angle < 0.0 {
          self.target.heading = runway.heading + turn_amount;
        } else if delta_angle > 0.0 {
          self.target.heading = runway.heading - turn_amount;
        }
        // Else, if we aren't on approach, check if we have landed
      } else if delta_angle.abs().round() == 180.0 && self.altitude == 0.0 {
        self.state = AircraftState::Deleted;
      }
    }

    false
  }

  fn update_leave_airspace(&mut self, airspace_size: f32) {
    let airspace_center = Vec2::splat(airspace_size * 0.5);
    let distance = self.pos.distance_squared(airspace_center);
    let max_distance = (airspace_size * 0.5).powf(2.0);

    if distance >= max_distance {
      self.state = AircraftState::Deleted;
    }
  }

  fn update_targets(&mut self, dt: f32) {
    // Don't update aircraft waiting to depart
    if matches!(self.state, AircraftState::WillDepart { .. }) {
      return;
    }

    // In feet per second
    let climb_speed = TIME_SCALE * (2000.0_f32 / 60.0_f32).round() * dt;
    // In degrees per second
    let turn_speed = TIME_SCALE * 2.0 * dt;
    // In knots per second
    let speed_speed = TIME_SCALE * 1.0 * dt;

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

  pub fn update(&mut self, airspace_size: f32, dt: f32) -> bool {
    self.update_takeoff();

    let went_around = self.update_ils();
    self.update_targets(dt);
    self.update_position(dt);
    self.update_leave_airspace(airspace_size);

    went_around
  }
}
