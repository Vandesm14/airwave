use std::f32::consts::PI;

use glam::Vec2;
use rand::Rng;

pub mod engine;
pub mod structs;

pub const TIME_SCALE: f32 = 1.0;

pub const NAUTICALMILES_TO_FEET: f32 = 6076.115;
pub const FEET_PER_UNIT: f32 = 0.005;
pub const KNOT_TO_FEET_PER_SECOND: f32 = 1.68781 * TIME_SCALE;

pub fn move_point(point: Vec2, degrees: f32, length: f32) -> Vec2 {
  // Convert degrees to radians
  let radians = degrees * (PI / 180.0);

  // Calculate x and y components
  let x = length * radians.cos();
  let y = length * radians.sin();

  // Create and return the new Vec2
  point + Vec2::new(x, y)
}

pub fn heading_to_degrees(heading: f32) -> f32 {
  (heading + 270.0) % 360.0
}

pub fn degrees_to_heading(heading: f32) -> f32 {
  (heading + 90.0) % 360.0
}

pub fn delta_angle(current: f32, target: f32) -> f32 {
  ((target - current + 540.0) % 360.0) - 180.0
}

pub fn inverse_degrees(degrees: f32) -> f32 {
  (degrees + 180.0) % 360.0
}

pub fn angle_between_points(a: Vec2, b: Vec2) -> f32 {
  let dx = b.x - a.x;
  let dy = b.y - a.y;

  (dy.atan2(dx).to_degrees() + 360.0) % 360.0
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
