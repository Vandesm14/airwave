use std::f32::consts::PI;

use crate::{
  command::{CommandReply, CommandWithFreq},
  engine::Bundle,
  entities::world::{closest_airport, closest_airspace},
  geometry::{
    add_degrees, angle_between_points, calculate_ils_altitude,
    closest_point_on_line, delta_angle, duration_now, inverse_degrees,
    move_point, normalize_angle,
  },
  line::Line,
  pathfinder::{NodeBehavior, NodeKind},
  AIRSPACE_RADIUS, KNOT_TO_FEET_PER_SECOND, MIN_CRUISE_ALTITUDE,
  NAUTICALMILES_TO_FEET, TRANSITION_ALTITUDE,
};

use super::{
  events::{AircraftEvent, EventKind},
  Aircraft, AircraftState, FlightSegment, LandingState, TCAS,
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

    let target_altitude = match aircraft.tcas {
      TCAS::Idle | TCAS::Warning => aircraft.target.altitude,
      TCAS::Hold => aircraft.altitude,
      TCAS::Climb => aircraft.altitude + 1000.0,
      TCAS::Descend => aircraft.altitude - 1000.0,
    };

    // Snap values if they're close enough
    if (altitude - target_altitude).abs() < climb_speed {
      altitude = target_altitude;
    }
    if (heading - aircraft.target.heading).abs() < turn_speed {
      heading = aircraft.target.heading;
    }
    if (speed - aircraft.target.speed).abs() < speed_speed {
      speed = aircraft.target.speed;
    }

    // Change if not equal
    if altitude != target_altitude {
      if altitude < target_altitude {
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

    let distance_to_runway = aircraft.pos.distance(runway.start);
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
    let distance_to_runway = aircraft.pos.distance(runway.start);

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
impl AircraftUpdateFlyingEffect {
  fn prune_waypoints(aircraft: &mut Aircraft) {
    if let AircraftState::Flying = &mut aircraft.state {
      let flight_plan = &mut aircraft.flight_plan;
      if flight_plan.waypoints.len() < 2 {
        return;
      }

      let mut skip_amount = 0;
      for (i, wp) in flight_plan.waypoints.windows(2).enumerate() {
        let a = wp.first().unwrap();
        let b = wp.last().unwrap();

        // Distance between waypoint A and B
        let wp_distance = a.data.pos.distance_squared(b.data.pos);
        // Distance between the aircraft and waypoint B
        let distance = aircraft.pos.distance_squared(b.data.pos);

        // If the aircraft is closer to B than A is to B, just go to B
        if distance < wp_distance {
          skip_amount = i + 1;
        }
      }

      // Only set if we are skipping new waypoints. Don't decrease the index.
      if skip_amount > flight_plan.waypoint_index {
        flight_plan.set_index(skip_amount);
      }

      // Update targets based on waypoint limits.
      aircraft.target = aircraft.target_waypoint_limits();
    }
  }
}

impl AircraftEffect for AircraftUpdateFlyingEffect {
  fn run(aircraft: &mut Aircraft, bundle: &mut Bundle) {
    if aircraft.altitude < 2000.0 {
      return;
    }

    let dt = aircraft.dt_enroute(bundle.dt);
    let speed_in_feet = aircraft.speed * KNOT_TO_FEET_PER_SECOND * dt;

    AircraftUpdateFlyingEffect::prune_waypoints(aircraft);

    if let AircraftState::Flying = &mut aircraft.state {
      if let Some(current) = aircraft.flight_plan.waypoint() {
        let heading = angle_between_points(aircraft.pos, current.data.pos);

        aircraft.target.heading = heading;

        let distance = aircraft.pos.distance_squared(current.data.pos);
        let movement_speed = speed_in_feet.powf(2.0);

        if movement_speed >= distance {
          aircraft.pos = current.data.pos;

          for e in current.data.events.iter() {
            bundle
              .events
              .push(AircraftEvent::new(aircraft.id, e.clone()).into());
          }

          aircraft.flight_plan.inc_index();
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
        let heading = angle_between_points(aircraft.pos, waypoint.data);

        aircraft.heading = heading;
        aircraft.target.heading = heading;

        let distance = aircraft.pos.distance_squared(waypoint.data);
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
            };

            aircraft.flip_flight_plan();
            aircraft.flight_time = None;
          }

          // Runway specific
          NodeBehavior::LineUp => {
            if current.kind == NodeKind::Runway {
              if let Some(runway) =
                closest_airport(&bundle.world.airspaces, aircraft.pos)
                  .and_then(|x| x.runways.iter().find(|r| r.id == current.name))
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
        let distance = aircraft.pos.distance_squared(waypoint.data);

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

pub struct AircraftUpdateSegmentEffect;
impl AircraftEffect for AircraftUpdateSegmentEffect {
  fn run(aircraft: &mut Aircraft, bundle: &mut Bundle) {
    let mut segment: Option<FlightSegment> = None;

    // Assert Dormant.
    if aircraft.flight_time.is_none() {
      segment = Some(FlightSegment::Dormant);
    }

    // Assert Boarding and Parked.
    if let AircraftState::Parked { .. } = aircraft.state {
      if let Some(flight_time) = aircraft.flight_time {
        // Assert Boarding.
        if duration_now() < flight_time {
          segment = Some(FlightSegment::Boarding);

          // Assert Parked.
        } else if duration_now() >= flight_time {
          segment = Some(FlightSegment::Parked);
        }
      }
    }

    // Assert Taxi.
    if let AircraftState::Taxiing { .. } = aircraft.state {
      let airport = closest_airport(&bundle.world.airspaces, aircraft.pos);
      if let Some(airport) = airport {
        // Assert TaxiDep.
        if airport.id == aircraft.flight_plan.departing {
          segment = Some(FlightSegment::TaxiDep);

        // Assert TaxiArr.
        } else if airport.id == aircraft.flight_plan.arriving {
          segment = Some(FlightSegment::TaxiArr);
        }
      }
    }

    if let AircraftState::Flying = aircraft.state {
      let airspace = closest_airspace(&bundle.world.airspaces, aircraft.pos)
        .filter(|a| {
          a.pos.distance_squared(aircraft.pos) <= AIRSPACE_RADIUS.powf(2.0)
        });
      if let Some(airspace) = airspace {
        // Assert Departure.
        if airspace.id == aircraft.flight_plan.departing {
          segment = Some(FlightSegment::Departure);

        // Assert Approach.
        } else if airspace.id == aircraft.flight_plan.arriving {
          segment = Some(FlightSegment::Approach);
        }

        // Assert Climb.
      } else if aircraft.altitude < aircraft.target.altitude {
        segment = Some(FlightSegment::Climb);

        // Assert Cruise.
      } else if aircraft.altitude == aircraft.target.altitude
        && aircraft.altitude >= MIN_CRUISE_ALTITUDE
      {
        segment = Some(FlightSegment::Cruise);

        // Assert Arrival.
      } else if aircraft.altitude <= MIN_CRUISE_ALTITUDE
        || aircraft.altitude >= aircraft.target.altitude
      {
        segment = Some(FlightSegment::Arrival);
      }

      // Assert Takeoff.
      if aircraft.altitude == 0.0 {
        segment = Some(FlightSegment::Takeoff);
      }
    }

    // Assert Landing.
    if let AircraftState::Landing { .. } = aircraft.state {
      segment = Some(FlightSegment::Landing);
    }

    let segment = segment.unwrap_or_default();
    if segment != aircraft.segment {
      if segment == FlightSegment::Unknown {
        tracing::warn!("Aircraft has an unknown segment: {:#?}", aircraft);
      }

      bundle.events.push(
        AircraftEvent {
          id: aircraft.id,
          kind: EventKind::Segment(segment),
        }
        .into(),
      );
    }
  }
}
