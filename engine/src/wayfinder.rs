use glam::Vec2;
use internment::Intern;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
  TRANSITION_ALTITUDE, ToText,
  entities::aircraft::{Aircraft, events::EventKind},
  geometry::{angle_between_points, delta_angle, normalize_angle},
  pathfinder::{Node, NodeBehavior, NodeKind},
  sign3,
};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub enum VORLimit {
  #[default]
  None,

  At(f32),
  AtOrAbove(f32),
  AtOrBelow(f32),
}

impl VORLimit {
  pub fn test(&self, value: f32) -> bool {
    match self {
      Self::None => true,
      Self::At(limit) => value == *limit,
      Self::AtOrAbove(limit) => value >= *limit,
      Self::AtOrBelow(limit) => value <= *limit,
    }
  }

  pub fn diff(&self, value: f32) -> f32 {
    match self {
      Self::None => 0.0,
      Self::At(limit) => *limit - value,
      Self::AtOrAbove(limit) => {
        if value >= *limit {
          0.0
        } else {
          *limit - value
        }
      }
      Self::AtOrBelow(limit) => {
        if value <= *limit {
          0.0
        } else {
          *limit - value
        }
      }
    }
  }

  pub fn is_none(&self) -> bool {
    matches!(self, Self::None)
  }

  pub fn is_some(&self) -> bool {
    !self.is_none()
  }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct VORLimits {
  pub altitude: VORLimit,
  pub speed: VORLimit,
}

impl VORLimits {
  pub fn new() -> Self {
    Self {
      altitude: VORLimit::None,
      speed: VORLimit::None,
    }
  }

  pub fn with_altitude(mut self, altitude: VORLimit) -> Self {
    self.altitude = altitude;
    self
  }

  pub fn with_speed(mut self, speed: VORLimit) -> Self {
    self.speed = speed;
    self
  }

  pub fn is_none(&self) -> bool {
    self.altitude.is_none() && self.speed.is_none()
  }

  pub fn is_some(&self) -> bool {
    !self.is_none()
  }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct VORData {
  #[ts(as = "(f32, f32)")]
  pub pos: Vec2,
  #[serde(skip)]
  pub events: Vec<EventKind>,
  #[serde(skip)]
  pub limits: VORLimits,
}

impl VORData {
  pub fn new(to: Vec2) -> Self {
    Self {
      pos: to,
      events: vec![],
      limits: VORLimits::default(),
    }
  }
}

pub fn new_vor(name: Intern<String>, to: Vec2) -> Node<VORData> {
  Node {
    name,
    kind: NodeKind::VOR,
    behavior: NodeBehavior::GoTo,
    data: VORData::new(to),
  }
}

impl Node<VORData> {
  pub fn with_action(mut self, behavior: EventKind) -> Self {
    self.data.events.push(behavior);
    self
  }

  pub fn with_actions(mut self, behavior: Vec<EventKind>) -> Self {
    self.data.events = behavior;
    self
  }

  pub fn with_limits(mut self, limits: VORLimits) -> Self {
    self.data.limits = limits;
    self
  }

  pub fn with_altitude_limit(mut self, limits: VORLimit) -> Self {
    self.data.limits.altitude = limits;
    self
  }

  pub fn with_speed_limit(mut self, limits: VORLimit) -> Self {
    self.data.limits.speed = limits;
    self
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FlightPlan {
  // To and From
  #[ts(as = "String")]
  pub arriving: Intern<String>,
  #[ts(as = "String")]
  pub departing: Intern<String>,

  pub waypoints: Vec<Node<VORData>>,
  pub waypoint_index: usize,

  pub follow: bool,
  pub course_offset: f32,

  // Initial Clearance
  pub speed: f32,
  pub altitude: f32,
}

impl Default for FlightPlan {
  fn default() -> Self {
    Self {
      arriving: Intern::from_ref(""),
      departing: Intern::from_ref(""),

      waypoints: Vec::new(),
      waypoint_index: 0,

      follow: true,
      course_offset: 0.0,

      speed: 450.0,
      altitude: TRANSITION_ALTITUDE,
    }
  }
}

impl ToText for FlightPlan {
  fn to_text(&self, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
    write!(
      w,
      "Flight Plan: {} to {} (dep alt: {}ft)",
      self.departing,
      self.arriving,
      self.altitude.round()
    )?;
    if !self.waypoints.is_empty() {
      writeln!(w)?;
      write!(w, "Waypoints: ")?;
      for (i, wp) in self
        .waypoints
        .iter()
        .skip(self.waypoint_index)
        .rev()
        .enumerate()
        .rev()
      {
        write!(w, "{}", wp.name)?;
        if i != 0 {
          write!(w, " ")?;
        }
      }
    }

    Ok(())
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

  pub fn with_waypoints(mut self, waypoints: Vec<Node<VORData>>) -> Self {
    self.waypoints = waypoints;
    self.waypoint_index = 0;

    self
  }
}

impl FlightPlan {
  pub fn clear_waypoints(&mut self) {
    self.waypoints.clear();
    self.waypoint_index = 0;
    self.start_following();
  }

  pub fn active_waypoints(&self) -> Vec<Node<VORData>> {
    self
      .waypoints
      .iter()
      .skip(self.waypoint_index)
      .cloned()
      .collect()
  }

  pub fn waypoint(&self) -> Option<&Node<VORData>> {
    if self.follow {
      self.waypoints.get(self.waypoint_index)
    } else {
      None
    }
  }

  pub fn stop_following(&mut self) {
    self.follow = false;
  }

  pub fn start_following(&mut self) {
    self.follow = true;
  }

  pub fn inc_index(&mut self) {
    self.set_index(self.waypoint_index + 1);
    self.clamp_index();
  }

  pub fn dec_index(&mut self) {
    self.set_index(self.waypoint_index + 1);
    self.clamp_index();
  }

  pub fn set_index(&mut self, index: usize) {
    self.start_following();
    self.waypoint_index = index;
    self.clamp_index();
  }

  fn clamp_index(&mut self) {
    if self.waypoints.is_empty() {
      self.waypoint_index = 0;
    } else {
      self.waypoint_index = self.waypoint_index.clamp(0, self.waypoints.len());
    }
  }

  pub fn index(&self) -> usize {
    self.waypoint_index
  }

  pub fn at_end(&self) -> bool {
    self.waypoint_index == self.waypoints.len() || self.waypoints.is_empty()
  }

  pub fn amend_end(&mut self, waypoints: Vec<Node<VORData>>) {
    let len = waypoints.len();
    let already_exists = self
      .waypoints
      .iter()
      .rev()
      .take(len)
      .rev()
      .enumerate()
      .all(|(i, wp)| {
        if let Some(new_wp) = waypoints.get(i) {
          wp == new_wp
        } else {
          false
        }
      });

    if self.waypoints.is_empty() || !already_exists {
      self.waypoints.extend(waypoints);
    }

    self.set_index(self.waypoints.len() - len);
  }

  pub fn distances(&self, pos: Vec2) -> Vec<f32> {
    let mut pos = pos;
    let mut distance = 0.0;
    let mut distances = Vec::with_capacity(self.active_waypoints().len());
    for wp in self.active_waypoints() {
      let dist = pos.distance(wp.data.pos) + distance;
      distance = dist;
      pos = wp.data.pos;

      distances.push(dist);
    }

    distances
  }

  pub fn heading(&self, pos: Vec2) -> Option<f32> {
    self
      .waypoint()
      .map(|wp| angle_between_points(pos, wp.data.pos))
  }

  pub fn course_heading(&self, aircraft: &Aircraft) -> Option<f32> {
    if !self.follow {
      return None;
    }

    self
      .heading(aircraft.pos)
      .map(|heading| normalize_angle(heading + self.course_offset))
  }

  pub fn next_heading(&self) -> Option<f32> {
    let next_two = self.active_waypoints();
    let mut next_two = next_two.iter();
    let next_two = next_two.next().zip(next_two.next());
    if let Some((a, b)) = next_two {
      let angle = angle_between_points(a.data.pos, b.data.pos);

      Some(angle)
    } else {
      None
    }
  }

  pub fn turn_bias(&self, aircraft: &Aircraft) -> f32 {
    if self.active_waypoints().len() == 1 {
      return 0.0;
    }

    let mut bias = 0.0;
    let mut last_pos = aircraft.pos;
    let mut last_hdg = aircraft.heading;
    for active in self.active_waypoints() {
      let course = angle_between_points(last_pos, active.data.pos);
      let diff = delta_angle(last_hdg, course);

      bias += sign3(diff);

      last_pos = active.data.pos;
      last_hdg = course;
    }

    bias
  }
}
