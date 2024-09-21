use std::{collections::HashMap, f32::consts::PI};

use entities::airport::{Runway, Taxiway, Terminal};
use glam::Vec2;
use internment::Intern;
// use objects::airport::{Runway, Taxiway, Terminal};
use serde::{Deserialize, Serialize, Serializer};
use turborand::{rng::Rng, TurboRand};

pub mod engine;
pub mod pathfinder;

pub mod command;
pub mod entities;

pub const TIME_SCALE: f32 = 1.0;

pub const NAUTICALMILES_TO_FEET: f32 = 6076.115;
pub const KNOT_TO_FEET_PER_SECOND: f32 = 1.68781 * TIME_SCALE;

pub const UP: f32 = 0.0;
pub const DOWN: f32 = 180.0;
pub const LEFT: f32 = 270.0;
pub const RIGHT: f32 = 90.0;

pub const CLOCKWISE: f32 = 90.0;
pub const COUNTERCLOCKWISE: f32 = 270.0;

pub fn normalize_angle(angle: f32) -> f32 {
  (360.0 + angle) % 360.0
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Line(pub Vec2, pub Vec2);

impl Line {
  pub fn new(a: Vec2, b: Vec2) -> Self {
    Self(a, b)
  }

  pub fn midpoint(&self) -> Vec2 {
    self.0.midpoint(self.1)
  }

  pub fn extend(&self, padding: f32) -> Self {
    Self(
      self.0.move_towards(self.1, -padding),
      self.1.move_towards(self.0, -padding),
    )
  }

  pub fn length(&self) -> f32 {
    self.0.distance(self.1)
  }
}

impl From<Runway> for Line {
  fn from(value: Runway) -> Self {
    Line::new(value.start(), value.end())
  }
}

impl From<Taxiway> for Line {
  fn from(value: Taxiway) -> Self {
    Line::new(value.a, value.b)
  }
}

impl From<Terminal> for Line {
  fn from(value: Terminal) -> Self {
    value.apron
  }
}

pub fn calculate_ils_altitude(distance: f32) -> f32 {
  let slope_radians = 7.0_f32.to_radians();
  distance * slope_radians.tan()
}

pub fn move_point(point: Vec2, degrees: f32, length: f32) -> Vec2 {
  // Convert degrees to radians
  let radians = degrees * (PI / 180.0);

  // Calculate x and y components
  let x = length * radians.sin();
  let y = length * radians.cos();

  // Create and return the new Vec2
  point + Vec2::new(x, y)
}

pub fn add_degrees(degrees: f32, add: f32) -> f32 {
  (degrees + add) % 360.0
}

pub fn subtract_degrees(degrees: f32, subtract: f32) -> f32 {
  (360.0 + degrees - subtract) % 360.0
}

pub fn inverse_degrees(degrees: f32) -> f32 {
  add_degrees(degrees, 180.0)
}

pub fn delta_angle(current: f32, target: f32) -> f32 {
  ((target - current + 540.0) % 360.0) - 180.0
}

pub fn angle_between_points(a: Vec2, b: Vec2) -> f32 {
  let dx = b.x - a.x;
  let dy = b.y - a.y;
  let angle = dx.atan2(dy).to_degrees();
  if angle < 0.0 {
    angle + 360.0
  } else {
    angle
  }
}

pub fn find_line_intersection(a: Line, b: Line) -> Option<Vec2> {
  // Calculate direction vectors
  let line1_dir = a.1 - a.0;
  let line2_dir = b.1 - b.0;

  // Calculate the determinant
  let det = line1_dir.x * line2_dir.y - line1_dir.y * line2_dir.x;

  // Check if lines are parallel (or coincident)
  if det.abs() < f32::EPSILON {
    return None;
  }

  // Calculate the differences between start points
  let dp = b.0 - a.0;

  // Calculate the parameters t and s
  let t = (dp.x * line2_dir.y - dp.y * line2_dir.x) / det;
  let s = (dp.x * line1_dir.y - dp.y * line1_dir.x) / det;

  // Check if the intersection point lies on both line segments
  if !(0.0..=1.0).contains(&t) || !(0.0..=1.0).contains(&s) {
    return None;
  }

  // Calculate the intersection point
  let intersection = a.0 + line1_dir * t;

  Some(intersection)
}

pub fn find_projected_intersection(a: Line, b: Line) -> Option<Vec2> {
  // Calculate direction vectors
  let line1_dir = a.1 - a.0;
  let line2_dir = b.1 - b.0;

  // Calculate the determinant
  let det = line1_dir.x * line2_dir.y - line1_dir.y * line2_dir.x;

  // Check if lines are parallel (or coincident)
  if det.abs() < f32::EPSILON {
    return None;
  }

  // Calculate the differences between start points
  let dp = b.0 - a.0;

  // Calculate the parameters t and s
  let t = (dp.x * line2_dir.y - dp.y * line2_dir.x) / det;

  // Calculate the intersection point
  let intersection = a.0 + line1_dir * t;

  Some(intersection)
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CirclePoint {
  pub position: Vec2,
  pub angle: f32,
}

pub fn get_random_point_on_circle(
  center: Vec2,
  radius: f32,
  rng: &mut Rng,
) -> CirclePoint {
  // Generate a random angle in radians
  let random_angle = rng.f32() * 2.0 * PI;

  // Calculate the position of the point on the circle
  let offset = Vec2::from_angle(random_angle) * radius;
  let position = center + offset;

  CirclePoint {
    position,
    angle: random_angle,
  }
}

pub fn heading_to_direction(heading: f32) -> &'static str {
  // Normalize the heading to be between 0 and 360
  let normalized_heading = heading.rem_euclid(360.0);

  // Define the directions and their corresponding angle ranges
  let directions = [
    ("North", 0.0, 22.5),
    ("Northeast", 22.5, 67.5),
    ("East", 67.5, 112.5),
    ("Southeast", 112.5, 157.5),
    ("South", 157.5, 202.5),
    ("Southwest", 202.5, 247.5),
    ("West", 247.5, 292.5),
    ("Northwest", 292.5, 337.5),
    ("North", 337.5, 360.0),
  ];

  // Find the matching direction
  for (direction, start, end) in directions.iter() {
    if normalized_heading >= *start && normalized_heading < *end {
      return direction;
    }
  }

  // This should never happen, but we'll return "Unknown" just in case
  "Unknown"
}

/// Abbreviates an altitude to feet or flight level (depending on the altitude).
pub fn abbreviate_altitude(altitude: f32) -> String {
  if altitude < 13000.0 {
    format!("{:?} feet", altitude)
  } else {
    format!("Flight Level {:?}", altitude)
  }
}

fn closest_point_on_line(
  point: Vec2,
  line_start: Vec2,
  line_end: Vec2,
) -> Vec2 {
  // Calculate the direction vector of the line
  let line_dir = line_end - line_start;

  // Normalize the direction vector
  let line_dir_normalized = line_dir.normalize();

  // Calculate the vector from line_start to the point
  let point_vector = point - line_start;

  // Calculate the projection of point_vector onto the line direction
  let projection = point_vector.dot(line_dir_normalized);

  // Calculate the closest point on the line
  line_start + line_dir_normalized * projection
}

pub fn circle_circle_intersection(
  lhs_position: Vec2,
  rhs_position: Vec2,
  lhs_radius: f32,
  rhs_radius: f32,
) -> bool {
  let dx = lhs_position.x - rhs_position.x;
  let dy = lhs_position.y - rhs_position.y;
  let d = (dx * dx + dy * dy).sqrt();
  d <= lhs_radius + rhs_radius
}

#[cfg(test)]
mod tests {
  use super::*;

  mod angle_between_points {
    use super::*;

    #[test]
    fn test_angle_between_points_origin() {
      let a = Vec2::new(0.0, 0.0);

      // Zero
      let b = Vec2::new(a.x, a.y);
      let angle = angle_between_points(a, b);
      assert_eq!(angle, 0.0);

      // Up
      let b = Vec2::new(a.x, a.y + 1.0);
      let angle = angle_between_points(a, b);
      assert_eq!(angle, 0.0);

      // Down
      let b = Vec2::new(a.x, a.y - 1.0);
      let angle = angle_between_points(a, b);
      assert_eq!(angle, 180.0);

      // Right
      let b = Vec2::new(a.x + 1.0, a.y);
      let angle = angle_between_points(a, b);
      assert_eq!(angle, 90.0);

      // Left
      let b = Vec2::new(a.x - 1.0, a.y);
      let angle = angle_between_points(a, b);
      assert_eq!(angle, 270.0);
    }

    #[test]
    fn test_angle_between_points_top_right() {
      let a = Vec2::new(10.0, 10.0);

      // Zero
      let b = Vec2::new(a.x, a.y);
      let angle = angle_between_points(a, b);
      assert_eq!(angle, 0.0);

      // Up
      let b = Vec2::new(a.x, a.y + 1.0);
      let angle = angle_between_points(a, b);
      assert_eq!(angle, 0.0);

      // Down
      let b = Vec2::new(a.x, a.y - 1.0);
      let angle = angle_between_points(a, b);
      assert_eq!(angle, 180.0);

      // Right
      let b = Vec2::new(a.x + 1.0, a.y);
      let angle = angle_between_points(a, b);
      assert_eq!(angle, 90.0);

      // Left
      let b = Vec2::new(a.x - 1.0, a.y);
      let angle = angle_between_points(a, b);
      assert_eq!(angle, 270.0);
    }

    #[test]
    fn test_angle_between_points_top_left() {
      let a = Vec2::new(-10.0, 10.0);

      // Zero
      let b = Vec2::new(a.x, a.y);
      let angle = angle_between_points(a, b);
      assert_eq!(angle, 0.0);

      // Up
      let b = Vec2::new(a.x, a.y + 1.0);
      let angle = angle_between_points(a, b);
      assert_eq!(angle, 0.0);

      // Down
      let b = Vec2::new(a.x, a.y - 1.0);
      let angle = angle_between_points(a, b);
      assert_eq!(angle, 180.0);

      // Right
      let b = Vec2::new(a.x + 1.0, a.y);
      let angle = angle_between_points(a, b);
      assert_eq!(angle, 90.0);

      // Left
      let b = Vec2::new(a.x - 1.0, a.y);
      let angle = angle_between_points(a, b);
      assert_eq!(angle, 270.0);
    }

    #[test]
    fn test_angle_between_points_bottom_right() {
      let a = Vec2::new(10.0, -10.0);

      // Zero
      let b = Vec2::new(a.x, a.y);
      let angle = angle_between_points(a, b);
      assert_eq!(angle, 0.0);

      // Up
      let b = Vec2::new(a.x, a.y + 1.0);
      let angle = angle_between_points(a, b);
      assert_eq!(angle, 0.0);

      // Down
      let b = Vec2::new(a.x, a.y - 1.0);
      let angle = angle_between_points(a, b);
      assert_eq!(angle, 180.0);

      // Right
      let b = Vec2::new(a.x + 1.0, a.y);
      let angle = angle_between_points(a, b);
      assert_eq!(angle, 90.0);

      // Left
      let b = Vec2::new(a.x - 1.0, a.y);
      let angle = angle_between_points(a, b);
      assert_eq!(angle, 270.0);
    }

    #[test]
    fn test_angle_between_points_bottom_left() {
      let a = Vec2::new(-10.0, -10.0);

      // Zero
      let b = Vec2::new(a.x, a.y);
      let angle = angle_between_points(a, b);
      assert_eq!(angle, 0.0);

      // Up
      let b = Vec2::new(a.x, a.y + 1.0);
      let angle = angle_between_points(a, b);
      assert_eq!(angle, 0.0);

      // Down
      let b = Vec2::new(a.x, a.y - 1.0);
      let angle = angle_between_points(a, b);
      assert_eq!(angle, 180.0);

      // Right
      let b = Vec2::new(a.x + 1.0, a.y);
      let angle = angle_between_points(a, b);
      assert_eq!(angle, 90.0);

      // Left
      let b = Vec2::new(a.x - 1.0, a.y);
      let angle = angle_between_points(a, b);
      assert_eq!(angle, 270.0);
    }
  }

  mod find_line_intersection {
    use super::*;

    #[test]
    fn test_find_line_intersection() {
      let a = Line(Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0));
      let b = Line(Vec2::new(0.0, 0.0), Vec2::new(0.0, 10.0));

      let intersection = find_line_intersection(a, b);
      assert_eq!(intersection, Some(Vec2::new(0.0, 0.0)));
    }

    #[test]
    fn test_find_line_intersection_equal() {
      let a = Line(Vec2::new(-5.0, 0.0), Vec2::new(5.0, 0.0));
      let b = Line(Vec2::new(0.0, -5.0), Vec2::new(0.0, 5.0));

      let intersection = find_line_intersection(a, b);
      assert_eq!(intersection, Some(Vec2::new(0.0, 0.0)));
    }
  }
}
