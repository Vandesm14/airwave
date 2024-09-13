use std::{
  ops::{Add, RangeInclusive},
  time::{Duration, SystemTime},
};

use glam::Vec2;
use rand::{seq::SliceRandom, thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::{
  angle_between_points, calculate_ils_altitude, closest_point_on_line,
  delta_angle, deserialize_vec2,
  engine::OutgoingReply,
  inverse_degrees, move_point,
  objects::{
    airport::Runway,
    airspace::Airspace,
    command::{CommandReply, CommandReplyKind, CommandWithFreq},
    world::{find_random_arrival, find_random_departure, World},
  },
  pathfinder::{Node, NodeBehavior, NodeKind, Pathfinder},
  serialize_vec2, Line, KNOT_TO_FEET_PER_SECOND, NAUTICALMILES_TO_FEET,
  TIME_SCALE,
};

use super::{airport::Gate, world::find_random_airspace};

const DEPARTURE_WAIT_RANGE: RangeInclusive<u64> = 120..=600;

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
pub enum GoAroundReason {
  TooHigh,
  WrongAngle,

  None,
}

impl Aircraft {
  pub fn created_now(&mut self) {
    self.created = SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH)
      .unwrap_or(Duration::from_millis(0))
      .as_millis();
  }

  pub fn sync_targets(&mut self) {
    self.target.heading = self.heading;
    self.target.speed = self.speed;
    self.target.altitude = self.altitude;
  }

  pub fn with_synced_targets(mut self) -> Self {
    self.sync_targets();
    self
  }

  pub fn departure_from_arrival(&mut self, airspaces: &[Airspace]) {
    let mut rng = thread_rng();
    // TODO: true when airports
    let arrival = find_random_airspace(airspaces);

    // TODO: when airport as destination
    // TODO: handle errors
    if let Some(arrival) = arrival {
      self.flight_plan = (self.airspace.clone().unwrap(), arrival.id.clone());
      self.created = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::from_millis(0))
        .add(Duration::from_secs(rng.gen_range(DEPARTURE_WAIT_RANGE)))
        .as_millis();
    }
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

  pub fn random_parked(gate: Gate) -> Self {
    Self {
      callsign: Self::random_callsign(),
      is_colliding: false,
      flight_plan: (String::new(), String::new()),
      state: AircraftState::Taxiing {
        current: gate.clone().into(),
        waypoints: Vec::new(),
      },
      pos: gate.pos,
      heading: gate.heading,
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
      aircraft.speed = 400.0;
      aircraft.altitude = 13000.0;

      aircraft.sync_targets();

      aircraft.frequency = arrival.frequencies.center;

      Some(aircraft)
    } else {
      None
    }
  }

  pub fn random_callsign() -> String {
    let mut string = String::new();
    let airlines = ["AAL", "SKW", "JBU"];

    let mut rng = thread_rng();
    let airline = airlines.choose(&mut rng).unwrap();

    string.push_str(airline);

    string.push_str(&rng.gen_range(0..=9).to_string());
    string.push_str(&rng.gen_range(0..=9).to_string());
    string.push_str(&rng.gen_range(0..=9).to_string());
    string.push_str(&rng.gen_range(0..=9).to_string());

    string
  }
}

impl Aircraft {
  pub fn speed_in_feet(&self) -> f32 {
    self.speed * KNOT_TO_FEET_PER_SECOND
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
}

impl Aircraft {
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

      tracing::debug!("target_altitude: {}", target_altitude);
      // If we are too high, descend.
      if self.altitude > target_altitude {
        self.target.altitude = target_altitude;
      }
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

impl Aircraft {
  pub fn do_clear_waypoints(&mut self) {
    if let AircraftState::Flying { .. } = self.state {
      self.state = AircraftState::Flying {
        waypoints: Vec::new(),
      };
    } else if let AircraftState::Taxiing { waypoints, .. } = &mut self.state {
      waypoints.clear();
    }
  }

  pub fn do_resume_own_navigation(&mut self, pos: Vec2) {
    if let AircraftState::Flying { .. } = &self.state {
      let heading = angle_between_points(self.pos, pos);
      self.target.heading = heading;
      self.target.speed = 400.0;
      self.target.altitude = 13000.0;
    }
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
        reply: CommandReply {
          callsign: self.callsign.clone(),
          kind: CommandReplyKind::WithCallsign { text },
        }
        .to_string(),
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
      let destinations = waypoints.iter();
      let mut all_waypoints: Vec<Node<Vec2>> = Vec::new();

      let mut pos = self.pos;
      let mut heading = self.heading;
      let mut current: Node<Vec2> = current.clone();
      for destination in destinations {
        let path = pathfinder.path_to(
          Node {
            name: current.name.clone(),
            kind: current.kind,
            behavior: current.behavior,
            value: (),
          },
          destination.clone(),
          pos,
          heading,
        );

        if let Some(path) = path {
          pos = path.final_pos;
          heading = path.final_heading;
          current = path.path.last().unwrap().clone();

          all_waypoints.extend(path.path);
        } else {
          tracing::error!(
            "Failed to find path for destination: {:?}, from: {:?}",
            destination,
            current
          );
          return;
        }
      }

      all_waypoints.reverse();
      *wps = all_waypoints;

      tracing::info!(
        "Initiating taxi for {}: {:?}",
        self.callsign,
        wps.iter().map(|w| w.name.clone()).collect::<Vec<_>>()
      );

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
}
