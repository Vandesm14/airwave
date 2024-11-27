use core::str::FromStr;
use std::{
  fs,
  net::{IpAddr, Ipv4Addr, SocketAddr},
  path::PathBuf,
  sync::Arc,
  time::SystemTime,
};

use glam::Vec2;
use internment::Intern;
use tokio::sync::mpsc;
use turborand::{rng::Rng, SeededCore};

use engine::entities::{airport::Airport, airspace::Airspace};
use server::{
  airport::new_v_pattern, config::Config, http, job::JobReq, runner::Runner,
  Cli, CLI, MANUAL_TOWER_AIRSPACE_RADIUS,
};

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt::init();
  if let Err(e) = dotenv::dotenv() {
    tracing::warn!(".env file was not provided: {}", e);
  }

  let openai_api_key: Arc<str> = std::env::var("OPENAI_API_KEY")
    .expect("OPENAI_API_KEY must be set")
    .into();

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

  let (job_tx, job_rx) = mpsc::unbounded_channel::<JobReq>();

  let seed = seed.unwrap_or(
    config
      .world
      .and_then(|w| w.seed)
      .unwrap_or(SystemTime::now().elapsed().unwrap().as_secs()),
  );
  let rng = Rng::with_seed(seed);
  let mut world_rng = Rng::with_seed(0);
  let mut runner = Runner::new(
    job_rx,
    Some(PathBuf::from_str("assets/world.json").unwrap()),
    rng,
  );

  let mut player_airspace = Airspace {
    id: Intern::from_ref("KSFO"),
    pos: Vec2::ZERO,
    radius: MANUAL_TOWER_AIRSPACE_RADIUS,
    airports: vec![],
    frequencies: config.frequencies.unwrap_or_default(),
  };

  let mut airport_ksfo = Airport {
    id: Intern::from_ref("KSFO"),
    center: player_airspace.pos,
    ..Default::default()
  };

  new_v_pattern::setup(&mut airport_ksfo);

  airport_ksfo.calculate_waypoints();
  player_airspace.airports.push(airport_ksfo);

  runner.world.airspace = player_airspace;

  runner.generate_airspaces(&mut world_rng);
  runner.fill_gates();

  //

  tracing::info!("Starting game loop...");
  tokio::task::spawn_blocking(move || runner.begin_loop());

  let _ = tokio::spawn(http::run(address, job_tx)).await;

  // let listener = TcpListener::bind(address).await.unwrap();
  // tracing::info!("Listening on {address}");

  // loop {
  //   let openai_api_key = openai_api_key.clone();
  //   let command_tx = command_tx.clone();
  //   let update_rx = update_rx.clone();

  //   let (stream, _) = match listener.accept().await {
  //     Ok(stream) => stream,
  //     Err(e) => {
  //       tracing::error!("Unable to accept TCP stream: {e}");
  //       continue;
  //     }
  //   };

  //   let stream = match tokio_tungstenite::accept_async(stream).await {
  //     Ok(stream) => stream,
  //     Err(e) => {
  //       tracing::error!("Unable to accept WebSocket stream: {e}");
  //       continue;
  //     }
  //   };

  //   let (writer, reader) = stream.split();

  //   command_tx.send(IncomingUpdate::Connect).await.unwrap();

  //   tokio::spawn(broadcast_updates_to(writer, update_rx));
  //   tokio::spawn(receive_commands_from(
  //     openai_api_key,
  //     reader,
  //     update_tx.clone(),
  //     command_tx,
  //   ));
  // }
}
