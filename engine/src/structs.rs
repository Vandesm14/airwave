use std::{
  ops::Add,
  sync::mpsc::Sender,
  time::{Duration, SystemTime},
};

use glam::Vec2;
use rand::{seq::SliceRandom, thread_rng, Rng};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{
  add_degrees, angle_between_points, closest_point_on_line, delta_angle,
  engine::OutgoingReply, find_line_intersection, get_random_point_on_circle,
  inverse_degrees, move_point, KNOT_TO_FEET_PER_SECOND, NAUTICALMILES_TO_FEET,
  TIME_SCALE,
};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct World {
  pub airspaces: Vec<Airspace>,
  pub aircraft: Vec<Aircraft>,
}

impl World {
  pub fn closest_airport(&self, point: Vec2) -> Option<&Airport> {
    let mut closest: Option<&Airport> = None;
    let mut distance = f32::MAX;
    for airspace in self.airspaces.iter().filter(|a| a.contains_point(point)) {
      for airport in airspace.airports.iter() {
        if airport.center.distance_squared(point) < distance {
          distance = airport.center.distance_squared(point);
          closest = Some(airport);
        }
      }
    }

    closest
  }
}

// TODO: Support non-circular (regional) airspaces
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Airspace {
  pub id: String,
  #[serde(serialize_with = "serialize_vec2")]
  #[serde(deserialize_with = "deserialize_vec2")]
  pub pos: Vec2,
  pub size: f32,
  pub airports: Vec<Airport>,
}

impl Airspace {
  pub fn contains_point(&self, point: Vec2) -> bool {
    let distance = point.distance_squared(self.pos);
    distance <= self.size.powf(2.0)
  }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Airport {
  pub id: String,
  #[serde(serialize_with = "serialize_vec2")]
  #[serde(deserialize_with = "deserialize_vec2")]
  pub center: Vec2,
  pub runways: Vec<Runway>,
  pub taxiways: Vec<Taxiway>,
  pub terminals: Vec<Terminal>,
  pub altitude_range: [f32; 2],
}

impl Airport {
  pub fn new(id: String, center: Vec2) -> Self {
    Self {
      id,
      center,
      runways: Vec::new(),
      taxiways: Vec::new(),
      terminals: Vec::new(),
      altitude_range: [0.0, 0.0],
    }
  }

  pub fn add_taxiway(&mut self, taxiway: Taxiway) {
    let taxiway = taxiway.extend_ends_by(100.0);
    self.taxiways.push(taxiway);
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Line(pub Vec2, pub Vec2);

impl Line {
  pub fn new(a: Vec2, b: Vec2) -> Self {
    Self(a, b)
  }
}

impl From<Runway> for Line {
  fn from(value: Runway) -> Self {
    let inverse_angle = inverse_degrees(value.heading);
    let half_length = value.length * 0.5;

    let a = move_point(value.pos, value.heading, half_length);
    let b = move_point(value.pos, inverse_angle, half_length);

    Line::new(a, b)
  }
}

impl From<Taxiway> for Line {
  fn from(value: Taxiway) -> Self {
    Line::new(value.a, value.b)
  }
}

impl From<Terminal> for Line {
  fn from(value: Terminal) -> Self {
    // TODO: This means that terminals can only have one enterance, AB

    Line::new(value.a, value.b)
  }
}

impl From<TaxiPoint> for Line {
  fn from(value: TaxiPoint) -> Self {
    match value {
      TaxiPoint::Taxiway(x) => x.into(),
      TaxiPoint::Runway(x) => x.into(),
      TaxiPoint::Gate(x, _) => x.into(),
    }
  }
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HoldDirection {
  Right,
  Left,
}

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
  Takeoff(String),
  #[serde(rename = "resume")]
  ResumeOwnNavigation,

  #[serde(rename = "taxi-runway")]
  TaxiRunway {
    runway: String,
    waypoints: Vec<(String, bool)>,
  },
  #[serde(rename = "taxi-gate")]
  TaxiGate {
    gate: String,
    waypoints: Vec<(String, bool)>,
  },
  #[serde(rename = "taxi-hold")]
  TaxiHold,
  #[serde(rename = "taxi-continue")]
  TaxiContinue,

  #[serde(rename = "hold")]
  HoldPattern(HoldDirection),
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
    move_point(self.pos, inverse_degrees(self.heading), self.length * 0.5)
  }

  pub fn end(&self) -> Vec2 {
    move_point(self.pos, self.heading, self.length * 0.5)
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type", content = "value")]
pub enum TaxiWaypointBehavior {
  GoTo,
  HoldShort,
  TakeOff,
  LineUp,
  Park,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaxiWaypoint {
  #[serde(serialize_with = "serialize_vec2")]
  #[serde(deserialize_with = "deserialize_vec2")]
  pub pos: Vec2,
  pub wp: TaxiPoint,
  pub behavior: TaxiWaypointBehavior,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type", content = "value")]
pub enum TaxiPoint {
  Taxiway(Taxiway),
  Runway(Runway),
  Gate(Terminal, Gate),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type", content = "value")]
pub enum AircraftState {
  Flying,
  Landing(Runway),
  HoldingPattern(HoldDirection),
  Taxiing {
    current: TaxiWaypoint,
    waypoints: Vec<TaxiWaypoint>,
  },
  TakingOff(Runway),

  Deleted,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type", content = "value")]
pub enum AircraftIntention {
  Land,
  Flyover,
  Depart { has_notified: bool, heading: f32 },
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
  pub intention: AircraftIntention,
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
#[serde(rename_all = "lowercase")]
#[serde(tag = "type", content = "value")]
pub enum TaxiwayKind {
  Normal,
  HoldShort(String),
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

  pub kind: TaxiwayKind,
}

impl Taxiway {
  pub fn new(id: String, a: Vec2, b: Vec2, kind: TaxiwayKind) -> Self {
    Self { id, a, b, kind }
  }

  pub fn extend_ends_by(mut self, padding: f32) -> Self {
    self.a = self.a.move_towards(self.b, -padding);
    self.b = self.b.move_towards(self.a, -padding);

    self
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Gate {
  pub id: String,
  #[serde(serialize_with = "serialize_vec2")]
  #[serde(deserialize_with = "deserialize_vec2")]
  pub pos: Vec2,
  pub heading: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GoAroundReason {
  TooHigh,
  WrongAngle,

  None,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Terminal {
  pub id: char,
  #[serde(serialize_with = "serialize_vec2")]
  #[serde(deserialize_with = "deserialize_vec2")]
  pub a: Vec2,
  #[serde(serialize_with = "serialize_vec2")]
  #[serde(deserialize_with = "deserialize_vec2")]
  pub b: Vec2,
  #[serde(serialize_with = "serialize_vec2")]
  #[serde(deserialize_with = "deserialize_vec2")]
  pub c: Vec2,
  #[serde(serialize_with = "serialize_vec2")]
  #[serde(deserialize_with = "deserialize_vec2")]
  pub d: Vec2,

  pub gates: Vec<Gate>,
}

impl Aircraft {
  pub fn random_to_land(airspace: &Airspace, frequency: f32) -> Self {
    let point = get_random_point_on_circle(airspace.pos, airspace.size);

    Self {
      callsign: Self::random_callsign(),
      is_colliding: false,
      intention: AircraftIntention::Land,
      state: AircraftState::Flying,
      pos: point.position,
      heading: angle_between_points(point.position, airspace.pos),
      speed: 250.0,
      altitude: 7000.0,
      frequency,
      target: AircraftTargets {
        heading: angle_between_points(point.position, airspace.pos),
        speed: 250.0,
        altitude: 7000.0,
      },
      created: SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::from_millis(0))
        .as_millis(),
    }
  }

  pub fn random_to_depart(
    frequency: f32,
    terminal: Terminal,
    gates: Vec<Gate>,
  ) -> Self {
    let mut rng = thread_rng();
    let gate = gates.choose(&mut rng).unwrap();
    Self {
      callsign: Self::random_callsign(),
      is_colliding: false,
      intention: AircraftIntention::Depart {
        has_notified: false,
        heading: rng.gen_range(0.0_f32..36.0).round() * 10.0,
      },
      state: AircraftState::Taxiing {
        current: TaxiWaypoint {
          pos: gate.pos,
          wp: TaxiPoint::Gate(terminal.clone(), gate.clone()),
          behavior: TaxiWaypointBehavior::GoTo,
        },
        waypoints: Vec::new(),
      },
      pos: gate.pos,
      heading: 0.0,
      speed: 0.0,
      altitude: 0.0,
      frequency,
      target: AircraftTargets {
        heading: 0.0,
        speed: 0.0,
        altitude: 0.0,
      },
      created: SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::from_millis(0))
        .as_millis(),
    }
  }

  pub fn departure_from_arrival(&mut self) {
    let mut rng = thread_rng();
    self.intention = AircraftIntention::Depart {
      has_notified: false,
      heading: rng.gen_range(0.0_f32..36.0).round() * 10.0,
    };
    self.created = SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH)
      .unwrap_or(Duration::from_millis(0))
      .add(Duration::from_secs(rng.gen_range(60..=180)))
      .as_millis();
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
    self.speed * KNOT_TO_FEET_PER_SECOND
  }

  pub fn do_hold_pattern(&mut self, direction: HoldDirection) {
    if let AircraftState::Flying = self.state {
      self.state = AircraftState::HoldingPattern(direction);
    }
  }

  pub fn do_go_around(
    &mut self,
    sender: &Sender<OutgoingReply>,
    reason: GoAroundReason,
  ) {
    if let AircraftState::Landing(_) = &self.state {
      if self.target.speed < 250.0 {
        self.target.speed = 250.0;
      }

      if self.target.altitude < 3000.0 {
        self.target.altitude = 3000.0;
      }
    }

    self.state = AircraftState::Flying;

    let text = match reason {
      GoAroundReason::TooHigh => {
        format!("Tower, {} is going around, too high.", self.callsign)
      }
      GoAroundReason::WrongAngle => {
        format!("Tower, {} is going around, missed approach.", self.callsign)
      }
      GoAroundReason::None => return,
    };
    sender
      .send(OutgoingReply::Reply(CommandWithFreq {
        id: self.callsign.clone(),
        frequency: self.frequency,
        reply: text,
        tasks: Vec::new(),
      }))
      .unwrap()
  }

  pub fn do_takeoff(&mut self, runway: &Runway) {
    self.pos = runway.start();
    self.heading = runway.heading;
    self.target.heading = runway.heading;

    self.state = AircraftState::TakingOff(runway.clone());
  }

  pub fn do_taxi(&mut self, blank_waypoints: Vec<TaxiWaypoint>) {
    if let AircraftState::Taxiing {
      waypoints,
      current: current_pos,
      ..
    } = &mut self.state
    {
      let mut current = current_pos.clone();
      let mut new_waypoints: Vec<TaxiWaypoint> = Vec::new();
      for mut waypoint in blank_waypoints.into_iter() {
        let current_line: Line = current.wp.clone().into();
        let waypoint_line: Line = waypoint.wp.clone().into();
        let intersection = find_line_intersection(
          current_line.0,
          current_line.1,
          waypoint_line.0,
          waypoint_line.1,
        );

        if let Some(intersection) = intersection {
          waypoint.pos = intersection;

          if let TaxiWaypointBehavior::HoldShort = waypoint.behavior {
            let angle = angle_between_points(waypoint.pos, current.pos);
            let hold_point = move_point(waypoint.pos, angle, 300.0);

            new_waypoints.push(TaxiWaypoint {
              pos: hold_point,
              wp: current.wp.clone(),
              behavior: TaxiWaypointBehavior::HoldShort,
            });

            waypoint.behavior = TaxiWaypointBehavior::GoTo;
          } else if let TaxiPoint::Gate(_, gate) = &waypoint.wp {
            new_waypoints.push(waypoint.clone());
            current = waypoint.clone();

            new_waypoints.push(TaxiWaypoint {
              pos: gate.pos,
              wp: waypoint.wp.clone(),
              behavior: TaxiWaypointBehavior::GoTo,
            });

            continue;
          }

          new_waypoints.push(waypoint.clone());
          current = waypoint.clone();
        } else {
          tracing::warn!("handle no intersection {current:#?}, {waypoint:#?}");
          return;
        }
      }

      *waypoints = new_waypoints;
      waypoints.reverse();
      current_pos.behavior = TaxiWaypointBehavior::GoTo;
    }

    if let AircraftState::Taxiing { .. } = self.state {
      self.do_continue_taxi()
    }
  }

  pub fn do_hold_taxi(&mut self, _fast: bool) {
    // TODO: if we don't do a fast stop for taxiing, holding short won't hold
    // short of the waypoint since we slow *after* reaching it, such that
    // we will overshoot and won't be directly over our destination.
    self.target.speed = 0.0;
    if self.speed <= 20.0 {
      self.speed = 0.0;
    }
  }

  pub fn do_continue_taxi(&mut self) {
    // TODO: Both hold and continue should modify speed only. Once an aircraft
    // passes a HoldShort waypoint, it should be deleted such that even if it
    // passes the waypoint, it can still continue.
    // if let AircraftState::Taxiing { current, .. } = &mut self.state {
    //   if current.pos == self.pos {
    //     current.behavior = TaxiWaypointBehavior::GoTo;
    //   }
    // }

    self.target.speed = 20.0;
  }

  pub fn resume_own_navigation(&mut self) {
    if let AircraftState::Flying = &self.state {
      if let AircraftIntention::Depart { heading, .. } = &self.intention {
        self.target.heading = *heading;
        self.target.speed = 400.0;
        self.target.altitude = 13000.0;
      }
    }
  }

  fn update_holding_pattern(&mut self) {
    if let AircraftState::HoldingPattern(direction) = &mut self.state {
      match direction {
        HoldDirection::Right => {
          self.target.heading = add_degrees(self.heading, 10.0)
        }
        HoldDirection::Left => {
          self.target.heading = add_degrees(self.heading, -10.0)
        }
      };
    }
  }

  fn update_position(&mut self, dt: f32) {
    let pos = move_point(self.pos, self.heading, self.speed_in_pixels() * dt);
    self.pos = pos;
  }

  fn update_to_departure(&mut self) {
    if let AircraftIntention::Land = self.intention {
      if let AircraftState::Taxiing { current, .. } = &self.state {
        if let TaxiPoint::Gate(_, gate) = &current.wp {
          if self.pos == gate.pos {
            self.departure_from_arrival();
            self.do_hold_taxi(true);
          }
        }
      }
    }
  }

  fn update_taxi(&mut self) {
    let speed_in_pixels = self.speed_in_pixels();
    if let AircraftState::Taxiing { current, waypoints } = &mut self.state {
      if let TaxiWaypoint {
        wp: TaxiPoint::Runway(r),
        behavior: TaxiWaypointBehavior::TakeOff,
        ..
      } = current
      {
        // TODO: I removed the position check from this due to floating point
        // errors. Ideally, we should lerp position on taxiways instead of
        // manually moving them
        self.pos = r.start();
        self.heading = r.heading;
        self.target.heading = r.heading;
        self.state = AircraftState::TakingOff(r.clone());
      } else if matches!(current.behavior, TaxiWaypointBehavior::HoldShort)
        || waypoints.is_empty()
      {
        self.do_hold_taxi(false)
      } else {
        let waypoint = waypoints.last().cloned();
        if let Some(waypoint) = waypoint {
          let heading = angle_between_points(self.pos, waypoint.pos);

          self.heading = heading;
          self.target.heading = heading;

          let distance = self.pos.distance_squared(waypoint.pos);
          let movement_speed = speed_in_pixels.powf(2.0);

          if movement_speed >= distance {
            *current = waypoints.pop().unwrap();
            self.pos = waypoint.pos;

            if let TaxiWaypointBehavior::HoldShort = current.behavior {
              // TODO: This solution seems to break holding short of a runway
              // if let Some(popped) = waypoints.pop() {
              //   *current = popped;
              // }
              self.do_hold_taxi(false);
            }
          }
        }
      }
    }
  }

  fn update_takeoff(&mut self) {
    if let AircraftState::TakingOff(runway) = &self.state {
      if self.pos == runway.start() && self.heading == runway.heading {
        self.heading = runway.heading;
        self.target.heading = runway.heading;

        self.speed = 170.0;
        self.target.speed = 220.0;

        self.target.altitude = 3000.0;

        self.state = AircraftState::Flying;
      } else {
        todo!("not at or lined up to runway {}", runway.id)
      }
    }
  }

  fn update_landing(&mut self, dt: f32, sender: &Sender<OutgoingReply>) {
    if let AircraftState::Landing(runway) = &self.state {
      let ils_line = Line::new(
        move_point(runway.start(), runway.heading, 500.0),
        move_point(
          runway.start(),
          inverse_degrees(runway.heading),
          NAUTICALMILES_TO_FEET * 10.0,
        ),
      );
      let start_descent_distance = NAUTICALMILES_TO_FEET * 6.0;

      let distance_to_runway = self.pos.distance(runway.start());

      let climb_speed = self.dt_climb_speed(dt);
      let seconds_for_descent = self.altitude / (climb_speed / dt);

      let target_speed_ft_s = distance_to_runway / seconds_for_descent;
      let target_knots = target_speed_ft_s / KNOT_TO_FEET_PER_SECOND;

      if distance_to_runway <= 1.0 {
        self.altitude = 0.0;
        self.target.altitude = 0.0;

        self.heading = runway.heading;
        self.target.heading = runway.heading;

        self.target.speed = 0.0;

        self.state = AircraftState::Taxiing {
          current: TaxiWaypoint {
            pos: runway.end(),
            wp: TaxiPoint::Runway(runway.clone()),
            behavior: TaxiWaypointBehavior::GoTo,
          },
          waypoints: Vec::new(),
        };

        return;
      }

      self.target.altitude =
        4000.0 * (distance_to_runway / start_descent_distance).min(1.0);

      if (165.0..=175.0).contains(&target_knots) {
        self.target.speed = target_knots;
      } else if target_knots < 165.0 {
        self.do_go_around(sender, GoAroundReason::TooHigh);
        return;
      }

      let closest_point =
        closest_point_on_line(self.pos, ils_line.0, ils_line.1);

      let landing_point =
        move_point(closest_point, runway.heading, NAUTICALMILES_TO_FEET * 0.4);

      let heading_to_point = angle_between_points(self.pos, landing_point);
      self.target.heading = heading_to_point;
    }
  }

  fn update_leave_airspace(&mut self, airspace_size: f32) {
    // TODO: reimplement leave airspace
    // let airspace_center = Vec2::splat(airspace_size * 0.5);
    // let distance = self.pos.distance_squared(airspace_center);
    // let max_distance = (airspace_size * 0.5).powf(2.0);

    // if distance >= max_distance {
    //   self.state = AircraftState::Deleted;
    // }
  }

  fn dt_climb_speed(&self, dt: f32) -> f32 {
    TIME_SCALE * (2000.0_f32 / 60.0_f32).round() * dt
  }

  fn dt_turn_speed(&self, dt: f32) -> f32 {
    TIME_SCALE * 2.0 * dt
  }

  fn dt_speed_speed(&self, dt: f32) -> f32 {
    // Taxi speed
    if self.altitude == 0.0 {
      // If landing
      if self.speed > 20.0 {
        TIME_SCALE * 6.0 * dt
        // If taxiing
      } else {
        TIME_SCALE * 5.0 * dt
      }
      // Air speed
    } else {
      TIME_SCALE * 2.0 * dt
    }
  }

  fn update_targets(&mut self, dt: f32) {
    // TODO: change speeds for takeoff and taxi (turn and speed speeds)

    // In feet per second
    let climb_speed = self.dt_climb_speed(dt);
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

  pub fn update(&mut self, dt: f32, sender: &Sender<OutgoingReply>) {
    self.update_landing(dt, sender);
    self.update_holding_pattern();
    self.update_targets(dt);
    self.update_taxi();
    self.update_to_departure();
    self.update_takeoff();
    self.update_position(dt);
    // self.update_leave_airspace(airspace_size);
  }
}
