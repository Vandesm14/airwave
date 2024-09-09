use core::fmt;
use std::{
  ops::Add,
  time::{Duration, SystemTime},
};

use glam::Vec2;
use rand::{seq::SliceRandom, thread_rng, Rng};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tracing::{debug, error, info};

use crate::{
  angle_between_points, calculate_ils_altitude, closest_point_on_line,
  delta_angle,
  engine::OutgoingReply,
  inverse_degrees, move_point,
  pathfinder::{Node, NodeBehavior, NodeKind, Object, Pathfinder},
  Line, KNOT_TO_FEET_PER_SECOND, NAUTICALMILES_TO_FEET, TIME_SCALE,
};

pub fn find_random_airspace(
  airspaces: &[Airspace],
  auto: bool,
  with_airports: bool,
) -> Option<&Airspace> {
  let mut rng = thread_rng();
  let filtered_airspaces: Vec<&Airspace> = airspaces
    .iter()
    .filter(|a| {
      if auto != a.auto {
        return false;
      }
      if with_airports {
        return !a.airports.is_empty();
      }
      true
    })
    .collect();

  filtered_airspaces.choose(&mut rng).copied()
}

pub fn find_random_departure(airspaces: &[Airspace]) -> Option<&Airspace> {
  // TODO: We should probably do `true` for the second bool, which specifies
  // that a departure airspace needs an airport. This just saves us time
  // when testing and messing about with single airspaces instead of those
  // plus an airport.
  find_random_airspace(airspaces, true, false)
}

pub fn find_random_arrival(airspaces: &[Airspace]) -> Option<&Airspace> {
  find_random_airspace(airspaces, false, true)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct World {
  pub airspaces: Vec<Airspace>,
  pub aircraft: Vec<Aircraft>,
  pub waypoints: Vec<Node<Vec2>>,
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Airspace {
  pub id: String,
  #[serde(serialize_with = "serialize_vec2")]
  #[serde(deserialize_with = "deserialize_vec2")]
  pub pos: Vec2,
  pub size: f32,
  pub airports: Vec<Airport>,

  /// Determines whether the airspace is automatically controlled.
  pub auto: bool,
}

impl Airspace {
  pub fn contains_point(&self, point: Vec2) -> bool {
    let distance = point.distance_squared(self.pos);
    distance <= self.size.powf(2.0)
  }

  pub fn find_random_airport(&self) -> Option<&Airport> {
    let mut rng = thread_rng();
    let airports: Vec<&Airport> = self.airports.iter().collect();

    airports.choose(&mut rng).copied()
  }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Airport {
  pub id: String,
  #[serde(serialize_with = "serialize_vec2")]
  #[serde(deserialize_with = "deserialize_vec2")]
  pub center: Vec2,
  pub runways: Vec<Runway>,
  pub taxiways: Vec<Taxiway>,
  pub terminals: Vec<Terminal>,
  pub altitude_range: [f32; 2],

  #[serde(skip)]
  pub pathfinder: Pathfinder,
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

      pathfinder: Pathfinder::new(),
    }
  }

  pub fn add_taxiway(&mut self, taxiway: Taxiway) {
    let taxiway = taxiway.extend_ends_by(100.0);
    self.taxiways.push(taxiway);
  }

  pub fn add_runway(&mut self, mut runway: Runway) {
    runway.length += 200.0;
    self.runways.push(runway);
  }

  pub fn cache_waypoints(&mut self) {
    let mut nodes: Vec<Object> = Vec::new();
    nodes.extend(self.runways.iter().map(|r| r.clone().into()));
    nodes.extend(self.taxiways.iter().map(|t| t.clone().into()));
    nodes.extend(self.terminals.iter().map(|g| g.clone().into()));

    self.pathfinder.calculate(nodes);
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

  #[serde(rename = "taxi")]
  Taxi(Vec<Node<()>>),
  #[serde(rename = "taxi-hold")]
  TaxiHold,
  #[serde(rename = "taxi-continue")]
  TaxiContinue,

  Direct(Vec<String>),
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
  // TODO: Should this be converted to CommandReply so that the front-end can
  //       handle formatting?
  pub reply: String,
  pub tasks: Vec<Task>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandReply {
  pub callsign: String,
  pub kind: CommandReplyKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CommandReplyKind {
  AircraftArrivedInTowerAirspace { direction: String },
}

impl fmt::Display for CommandReply {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match &self.kind {
      CommandReplyKind::AircraftArrivedInTowerAirspace { direction } => {
        write!(
          f,
          "Tower, {} is {direction} of the airport, with you.",
          self.callsign
        )
      }
    }
  }
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
pub enum AircraftState {
  Flying {
    waypoints: Vec<Node<Vec2>>,
  },
  Landing(Runway),
  Taxiing {
    current: Node<Vec2>,
    waypoints: Vec<Node<Vec2>>,
  },
  TakingOff(Runway),

  Deleted,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AircraftTargets {
  pub heading: f32,
  pub speed: f32,
  pub altitude: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum AircraftUpdate {
  #[default]
  None,

  NewDeparture,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Aircraft {
  pub callsign: String,

  pub is_colliding: bool,
  pub flight_plan: (String, String),
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

  pub airspace: Option<String>,
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

impl Taxiway {
  pub fn new(id: String, a: Vec2, b: Vec2) -> Self {
    Self { id, a, b }
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
  pub apron: Line,
}

impl Aircraft {
  pub fn sync_targets(&mut self) {
    self.target.heading = self.heading;
    self.target.speed = self.speed;
    self.target.altitude = self.altitude;
  }

  pub fn with_synced_targets(mut self) -> Self {
    self.sync_targets();
    self
  }

  pub fn random_flying() -> Self {
    Self {
      callsign: Self::random_callsign(),
      is_colliding: false,
      flight_plan: (String::new(), String::new()),
      state: AircraftState::Flying {
        waypoints: Vec::new(),
      },
      pos: Vec2::default(),
      heading: 0.0,
      speed: 250.0,
      altitude: 7000.0,
      frequency: 118.5,
      target: AircraftTargets::default(),
      created: SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::from_millis(0))
        .as_millis(),
      airspace: None,
    }
    .with_synced_targets()
  }

  pub fn random_parked(current: Node<Vec2>) -> Self {
    Self {
      callsign: Self::random_callsign(),
      is_colliding: false,
      flight_plan: (String::new(), String::new()),
      state: AircraftState::Taxiing {
        current: current.clone(),
        waypoints: Vec::new(),
      },
      pos: current.value,
      heading: 0.0,
      speed: 0.0,
      altitude: 00.0,
      frequency: 118.6,
      target: AircraftTargets::default(),
      created: SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::from_millis(0))
        .as_millis(),
      airspace: None,
    }
    .with_synced_targets()
  }

  pub fn random_to_arrive(world: &World) -> Option<Self> {
    let departure = find_random_departure(&world.airspaces);
    let arrival = find_random_arrival(&world.airspaces);

    if let Some((departure, arrival)) = departure.zip(arrival) {
      // We can unwrap this because we already filtered out airspaces with no
      // airports.

      // TODO: when depart from airport
      // let dep_airport = departure.find_random_airport().unwrap();
      // let arr_airport = arrival.find_random_airport().unwrap();

      // let mut aircraft = Aircraft::random_flying();
      // aircraft.flight_plan = (dep_airport.id.clone(), arr_airport.id.clone());
      // aircraft.pos = dep_airport.center;
      // aircraft.heading =
      //   angle_between_points(dep_airport.center, arr_airport.center);

      let arr_airport = arrival.find_random_airport().unwrap();

      let mut aircraft = Aircraft::random_flying();
      aircraft.flight_plan = (departure.id.clone(), arr_airport.id.clone());
      aircraft.pos = departure.pos;
      aircraft.heading =
        angle_between_points(departure.pos, arr_airport.center);

      aircraft.sync_targets();

      Some(aircraft)
    } else {
      None
    }
  }

  pub fn created_now(&mut self) {
    self.created = SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH)
      .unwrap_or(Duration::from_millis(0))
      .as_millis();
  }

  pub fn departure_from_arrival(&mut self, airspaces: &[Airspace]) {
    let mut rng = thread_rng();
    // TODO: true when airports
    let arrival = find_random_airspace(airspaces, true, false);

    // TODO: when airport as destination
    // TODO: handle errors
    if let Some(arrival) = arrival {
      self.flight_plan = (self.airspace.clone().unwrap(), arrival.id.clone());
      self.created = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::from_millis(0))
        .add(Duration::from_secs(rng.gen_range(60..=180)))
        .as_millis();
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

  pub fn speed_in_feet(&self) -> f32 {
    self.speed * KNOT_TO_FEET_PER_SECOND
  }

  pub fn do_go_around(
    &mut self,
    sender: &async_broadcast::Sender<OutgoingReply>,
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

    self.state = AircraftState::Flying {
      waypoints: Vec::new(),
    };

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
      .try_broadcast(OutgoingReply::Reply(CommandWithFreq {
        id: self.callsign.clone(),
        frequency: self.frequency,
        reply: text,
        tasks: Vec::new(),
      }))
      .unwrap();
  }

  pub fn do_takeoff(&mut self, runway: &Runway) {
    self.pos = runway.start();
    self.heading = runway.heading;
    self.target.heading = runway.heading;

    self.state = AircraftState::TakingOff(runway.clone());
  }

  pub fn do_taxi(&mut self, waypoints: Vec<Node<()>>, pathfinder: &Pathfinder) {
    if let AircraftState::Taxiing {
      waypoints: wps,
      current,
      ..
    } = &mut self.state
    {
      let waypoints = pathfinder.path_to(
        Node {
          name: current.name.clone(),
          kind: current.kind,
          behavior: NodeBehavior::GoTo,
          value: (),
        },
        waypoints.last().unwrap().clone(),
        waypoints,
        self.pos,
        self.heading,
      );

      if let Some(mut waypoints) = waypoints {
        waypoints.reverse();
        *wps = waypoints;

        info!("Initiating taxi for {}: {:?}", self.callsign, wps);
      } else {
        error!("Failed to find waypoints for {:?}", &self);

        return;
      }

      if wps.is_empty() {
        return;
      }
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

  pub fn clear_waypoints(&mut self) {
    if let AircraftState::Flying { .. } = self.state {
      self.state = AircraftState::Flying {
        waypoints: Vec::new(),
      };
    }
  }

  pub fn resume_own_navigation(&mut self) {
    if let AircraftState::Flying { .. } = &self.state {
      // TODO: new system
      // if let AircraftIntention::Depart { heading, .. } = &self.intention {
      //   self.target.heading = *heading;
      //   self.target.speed = 400.0;
      //   self.target.altitude = 13000.0;
      // }
    }
  }

  fn update_position(&mut self, dt: f32) {
    let pos = move_point(self.pos, self.heading, self.speed_in_feet() * dt);
    self.pos = pos;
  }

  fn update_to_departure(&mut self) -> AircraftUpdate {
    if let AircraftState::Taxiing {
      current:
        Node {
          kind: NodeKind::Gate,
          value,
          ..
        },
      waypoints,
    } = &self.state
    {
      if self.pos == *value
        && waypoints.is_empty()
        && Some(self.flight_plan.1.clone()) == self.airspace
      {
        self.do_hold_taxi(true);
        return AircraftUpdate::NewDeparture;
      }
    }

    AircraftUpdate::default()
  }

  fn update_taxi(&mut self) {
    let speed_in_feet = self.speed_in_feet();
    if let AircraftState::Taxiing {
      current, waypoints, ..
    } = &mut self.state
    {
      let waypoint = waypoints.last().cloned();
      if let Some(waypoint) = waypoint {
        let heading = angle_between_points(self.pos, waypoint.value);

        self.heading = heading;
        self.target.heading = heading;

        let distance = self.pos.distance_squared(waypoint.value);
        let movement_speed = speed_in_feet.powf(2.0);

        if movement_speed >= distance {
          *current = waypoints.pop().unwrap();
          self.pos = waypoint.value;
        }
      } else {
        self.do_hold_taxi(false);
      }
    }

    if let AircraftState::Taxiing { waypoints, .. } = &mut self.state {
      let waypoint = waypoints.last_mut();
      if let Some(waypoint) = waypoint {
        let distance = self.pos.distance_squared(waypoint.value);

        if NodeBehavior::HoldShort == waypoint.behavior
          && distance <= 250.0_f32.powf(2.0)
        {
          waypoint.behavior = NodeBehavior::GoTo;
          self.do_hold_taxi(false);
        }
      }
    }
  }

  fn update_flying(&mut self) {
    let speed_in_feet = self.speed_in_feet();
    if let AircraftState::Flying { waypoints, .. } = &mut self.state {
      if let Some(current) = waypoints.last() {
        let heading = angle_between_points(self.pos, current.value);

        self.target.heading = heading;

        let distance = self.pos.distance_squared(current.value);
        let movement_speed = speed_in_feet.powf(2.0);

        if movement_speed >= distance {
          self.pos = current.value;
          waypoints.pop();
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

        self.state = AircraftState::Flying {
          waypoints: Vec::new(),
        };
      } else {
        todo!("not at or lined up to runway {}", runway.id)
      }
    }
  }

  fn update_landing(
    &mut self,
    dt: f32,
    sender: &async_broadcast::Sender<OutgoingReply>,
  ) {
    if let AircraftState::Landing(runway) = &self.state {
      let ils_line = Line::new(
        move_point(runway.end(), runway.heading, 500.0),
        move_point(
          runway.end(),
          inverse_degrees(runway.heading),
          NAUTICALMILES_TO_FEET * 10.0 + runway.length,
        ),
      );

      let start_descent_distance = NAUTICALMILES_TO_FEET * 10.0;
      let distance_to_runway = self.pos.distance(runway.start());
      let distance_to_end = self.pos.distance_squared(runway.end());

      let climb_speed = self.dt_climb_speed(dt);
      let seconds_for_descent = self.altitude / (climb_speed / dt);

      let target_speed_ft_s = distance_to_runway / seconds_for_descent;
      let target_knots = target_speed_ft_s / KNOT_TO_FEET_PER_SECOND;

      let target_altitude = calculate_ils_altitude(distance_to_runway);

      let angle_to_runway =
        inverse_degrees(angle_between_points(runway.start(), self.pos));
      let angle_range = (runway.heading - 5.0)..=(runway.heading + 5.0);

      // If we are too high, go around.
      if self.altitude - target_altitude > 100.0 {
        self.do_go_around(sender, GoAroundReason::TooHigh);
        return;
      }

      // If we have passed the start of the runway (landed),
      // set our state to taxiing.
      if distance_to_end <= runway.length.powf(2.0) {
        self.altitude = 0.0;
        self.target.altitude = 0.0;

        self.heading = runway.heading;
        self.target.heading = runway.heading;

        self.target.speed = 0.0;

        self.state = AircraftState::Taxiing {
          current: Node {
            name: runway.id.clone(),
            kind: NodeKind::Runway,
            behavior: NodeBehavior::GoTo,
            value: self.pos,
          },
          waypoints: Vec::new(),
        };

        return;
      }

      let closest_point =
        closest_point_on_line(self.pos, ils_line.0, ils_line.1);

      let landing_point =
        move_point(closest_point, runway.heading, NAUTICALMILES_TO_FEET * 0.4);

      let heading_to_point = angle_between_points(self.pos, landing_point);
      self.target.heading = heading_to_point;

      // If we aren't within the localizer beacon (+/- 5 degrees), don't do
      // anything.
      if !angle_range.contains(&angle_to_runway)
        || distance_to_runway > start_descent_distance
      {
        return;
      }

      self.target.speed = target_knots.min(180.0);

      debug!("target_altitude: {}", target_altitude);
      // If we are too high, descend.
      if self.altitude > target_altitude {
        self.target.altitude = target_altitude;
      }
    }
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
        TIME_SCALE * 4.0 * dt
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

  pub fn update(
    &mut self,
    dt: f32,
    sender: &async_broadcast::Sender<OutgoingReply>,
  ) -> AircraftUpdate {
    self.update_landing(dt, sender);
    self.update_targets(dt);
    self.update_taxi();
    self.update_flying();
    let update = self.update_to_departure();
    self.update_takeoff();
    self.update_position(dt);

    update
  }
}
