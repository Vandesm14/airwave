use std::f32::consts::PI;

use crate::{
  KNOT_TO_FEET_PER_SECOND, MAX_TAXI_SPEED, MIN_CRUISE_ALTITUDE,
  NAUTICALMILES_TO_FEET, TRANSITION_ALTITUDE,
  command::{CommandReply, CommandWithFreq},
  engine::Event,
  entities::world::World,
  geometry::{
    add_degrees, angle_between_points, calculate_ils_altitude,
    closest_point_on_line, delta_angle, inverse_degrees, move_point,
    normalize_angle,
  },
  line::Line,
  pathfinder::{Node, NodeBehavior, NodeKind},
};

use super::{
  Aircraft, AircraftState, FlightSegment, LandingState, TCAS,
  events::{AircraftEvent, EventKind},
};

// Engine Effects.
impl Aircraft {
  pub fn update_from_targets(&mut self, dt: f32) {
    // In feet per second
    let climb_speed = self.dt_climb_speed(dt);
    // In degrees per second
    let turn_speed = self.dt_turn_speed(dt);
    // In knots per second
    let speed_speed = self.dt_speed_speed(dt);

    let mut altitude = self.altitude;
    let mut heading = self.heading;
    let mut speed = self.speed;

    let target_altitude = match self.tcas {
      TCAS::Idle | TCAS::Warning => self.target.altitude,
      TCAS::Hold => self.altitude,
      TCAS::Climb => self.altitude + 1000.0,
      TCAS::Descend => self.altitude - 1000.0,
    };

    // Snap values if they're close enough
    if (altitude - target_altitude).abs() < climb_speed {
      altitude = target_altitude;
    }
    if (heading - self.target.heading).abs() < turn_speed {
      heading = self.target.heading;
    }
    if (speed - self.target.speed).abs() < speed_speed {
      speed = self.target.speed;
    }

    // Change if not equal
    if altitude != target_altitude {
      if altitude < target_altitude {
        altitude += climb_speed;
      } else {
        altitude -= climb_speed;
      }
    }
    if heading != self.target.heading {
      let delta_angle = delta_angle(heading, self.target.heading);
      if delta_angle < 0.0 {
        heading -= turn_speed;
      } else {
        heading += turn_speed;
      }
    }
    if speed != self.target.speed {
      if speed < self.target.speed {
        speed += speed_speed;
      } else {
        speed -= speed_speed;
      }
    }

    if altitude != self.altitude {
      self.altitude = altitude;
    }
    if heading != self.heading {
      self.heading = normalize_angle(heading);
    }
    if speed != self.speed {
      self.speed = speed;
    }
  }

  pub fn update_position(&mut self, dt: f32) {
    let pos = move_point(
      self.pos,
      self.heading,
      self.speed * KNOT_TO_FEET_PER_SECOND * dt,
    );

    if pos != self.pos {
      self.pos = pos;
    }
  }

  pub fn prune_waypoints(&mut self) {
    if let AircraftState::Flying = &mut self.state {
      let flight_plan = &mut self.flight_plan;
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
        let distance = self.pos.distance_squared(b.data.pos);

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
      self.target = self.target_waypoint_limits();
    }
  }

  pub fn update_flying(&mut self, events: &mut Vec<Event>, dt: f32) {
    if self.altitude < 2000.0 {
      return;
    }

    let speed_in_feet = self.speed * KNOT_TO_FEET_PER_SECOND * dt;

    self.prune_waypoints();

    if let AircraftState::Flying = &mut self.state {
      if let Some(current) = self.flight_plan.waypoint() {
        let heading = angle_between_points(self.pos, current.data.pos);

        self.target.heading = heading;

        let distance = self.pos.distance_squared(current.data.pos);
        let movement_speed = speed_in_feet.powf(2.0);

        if movement_speed >= distance {
          self.pos = current.data.pos;

          for e in current.data.events.iter() {
            events.push(AircraftEvent::new(self.id, e.clone()).into());
          }

          self.flight_plan.inc_index();
        }
      }
    }
  }

  pub fn update_taxiing(
    &mut self,
    events: &mut Vec<Event>,
    world: &World,
    dt: f32,
  ) {
    let speed_in_feet = self.speed * KNOT_TO_FEET_PER_SECOND * dt;
    if let AircraftState::Taxiing { current, .. } = &mut self.state {
      current.data = self.pos;
    }

    if let AircraftState::Taxiing {
      waypoints, current, ..
    } = &mut self.state
    {
      let waypoint = waypoints.last().cloned();
      if let Some(waypoint) = waypoint {
        let heading = angle_between_points(self.pos, waypoint.data);

        self.heading = heading;
        self.target.heading = heading;

        let distance = self.pos.distance_squared(waypoint.data);
        let movement_speed = speed_in_feet.powf(2.0);

        if movement_speed >= distance {
          if let Some(wp) = waypoints.pop() {
            *current = wp;
          }
        }
        // Only hold if we are not stopped and we are at or below taxi speed.
      } else if self.speed > 0.0 && self.speed <= 20.0 {
        events.push(
          AircraftEvent {
            id: self.id,
            kind: EventKind::TaxiHold { and_state: true },
          }
          .into(),
        );

        match current.behavior {
          NodeBehavior::GoTo => {}
          NodeBehavior::HoldShort => {}
          NodeBehavior::Park => {
            // bundle.events.push(
            //   AircraftEvent::new(
            //     aircraft.id,
            //     EventKind::Segment(FlightSegment::Parked),
            //   )
            //   .into(),
            // );
            self.state = AircraftState::Parked {
              at: current.clone(),
            };
          }

          // Runway specific
          NodeBehavior::LineUp => {
            if current.kind == NodeKind::Runway {
              if let Some(runway) = world
                .airports
                .iter()
                .find(|a| self.airspace.is_some_and(|id| a.id == id))
                .and_then(|x| x.runways.iter().find(|r| r.id == current.name))
              {
                self.heading = runway.heading;
                self.target.heading = runway.heading;
              }
            }
          }
          NodeBehavior::Takeoff => {
            if current.kind == NodeKind::Runway {
              events.push(
                AircraftEvent::new(self.id, EventKind::Takeoff(current.name))
                  .into(),
              );
            }
          }
        }
      }
    }

    if let AircraftState::Taxiing { waypoints, .. } = &self.state {
      let waypoint = waypoints.last();
      if let Some(waypoint) = waypoint {
        let distance = self.pos.distance_squared(waypoint.data);

        match waypoint.behavior {
          NodeBehavior::GoTo => {}
          NodeBehavior::Park => {}
          NodeBehavior::HoldShort => {
            if distance <= 250.0_f32.powf(2.0) {
              events.push(
                AircraftEvent {
                  id: self.id,
                  kind: EventKind::TaxiHold { and_state: true },
                }
                .into(),
              );
              if let AircraftState::Taxiing { waypoints, .. } = &mut self.state
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

  pub fn update_segment(
    &mut self,
    events: &mut Vec<Event>,
    world: &World,
    tick: usize,
  ) {
    let mut segment: Option<FlightSegment> = None;

    // Assert Dormant.
    if self.flight_time.is_none() {
      segment = Some(FlightSegment::Dormant);
    }

    // Assert Boarding and Parked.
    if let AircraftState::Parked { .. } = self.state {
      if let Some(flight_time) = self.flight_time {
        // Assert Boarding.
        if tick < flight_time {
          segment = Some(FlightSegment::Boarding);

          // Assert Parked.
        } else if tick >= flight_time {
          segment = Some(FlightSegment::Parked);
        }
      }
    }

    // Assert Taxi.
    if let AircraftState::Taxiing { .. } = self.state {
      if let Some(airspace) = self.airspace {
        // Assert TaxiDep.
        if airspace == self.flight_plan.departing {
          segment = Some(FlightSegment::TaxiDep);

        // Assert TaxiArr.
        } else if airspace == self.flight_plan.arriving {
          segment = Some(FlightSegment::TaxiArr);
        }
      }
    }

    if let AircraftState::Flying = self.state {
      let airport = world
        .airports
        .iter()
        .filter(|_| self.altitude <= TRANSITION_ALTITUDE)
        .filter(|a| {
          a.id == self.flight_plan.departing
            || a.id == self.flight_plan.arriving
        })
        .find(|a| self.airspace.is_some_and(|id| a.id == id));
      if let Some(airport) = airport {
        // Assert Departure.
        if airport.id == self.flight_plan.departing {
          segment = Some(FlightSegment::Departure);

        // Assert Approach.
        } else if airport.id == self.flight_plan.arriving {
          segment = Some(FlightSegment::Approach);
        }

        // Assert Climb.
      } else if self.altitude < self.target.altitude {
        segment = Some(FlightSegment::Climb);

        // Assert Cruise.
      } else if self.altitude == self.target.altitude
        && self.altitude >= MIN_CRUISE_ALTITUDE
      {
        segment = Some(FlightSegment::Cruise);

        // Assert Arrival.
      } else if self.altitude <= MIN_CRUISE_ALTITUDE
        || self.altitude >= self.target.altitude
      {
        segment = Some(FlightSegment::Arrival);
      }

      // Assert Takeoff.
      if self.altitude == 0.0 {
        segment = Some(FlightSegment::Takeoff);
      }
    }

    // Assert Landing.
    if let AircraftState::Landing { .. } = self.state {
      segment = Some(FlightSegment::Landing);
    }

    let segment = segment.unwrap_or_default();
    if segment != self.segment {
      if segment == FlightSegment::Unknown {
        tracing::warn!("Aircraft has an unknown segment: {:#?}", self);
      }

      events.push(
        AircraftEvent {
          id: self.id,
          kind: EventKind::Segment(segment),
        }
        .into(),
      );
    }
  }
}

// Landing Effect
impl Aircraft {
  fn state_before_turn(&mut self, dt: f32) {
    let degrees_per_sec = self.dt_turn_speed(dt);
    let AircraftState::Landing { runway, state } = &mut self.state else {
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
      turning_radius * self.speed * KNOT_TO_FEET_PER_SECOND * dt;
    let turning_radius = turning_radius / (2.0 * PI);
    let turning_radius = turning_radius * 2.0;

    let delta_ang = delta_angle(self.heading, runway.heading);
    let percent_of = delta_ang.abs() / 180.0;
    let percent_of = (percent_of * PI + PI * 1.5).sin() / 2.0 + 0.5;
    let turn_distance = turning_radius * percent_of;
    let turn_distance = turn_distance.powf(2.0);

    let closest_point = closest_point_on_line(self.pos, ils_line.0, ils_line.1);
    let distance_to_point = self.pos.distance_squared(closest_point);

    if distance_to_point <= turn_distance {
      self.target.heading = runway.heading;

      *state = LandingState::Turning;
    }

    let angle_to_runway =
      inverse_degrees(angle_between_points(runway.end(), self.pos));

    if self.heading.round() == runway.heading
      && (angle_to_runway.round() != runway.heading
        || distance_to_point.round() != 0.0)
    {
      if angle_to_runway > runway.heading {
        self.target.heading = add_degrees(runway.heading, 30.0);
      }

      if angle_to_runway < runway.heading {
        self.target.heading = add_degrees(runway.heading, -30.0);
      }

      *state = LandingState::Correcting;
    }

    if distance_to_point <= 50_f32.powf(2.0)
      && self.heading.round() == runway.heading
    {
      *state = LandingState::Localizer;
    }
  }

  fn state_touchdown(&mut self, events: &mut Vec<Event>) {
    let AircraftState::Landing { runway, state } = &mut self.state else {
      unreachable!("outer function asserts that aircraft is landing")
    };

    if *state != LandingState::Glideslope {
      return;
    }

    let distance_to_end = self.pos.distance_squared(runway.end());

    // If we have passed the start of the runway (landed),
    // set our state to taxiing.
    if distance_to_end <= runway.length.powf(2.0) {
      events.push(
        AircraftEvent {
          id: self.id,
          kind: EventKind::Touchdown,
        }
        .into(),
      );

      *state = LandingState::Touchdown
    }
  }

  fn state_go_around(&mut self, events: &mut Vec<Event>) {
    let AircraftState::Landing { runway, state } = &mut self.state else {
      unreachable!("outer function asserts that aircraft is landing")
    };

    if *state != LandingState::Glideslope {
      return;
    }

    let distance_to_runway = self.pos.distance(runway.start);
    let target_altitude = calculate_ils_altitude(distance_to_runway);

    // If we are too high, go around.
    if self.altitude - target_altitude > 100.0 {
      events.push(
        AircraftEvent {
          id: self.id,
          kind: EventKind::GoAround,
        }
        .into(),
      );
      events.push(
        AircraftEvent {
          id: self.id,
          kind: EventKind::Callout(CommandWithFreq::new(
            self.id.to_string(),
            self.frequency,
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

  pub fn update_landing(&mut self, events: &mut Vec<Event>, dt: f32) {
    if let AircraftState::Landing { .. } = &self.state {
      Self::state_touchdown(self, events);
      Self::state_go_around(self, events);
      Self::state_before_turn(self, dt);
      Self::state_glideslope(self, dt);
    }
  }

  pub fn update_airspace(&mut self, world: &World) {
    self.airspace = world.detect_airspace(self.pos).map(|a| a.id);
  }
}

// ATC Automation
impl Aircraft {
  pub fn update_auto_approach(
    &mut self,
    events: &mut Vec<Event>,
    world: &World,
  ) {
    if matches!(self.segment, FlightSegment::Approach)
      && self.airspace.is_some_and(|a| world.automated_arrivals(a))
    {
      if let Some(airport) = world
        .airports
        .iter()
        .find(|a| self.airspace.is_some_and(|id| id == a.id))
      {
        let runway = airport.runways.first().unwrap();
        let point_to = move_point(
          runway.start,
          runway.heading,
          -NAUTICALMILES_TO_FEET * 10.0,
        );

        let heading = angle_between_points(self.pos, point_to);
        let altitude = 4000.0;
        let speed = 200.0;

        if self.target.heading != heading {
          events.push(
            AircraftEvent::new(self.id, EventKind::Heading(heading)).into(),
          );
        }

        if self.target.altitude >= altitude {
          events.push(
            AircraftEvent::new(self.id, EventKind::Altitude(altitude)).into(),
          );
        }

        if self.target.speed >= speed {
          events
            .push(AircraftEvent::new(self.id, EventKind::Speed(speed)).into());
        }

        if matches!(self.state, AircraftState::Flying) {
          events.push(
            AircraftEvent::new(self.id, EventKind::Land(runway.id)).into(),
          )
        }
      }
    }
  }

  pub fn update_auto_ground(&mut self, events: &mut Vec<Event>, world: &World) {
    if matches!(self.segment, FlightSegment::TaxiArr)
      && self.airspace.is_some_and(|a| world.automated_arrivals(a))
    {
      if let AircraftState::Taxiing { waypoints, .. } = &self.state {
        if self.speed <= MAX_TAXI_SPEED
          && !waypoints.iter().any(|w| w.kind == NodeKind::Gate)
        {
          if let Some(airport) = world
            .airports
            .iter()
            .find(|a| self.airspace.is_some_and(|id| id == a.id))
          {
            let available_gate = airport
              .terminals
              .iter()
              .flat_map(|t| t.gates.iter())
              .find(|g| g.available);
            if let Some(gate) = available_gate {
              events.push(
                AircraftEvent::new(
                  self.id,
                  EventKind::Taxi(vec![Node::new(
                    gate.id,
                    NodeKind::Gate,
                    NodeBehavior::GoTo,
                    (),
                  )]),
                )
                .into(),
              );
            }
          }
        }
      }
    }
  }
}
