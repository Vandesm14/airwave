use core::str::FromStr;
use std::{fs, path::PathBuf, time::Instant};

use glam::Vec2;
use internment::Intern;
use tokio::sync::mpsc;
use turborand::{SeededCore, rng::Rng};

use engine::{
  AIRSPACE_RADIUS,
  entities::{airport::Airport, airspace::Airspace},
};
use server::{
  CLI, Cli, LOGS_DIR,
  config::Config,
  http,
  job::JobReq,
  runner::{ArgReqKind, ResKind, Runner, TinyReqKind},
};

#[tokio::main]
async fn main() {
  let Cli {
    address,
    ref audio_path,
    ref config_path,
    ..
  } = *CLI;

  let (file_log_non_blocking, _file_log_guard) = tracing_appender::non_blocking(
    tracing_appender::rolling::minutely(&*LOGS_DIR, "server.log"),
  );

  tracing_subscriber::fmt::fmt()
    .with_env_filter(
      tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(
        |_| {
          #[cfg(debug_assertions)]
          return concat!(env!("CARGO_CRATE_NAME"), "=", "trace").into();
          #[cfg(not(debug_assertions))]
          return concat!(env!("CARGO_CRATE_NAME"), "=", "info").into();
        },
      ),
    )
    .with_writer(file_log_non_blocking)
    .with_ansi(false)
    .init();

  if let Err(e) = dotenv::dotenv() {
    tracing::warn!(".env file was not provided: {}", e);
  }

  // Ensure that the API key is set.
  let _ = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");

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

  let (get_tx, get_rx) =
    mpsc::unbounded_channel::<JobReq<TinyReqKind, ResKind>>();
  let (post_tx, post_rx) =
    mpsc::unbounded_channel::<JobReq<ArgReqKind, ResKind>>();

  let seed = config.world().seed();

  tracing::info!("Seed: {seed}");

  let rng = Rng::with_seed(seed);
  let mut world_rng = Rng::with_seed(0);
  let mut runner = Runner::new(
    get_rx,
    post_rx,
    Some(PathBuf::from_str("assets/world.json").unwrap()),
    rng,
  );

  runner.load_assets();

  let mut player_airspace = Airspace {
    id: Intern::from_ref("KSFO"),
    pos: Vec2::ZERO,
    radius: AIRSPACE_RADIUS,
    airports: vec![],
    auto: false,
  };

  let mut main_airport: Airport = match config.world().airport() {
    Some(id) => match runner.airport(id) {
      Some(airport) => {
        tracing::info!(r#"Using airport: "{}""#, airport.id);
        airport.clone()
      }
      None => {
        tracing::error!(
          r#"Failed to load airport "{id}": Could not find assets "{id}.json" or "{id}.lua" (assets are case-sensetive)."#
        );
        std::process::exit(1);
      }
    },
    None => match runner.default_airport() {
      Some(airport) => {
        tracing::info!(r#"Using default airport: "{}""#, airport.id);
        airport.clone()
      }
      None => {
        tracing::error!("Could not find default airport");
        std::process::exit(1);
      }
    },
  };
  if let Some(frequencies) = config.frequencies() {
    main_airport.frequencies = frequencies.clone();
  }

  player_airspace.airports.push(main_airport.clone());
  runner.world.airspaces.push(player_airspace);

  runner.generate_airspaces(&mut world_rng, &main_airport.frequencies);
  runner.generate_waypoints();
  runner.world.reset_statuses();
  runner.fill_gates();

  //

  tracing::info!("Quick start loop...");
  let start = Instant::now();
  let ticks_ran = runner.quick_start();
  let duration = start.elapsed();
  let simulated_seconds = ticks_ran as f32 / runner.rate as f32;
  let simulated_minutes = (simulated_seconds / 60.0).floor();
  tracing::info!(
    "Simulated {} ticks (relative time: {:.0}m{:.0}s) in {:.2} secs (approx. {:.2}x speed).",
    ticks_ran,
    simulated_minutes,
    simulated_seconds % 60.0,
    duration.as_secs_f32(),
    simulated_seconds / duration.as_secs_f32()
  );

  tracing::info!("Starting game loop...");
  tokio::task::spawn_blocking(move || runner.begin_loop());

  let address = address.unwrap_or(config.server().address());
  let _ = tokio::spawn(http::run(address, get_tx, post_tx)).await;
}
