pub mod effects;
pub mod events;

use std::{f32::consts::PI, ops::Sub};

use glam::Vec2;
use internment::Intern;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use turborand::{TurboRand, rng::Rng};

use crate::{
  KNOT_TO_FEET_PER_SECOND, NAUTICALMILES_TO_FEET, ToText,
  geometry::delta_angle, pathfinder::Node, wayfinder::FlightPlan,
};

use super::airport::{Airport, Gate, Runway};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AircraftTargets {
  pub heading: f32,
  pub speed: f32,
  pub altitude: f32,
}

impl ToText for AircraftTargets {
  fn to_text(&self, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
    write!(
      w,
      "Targets: {}° {}kt {}ft",
      self.heading, self.speed, self.altitude
    )
  }
}

#[derive(
  Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize, TS,
)]
#[ts(export)]
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

impl LandingState {
  pub fn established(&self) -> bool {
    matches!(self, Self::Localizer | Self::Glideslope)
  }
}

#[derive(
  Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize, TS,
)]
#[ts(export)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type", content = "value")]
#[ts(export)]
pub enum AircraftState {
  Flying,
  Landing {
    runway: Runway,
    state: LandingState,
  },
  Taxiing {
    #[ts(as = "Node<(f32, f32)>")]
    current: Node<Vec2>,
    #[ts(as = "Vec<Node<(f32, f32)>>")]
    waypoints: Vec<Node<Vec2>>,
    state: TaxiingState,
  },
  Parked {
    #[ts(as = "Node<(f32, f32)>")]
    at: Node<Vec2>,
  },
}

impl Default for AircraftState {
  fn default() -> Self {
    Self::Flying
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
        turn_speed: 2.0,
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
#[derive(
  Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize, TS,
)]
#[serde(rename_all = "kebab-case")]
#[repr(u8)]
#[ts(export)]
pub enum FlightSegment {
  #[default]
  // Unknown state.
  Unknown,

  /// Inactive and parked.
  Dormant,
  /// Boarding for departure.
  Boarding,
  /// Parked and ready for taxi.
  Parked,

  /// Taxiing as a departure.
  TaxiDep,
  /// Taking off (not yet in the air).
  Takeoff,
  /// Flying within departure airspace, most likely via a SID.
  Departure,
  /// Climbing to cruise altitude, outside of terminal airspace.
  Climb,
  /// Outside of terminal airspace, at cruise altitude and speed.
  Cruise,

  /// Descending from cruise, most likely via a STAR.
  Arrival,
  /// Within a terminal airspace for vectors to final.
  Approach,
  /// Following ILS for landing.
  Landing,
  /// Taxiing as an arrival.
  TaxiArr,
}

// TODO: Implement these tests into the segment effect in effect.rs.
impl FlightSegment {
  pub const fn on_ground(&self) -> bool {
    matches!(
      self,
      Self::Dormant
        | Self::Boarding
        | Self::Parked
        | Self::TaxiDep
        | Self::TaxiArr
    )
  }

  pub const fn in_air(&self) -> bool {
    matches!(
      self,
      Self::Departure
        | Self::Climb
        | Self::Cruise
        | Self::Arrival
        | Self::Approach
        | Self::Landing
    )
  }
}

#[derive(
  Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize, TS,
)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
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

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SeparationMinima {
  pub separation_distance: f32,
  pub max_speed: f32,
  pub min_speed: f32,
  pub max_deviation_angle: f32,
}

impl SeparationMinima {
  pub fn new(
    separation_distance: f32,
    max_speed: f32,
    min_speed: f32,
    max_deviation_angle: f32,
  ) -> Self {
    Self {
      separation_distance,
      max_speed,
      min_speed,
      max_deviation_angle,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Aircraft {
  #[ts(as = "String")]
  pub id: Intern<String>,

  #[ts(as = "(f32, f32)")]
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
  #[ts(as = "Option<String>")]
  pub airspace: Option<Intern<String>>,

  pub flight_time: Option<usize>,
}

impl ToText for Aircraft {
  fn to_text(&self, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
    writeln!(w, "Callsign: {}", self.id)?;
    writeln!(
      w,
      "Current: {}° {}kt {}ft",
      self.heading, self.speed, self.altitude
    )?;
    self.target.to_text(w)?;
    writeln!(w)?;

    // TODO: State
    // TODO: TCAS
    // TODO: Flight Plan
    // TODO: Freq
    // TODO: Segment
    // TODO: Flight Time

    Ok(())
  }
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
    let airlines = ["AAL", "SKW", "JBU", "DAL", "UAL", "BAW", "SWA"];

    let airline = rng.sample(&airlines).unwrap();

    string.push_str(airline);

    string.push_str(&rng.sample_iter(0..=9).unwrap().to_string());
    string.push_str(&rng.sample_iter(0..=9).unwrap().to_string());
    string.push_str(&rng.sample_iter(0..=9).unwrap().to_string());
    string.push_str(&rng.sample_iter(0..=9).unwrap().to_string());

    string
  }

  pub fn random_dormant(gate: &Gate, rng: &mut Rng, airport: &Airport) -> Self {
    Self {
      id: Intern::from(Self::random_callsign(rng)),

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
      segment: FlightSegment::Dormant,
      airspace: None,

      flight_time: None,
    }
    .with_synced_targets()
  }

  pub fn flip_flight_plan(&mut self) {
    if self
      .airspace
      .is_some_and(|a| self.flight_plan.arriving == a)
    {
      let d = self.flight_plan.departing;
      let a = self.flight_plan.arriving;

      self.flight_plan.departing = a;
      self.flight_plan.arriving = d;
    }
  }

  pub fn find_airport<'a>(
    &self,
    airports: &'a [Airport],
  ) -> Option<&'a Airport> {
    airports
      .iter()
      .find(|a| self.airspace.is_some_and(|id| a.id == id))
  }
}

// Performance stats
impl Aircraft {
  pub fn separation_minima(&self) -> SeparationMinima {
    let separation_distance = NAUTICALMILES_TO_FEET * 7.5;
    if matches!(self.segment, FlightSegment::Approach) {
      SeparationMinima::new(separation_distance, 230.0, 150.0, 60.0)
    } else if matches!(self.segment, FlightSegment::Departure) {
      SeparationMinima::new(separation_distance, 250.0, 180.0, 60.0)
    } else if matches!(
      self.segment,
      FlightSegment::Climb | FlightSegment::Cruise | FlightSegment::Arrival
    ) {
      SeparationMinima::new(
        separation_distance,
        self.flight_plan.speed,
        350.0,
        30.0,
      )
    } else {
      SeparationMinima::new(separation_distance, 250.0, 180.0, 30.0)
    }
  }

  pub fn climb_speed(&self) -> f32 {
    // When taking off or taxiing (no climb until V2)
    if self.speed < 140.0 {
      0.0
    } else {
      // Flying
      (2000.0_f32 / 60.0_f32).round()
    }
  }

  pub fn turn_speed(&self) -> f32 {
    AircraftKind::A21N.stats().turn_speed
  }

  pub fn speed_speed(&self) -> f32 {
    // Taxi speed
    if self.altitude == 0.0 {
      // If landing
      if self.speed > 20.0 {
        3.3
        // If taxiing
      } else {
        5.0
      }
    } else if self.altitude <= 1000.0 {
      // When taking off
      5.0
    } else {
      // Flying
      2.0
    }
  }

  pub fn turn_distance(&self, new_angle: f32) -> f32 {
    let delta_ang = delta_angle(self.heading, new_angle).abs();

    let degrees_per_sec = self.turn_speed();
    let turning_radius = 360.0 / degrees_per_sec;
    let turning_radius = turning_radius * self.speed * KNOT_TO_FEET_PER_SECOND;
    let turning_radius = turning_radius / (2.0 * PI);
    let turning_radius = turning_radius * 2.0;

    let percent_of = delta_ang.abs() / 180.0;
    let percent_of = (percent_of * PI + PI * 1.5).sin() / 2.0 + 0.5;

    turning_radius * percent_of
  }

  /// Outputs the distance in feet traveled until the current speed matches
  /// the new speed.
  pub fn distance_to_change_speed(&self, new_speed: f32) -> f32 {
    if self.speed == new_speed {
      return 0.0;
    }

    let mut distance = 0.0;
    let mut speed = self.speed;
    while speed.sub(new_speed).abs() >= self.speed_speed() {
      if speed > new_speed {
        speed -= self.speed_speed();
      } else {
        speed += self.speed_speed();
      }

      distance += speed * KNOT_TO_FEET_PER_SECOND;
    }

    distance
  }

  /// Outputs the distance in feet traveled until the current altitude matches
  /// the new altitude.
  pub fn distance_to_change_altitude(&self, new_altitude: f32) -> f32 {
    if self.altitude == new_altitude {
      return 0.0;
    }

    let mut distance = 0.0;
    let mut altitude = self.altitude;
    while altitude.sub(new_altitude).abs() >= self.climb_speed() {
      if altitude > new_altitude {
        altitude -= self.climb_speed();
      } else {
        altitude += self.climb_speed();
      }

      distance += self.speed * KNOT_TO_FEET_PER_SECOND;
    }

    distance
  }

  pub fn target_waypoint_limits(&self) -> AircraftTargets {
    if !self.flight_plan.follow {
      return self.target.clone();
    }

    let mut altitude_target: Option<f32> = None;
    let mut speed_target: Option<f32> = None;

    let mut distance = 0.0;
    let mut pos = self.pos;
    for wp in self
      .flight_plan
      .waypoints
      .iter()
      .skip(self.flight_plan.waypoint_index)
    {
      distance += pos.distance(wp.data.pos);
      pos = wp.data.pos;

      if wp.data.limits.altitude.is_some() {
        let delta = wp.data.limits.altitude.diff(self.altitude);
        if delta != 0.0 {
          let distance_to_change =
            self.distance_to_change_altitude(self.altitude + delta);
          if distance <= distance_to_change && altitude_target.is_none() {
            altitude_target = Some(self.altitude + delta);
          }
        }

        // Put a hold on the altitude limit so further ones don't take effect.
        if altitude_target.is_none() {
          altitude_target = Some(self.target.altitude);
        }
      }

      if wp.data.limits.speed.is_some() {
        let delta = wp.data.limits.speed.diff(self.speed);
        if delta != 0.0 {
          let distance_to_change =
            self.distance_to_change_speed(self.speed + delta);
          if distance <= distance_to_change && speed_target.is_none() {
            speed_target = Some(self.speed + delta);
          }
        }

        // Put a hold on the speed limit so further ones don't take effect.
        if speed_target.is_none() {
          speed_target = Some(self.target.speed);
        }
      }
    }

    AircraftTargets {
      altitude: altitude_target.unwrap_or(self.target.altitude),
      speed: speed_target.unwrap_or(self.target.speed),
      heading: self.target.heading,
    }
  }
}
