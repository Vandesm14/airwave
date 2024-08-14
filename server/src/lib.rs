use std::f32::consts::PI;

use glam::Vec2;
use rand::Rng;

pub mod engine;
pub mod structs;

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
