use std::time::SystemTime;

use super::{
  actions::ActionKind,
  events::{Event, EventKind},
  Action, Aircraft, AircraftState,
};

use crate::{
  angle_between_points, calculate_ils_altitude, closest_point_on_line,
  command::{CommandReply, CommandWithFreq},
  delta_angle,
  engine::Bundle,
  inverse_degrees, move_point, normalize_angle,
  pathfinder::{NodeBehavior, NodeKind},
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

    let mut altitude = aircraft.altitude;
    let mut heading = aircraft.heading;
    let mut speed = aircraft.speed;

    // Snap values if they're close enough
    if (altitude - aircraft.target.altitude).abs() < climb_speed {
      altitude = aircraft.target.altitude;
    }
    if (heading - aircraft.target.heading).abs() < turn_speed {
      heading = aircraft.target.heading;
    }
    if (speed - aircraft.target.speed).abs() < speed_speed {
      speed = aircraft.target.speed;
    }

    // Change if not equal
    if altitude != aircraft.target.altitude {
      if altitude < aircraft.target.altitude {
        altitude += climb_speed;
      } else {
        altitude -= climb_speed;
      }
    }
    if heading != aircraft.target.heading {
      let delta_angle = delta_angle(heading, aircraft.target.heading);
      if delta_angle < 0.0 {
        heading -= turn_speed;
      } else {
        heading += turn_speed;
      }
    }
    if speed != aircraft.target.speed {
      if speed < aircraft.target.speed {
        speed += speed_speed;
      } else {
        speed -= speed_speed;
      }
    }

    if altitude != aircraft.altitude {
      bundle
        .actions
        .push(Action::new(aircraft.id, ActionKind::Altitude(altitude)));
    }
    if heading != aircraft.heading {
      bundle.actions.push(Action::new(
        aircraft.id,
        ActionKind::Heading(normalize_angle(heading)),
      ));
    }
    if speed != aircraft.speed {
      bundle
        .actions
        .push(Action::new(aircraft.id, ActionKind::Speed(speed)));
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

    if pos != aircraft.pos {
      bundle
        .actions
        .push(Action::new(aircraft.id, ActionKind::Pos(pos)));
    }
  }
}

pub struct AircraftUpdateAirspaceEffect;
impl AircraftEffect for AircraftUpdateAirspaceEffect {
  fn run(aircraft: &Aircraft, bundle: &mut Bundle) {
    let airspace = bundle
      .airspaces
      .iter()
      .find(|a| a.contains_point(aircraft.pos))
      .map(|a| a.id);
    if airspace != aircraft.airspace {
      bundle
        .actions
        .push(Action::new(aircraft.id, ActionKind::Airspace(airspace)));
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

      // If we have passed the start of the runway (landed),
      // set our state to taxiing.
      if distance_to_end <= runway.length.powf(2.0) {
        bundle.events.push(Event {
          id: aircraft.id,
          kind: EventKind::Touchdown,
        });
        return;
      }

      if !angle_range.contains(&angle_to_runway) {
        return;
      }

      // If we are too high, go around.
      if aircraft.altitude - target_altitude > 100.0 {
        bundle.events.push(Event {
          id: aircraft.id,
          kind: EventKind::GoAround,
        });
        return;
      }

      let landing_point = if distance_to_runway <= start_descent_distance {
        let closest_point =
          closest_point_on_line(aircraft.pos, ils_line.0, ils_line.1);

        move_point(closest_point, runway.heading, NAUTICALMILES_TO_FEET * 0.5)
      } else {
        ils_line.1
      };

      let heading_to_point = angle_between_points(aircraft.pos, landing_point);
      bundle.actions.push(Action::new(
        aircraft.id,
        ActionKind::TargetHeading(heading_to_point),
      ));

      // If we aren't within the localizer beacon (+/- 5 degrees), don't do
      // anything.
      if distance_to_runway > start_descent_distance {
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
        // Only hold if we are not stopped and we are at or below taxi speed.
      } else if aircraft.speed > 0.0 && aircraft.speed <= 20.0 {
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

pub struct AircraftIsNowParkedEffect;
impl AircraftEffect for AircraftIsNowParkedEffect {
  fn run(aircraft: &Aircraft, bundle: &mut Bundle) {
    if let AircraftState::Taxiing { current, .. } = &aircraft.state {
      if aircraft.speed == 0.0
        && current.kind == NodeKind::Gate
        && aircraft.pos == current.value
        && Some(aircraft.flight_plan.arriving) == aircraft.airspace
      {
        bundle
          .events
          .push(Event::new(aircraft.id, EventKind::DepartureFromArrival));
      }
    }
  }
}

pub struct AircraftContactCenterEffect;
impl AircraftEffect for AircraftContactCenterEffect {
  fn run(aircraft: &Aircraft, bundle: &mut Bundle) {
    if let AircraftState::Flying { .. } = aircraft.state {
      if bundle.prev.airspace.is_some() && aircraft.airspace.is_none() {
        bundle.events.push(Event::new(
          aircraft.id,
          EventKind::Callout(CommandWithFreq::new_reply(
            aircraft.id.to_string(),
            aircraft.frequency,
            CommandReply::ContactCenter {
              altitude: aircraft.altitude,
            },
          )),
        ));
      }
    }
  }
}

pub struct AircraftContactClearanceEffect;
impl AircraftEffect for AircraftContactClearanceEffect {
  fn run(aircraft: &Aircraft, bundle: &mut Bundle) {
    if let AircraftState::Taxiing { current, .. } = &aircraft.state {
      let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
      if aircraft.created < now
        && !aircraft.callouts.clearance
        && current.kind == NodeKind::Gate
        && aircraft.airspace == Some(aircraft.flight_plan.departing)
      {
        bundle.events.push(Event::new(
          aircraft.id,
          EventKind::Callout(CommandWithFreq::new_reply(
            aircraft.id.to_string(),
            aircraft.frequency,
            CommandReply::ContactClearance {
              arrival: aircraft.flight_plan.arriving.to_string(),
            },
          )),
        ));
        bundle.actions.push(Action::new(
          aircraft.id,
          ActionKind::Callouts(aircraft.callouts.mark_clearance()),
        ));
      }
    }
  }
}
