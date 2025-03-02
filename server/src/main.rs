use core::str::FromStr;
use std::{
  fs,
  net::{IpAddr, Ipv4Addr, SocketAddr},
  path::PathBuf,
  time::{Instant, SystemTime},
};

use glam::Vec2;
use internment::Intern;
use tokio::sync::mpsc;
use turborand::{rng::Rng, SeededCore};

use engine::entities::{airport::Airport, airspace::Airspace};
use server::{
  airport::new_v_pattern,
  config::Config,
  http,
  job::JobReq,
  runner::{ArgReqKind, ResKind, Runner, TinyReqKind},
  Cli, CLI, MANUAL_TOWER_AIRSPACE_RADIUS,
};

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt::init();
  if let Err(e) = dotenv::dotenv() {
    tracing::warn!(".env file was not provided: {}", e);
  }
  let openai_api_key =
    std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");

  let Cli {
    address,
    seed,
    ref audio_path,
    ref config_path,
  } = *CLI;

  if let Some(audio_path) = audio_path {
    if !audio_path.exists() {
      match std::fs::create_dir_all(audio_path) {
        Ok(()) => {}
        Err(e) => {
          tracing::error!("Unable to create directory: {e}");
          return;
        }
      }
    }
  }

  let default_path = PathBuf::from_str("config.toml").unwrap();
  let path = config_path.clone().unwrap_or(default_path);

  let config: Config = if fs::exists(&path).ok() == Some(true) {
    tracing::info!("Reading config at {}.", path.to_string_lossy());
    Config::from_path(path).unwrap()
  } else {
    tracing::info!("Using default config.");
    Config::default()
  };

  let address = address
    .or_else(|| config.server.and_then(|s| s.address))
    .unwrap_or(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9001));

  let (get_tx, get_rx) =
    mpsc::unbounded_channel::<JobReq<TinyReqKind, ResKind>>();
  let (post_tx, post_rx) =
    mpsc::unbounded_channel::<JobReq<ArgReqKind, ResKind>>();

  let seed = seed.unwrap_or(
    config
      .world
      .and_then(|w| w.seed)
      .unwrap_or(SystemTime::now().elapsed().unwrap().as_secs()),
  );

  tracing::info!("Seed: {seed}");

  let rng = Rng::with_seed(seed);
  let mut world_rng = Rng::with_seed(0);
  let mut runner = Runner::new(
    get_rx,
    post_rx,
    Some(PathBuf::from_str("assets/world.json").unwrap()),
    rng,
  );

  let mut player_airspace = Airspace {
    id: Intern::from_ref("KSFO"),
    pos: Vec2::ZERO,
    radius: MANUAL_TOWER_AIRSPACE_RADIUS,
    airports: vec![],
    auto: false,
  };

  let mut airport_ksfo = Airport {
    id: Intern::from_ref("KSFO"),
    center: player_airspace.pos,
    frequencies: config.frequencies.unwrap_or_default(),
    ..Default::default()
  };

  new_v_pattern::setup(&mut airport_ksfo);

  airport_ksfo.calculate_waypoints();
  player_airspace.airports.push(airport_ksfo);

  runner.world.airspaces.push(player_airspace);

  runner.generate_airspaces(&mut world_rng);
  runner.fill_gates();

  //

  tracing::info!("Quick start loop...");
  let start = Instant::now();
  let ticks_ran = runner.quick_start();
  let duration = start.elapsed();
  tracing::info!(
    "Preloaded {ticks_ran} ticks in {} seconds.",
    duration.as_secs_f32()
  );

  tracing::info!("Starting game loop...");
  tokio::task::spawn_blocking(move || runner.begin_loop());

  let _ =
    tokio::spawn(http::run(address, get_tx, post_tx, openai_api_key.into()))
      .await;
}
