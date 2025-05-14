use core::ops::RangeInclusive;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub mod assets;
pub mod command;
pub mod compile;
pub mod engine;
pub mod entities;
pub mod geometry;
pub mod line;
pub mod pathfinder;
pub mod wayfinder;
pub mod wordify;

pub const DEFAULT_TICK_RATE_TPS: usize = 15;

pub const NAUTICALMILES_TO_FEET: f32 = 6076.115;
pub const KNOT_TO_FEET_PER_SECOND: f32 = 1.68781;

pub const AIRSPACE_RADIUS: f32 = NAUTICALMILES_TO_FEET * 30.0;
pub const AIRSPACE_PADDING_RADIUS: f32 = NAUTICALMILES_TO_FEET * 20.0;
pub const WORLD_RADIUS: f32 = NAUTICALMILES_TO_FEET * 500.0;

pub const UP: f32 = 0.0;
pub const DOWN: f32 = 180.0;
pub const LEFT: f32 = 270.0;
pub const RIGHT: f32 = 90.0;
pub const CLOCKWISE: f32 = 90.0;
pub const COUNTERCLOCKWISE: f32 = 270.0;

pub const DEPARTURE_WAIT_RANGE: RangeInclusive<u64> = 180..=900;

pub const MIN_CRUISE_ALTITUDE: f32 = 28000.0;
pub const EAST_CRUISE_ALTITUDE: f32 = 37000.0;
pub const WEST_CRUISE_ALTITUDE: f32 = 38000.0;
pub const TRANSITION_ALTITUDE: f32 = 18000.0;
pub const ARRIVAL_ALTITUDE: f32 = 10000.0;
pub const APPROACH_ALTITUDE: f32 = 3000.0;

pub const MAX_TAXI_SPEED: f32 = 20.0;

pub trait ToText {
  fn to_text(&self, w: &mut dyn std::fmt::Write) -> std::fmt::Result;
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename = "Vec2")]
pub struct ExportedVec2(f32, f32);

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename = "Duration")]
pub struct ExportedDuration {
  secs: f32,
  nanos: f32,
}

pub fn sign3(x: f32) -> f32 {
  if x > 0.0 {
    1.0
  } else if x < 0.0 {
    -1.0
  } else {
    0.0
  }
}

pub fn duration_now() -> Duration {
  SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
}

pub fn heading_to_direction(heading: f32) -> &'static str {
  // Normalize the heading to be between 0 and 360
  let normalized_heading = heading.rem_euclid(360.0);

  match normalized_heading {
    0.0..=22.5 => "North",
    22.6..=67.5 => "Northeast",
    67.6..=112.5 => "East",
    112.6..=157.5 => "Southeast",
    157.6..=202.5 => "South",
    202.6..=247.5 => "Southwest",
    247.6..=292.5 => "West",
    292.6..=337.5 => "Northwest",
    337.6..=360.0 => "North",
    _ => unreachable!(),
  }
}

const NATO_ALPHABET: [(char, &str); 26] = [
  ('A', "Alfa"),
  ('B', "Bravo"),
  ('C', "Charlie"),
  ('D', "Delta"),
  ('E', "Echo"),
  ('F', "Foxtrot"),
  ('G', "Golf"),
  ('H', "Hotel"),
  ('I', "India"),
  ('J', "Juliett"),
  ('K', "Kilo"),
  ('L', "Lima"),
  ('M', "Mike"),
  ('N', "November"),
  ('O', "Oscar"),
  ('P', "Papa"),
  ('Q', "Quebec"),
  ('R', "Romeo"),
  ('S', "Sierra"),
  ('T', "Tango"),
  ('U', "Uniform"),
  ('V', "Victor"),
  ('W', "Whiskey"),
  ('X', "X-ray"),
  ('Y', "Yankee"),
  ('Z', "Zulu"),
];

const NATO_NUMBERS: [(char, &str); 10] = [
  ('0', "Zero"),
  ('1', "One"),
  ('2', "Two"),
  ('3', "Three"),
  ('4', "Four"),
  ('5', "Five"),
  ('6', "Six"),
  ('7', "Seven"),
  ('8', "Eight"),
  ('9', "Nine"),
];

pub fn nato_phonetic(string: impl AsRef<str>) -> String {
  let string = string.as_ref();
  let mut result = String::new();

  for c in string.chars() {
    if let Some(nato) = NATO_ALPHABET
      .into_iter()
      .find_map(|(ch, s)| (c == ch).then_some(s))
    {
      result.push_str(nato);
      result.push(' ');
    } else if let Some(nato) = NATO_NUMBERS
      .into_iter()
      .find_map(|(ch, s)| (c == ch).then_some(s))
    {
      result.push_str(nato);
      result.push(' ');
    } else {
      result.push(c);
    }
  }

  result.trim().to_string()
}

/// Abbreviates an altitude to feet or flight level (depending on the altitude).
pub fn abbreviate_altitude(altitude: f32) -> String {
  if altitude < 13000.0 {
    format!("{} thousand feet", (altitude / 1000.0).round())
  } else {
    format!("Flight Level {}", (altitude / 100.0).round())
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    NATO_ALPHABET, NATO_NUMBERS,
    geometry::{angle_between_points, delta_angle, find_line_intersection},
    line::Line,
    nato_phonetic,
  };

  #[test]
  fn test_nato_phonetic() {
    for (c, s) in NATO_NUMBERS.into_iter().chain(NATO_ALPHABET) {
      assert_eq!(s, nato_phonetic(c.to_string()));
    }
  }

  mod delta_angle {
    use super::*;

    #[test]
    fn test_delta_angle_zero() {
      assert_eq!(delta_angle(0.0, 0.0), 0.0)
    }

    #[test]
    fn test_delta_angle_90() {
      assert_eq!(delta_angle(0.0, 90.0), 90.0)
    }

    #[test]
    fn test_delta_angle_negative() {
      assert_eq!(delta_angle(0.0, -90.0), -90.0)
    }

    #[test]
    fn test_delta_angle_not_zero() {
      assert_eq!(delta_angle(90.0, 180.0), 90.0)
    }

    #[test]
    fn test_delta_angle_not_zero_negative() {
      assert_eq!(delta_angle(180.0, 90.0), -90.0)
    }
  }

  mod angle_between_points {
    use glam::Vec2;

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
    use glam::Vec2;

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
