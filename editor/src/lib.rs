use glam::Vec2;
use nannou::geom;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct WorldFile {
  pub points: Vec<Vec2>,
}

impl WorldFile {
  pub fn new(points: Vec<Vec2>) -> Self {
    Self { points }
  }

  pub fn find_closest_point(
    &self,
    point: Vec2,
    threshold: f32,
  ) -> Option<usize> {
    let mut smallest_distance = threshold;
    let mut index: Option<usize> = None;
    for (i, p) in self.points.iter().enumerate() {
      let distance = p.distance_squared(point);
      if distance < smallest_distance {
        smallest_distance = distance;
        index = Some(i);
      }
    }
    index
  }
}

pub fn geom_to_glam(geom: geom::Vec2) -> Vec2 {
  Vec2::new(geom.x, geom.y)
}
