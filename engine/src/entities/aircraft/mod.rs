pub mod actions;
pub mod effects;
pub mod events;

use std::{
  ops::{Add, RangeInclusive},
  time::{Duration, SystemTime},
};

use actions::Action;
use glam::Vec2;
use internment::Intern;
use serde::{Deserialize, Serialize};
use turborand::{rng::Rng, TurboRand};

use crate::{
  angle_between_points,
  pathfinder::{Node, NodeBehavior, NodeKind, NodeVORData},
};

use super::{
  airport::{Gate, Runway},
  airspace::Airspace,
  world::find_random_airspace,
};

const DEPARTURE_WAIT_RANGE: RangeInclusive<u64> = 600..=1200;

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct AircraftCallouts {
  pub clearance: bool,
}

impl AircraftCallouts {
  pub fn mark_clearance(mut self) -> Self {
    self.clearance = true;
    self
  }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AircraftTargets {
  pub heading: f32,
  pub speed: f32,
  pub altitude: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type", content = "value")]
pub enum AircraftState {
  Flying {
    waypoints: Vec<Node<NodeVORData>>,
  },
  Landing(Runway),
  Taxiing {
    current: Node<Vec2>,
    waypoints: Vec<Node<Vec2>>,
  },
}

impl Default for AircraftState {
  fn default() -> Self {
    Self::Flying {
      waypoints: Vec::new(),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlightPlan {
  // To and From
  pub arriving: Intern<String>,
  pub departing: Intern<String>,

  // IFR Clearance
  pub speed: f32,
  pub altitude: f32,
  pub waypoints: Vec<Node<NodeVORData>>,
}

impl Default for FlightPlan {
  fn default() -> Self {
    Self {
      arriving: Intern::from_ref("arriving"),
      departing: Intern::from_ref("departing"),

      speed: 220.0,
      altitude: 3000.0,
      waypoints: Vec::new(),
    }
  }
}

impl FlightPlan {
  pub fn new(departing: Intern<String>, arriving: Intern<String>) -> Self {
    Self {
      departing,
      arriving,
      ..Self::default()
    }
  }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Aircraft {
  pub id: Intern<String>,
  pub is_colliding: bool,

  pub pos: Vec2,
  pub speed: f32,
  pub heading: f32,
  pub altitude: f32,

  pub state: AircraftState,
  pub target: AircraftTargets,
  pub flight_plan: FlightPlan,
  pub callouts: AircraftCallouts,

  pub frequency: f32,
  pub created: Duration,
  pub airspace: Option<Intern<String>>,
}

// Helper methods
impl Aircraft {
  pub fn sync_targets_to_vals(&mut self) {
    self.target.heading = self.heading;
    self.target.speed = self.speed;
    self.target.altitude = self.altitude;
  }

  pub fn with_synced_targets(mut self) -> Self {
    self.sync_targets_to_vals();
    self
  }

  pub fn random_callsign(rng: &mut Rng) -> String {
    let mut string = String::new();
    let airlines = ["AAL", "SKW", "JBU"];

    let airline = rng.sample(&airlines).unwrap();

    string.push_str(airline);

    string.push_str(&rng.sample_iter(0..=9).unwrap().to_string());
    string.push_str(&rng.sample_iter(0..=9).unwrap().to_string());
    string.push_str(&rng.sample_iter(0..=9).unwrap().to_string());
    string.push_str(&rng.sample_iter(0..=9).unwrap().to_string());

    string
  }

  pub fn random_parked(gate: Gate, rng: &mut Rng, airspace: &Airspace) -> Self {
    Self {
      id: Intern::from(Self::random_callsign(rng)),
      is_colliding: false,

      pos: gate.pos,
      speed: 0.0,
      heading: gate.heading,
      altitude: 0.0,

      state: AircraftState::Taxiing {
        current: gate.clone().into(),
        waypoints: Vec::new(),
      },
      target: AircraftTargets::default(),
      flight_plan: FlightPlan::new(
        Intern::from(String::new()),
        Intern::from(String::new()),
      ),
      callouts: AircraftCallouts::default(),

      frequency: airspace.frequencies.clearance,
      created: SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap(),
      airspace: Some(airspace.id),
    }
    .with_synced_targets()
  }

  pub fn random_to_arrive(
    departure: &Airspace,
    arrival: &Airspace,
    rng: &mut Rng,
  ) -> Self {
    let mut aircraft = Aircraft {
      id: Intern::from(Aircraft::random_callsign(rng)),
      pos: departure.pos,
      speed: 250.0,
      heading: angle_between_points(departure.pos, arrival.pos),
      altitude: 1000.0,
      state: AircraftState::Flying {
        waypoints: vec![Node::new(
          arrival.id,
          NodeKind::VOR,
          NodeBehavior::GoTo,
          NodeVORData {
            to: arrival.pos,
            then: Vec::new(),
          },
        )],
      },
      flight_plan: FlightPlan::new(departure.id, arrival.id),
      frequency: arrival.frequencies.center,
      airspace: Some(departure.id),
      ..Default::default()
    };

    aircraft.sync_targets_to_vals();
    aircraft.target.speed = 400.0;
    aircraft.target.altitude = 13000.0;

    aircraft
  }

  pub fn make_random_departure(
    &mut self,
    airspaces: &[Airspace],
    rng: &mut Rng,
  ) {
    let departure =
      airspaces.iter().find(|a| a.id == self.flight_plan.arriving);
    let arrival = find_random_airspace(airspaces, rng);

    if let Some((arrival, departure)) = arrival.zip(departure) {
      self.departure_from_arrival(
        departure.id,
        arrival.id,
        Duration::from_secs(rng.sample_iter(DEPARTURE_WAIT_RANGE).unwrap()),
      );
      self.frequency = departure.frequencies.clearance;
    }
  }

  pub fn departure_from_arrival(
    &mut self,
    departure: Intern<String>,
    arrival: Intern<String>,
    wait_time: Duration,
  ) {
    self.flight_plan = FlightPlan::new(departure, arrival);
    self.created = SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH)
      .unwrap()
      .add(wait_time);
    self.callouts = AircraftCallouts::default();
  }

  pub fn created_now(&mut self) {
    self.created = SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH)
      .unwrap();
  }
}

// Performance stats
impl Aircraft {
  pub fn dt_climb_speed(&self, dt: f32) -> f32 {
    // When taking off or taxiing (no climb until V2)
    if self.speed < 140.0 {
      0.0
    } else {
      // Flying
      (2000.0_f32 / 60.0_f32).round() * dt
    }
  }

  pub fn dt_turn_speed(&self, dt: f32) -> f32 {
    2.0 * dt
  }

  pub fn dt_speed_speed(&self, dt: f32) -> f32 {
    // Taxi speed
    if self.altitude == 0.0 {
      // If landing
      if self.speed > 20.0 {
        3.3 * dt
        // If taxiing
      } else {
        5.0 * dt
      }
    } else if self.altitude <= 1000.0 {
      // When taking off
      5.0 * dt
    } else {
      // Flying
      2.0 * dt
    }
  }
}
