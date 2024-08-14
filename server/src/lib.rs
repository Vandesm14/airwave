use std::f32::consts::PI;

use glam::Vec2;

pub mod engine;
pub mod structs;

pub fn move_point(point: Vec2, degrees: f32, length: f32) -> Vec2 {
  // Convert degrees to radians
  let radians = degrees * PI / 180.0;

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
