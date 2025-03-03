use glam::Vec2;
use nannou::geom;
use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, SlotMap};

new_key_type! { pub struct PointKey; }

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorldFile {
  pub points: SlotMap<PointKey, Vec2>,
}

impl WorldFile {
  pub fn find_closest_point(
    &self,
    test_point: Vec2,
    threshold: f32,
  ) -> Option<(PointKey, Vec2)> {
    let mut smallest_distance = threshold;
    let mut point: Option<(PointKey, Vec2)> = None;
    for p in self.points.iter() {
      let distance = p.1.distance_squared(test_point);
      if distance < smallest_distance {
        smallest_distance = distance;
        point = Some((p.0, *p.1));
      }
    }

    point
  }
}

pub fn geom_to_glam(geom: geom::Vec2) -> Vec2 {
  Vec2::new(geom.x, geom.y)
}
