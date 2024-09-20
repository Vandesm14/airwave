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
  deserialize_vec2,
  pathfinder::{Node, NodeVORData},
  serialize_vec2,
};

use super::{
  airport::{Gate, Runway},
  airspace::Airspace,
  world::find_random_airspace,
};

const DEPARTURE_WAIT_RANGE: RangeInclusive<u64> = 600..=1200;

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

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FlightPlan {
  pub departing: Intern<String>,
  pub arriving: Intern<String>,
  pub altitude: f32,
  pub speed: f32,
  pub waypoints: Vec<Node<NodeVORData>>,
}

impl FlightPlan {
  pub fn new(departing: Intern<String>, arriving: Intern<String>) -> Self {
    Self {
      departing,
      arriving,
      ..Default::default()
    }
  }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Aircraft {
  pub id: Intern<String>,

  #[serde(serialize_with = "serialize_vec2")]
  #[serde(deserialize_with = "deserialize_vec2")]
  pub pos: Vec2,
  pub speed: f32,
  pub heading: f32,
  pub altitude: f32,

  pub state: AircraftState,
  pub target: AircraftTargets,
  pub flight_plan: FlightPlan,

  pub frequency: f32,
  pub created: u128,
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

  pub fn random_parked(gate: Gate, rng: &mut Rng) -> Self {
    Self {
      id: Intern::from(Self::random_callsign(rng)),

      pos: gate.pos,
      speed: 0.0,
      heading: gate.heading,
      altitude: 00.0,

      state: AircraftState::Taxiing {
        current: gate.clone().into(),
        waypoints: Vec::new(),
      },
      target: AircraftTargets::default(),
      flight_plan: FlightPlan::new(
        Intern::from(String::new()),
        Intern::from(String::new()),
      ),

      frequency: 118.6,
      created: SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::from_millis(0))
        .as_millis(),
      airspace: None,
    }
    .with_synced_targets()
  }

  pub fn departure_from_arrival(
    &mut self,
    airspaces: &[Airspace],
    rng: &mut Rng,
  ) {
    // TODO: true when airports
    let departure =
      airspaces.iter().find(|a| a.id == self.flight_plan.arriving);
    let arrival = find_random_airspace(airspaces, rng);

    // TODO: when airport as destination
    // TODO: handle errors
    if let Some(((arrival, departure), airspace)) =
      arrival.zip(departure).zip(self.airspace)
    {
      self.flight_plan = FlightPlan::new(airspace, arrival.id);

      // TODO: created and frequency
      self.created = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::from_millis(0))
        .add(Duration::from_secs(
          rng.sample_iter(DEPARTURE_WAIT_RANGE).unwrap(),
        ))
        .as_millis();
      self.frequency = departure.frequencies.clearance;
    }
  }
}

// Performance stats
impl Aircraft {
  pub fn dt_climb_speed(&self, dt: f32) -> f32 {
    (2000.0_f32 / 60.0_f32).round() * dt
  }

  pub fn dt_turn_speed(&self, dt: f32) -> f32 {
    2.0 * dt
  }

  pub fn dt_speed_speed(&self, dt: f32) -> f32 {
    // Taxi speed
    if self.altitude == 0.0 {
      // If landing
      if self.speed > 20.0 {
        4.0 * dt
        // If taxiing
      } else {
        5.0 * dt
      }
      // Air speed
    } else {
      2.0 * dt
    }
  }
}
