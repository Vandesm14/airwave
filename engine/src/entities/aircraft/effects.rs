use crate::{delta_angle, move_point, normalize_angle};

use super::{Action, Aircraft, AircraftEffect, Bundle};

pub struct AircraftUpdateFromTargetsEffect;
impl AircraftEffect for AircraftUpdateFromTargetsEffect {
  fn run(aircraft: &Aircraft, bundle: &mut Bundle) {
    // In feet per second
    let climb_speed = aircraft.dt_climb_sp(bundle.dt);
    // In degrees per second
    let turn_speed = aircraft.dt_turn_speed(bundle.dt);
    // In knots per second
    let speed_speed = aircraft.dt_speed_speed(bundle.dt);

    if (aircraft.altitude - aircraft.target.altitude).abs() < climb_speed {
      bundle
        .actions
        .push(Action::Altitude(aircraft.target.altitude));
    }
    if (aircraft.heading - aircraft.target.heading).abs() < turn_speed {
      bundle
        .actions
        .push(Action::Heading(normalize_angle(aircraft.target.heading)));
    }
    if (aircraft.speed - aircraft.target.speed).abs() < speed_speed {
      bundle.actions.push(Action::Speed(aircraft.target.speed));
    }

    // Change based on speed if not equal
    if aircraft.altitude != aircraft.target.altitude {
      if aircraft.altitude < aircraft.target.altitude {
        bundle
          .actions
          .push(Action::Altitude(aircraft.altitude + climb_speed));
      } else {
        bundle
          .actions
          .push(Action::Altitude(aircraft.altitude - climb_speed));
      }
    }
    if aircraft.heading != aircraft.target.heading {
      let delta_angle = delta_angle(aircraft.heading, aircraft.target.heading);
      if delta_angle < 0.0 {
        // aircraft.heading -= turn_speed;
      } else {
        bundle.actions.push(Action::Heading(normalize_angle(
          aircraft.heading + turn_speed,
        )));
      }
    }
    if aircraft.speed != aircraft.target.speed {
      if aircraft.speed < aircraft.target.speed {
        bundle
          .actions
          .push(Action::Speed(aircraft.speed + speed_speed));
      } else {
        bundle
          .actions
          .push(Action::Speed(aircraft.speed - speed_speed));
      }
    }
  }
}

pub struct AircraftUpdatePositionEffect;
impl AircraftEffect for AircraftUpdatePositionEffect {
  fn run(aircraft: &Aircraft, bundle: &mut Bundle) {
    let pos = move_point(
      aircraft.pos,
      aircraft.heading,
      aircraft.speed_in_feet() * bundle.dt,
    );
    bundle.actions.push(Action::Pos(pos));
  }
}