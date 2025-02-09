use std::f32::consts::PI;

use crate::{
  add_degrees, angle_between_points, calculate_ils_altitude,
  closest_point_on_line,
  command::{CommandReply, CommandWithFreq},
  delta_angle,
  engine::Bundle,
  inverse_degrees, move_point, normalize_angle,
  pathfinder::{NodeBehavior, NodeKind},
  Line, KNOT_TO_FEET_PER_SECOND, NAUTICALMILES_TO_FEET,
};

use super::{
  events::{AircraftEvent, EventKind},
  Aircraft, AircraftState, LandingState,
};

pub trait AircraftEffect {
  fn run(aircraft: &mut Aircraft, bundle: &mut Bundle);
}

pub struct AircraftUpdateFromTargetsEffect;
impl AircraftEffect for AircraftUpdateFromTargetsEffect {
  fn run(aircraft: &mut Aircraft, bundle: &mut Bundle) {
    let dt = aircraft.dt_enroute(bundle.dt);

    // In feet per second
    let climb_speed = aircraft.dt_climb_speed(dt);
    // In degrees per second
    let turn_speed = aircraft.dt_turn_speed(dt);
    // In knots per second
    let speed_speed = aircraft.dt_speed_speed(dt);

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
      aircraft.altitude = altitude;
    }
    if heading != aircraft.heading {
      aircraft.heading = normalize_angle(heading);
    }
    if speed != aircraft.speed {
      aircraft.speed = speed;
    }
  }
}

pub struct AircraftUpdatePositionEffect;
impl AircraftEffect for AircraftUpdatePositionEffect {
  fn run(aircraft: &mut Aircraft, bundle: &mut Bundle) {
    let dt = aircraft.dt_enroute(bundle.dt);

    let pos = move_point(
      aircraft.pos,
      aircraft.heading,
      aircraft.speed * KNOT_TO_FEET_PER_SECOND * dt,
    );

    if pos != aircraft.pos {
      aircraft.pos = pos;
    }
  }
}

pub struct AircraftUpdateLandingEffect;
impl AircraftUpdateLandingEffect {
  fn state_before_turn(aircraft: &mut Aircraft, _: &mut Bundle, dt: f32) {
    let degrees_per_sec = aircraft.dt_turn_speed(dt);
    let AircraftState::Landing { runway, state } = &mut aircraft.state else {
      unreachable!("outer function asserts that aircraft is landing")
    };

    let ils_line = Line::new(
      move_point(runway.end(), runway.heading, 500.0),
      move_point(
        runway.end(),
        inverse_degrees(runway.heading),
        NAUTICALMILES_TO_FEET * 18.0 + runway.length,
      ),
    );

    let turning_radius = 360.0 / degrees_per_sec;
    let turning_radius =
      turning_radius * aircraft.speed * KNOT_TO_FEET_PER_SECOND * dt;
    let turning_radius = turning_radius / (2.0 * PI);
    let turning_radius = turning_radius * 2.0;

    let delta_ang = delta_angle(aircraft.heading, runway.heading);
    let percent_of = delta_ang.abs() / 180.0;
    let percent_of = (percent_of * PI + PI * 1.5).sin() / 2.0 + 0.5;
    let turn_distance = turning_radius * percent_of;
    let turn_distance = turn_distance.powf(2.0);

    let closest_point =
      closest_point_on_line(aircraft.pos, ils_line.0, ils_line.1);
    let distance_to_point = aircraft.pos.distance_squared(closest_point);

    if distance_to_point <= turn_distance {
      aircraft.target.heading = runway.heading;

      *state = LandingState::Turning;
    } else if aircraft.speed > aircraft.target.speed {
      aircraft.target.heading = aircraft.heading;

      *state = LandingState::BeforeTurn;
    }

    let angle_to_runway =
      inverse_degrees(angle_between_points(runway.end(), aircraft.pos));

    if aircraft.heading.round() == runway.heading
      && (angle_to_runway.round() != runway.heading
        || distance_to_point.round() != 0.0)
    {
      if angle_to_runway > runway.heading {
        aircraft.target.heading = add_degrees(runway.heading, 20.0);
      }

      if angle_to_runway < runway.heading {
        aircraft.target.heading = add_degrees(runway.heading, -20.0);
      }

      *state = LandingState::Correcting;
    }

    if distance_to_point <= 50_f32.powf(2.0)
      && aircraft.heading.round() == runway.heading
    {
      *state = LandingState::Localizer;
    }
  }

  fn state_touchdown(aircraft: &mut Aircraft, bundle: &mut Bundle) {
    let AircraftState::Landing { runway, state } = &mut aircraft.state else {
      unreachable!("outer function asserts that aircraft is landing")
    };

    if *state != LandingState::Glideslope {
      return;
    }

    let distance_to_end = aircraft.pos.distance_squared(runway.end());

    // If we have passed the start of the runway (landed),
    // set our state to taxiing.
    if distance_to_end <= runway.length.powf(2.0) {
      bundle.events.push(
        AircraftEvent {
          id: aircraft.id,
          kind: EventKind::Touchdown,
        }
        .into(),
      );

      *state = LandingState::Touchdown
    }
  }

  fn state_go_around(aircraft: &mut Aircraft, bundle: &mut Bundle) {
    let AircraftState::Landing { runway, state } = &mut aircraft.state else {
      unreachable!("outer function asserts that aircraft is landing")
    };

    if *state != LandingState::Glideslope {
      return;
    }

    let distance_to_runway = aircraft.pos.distance(runway.start());
    let target_altitude = calculate_ils_altitude(distance_to_runway);

    // If we are too high, go around.
    if aircraft.altitude - target_altitude > 100.0 {
      bundle.events.push(
        AircraftEvent {
          id: aircraft.id,
          kind: EventKind::GoAround,
        }
        .into(),
      );
      bundle.events.push(
        AircraftEvent {
          id: aircraft.id,
          kind: EventKind::Callout(CommandWithFreq::new(
            aircraft.id.to_string(),
            aircraft.frequency,
            CommandReply::GoAround {
              runway: runway.id.to_string(),
            },
            vec![],
          )),
        }
        .into(),
      );

      *state = LandingState::GoAround;
    }
  }

  fn state_glideslope(aircraft: &mut Aircraft, dt: f32) {
    let climb_speed = aircraft.dt_climb_speed(dt);

    let AircraftState::Landing { runway, state } = &mut aircraft.state else {
      unreachable!("outer function asserts that aircraft is landing")
    };

    if !(*state == LandingState::Localizer
      || *state == LandingState::Glideslope)
    {
      return;
    }

    let start_descent_distance = NAUTICALMILES_TO_FEET * 10.0;
    let distance_to_runway = aircraft.pos.distance(runway.start());

    let angle_to_runway =
      inverse_degrees(angle_between_points(runway.end(), aircraft.pos));
    let angle_range = (runway.heading - 5.0)..=(runway.heading + 5.0);

    let seconds_for_descent = aircraft.altitude / (climb_speed / dt);

    let target_speed_ft_s = distance_to_runway / seconds_for_descent;
    let target_knots = target_speed_ft_s / KNOT_TO_FEET_PER_SECOND;

    let target_altitude = calculate_ils_altitude(distance_to_runway);

    // If we aren't within the localizer beacon (+/- 5 degrees), don't do
    // anything.
    if angle_range.contains(&angle_to_runway)
      && distance_to_runway <= start_descent_distance
    {
      aircraft.target.speed = target_knots.min(180.0);

      // If we are too high, descend.
      if aircraft.altitude > target_altitude {
        aircraft.target.altitude = target_altitude;

        *state = LandingState::Glideslope;
      }
    }
  }
}

impl AircraftEffect for AircraftUpdateLandingEffect {
  fn run(aircraft: &mut Aircraft, bundle: &mut Bundle) {
    let dt = aircraft.dt_enroute(bundle.dt);

    if let AircraftState::Landing { .. } = &aircraft.state {
      Self::state_touchdown(aircraft, bundle);
      Self::state_go_around(aircraft, bundle);
      Self::state_before_turn(aircraft, bundle, dt);
      Self::state_glideslope(aircraft, dt);
    }
  }
}

pub struct AircraftUpdateFlyingEffect;
impl AircraftEffect for AircraftUpdateFlyingEffect {
  fn run(aircraft: &mut Aircraft, bundle: &mut Bundle) {
    if aircraft.altitude < 2000.0 {
      return;
    }

    let dt = aircraft.dt_enroute(bundle.dt);
    let speed_in_feet = aircraft.speed * KNOT_TO_FEET_PER_SECOND * dt;
    if let AircraftState::Flying { waypoints, .. } = &mut aircraft.state {
      if let Some(current) = waypoints.last() {
        let heading = angle_between_points(aircraft.pos, current.value.to);

        aircraft.target.heading = heading;

        let distance = aircraft.pos.distance_squared(current.value.to);
        let movement_speed = speed_in_feet.powf(2.0);

        if movement_speed >= distance {
          aircraft.pos = current.value.to;

          for e in current.value.then.iter() {
            bundle
              .events
              .push(AircraftEvent::new(aircraft.id, e.clone()).into());
          }

          waypoints.pop();
        }
      }
    }
  }
}

pub struct AircraftUpdateTaxiingEffect;
impl AircraftEffect for AircraftUpdateTaxiingEffect {
  fn run(aircraft: &mut Aircraft, bundle: &mut Bundle) {
    let speed_in_feet = aircraft.speed * KNOT_TO_FEET_PER_SECOND * bundle.dt;
    if let AircraftState::Taxiing {
      waypoints, current, ..
    } = &mut aircraft.state
    {
      let waypoint = waypoints.last().cloned();
      if let Some(waypoint) = waypoint {
        let heading = angle_between_points(aircraft.pos, waypoint.value);

        aircraft.heading = heading;
        aircraft.target.heading = heading;

        let distance = aircraft.pos.distance_squared(waypoint.value);
        let movement_speed = speed_in_feet.powf(2.0);

        if movement_speed >= distance {
          if let Some(wp) = waypoints.pop() {
            *current = wp;
          }
        }
        // Only hold if we are not stopped and we are at or below taxi speed.
      } else if aircraft.speed > 0.0 && aircraft.speed <= 20.0 {
        bundle.events.push(
          AircraftEvent {
            id: aircraft.id,
            kind: EventKind::TaxiHold { and_state: true },
          }
          .into(),
        );

        match current.behavior {
          NodeBehavior::GoTo => {}
          NodeBehavior::HoldShort => {}
          NodeBehavior::Park => {
            aircraft.state = AircraftState::Parked {
              at: current.clone(),
              // Only become inactive if we are arriving at the player's airspace.
              // If we are departing, keep us as active.
              active: aircraft.flight_plan.arriving != bundle.world.airspace.id,
            };
            bundle.events.push(
              AircraftEvent {
                id: aircraft.id,
                kind: EventKind::CompleteFlight,
              }
              .into(),
            );
          }

          // Runway specific
          NodeBehavior::LineUp => {
            if current.kind == NodeKind::Runway {
              if let Some(runway) = bundle
                .world
                .airspace
                .airports
                .iter()
                .flat_map(|a| a.runways.iter())
                .find(|r| r.id == current.name)
              {
                aircraft.heading = runway.heading;
                aircraft.target.heading = runway.heading;
              }
            }
          }
          NodeBehavior::Takeoff => {
            if current.kind == NodeKind::Runway {
              bundle.events.push(
                AircraftEvent::new(
                  aircraft.id,
                  EventKind::Takeoff(current.name),
                )
                .into(),
              );
            }
          }
        }
      }
    }

    if let AircraftState::Taxiing { waypoints, .. } = &aircraft.state {
      let waypoint = waypoints.last();
      if let Some(waypoint) = waypoint {
        let distance = aircraft.pos.distance_squared(waypoint.value);

        match waypoint.behavior {
          NodeBehavior::GoTo => {}
          NodeBehavior::Park => {}
          NodeBehavior::HoldShort => {
            if distance <= 250.0_f32.powf(2.0) {
              bundle.events.push(
                AircraftEvent {
                  id: aircraft.id,
                  kind: EventKind::TaxiHold { and_state: true },
                }
                .into(),
              );
              if let AircraftState::Taxiing { waypoints, .. } =
                &mut aircraft.state
              {
                if let Some(last) = waypoints.last_mut() {
                  last.behavior = NodeBehavior::GoTo;
                }
              }
            }
          }

          // Runway specific
          NodeBehavior::LineUp => {}
          NodeBehavior::Takeoff => {}
        }
      }
    }
  }
}
