use core::str::FromStr;
use std::{path::PathBuf, sync::Arc};

use futures_util::StreamExt as _;
use glam::Vec2;
use internment::Intern;
use tokio::net::TcpListener;
use turborand::{rng::Rng, SeededCore, TurboRand};

use engine::{
  circle_circle_intersection,
  entities::{
    aircraft::Aircraft,
    airport::Airport,
    airspace::{Airspace, Frequencies},
    world::{Connection, ConnectionState},
  },
};
use server::{
  airport::new_v_pattern,
  runner::{IncomingUpdate, OutgoingReply, Runner},
  Cli, AUTO_TOWER_AIRSPACE_RADIUS, CLI, MANUAL_TOWER_AIRSPACE_RADIUS,
  TOWER_AIRSPACE_PADDING_RADIUS, WORLD_RADIUS,
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

  let (command_tx, command_rx) = async_channel::unbounded::<IncomingUpdate>();
  let (mut update_tx, update_rx) =
    async_broadcast::broadcast::<OutgoingReply>(16);

  update_tx.set_overflow(true);

  let rng = Rng::with_seed(seed);
  let world_rng = Rng::with_seed(0);
  let mut runner = Runner::new(
    command_rx,
    update_tx.clone(),
    command_tx.clone(),
    Some(PathBuf::from_str("assets/world.json").unwrap()),
    rng,
  );

  let player_one_frequencies = Frequencies {
    approach: 118.6,
    departure: 118.6,
    tower: 118.5,
    ground: 118.5,
    center: 118.7,
  };

  let airspace_names = [
    "KLAX", "KPHL", "KJFK", "KMGM", "KCLT", "KDFW", "KATL", "KMCO", "EGLL",
    "EGLC", "EGNV", "EGNT", "EGGP", "EGCC", "EGKK", "EGHI",
  ];

  let mut player_airspace = Airspace {
    id: Intern::from_ref("KSFO"),
    pos: Vec2::ZERO,
    radius: MANUAL_TOWER_AIRSPACE_RADIUS,
    airports: vec![],
    frequencies: player_one_frequencies.clone(),
  };

  let mut airport_ksfo = Airport {
    id: Intern::from_ref("KSFO"),
    center: player_airspace.pos,
    ..Default::default()
  };

  new_v_pattern::setup(&mut airport_ksfo);

  airport_ksfo.calculate_waypoints();
  player_airspace.airports.push(airport_ksfo);

  // Generate randomly positioned uncontrolled airspaces.
  for airspace_name in airspace_names {
    // TODO: This is a brute-force approach. A better solution would be to use
    //       some form of jitter or other, potentially, less infinite-loop-prone
    //       solution.

    let mut i = 0;

    let airspace_position = 'outer: loop {
      if i >= 1000 {
        tracing::error!(
          "Unable to find a place for airspace '{airspace_name}'"
        );
        std::process::exit(1);
      }

      i += 1;

      let position = Vec2::new(
        (world_rng.f32() - 0.5) * WORLD_RADIUS,
        (world_rng.f32() - 0.5) * WORLD_RADIUS,
      );

      for airport in runner.world.connections.iter() {
        if circle_circle_intersection(
          position,
          airport.pos,
          AUTO_TOWER_AIRSPACE_RADIUS + TOWER_AIRSPACE_PADDING_RADIUS,
          AUTO_TOWER_AIRSPACE_RADIUS + TOWER_AIRSPACE_PADDING_RADIUS,
        ) {
          continue 'outer;
        }
      }

      break position;
    };

    let connection = Connection {
      id: Intern::from_ref(airspace_name),
      state: ConnectionState::Active,
      pos: airspace_position,
      transition: player_airspace
        .pos
        .move_towards(airspace_position, MANUAL_TOWER_AIRSPACE_RADIUS),
    };

    runner.world.connections.push(connection);
  }

  let mut aircrafts: Vec<Aircraft> = Vec::new();
  for airport in player_airspace.airports.iter() {
    for gate in airport.terminals.iter().flat_map(|t| t.gates.iter()) {
      if runner.rng.f32() < 0.9 {
        let mut aircraft = Aircraft::random_parked(
          gate.clone(),
          &mut runner.rng,
          &player_airspace,
        );
        aircraft.flight_plan.departing = player_airspace.id;
        aircraft.flight_plan.arriving = runner
          .rng
          .sample(&runner.world.connections)
          .map(|c| c.id)
          .unwrap_or_default();

        aircrafts.push(aircraft);
      }
    }
  }

  for aircraft in aircrafts.drain(..) {
    runner.add_aircraft(aircraft);
  }

  runner.world.airspace = player_airspace;

  //

  tracing::info!("Preparing spawn area...");
  runner.prepare();

  tracing::info!("Starting game loop...");
  tokio::task::spawn_blocking(move || runner.begin_loop());

  let listener = TcpListener::bind(address).await.unwrap();
  tracing::info!("Listening on {address}");

  loop {
    let openai_api_key = openai_api_key.clone();
    let command_tx = command_tx.clone();
    let update_rx = update_rx.clone();

    let (stream, _) = match listener.accept().await {
      Ok(stream) => stream,
      Err(e) => {
        tracing::error!("Unable to accept TCP stream: {e}");
        continue;
      }
    };

    let stream = match tokio_tungstenite::accept_async(stream).await {
      Ok(stream) => stream,
      Err(e) => {
        tracing::error!("Unable to accept WebSocket stream: {e}");
        continue;
      }
    };

    let (writer, reader) = stream.split();

    command_tx.send(IncomingUpdate::Connect).await.unwrap();

    tokio::spawn(server::broadcast_updates_to(writer, update_rx));
    tokio::spawn(server::receive_commands_from(
      openai_api_key,
      reader,
      update_tx.clone(),
      command_tx,
    ));
  }
}
