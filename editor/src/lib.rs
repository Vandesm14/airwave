use glam::Vec2;
use nannou::geom;

pub mod draw;

pub fn glam_to_geom(v: Vec2) -> geom::Vec2 {
  geom::Vec2::new(v.x, v.y)
}

pub fn scale_point(point: Vec2, offset: Vec2, scale: f32) -> Vec2 {
  (point + offset) * scale
}

pub fn unscale_point(point: Vec2, offset: Vec2, scale: f32) -> Vec2 {
  (point / scale) - offset
}

// Helper function to get midpoint between two points
pub fn midpoint(a: Vec2, b: Vec2) -> Vec2 {
  (a + b) * 0.5
}
