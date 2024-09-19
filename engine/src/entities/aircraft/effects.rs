use super::{
  actions::ActionKind,
  events::{Event, EventKind},
  Action, Aircraft, AircraftState,
};

use crate::{
  angle_between_points, calculate_ils_altitude, closest_point_on_line,
  delta_angle, engine::Bundle, inverse_degrees, move_point, normalize_angle,
  pathfinder::NodeBehavior, Line, KNOT_TO_FEET_PER_SECOND,
  NAUTICALMILES_TO_FEET,
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
        bundle.events.push(Event {
          id: aircraft.id,
          kind: EventKind::GoAround,
        });
        return;
      }

      // If we have passed the start of the runway (landed),
      // set our state to taxiing.
      if distance_to_end <= runway.length.powf(2.0) {
        bundle.events.push(Event {
          id: aircraft.id,
          kind: EventKind::Touchdown,
        });
        return;
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

pub struct AircraftUpdateFlyingEffect;
impl AircraftEffect for AircraftUpdateFlyingEffect {
  fn run(aircraft: &Aircraft, bundle: &mut Bundle) {
    let speed_in_feet = aircraft.speed * KNOT_TO_FEET_PER_SECOND;
    if let AircraftState::Flying { waypoints, .. } = &aircraft.state {
      if let Some(current) = waypoints.last() {
        let heading = angle_between_points(aircraft.pos, current.value.to);

        bundle.actions.push(Action {
          id: aircraft.id,
          kind: ActionKind::TargetHeading(heading),
        });

        let distance = aircraft.pos.distance_squared(current.value.to);
        let movement_speed = speed_in_feet.powf(2.0);

        if movement_speed >= distance {
          bundle.actions.push(Action {
            id: aircraft.id,
            kind: ActionKind::Pos(current.value.to),
          });
          bundle.actions.push(Action {
            id: aircraft.id,
            kind: ActionKind::PopWaypoint,
          });
        }
      }
    }
  }
}

pub struct AircraftUpdateTaxiingEffect;
impl AircraftEffect for AircraftUpdateTaxiingEffect {
  fn run(aircraft: &Aircraft, bundle: &mut Bundle) {
    let speed_in_feet = aircraft.speed * KNOT_TO_FEET_PER_SECOND;
    if let AircraftState::Taxiing { waypoints, .. } = &aircraft.state {
      let waypoint = waypoints.last().cloned();
      if let Some(waypoint) = waypoint {
        let heading = angle_between_points(aircraft.pos, waypoint.value);

        bundle.actions.push(Action {
          id: aircraft.id,
          kind: ActionKind::Heading(heading),
        });
        bundle.actions.push(Action {
          id: aircraft.id,
          kind: ActionKind::TargetHeading(heading),
        });

        let distance = aircraft.pos.distance_squared(waypoint.value);
        let movement_speed = speed_in_feet.powf(2.0);

        if movement_speed >= distance {
          bundle.actions.push(Action {
            id: aircraft.id,
            kind: ActionKind::PopWaypoint,
          });
        }
      } else {
        bundle.events.push(Event {
          id: aircraft.id,
          kind: EventKind::TaxiHold,
        });
      }
    }

    if let AircraftState::Taxiing { waypoints, .. } = &aircraft.state {
      let waypoint = waypoints.last();
      if let Some(waypoint) = waypoint {
        let distance = aircraft.pos.distance_squared(waypoint.value);

        if NodeBehavior::HoldShort == waypoint.behavior
          && distance <= 250.0_f32.powf(2.0)
        {
          bundle.actions.push(Action {
            id: aircraft.id,
            kind: ActionKind::TaxiLastAsGoto,
          });
          bundle.events.push(Event {
            id: aircraft.id,
            kind: EventKind::TaxiHold,
          });
        }
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
