use core::{
  net::{IpAddr, Ipv4Addr, SocketAddr},
  str::FromStr as _,
};
use std::{path::PathBuf, sync::Arc};

use clap::Parser;
use engine::{
  engine::{Engine, IncomingUpdate, OutgoingReply},
  pathfinder::{Node, NodeBehavior, NodeKind},
  structs::{Aircraft, AircraftState, Airport, Airspace},
  NAUTICALMILES_TO_FEET,
};
use futures_util::StreamExt as _;
use glam::Vec2;
use server::airport;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt::init();
  dotenv::dotenv().unwrap();

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

  let mut airport = Airport::new("KSFO".into(), Vec2::new(0.0, 0.0));
  airport::v_pattern::setup(&mut airport);
  airport.cache_waypoints();

  tracing::debug!("Created {airport:?}");

  let airspace = Airspace {
    id: "KSFO".into(),
    pos: airport.center,
    size: NAUTICALMILES_TO_FEET * 30.0,
    // TODO: remove clone after debugging
    airports: vec![airport.clone()],
  };

  tracing::debug!("Created {airspace:?}");

  // DEBUGGING:
  {
    let runway = airport.runways.first().unwrap().clone();
    let mut aircraft = Aircraft::random_to_land(&airspace, 118.5);
    aircraft.state = AircraftState::Taxiing {
      current: Node {
        name: runway.id.clone(),
        kind: NodeKind::Runway,
        behavior: NodeBehavior::GoTo,
        value: runway.pos,
      },
      waypoints: Vec::new(),
    };
    // aircraft.pos = move_point(
    //   runway.start(),
    //   inverse_degrees(runway.heading),
    //   NAUTICALMILES_TO_FEET * 5.0,
    // );
    aircraft.pos = runway.pos;

    aircraft.speed = 0.0;
    aircraft.altitude = 0.0;
    aircraft.heading = runway.heading;

    aircraft.target.speed = aircraft.speed;
    aircraft.target.altitude = aircraft.altitude;
    aircraft.target.heading = aircraft.heading;

    tracing::debug!("Created {aircraft:?}");

    engine.world.aircraft.push(aircraft);
  }

  engine.world.airspaces.push(airspace);

  // engine.spawn_random_aircraft();

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
