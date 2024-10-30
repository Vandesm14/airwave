pub mod actions;
pub mod effects;
pub mod events;

use actions::Action;
use glam::Vec2;
use internment::Intern;
use serde::{Deserialize, Serialize};
use turborand::{rng::Rng, TurboRand};

use crate::pathfinder::{Node, NodeVORData};

use super::{
  airport::{Gate, Runway},
  airspace::Airspace,
};

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
    enroute: bool,
  },
  Landing(Runway),
  Taxiing {
    current: Node<Vec2>,
    waypoints: Vec<Node<Vec2>>,
  },
  Parked(Node<Vec2>),
}

impl Default for AircraftState {
  fn default() -> Self {
    Self::Flying {
      waypoints: Vec::new(),
      enroute: false,
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
}

impl Default for FlightPlan {
  fn default() -> Self {
    Self {
      arriving: Intern::from_ref("arriving"),
      departing: Intern::from_ref("departing"),

      speed: 220.0,
      altitude: 3000.0,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AircraftKind {
  // Airbus
  A321,
  A330,

  // Boeing
  B737,
  B747,
  B777,

  // Embraer
  CRJ700,
  ERJ170,
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

  pub frequency: f32,
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

      state: AircraftState::Parked(gate.clone().into()),
      target: AircraftTargets::default(),
      flight_plan: FlightPlan::new(
        Intern::from(String::new()),
        Intern::from(String::new()),
      ),

      frequency: airspace.frequencies.ground,
    }
    .with_synced_targets()
  }

  pub fn random_flying(frequency: f32, flight_plan: FlightPlan) -> Self {
    Self {
      id: Intern::from(Aircraft::random_callsign(&mut Default::default())),
      is_colliding: false,

      pos: Vec2::ZERO,
      speed: 250.0,
      heading: 0.0,
      altitude: 7000.0,

      state: AircraftState::Flying {
        waypoints: Vec::new(),
        enroute: false,
      },
      target: AircraftTargets::default(),
      flight_plan,

      frequency,
    }
    .with_synced_targets()
  }

  pub fn flip_flight_plan(&mut self) {
    let d = self.flight_plan.departing;
    let a = self.flight_plan.arriving;

    self.flight_plan.departing = a;
    self.flight_plan.arriving = d;
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

  pub fn dt_enroute(&self, dt: f32) -> f32 {
    if let AircraftState::Flying { enroute, .. } = &self.state {
      if *enroute {
        dt * 10.0
      } else {
        dt
      }
    } else {
      dt
    }
  }
}
