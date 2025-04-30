use glam::Vec2;
use internment::Intern;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
  entities::aircraft::events::EventKind,
  pathfinder::{Node, NodeBehavior, NodeKind},
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
  pub fn with_actions(mut self, behavior: Vec<EventKind>) -> Self {
    self.data.events = behavior;
    self
  }

  pub fn with_limits(mut self, limits: VORLimits) -> Self {
    self.data.limits = limits;
    self
  }
}
