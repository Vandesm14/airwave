pub mod effects;
pub mod events;

use glam::Vec2;
use internment::Intern;
use serde::{Deserialize, Serialize};
use turborand::{rng::Rng, TurboRand};

use crate::pathfinder::{Node, NodeVORData};

use super::airport::{Airport, Gate, Runway};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AircraftTargets {
  pub heading: f32,
  pub speed: f32,
  pub altitude: f32,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LandingState {
  #[default]
  /// Do nothing.
  BeforeTurn,

  /// Turn once we should to line up with the localizer.
  Turning,

  /// Correct our position if we are off of the localizer.
  Correcting,

  /// Once on the localizer.
  Localizer,

  /// Once established on the glideslope, descend.
  Glideslope,

  /// We have landed.
  Touchdown,

  /// Go around
  GoAround,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaxiingState {
  /// Normal operation, will stop if a collision is detected.
  #[default]
  Armed,

  /// Stopped, collision detected. Won't move until collision is cleared.
  Stopped,

  /// Palyer override. Will move despite a collision. Reset after collision is
  /// no longer detected.
  Override,

  /// Player or waypoint ovveride. Won't move unless a continue is given.
  Holding,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type", content = "value")]
pub enum AircraftState {
  Flying {
    waypoints: Vec<Node<NodeVORData>>,
  },
  Landing {
    runway: Runway,
    state: LandingState,
  },
  Taxiing {
    current: Node<Vec2>,
    waypoints: Vec<Node<Vec2>>,
    state: TaxiingState,
  },
  Parked {
    at: Node<Vec2>,
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
}

impl Default for FlightPlan {
  fn default() -> Self {
    Self {
      arriving: Intern::from_ref("arriving"),
      departing: Intern::from_ref("departing"),

      speed: 250.0,
      altitude: 7000.0,
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
pub struct AircraftStats {
  // Speed^2
  /// Thrust in kN
  pub thrust: f32,
  /// Drag in kN
  pub drag: f32,
  /// Rate of turn in degrees per second
  pub turn_speed: f32,
  /// Rate of climb in feet per minute
  pub roc: f32,
  /// Rate of descent in feet per minute
  pub rod: f32,

  // Limits
  /// Max altitude in feet
  pub max_altitude: f32,

  /// Minimum speed in knots
  pub min_speed: f32,
  /// Maximum speed in knots
  pub max_speed: f32,

  // Performance
  /// V2 speed in knots (when rotate)
  pub v2: f32,
  /// Minimum length of runway for takeoff (in feet)
  pub takeoff_length: f32,
  /// Minimum length of runway for landing (in feet)
  pub landing_length: f32,

  // Cargo
  /// Max takeoff weight in pounds
  pub max_takeoff_weight: f32,
  /// Max landing weight in pounds
  pub max_landing_weight: f32,
  /// Dry weight in pounds
  pub dry_weight: f32,
  /// Fuel capacity in pounds
  pub fuel_capacity: f32,
  /// Passenger capacity in capita
  pub seats: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AircraftKind {
  // Airbus
  /// https://contentzone.eurocontrol.int/aircraftperformance/details.aspx?ICAO=A21N
  A21N,
  /// https://contentzone.eurocontrol.int/aircraftperformance/details.aspx?ICAO=A333
  A333,

  // Boeing
  /// https://contentzone.eurocontrol.int/aircraftperformance/details.aspx?ICAO=B737
  B737,
  /// https://contentzone.eurocontrol.int/aircraftperformance/details.aspx?ICAO=B74S
  B747,
  /// https://contentzone.eurocontrol.int/aircraftperformance/details.aspx?ICAO=B77L
  B77L,

  // Embraer
  /// https://contentzone.eurocontrol.int/aircraftperformance/details.aspx?ICAO=CRJ7
  CRJ7,
  /// https://contentzone.eurocontrol.int/aircraftperformance/details.aspx?ICAO=E170
  E170,
}

impl AircraftKind {
  pub fn stats(&self) -> AircraftStats {
    match self {
      AircraftKind::A21N => AircraftStats {
        thrust: 140.96,
        // TODO: placeholder
        drag: 0.0,
        turn_speed: 1.0,
        roc: 1500.0,
        rod: 2500.0,
        max_altitude: 39000.0,
        min_speed: 140.0,
        max_speed: 450.0,
        v2: 145.0,
        takeoff_length: 7054.0,
        landing_length: 6070.0,
        max_takeoff_weight: 213800.0,
        max_landing_weight: 174606.0,
        dry_weight: 103000.0,
        fuel_capacity: 58232.5,
        seats: 200,
      },
      AircraftKind::A333 => todo!(),
      AircraftKind::B737 => todo!(),
      AircraftKind::B747 => todo!(),
      AircraftKind::B77L => todo!(),
      AircraftKind::CRJ7 => todo!(),
      AircraftKind::E170 => todo!(),
    }
  }
}

/// FlightSegment denotes the exact segment of flight that an aircraft is in.
///
/// This is simply a flag for denoting the segment of flight and does not
/// contain any data or further information. [`AircraftState`] is the primary
/// holder of state and data for an aircraft.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[repr(u8)]
pub enum FlightSegment {
  #[default]
  /// Parked and motionless.
  Parked,

  /// Taxiing as a departure.
  TaxiDep,
  /// Taking off (not yet in the air).
  Takeoff,
  /// Departing, most likely via a SID.
  Departure,
  /// Outside of terminal airspace, at cruise altitude and speed.
  Cruise,

  /// Descending from cruise, most likely via a STAR.
  Arrival,
  /// Within a terminal airspace for vectors to final.
  Approach,
  /// Following ILS for landing.
  Land,
  /// Taxiing as an arrival.
  TaxiArr,
}

// TODO: Implement these tests into the segment effect in effect.rs.
impl FlightSegment {
  pub fn on_ground(&self) -> bool {
    matches!(self, Self::Parked | Self::TaxiDep | Self::TaxiArr)
  }

  pub fn in_air(&self) -> bool {
    matches!(
      self,
      Self::Departure
        | Self::Cruise
        | Self::Arrival
        | Self::Approach
        | Self::Land
    )
  }

  pub fn departing(&self) -> bool {
    matches!(self, Self::TaxiDep | Self::Takeoff | Self::Departure)
  }

  pub fn en_route(&self) -> bool {
    matches!(self, Self::Departure | Self::Cruise)
  }

  pub fn arriving(&self) -> bool {
    matches!(
      self,
      Self::Arrival | Self::Approach | Self::Land | Self::TaxiArr
    )
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TCAS {
  #[default]
  Idle,
  Warning,
  Climb,
  Descend,
  Hold,
}

impl TCAS {
  pub fn is_idle(&self) -> bool {
    matches!(self, Self::Idle)
  }

  pub fn is_ta(&self) -> bool {
    matches!(self, Self::Warning)
  }

  pub fn is_ra(&self) -> bool {
    matches!(self, Self::Climb | Self::Descend | Self::Hold)
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
  pub tcas: TCAS,
  pub flight_plan: FlightPlan,

  pub frequency: f32,
  pub segment: FlightSegment,

  pub accepted: bool,
}

// Helper methods
impl Aircraft {
  pub fn is_parked(&self) -> bool {
    matches!(self.state, AircraftState::Parked { .. })
  }

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

  pub fn random_parked(gate: Gate, rng: &mut Rng, airport: &Airport) -> Self {
    Self {
      id: Intern::from(Self::random_callsign(rng)),
      is_colliding: false,

      pos: gate.pos,
      speed: 0.0,
      heading: gate.heading,
      altitude: 0.0,

      state: AircraftState::Parked { at: gate.into() },
      target: AircraftTargets::default(),
      flight_plan: FlightPlan::new(
        Intern::from(String::new()),
        Intern::from(String::new()),
      ),
      tcas: TCAS::default(),

      frequency: airport.frequencies.ground,
      segment: FlightSegment::Parked,

      accepted: false,
    }
    .with_synced_targets()
  }

  pub fn flip_flight_plan(&mut self) {
    let d = self.flight_plan.departing;
    let a = self.flight_plan.arriving;

    // TODO: Is this the best place to put this change?
    //
    // If a flight plan is flipped, it does mean its a new flight, so that's
    // why I put it here.
    self.accepted = false;

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
    dt
  }
}
