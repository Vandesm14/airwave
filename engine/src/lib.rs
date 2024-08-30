use std::f32::consts::PI;

use glam::Vec2;
use rand::Rng;

pub mod engine;
pub mod structs;

pub const TIME_SCALE: f32 = 1.0;

pub const NAUTICALMILES_TO_FEET: f32 = 6076.115;
// pub const FEET_PER_UNIT: f32 = 0.005;
pub const KNOT_TO_FEET_PER_SECOND: f32 = 1.68781 * TIME_SCALE;

pub const UP: f32 = 0.0;
pub const DOWN: f32 = 180.0;
pub const LEFT: f32 = 270.0;
pub const RIGHT: f32 = 90.0;

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

pub fn degrees_to_heading(heading: f32) -> f32 {
  add_degrees(heading, 90.0)
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

  add_degrees(dy.atan2(dx).to_degrees(), 360.0)
}

pub fn find_line_intersection(
  line1_start: Vec2,
  line1_end: Vec2,
  line2_start: Vec2,
  line2_end: Vec2,
) -> Option<Vec2> {
  // Calculate direction vectors
  let line1_dir = line1_end - line1_start;
  let line2_dir = line2_end - line2_start;

  // Calculate the determinant
  let det = line1_dir.x * line2_dir.y - line1_dir.y * line2_dir.x;

  // Check if lines are parallel (or coincident)
  if det.abs() < f32::EPSILON {
    return None;
  }

  // Calculate the differences between start points
  let dp = line2_start - line1_start;

  // Calculate the parameters t and s
  let t = (dp.x * line2_dir.y - dp.y * line2_dir.x) / det;
  let s = (dp.x * line1_dir.y - dp.y * line1_dir.x) / det;

  // Check if the intersection point lies on both line segments
  if !(0.0..=1.0).contains(&t) || !(0.0..=1.0).contains(&s) {
    return None;
  }

  // Calculate the intersection point
  let intersection = line1_start + line1_dir * t;

  Some(intersection)
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CirclePoint {
  pub position: Vec2,
  pub angle: f32,
}

pub fn get_random_point_on_circle(center: Vec2, radius: f32) -> CirclePoint {
  let mut rng = rand::thread_rng();

  // Generate a random angle in radians
  let random_angle = rng.r#gen::<f32>() * 2.0 * PI;

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
