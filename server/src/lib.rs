use core::net::SocketAddr;
use std::{path::PathBuf, sync::LazyLock};

use clap::Parser;

use engine::NAUTICALMILES_TO_FEET;

pub const MANUAL_TOWER_AIRSPACE_RADIUS: f32 = NAUTICALMILES_TO_FEET * 30.0;
pub const AUTO_TOWER_AIRSPACE_RADIUS: f32 = NAUTICALMILES_TO_FEET * 30.0;
pub const TOWER_AIRSPACE_PADDING_RADIUS: f32 = NAUTICALMILES_TO_FEET * 20.0;
pub const WORLD_RADIUS: f32 = NAUTICALMILES_TO_FEET * 500.0;

pub mod airport;
pub mod config;
pub mod http;
pub mod job;
pub mod message;
pub mod prompter;
pub mod ring;
pub mod runner;

pub static CLI: LazyLock<Cli> = LazyLock::new(Cli::parse);

#[derive(Parser)]
pub struct Cli {
  /// The socket address to bind the WebSocket server to.
  #[arg(short, long, default_value = None)]
  pub address: Option<SocketAddr>,

  /// The seed to use for the random number generator.
  #[arg(short, long)]
  pub seed: Option<u64>,

  /// Whether to and where to record incomming audio to.
  #[arg(long, default_value = None)]
  pub audio_path: Option<PathBuf>,

  /// The path to the config file.
  #[arg(short, long, default_value = None)]
  pub config_path: Option<PathBuf>,
}
