use std::{collections::HashMap, ops::RangeInclusive};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub mod command;
pub mod compile;
pub mod engine;
pub mod entities;
pub mod geometry;
pub mod line;
pub mod pathfinder;
pub mod wayfinder;
pub mod wordify;

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

  let nato_alphabet: HashMap<char, &str> = HashMap::from_iter(NATO_ALPHABET);
  let nato_numbers: HashMap<char, &str> = HashMap::from_iter(NATO_NUMBERS);

  for c in string.chars() {
    if c.is_alphabetic() {
      let c = c.to_ascii_uppercase();
      if let Some(nato) = nato_alphabet.get(&c) {
        result.push_str(nato);
        result.push(' ');
      }
    } else if c.is_numeric() {
      if let Some(nato) = nato_numbers.get(&c) {
        result.push_str(nato);
        result.push(' ');
      }
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
    geometry::{angle_between_points, delta_angle, find_line_intersection},
    line::Line,
  };

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
