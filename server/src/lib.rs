use core::net::SocketAddr;
use std::{path::PathBuf, sync::LazyLock};

use clap::Parser;

use engine::NAUTICALMILES_TO_FEET;
use glam::Vec2;
use itertools::Itertools;
use union_find::{QuickUnionUf, UnionBySize, UnionFind};

pub const MANUAL_TOWER_AIRSPACE_RADIUS: f32 = NAUTICALMILES_TO_FEET * 30.0;
pub const AUTO_TOWER_AIRSPACE_RADIUS: f32 = NAUTICALMILES_TO_FEET * 30.0;
pub const TOWER_AIRSPACE_PADDING_RADIUS: f32 = NAUTICALMILES_TO_FEET * 20.0;
pub const WORLD_RADIUS: f32 = NAUTICALMILES_TO_FEET * 500.0;

pub mod assets;
pub mod config;
pub mod http;
pub mod job;
pub mod parser;
pub mod prompter;
pub mod ring;
pub mod runner;
pub mod signal_gen;

pub static CLI: LazyLock<Cli> = LazyLock::new(Cli::parse);

#[derive(Parser)]
pub struct Cli {
  /// The socket address to bind the WebSocket server to.
  #[arg(short, long, default_value = None)]
  pub address: Option<SocketAddr>,

  /// Whether to and where to record incomming audio to.
  #[arg(long, default_value = None)]
  pub audio_path: Option<PathBuf>,

  /// The path to the config file.
  #[arg(short, long, default_value = None)]
  pub config_path: Option<PathBuf>,
}

pub fn merge_points(points: &[Vec2], min_distance: f32) -> Vec<Vec2> {
  let n = points.len();
  let mut uf = QuickUnionUf::<UnionBySize>::new(points.len());

  // Group points that are within min_distance of each other.
  for (i, j) in (0..n).tuple_combinations() {
    if points[i].distance_squared(points[j]) < min_distance.powf(2.0) {
      uf.union(i, j);
    }
  }

  // Group points by their root representative.
  let mut groups: std::collections::HashMap<usize, Vec<Vec2>> =
    std::collections::HashMap::new();
  for (i, point) in points.iter().enumerate() {
    groups.entry(uf.find(i)).or_default().push(*point);
  }

  // Compute the centroid for each group.
  groups
    .values()
    .map(|group| {
      group.iter().fold(Vec2::ZERO, |acc, p| acc + *p) / (group.len() as f32)
    })
    .collect()
}
