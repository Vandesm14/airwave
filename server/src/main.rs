use core::{
  net::{IpAddr, Ipv4Addr, SocketAddr},
  str::FromStr as _,
};
use std::{path::PathBuf, sync::Arc};

use clap::Parser;
use engine::{
  engine::{Engine, IncomingUpdate, OutgoingReply},
  objects::{
    aircraft::Aircraft,
    airport::Airport,
    airspace::{Airspace, Frequencies},
  },
  pathfinder::{Node, NodeBehavior, NodeKind},
  NAUTICALMILES_TO_FEET,
};
use futures_util::StreamExt as _;
use glam::Vec2;
use server::airport;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt::init();
  if let Err(e) = dotenv::dotenv() {
    tracing::warn!(".env file was not provided: {}", e);
  }

  let openai_api_key: Arc<str> = std::env::var("OPENAI_API_KEY")
    .expect("OPENAI_API_KEY must be set")
    .into();

  let Cli { address } = Cli::parse();

  let (command_tx, command_rx) = async_channel::unbounded::<IncomingUpdate>();
  let (mut update_tx, update_rx) =
    async_broadcast::broadcast::<OutgoingReply>(16);

  update_tx.set_overflow(true);

  let mut engine = Engine::new(
    command_rx,
    update_tx.clone(),
    Some(PathBuf::from_str("assets/world.json").unwrap()),
  );

  let player_one_frequencies = Frequencies {
    approach: 118.5,
    departure: 118.5,
    tower: 118.5,
    ground: 118.5,
    center: 118.5,
  };

  let player_two_frequencies = Frequencies {
    approach: 118.6,
    departure: 118.6,
    tower: 118.6,
    ground: 118.6,
    center: 118.6,
  };

  // Create a controlled KSFO airspace
  let mut airspace_ksfo = Airspace {
    id: "KSFO".into(),
    pos: Vec2::new(0.0, 0.0),
    size: NAUTICALMILES_TO_FEET * 30.0,
    airports: vec![],
    auto: false,
    frequencies: player_one_frequencies.clone(),
  };

  // Create a controlled EGLL airspace
  let mut airspace_egll = Airspace {
    id: "EGLL".into(),
    pos: Vec2::new(NAUTICALMILES_TO_FEET * 20.0, -NAUTICALMILES_TO_FEET * 70.0),
    size: NAUTICALMILES_TO_FEET * 30.0,
    airports: vec![],
    auto: false,
    frequencies: player_two_frequencies.clone(),
  };

  // Create an uncontrolled (auto) KLAX airspace
  let airspace_klax = Airspace {
    id: "KLAX".into(),
    pos: Vec2::new(
      -NAUTICALMILES_TO_FEET * 80.0,
      -NAUTICALMILES_TO_FEET * 40.0,
    ),
    size: NAUTICALMILES_TO_FEET * 20.0,
    airports: vec![],
    auto: true,
    frequencies: player_one_frequencies.clone(),
  };

  // Create an uncontrolled (auto) KPHL airspace
  let airspace_kphl = Airspace {
    id: "KPHL".into(),
    pos: Vec2::new(NAUTICALMILES_TO_FEET * 10.0, NAUTICALMILES_TO_FEET * 80.0),
    size: NAUTICALMILES_TO_FEET * 20.0,
    airports: vec![],
    auto: true,
    frequencies: player_one_frequencies.clone(),
  };

  // Create an uncontrolled (auto) KJFK airspace
  let airspace_kjfk = Airspace {
    id: "KJFK".into(),
    pos: Vec2::new(NAUTICALMILES_TO_FEET * 90.0, NAUTICALMILES_TO_FEET * -10.0),
    size: NAUTICALMILES_TO_FEET * 20.0,
    airports: vec![],
    auto: true,
    frequencies: player_one_frequencies.clone(),
  };

  let mut airport_ksfo =
    Airport::new(airspace_ksfo.id.clone(), airspace_ksfo.pos);
  airport::v_pattern::setup(
    &mut airport_ksfo,
    &mut engine.world.waypoints,
    &mut engine.world.waypoint_sets,
  );
  airport_ksfo.cache_waypoints();
  airspace_ksfo.airports.push(airport_ksfo);

  let mut airport_egll =
    Airport::new(airspace_egll.id.clone(), airspace_egll.pos);
  airport::parallel::setup(
    &mut airport_egll,
    &mut engine.world.waypoints,
    &mut engine.world.waypoint_sets,
  );
  airport_egll.cache_waypoints();
  airspace_egll.airports.push(airport_egll);

  engine.world.airspaces.push(airspace_ksfo);
  engine.world.airspaces.push(airspace_klax);
  engine.world.airspaces.push(airspace_kphl);
  engine.world.airspaces.push(airspace_kjfk);
  engine.world.airspaces.push(airspace_egll);

  engine.spawn_random_aircraft();

  // Fill all gates with random aircraft
  for airspace in engine.world.airspaces.iter() {
    if !airspace.auto {
      for airport in airspace.airports.iter() {
        let mut now = true;
        for gate in airport.terminals.iter().flat_map(|t| t.gates.iter()) {
          let mut aircraft = Aircraft::random_parked(Node {
            name: gate.id.clone(),
            kind: NodeKind::Gate,
            behavior: NodeBehavior::GoTo,
            value: gate.pos,
          });
          aircraft.airspace = Some(airspace.id.clone());
          aircraft.departure_from_arrival(&engine.world.airspaces);
          aircraft.frequency = airspace.frequencies.ground;

          if now {
            aircraft.created_now();
            now = false;
          }

          engine.world.aircraft.push(aircraft);
        }
      }
    }
  }

  tokio::task::spawn_blocking(move || engine.begin_loop());

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

#[derive(Parser)]
struct Cli {
  /// The socket address to bind the WebSocket server to.
  #[arg(short, long, default_value_t = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9001))]
  address: SocketAddr,
}
