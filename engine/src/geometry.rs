use std::f32::consts::PI;

use glam::Vec2;
use turborand::{TurboRand, rng::Rng};

use crate::line::Line;

pub fn normalize_angle(angle: f32) -> f32 {
  (360.0 + angle) % 360.0
}

pub trait Translate {
  fn translate(&mut self, offset: Vec2) -> &mut Self;
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
  if angle < 0.0 { angle + 360.0 } else { angle }
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

#[derive(Debug, Clone, Copy, Default)]
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

// TODO: Use [`Line`] instead
pub fn closest_point_on_line(
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
