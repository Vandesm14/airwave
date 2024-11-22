use std::{
  f32::consts::PI,
  ops::Add,
  time::{Duration, SystemTime},
};

use turborand::TurboRand;

use crate::{
  add_degrees, angle_between_points, calculate_ils_altitude,
  closest_point_on_line,
  command::{CommandReply, CommandWithFreq},
  delta_angle,
  engine::Bundle,
  entities::airport::Runway,
  inverse_degrees, move_point, normalize_angle,
  pathfinder::NodeBehavior,
  Line, DEPARTURE_WAIT_RANGE, KNOT_TO_FEET_PER_SECOND, NAUTICALMILES_TO_FEET,
};

use super::{
  actions::ActionKind,
  events::{AircraftEvent, EventKind},
  Action, Aircraft, AircraftState, LandingState, TaxiingState,
};

pub trait AircraftEffect {
  fn run(aircraft: &Aircraft, bundle: &mut Bundle);
}

pub struct AircraftUpdateFromTargetsEffect;
impl AircraftEffect for AircraftUpdateFromTargetsEffect {
  fn run(aircraft: &Aircraft, bundle: &mut Bundle) {
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
    let dt = aircraft.dt_enroute(bundle.dt);

    let pos = move_point(
      aircraft.pos,
      aircraft.heading,
      aircraft.speed * KNOT_TO_FEET_PER_SECOND * dt,
    );

    if pos != aircraft.pos {
      bundle
        .actions
        .push(Action::new(aircraft.id, ActionKind::Pos(pos)));
    }
  }
}

pub struct AircraftUpdateLandingEffect;
impl AircraftUpdateLandingEffect {
  fn state_before_turn(
    aircraft: &Aircraft,
    bundle: &mut Bundle,
    dt: f32,
    runway: &Runway,
    ils_line: Line,
    mut state: LandingState,
  ) -> LandingState {
    let degrees_per_sec = aircraft.dt_turn_speed(dt);
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
      bundle.actions.push(Action::new(
        aircraft.id,
        ActionKind::TargetHeading(runway.heading),
      ));

      state = LandingState::Turning;
    } else if aircraft.speed > aircraft.target.speed {
      bundle.actions.push(Action::new(
        aircraft.id,
        ActionKind::TargetHeading(aircraft.heading),
      ));

      state = LandingState::BeforeTurn;
    }

    let angle_to_runway =
      inverse_degrees(angle_between_points(runway.end(), aircraft.pos));

    if aircraft.heading.round() == runway.heading
      && (angle_to_runway.round() != runway.heading
        || distance_to_point.round() != 0.0)
    {
      if angle_to_runway > runway.heading {
        bundle.actions.push(Action::new(
          aircraft.id,
          ActionKind::TargetHeading(add_degrees(runway.heading, 20.0)),
        ));
      }

      if angle_to_runway < runway.heading {
        bundle.actions.push(Action::new(
          aircraft.id,
          ActionKind::TargetHeading(add_degrees(runway.heading, -20.0)),
        ));
      }

      state = LandingState::Correcting;
    }

    if distance_to_point <= 50_f32.powf(2.0)
      && aircraft.heading.round() == runway.heading
    {
      state = LandingState::Localizer;
    }

    state
  }

  fn state_touchdown(
    aircraft: &Aircraft,
    bundle: &mut Bundle,
    runway: &Runway,
    mut state: LandingState,
  ) -> LandingState {
    if state != LandingState::Glideslope {
      return state;
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

      state = LandingState::Touchdown
    }

    state
  }

  fn state_go_around(
    aircraft: &Aircraft,
    bundle: &mut Bundle,
    runway: &Runway,
    mut state: LandingState,
  ) -> LandingState {
    if state != LandingState::Glideslope {
      return state;
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

      state = LandingState::GoAround;
    }

    state
  }

  fn state_glideslope(
    aircraft: &Aircraft,
    bundle: &mut Bundle,
    dt: f32,
    runway: &Runway,
    mut state: LandingState,
  ) -> LandingState {
    if !(state == LandingState::Localizer || state == LandingState::Glideslope)
    {
      return state;
    }

    let start_descent_distance = NAUTICALMILES_TO_FEET * 10.0;
    let distance_to_runway = aircraft.pos.distance(runway.start());

    let angle_to_runway =
      inverse_degrees(angle_between_points(runway.end(), aircraft.pos));
    let angle_range = (runway.heading - 5.0)..=(runway.heading + 5.0);

    let climb_speed = aircraft.dt_climb_speed(dt);
    let seconds_for_descent = aircraft.altitude / (climb_speed / dt);

    let target_speed_ft_s = distance_to_runway / seconds_for_descent;
    let target_knots = target_speed_ft_s / KNOT_TO_FEET_PER_SECOND;

    let target_altitude = calculate_ils_altitude(distance_to_runway);

    // If we aren't within the localizer beacon (+/- 5 degrees), don't do
    // anything.
    if angle_range.contains(&angle_to_runway)
      && distance_to_runway <= start_descent_distance
    {
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

        state = LandingState::Glideslope;
      }
    }

    state
  }
}

impl AircraftEffect for AircraftUpdateLandingEffect {
  fn run(aircraft: &Aircraft, bundle: &mut Bundle) {
    if let AircraftState::Landing { runway, state } = &aircraft.state {
      let dt = aircraft.dt_enroute(bundle.dt);

      let ils_line = Line::new(
        move_point(runway.end(), runway.heading, 500.0),
        move_point(
          runway.end(),
          inverse_degrees(runway.heading),
          NAUTICALMILES_TO_FEET * 10.0 + runway.length,
        ),
      );

      let s = &Self::state_touchdown(aircraft, bundle, runway, *state);
      let s = &Self::state_go_around(aircraft, bundle, runway, *s);
      let s =
        &Self::state_before_turn(aircraft, bundle, dt, runway, ils_line, *s);
      let s = &Self::state_glideslope(aircraft, bundle, dt, runway, *s);

      bundle
        .actions
        .push(Action::new(aircraft.id, ActionKind::LandingState(*s)));
    }
  }
}

pub struct AircraftUpdateFlyingEffect;
impl AircraftEffect for AircraftUpdateFlyingEffect {
  fn run(aircraft: &Aircraft, bundle: &mut Bundle) {
    if aircraft.altitude < 2000.0 {
      return;
    }

    let dt = aircraft.dt_enroute(bundle.dt);
    let speed_in_feet = aircraft.speed * KNOT_TO_FEET_PER_SECOND * dt;
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

          for e in current.value.then.iter() {
            bundle
              .events
              .push(AircraftEvent::new(aircraft.id, e.clone()).into());
          }
        }
      }
    }
  }
}

pub struct AircraftUpdateTaxiingEffect;
impl AircraftEffect for AircraftUpdateTaxiingEffect {
  fn run(aircraft: &Aircraft, bundle: &mut Bundle) {
    let speed_in_feet = aircraft.speed * KNOT_TO_FEET_PER_SECOND;
    if let AircraftState::Taxiing {
      waypoints, current, ..
    } = &aircraft.state
    {
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
        bundle.events.push(
          AircraftEvent {
            id: aircraft.id,
            kind: EventKind::TaxiHold,
          }
          .into(),
        );
        bundle.actions.push(Action {
          id: aircraft.id,
          kind: ActionKind::TaxiingState(TaxiingState::Holding),
        });

        if let NodeBehavior::Park = current.behavior {
          bundle.actions.push(Action {
            id: aircraft.id,
            kind: ActionKind::Parked {
              at: current.clone(),
              ready_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .add(Duration::from_secs(
                  bundle.rng.sample_iter(DEPARTURE_WAIT_RANGE).unwrap(),
                )),
            },
          });
          bundle.actions.push(Action {
            id: aircraft.id,
            kind: ActionKind::FlipFlightPlan,
          });
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
              bundle.actions.push(Action {
                id: aircraft.id,
                kind: ActionKind::TaxiLastAsGoto,
              });
              bundle.actions.push(Action {
                id: aircraft.id,
                kind: ActionKind::TaxiingState(TaxiingState::Holding),
              });
              bundle.events.push(
                AircraftEvent {
                  id: aircraft.id,
                  kind: EventKind::TaxiHold,
                }
                .into(),
              );
            }
          }
        }
      }
    }
  }
}
