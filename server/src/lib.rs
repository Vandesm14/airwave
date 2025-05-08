use core::{net::SocketAddr, str::FromStr};
use std::{path::PathBuf, sync::LazyLock};

use clap::Parser;

use directories::ProjectDirs;
use glam::Vec2;
use itertools::Itertools;
use union_find::{QuickUnionUf, UnionBySize, UnionFind};

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
pub static PROJECT_DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| ProjectDirs::from("com", "airwavegame", "Airwave").expect("unable to retrieve a valid user home directory path from the operating system"));

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

  /// Overrides the directory path to store log files.
  #[arg(long)]
  pub logs_path: Option<PathBuf>,
  /// The number of log files to keep before rolling-over.
  #[arg(long, default_value_t = 10)]
  pub logs_max_files: usize,
  /// The duration between log file rotation.
  #[arg(long, default_value_t = LogRotation::Minutely)]
  pub logs_rotation: LogRotation,
  /// The minimum log level for the tty.
  #[arg(long, default_value_t = LogLevel::Info)]
  pub logs_tty_min_level: LogLevel,
  /// The minimum log level for the log files.
  #[arg(long, default_value_t = LogLevel::Trace)]
  pub logs_file_min_level: LogLevel,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogRotation {
  Minutely,
  Hourly,
  Daily,
  Never,
}

impl From<LogRotation> for tracing_appender::rolling::Rotation {
  fn from(value: LogRotation) -> Self {
    match value {
      LogRotation::Minutely => Self::MINUTELY,
      LogRotation::Hourly => Self::HOURLY,
      LogRotation::Daily => Self::DAILY,
      LogRotation::Never => Self::NEVER,
    }
  }
}

impl FromStr for LogRotation {
  type Err = ParseRotationError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "minutely" => Ok(Self::Minutely),
      "hourly" => Ok(Self::Hourly),
      "daily" => Ok(Self::Daily),
      "never" => Ok(Self::Never),
      _ => Err(ParseRotationError),
    }
  }
}

impl core::fmt::Display for LogRotation {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::Minutely => write!(f, "minutely"),
      Self::Hourly => write!(f, "hourly"),
      Self::Daily => write!(f, "daily"),
      Self::Never => write!(f, "never"),
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseRotationError;

impl core::fmt::Display for ParseRotationError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "invalid log rotation")
  }
}

impl core::error::Error for ParseRotationError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogLevel {
  Trace,
  Debug,
  Info,
  Warn,
  Error,
}

impl From<LogLevel> for tracing::Level {
  fn from(value: LogLevel) -> Self {
    match value {
      LogLevel::Trace => Self::TRACE,
      LogLevel::Debug => Self::DEBUG,
      LogLevel::Info => Self::INFO,
      LogLevel::Warn => Self::WARN,
      LogLevel::Error => Self::ERROR,
    }
  }
}

impl FromStr for LogLevel {
  type Err = ParseRotationError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "trace" => Ok(Self::Trace),
      "debug" => Ok(Self::Debug),
      "info" => Ok(Self::Info),
      "warn" => Ok(Self::Warn),
      "error" => Ok(Self::Error),
      _ => Err(ParseRotationError),
    }
  }
}

impl core::fmt::Display for LogLevel {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::Trace => write!(f, "trace"),
      Self::Debug => write!(f, "debug"),
      Self::Info => write!(f, "info"),
      Self::Warn => write!(f, "warn"),
      Self::Error => write!(f, "error"),
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseLogLevelError;

impl core::fmt::Display for ParseLogLevelError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "invalid log level")
  }
}

impl core::error::Error for ParseLogLevelError {}
