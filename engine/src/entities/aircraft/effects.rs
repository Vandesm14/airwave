use super::{actions::ActionKind, Action, Aircraft, AircraftState};

use crate::{
  angle_between_points, calculate_ils_altitude, closest_point_on_line,
  delta_angle, engine::Bundle, inverse_degrees, move_point, normalize_angle,
  Line, KNOT_TO_FEET_PER_SECOND, NAUTICALMILES_TO_FEET,
};

pub trait AircraftEffect {
  fn run(aircraft: &Aircraft, bundle: &mut Bundle);
}

pub struct AircraftUpdateFromTargetsEffect;
impl AircraftEffect for AircraftUpdateFromTargetsEffect {
  fn run(aircraft: &Aircraft, bundle: &mut Bundle) {
    // In feet per second
    let climb_speed = aircraft.dt_climb_speed(bundle.dt);
    // In degrees per second
    let turn_speed = aircraft.dt_turn_speed(bundle.dt);
    // In knots per second
    let speed_speed = aircraft.dt_speed_speed(bundle.dt);

    // Snap values if they're close enough
    if (aircraft.altitude - aircraft.target.altitude).abs() < climb_speed {
      bundle.actions.push(Action::new(
        aircraft.id,
        ActionKind::Altitude(aircraft.target.altitude),
      ));
    }
    if (aircraft.heading - aircraft.target.heading).abs() < turn_speed {
      bundle.actions.push(Action::new(
        aircraft.id,
        ActionKind::Heading(normalize_angle(aircraft.target.heading)),
      ));
    }
    if (aircraft.speed - aircraft.target.speed).abs() < speed_speed {
      bundle.actions.push(Action::new(
        aircraft.id,
        ActionKind::Speed(aircraft.target.speed),
      ));
    }

    // Change if not equal
    if aircraft.altitude != aircraft.target.altitude {
      if aircraft.altitude < aircraft.target.altitude {
        bundle.actions.push(Action::new(
          aircraft.id,
          ActionKind::Altitude(aircraft.altitude + climb_speed),
        ));
      } else {
        bundle.actions.push(Action::new(
          aircraft.id,
          ActionKind::Altitude(aircraft.altitude - climb_speed),
        ));
      }
    }
    if aircraft.heading != aircraft.target.heading {
      let delta_angle = delta_angle(aircraft.heading, aircraft.target.heading);
      if delta_angle < 0.0 {
        bundle.actions.push(Action::new(
          aircraft.id,
          ActionKind::Heading(normalize_angle(aircraft.heading - turn_speed)),
        ));
      } else {
        bundle.actions.push(Action::new(
          aircraft.id,
          ActionKind::Heading(normalize_angle(aircraft.heading + turn_speed)),
        ));
      }
    }
    if aircraft.speed != aircraft.target.speed {
      if aircraft.speed < aircraft.target.speed {
        bundle.actions.push(Action::new(
          aircraft.id,
          ActionKind::Speed(aircraft.speed + speed_speed),
        ));
      } else {
        bundle.actions.push(Action::new(
          aircraft.id,
          ActionKind::Speed(aircraft.speed - speed_speed),
        ));
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
      aircraft.speed * KNOT_TO_FEET_PER_SECOND * bundle.dt,
    );
    bundle
      .actions
      .push(Action::new(aircraft.id, ActionKind::Pos(pos)));
  }
}

pub struct AircraftUpdateAirspaceEffect;
impl AircraftEffect for AircraftUpdateAirspaceEffect {
  fn run(aircraft: &Aircraft, bundle: &mut Bundle) {
    for airspace in bundle.airspaces.iter() {
      if airspace.contains_point(aircraft.pos) {
        bundle
          .actions
          .push(Action::new(aircraft.id, ActionKind::Airspace(airspace.id)));

        break;
      }
    }
  }
}

pub struct AircraftUpdateLandingEffect;
impl AircraftEffect for AircraftUpdateLandingEffect {
  fn run(aircraft: &Aircraft, bundle: &mut Bundle) {
    if let AircraftState::Landing(runway) = &aircraft.state {
      let ils_line = Line::new(
        move_point(runway.end(), runway.heading, 500.0),
        move_point(
          runway.end(),
          inverse_degrees(runway.heading),
          NAUTICALMILES_TO_FEET * 10.0 + runway.length,
        ),
      );

      let start_descent_distance = NAUTICALMILES_TO_FEET * 10.0;
      let distance_to_runway = aircraft.pos.distance(runway.start());
      let distance_to_end = aircraft.pos.distance_squared(runway.end());

      let climb_speed = aircraft.dt_climb_speed(bundle.dt);
      let seconds_for_descent = aircraft.altitude / (climb_speed / bundle.dt);

      let target_speed_ft_s = distance_to_runway / seconds_for_descent;
      let target_knots = target_speed_ft_s / KNOT_TO_FEET_PER_SECOND;

      let target_altitude = calculate_ils_altitude(distance_to_runway);

      let angle_to_runway =
        inverse_degrees(angle_between_points(runway.start(), aircraft.pos));
      let angle_range = (runway.heading - 5.0)..=(runway.heading + 5.0);

      // If we are too high, go around.
      if aircraft.altitude - target_altitude > 100.0 {
        // TODO: Go around
        // aircraft.do_go_around(sender, GoAroundReason::TooHigh);
        return;
      }

      // If we have passed the start of the runway (landed),
      // set our state to taxiing.
      if distance_to_end <= runway.length.powf(2.0) {
        // TODO: Initiate taxi

        // self.altitude = 0.0;
        // self.target.altitude = 0.0;

        // self.heading = runway.heading;
        // self.target.heading = runway.heading;

        // self.target.speed = 0.0;

        // self.state = AircraftState::Taxiing {
        //   current: Node {
        //     name: runway.id.clone(),
        //     kind: NodeKind::Runway,
        //     behavior: NodeBehavior::GoTo,
        //     value: self.pos,
        //   },
        //   waypoints: Vec::new(),
        // };

        // return;
      }

      let closest_point =
        closest_point_on_line(aircraft.pos, ils_line.0, ils_line.1);

      let landing_point =
        move_point(closest_point, runway.heading, NAUTICALMILES_TO_FEET * 0.4);

      let heading_to_point = angle_between_points(aircraft.pos, landing_point);
      bundle.actions.push(Action::new(
        aircraft.id,
        ActionKind::TargetHeading(heading_to_point),
      ));

      // If we aren't within the localizer beacon (+/- 5 degrees), don't do
      // anything.
      if !angle_range.contains(&angle_to_runway)
        || distance_to_runway > start_descent_distance
      {
        return;
      }

      bundle.actions.push(Action::new(
        aircraft.id,
        ActionKind::TargetSpeed(target_knots.min(180.0)),
      ));

      // If we are too high, descend.
      if aircraft.altitude > target_altitude {
        bundle.actions.push(Action::new(
          aircraft.id,
          ActionKind::TargetAltitude(target_altitude),
        ));
      }
    }
  }
}

// Example of an effect that triggers a one-time event based on previous and current state
//
// pub struct AircraftIsPast205Effect;
// impl AircraftEffect for AircraftIsPast205Effect {
//   fn run(aircraft: &Aircraft, bundle: &mut Bundle) {
//     if bundle.prev.speed <= 205.0 && aircraft.speed >= 205.0 {
//       println!("Past 205");
//       bundle.events.push(Event::Land(Intern::from_ref("27")));
//     }
//   }
// }
