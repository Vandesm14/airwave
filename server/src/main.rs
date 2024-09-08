use core::{
  net::{IpAddr, Ipv4Addr, SocketAddr},
  str::FromStr as _,
};
use std::{path::PathBuf, sync::Arc};

use clap::Parser;
use engine::{
  engine::{Engine, IncomingUpdate, OutgoingReply},
  structs::{Airport, Airspace},
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

  let mut airspace_ksfo = Airspace {
    id: "KSFO".into(),
    pos: Vec2::new(0.0, 0.0),
    size: NAUTICALMILES_TO_FEET * 30.0,
    airports: vec![],
  };
  tracing::debug!("Created {airspace_ksfo:?}");

  let mut airport_ksfo = Airport::new("KSFO".into(), airspace_ksfo.pos);
  airport::v_pattern::setup(&mut airport_ksfo);
  airport_ksfo.cache_waypoints();
  tracing::debug!("Created {airport_ksfo:?}");

  airspace_ksfo.airports.push(airport_ksfo);

  let mut airspace_klax = Airspace {
    id: "KLAX".into(),
    pos: Vec2::new(
      -NAUTICALMILES_TO_FEET * 80.0,
      -NAUTICALMILES_TO_FEET * 40.0,
    ),
    size: NAUTICALMILES_TO_FEET * 30.0,
    airports: vec![],
  };
  tracing::debug!("Created {airspace_klax:?}");

  let mut airport_klax = Airport::new("KLAX".into(), airspace_klax.pos);
  airport::parallel::setup(&mut airport_klax);
  airport_klax.cache_waypoints();
  tracing::debug!("Created {airport_klax:?}");

  airspace_klax.airports.push(airport_klax);

  engine.world.airspaces.push(airspace_ksfo);
  engine.world.airspaces.push(airspace_klax);
  engine.spawn_random_aircraft();

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
