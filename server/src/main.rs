use core::{
  net::{IpAddr, Ipv4Addr, SocketAddr},
  str::FromStr as _,
};
use std::{path::PathBuf, sync::Arc};

use clap::Parser;
use engine::{
  circle_circle_intersection, engine::{Engine, IncomingUpdate, OutgoingReply}, objects::{
    aircraft::Aircraft,
    airport::Airport,
    airspace::{Airspace, Frequencies},
  }, pathfinder::{Node, NodeBehavior, NodeKind}, NAUTICALMILES_TO_FEET
};
use futures_util::StreamExt as _;
use glam::Vec2;
use rand::{seq::SliceRandom, thread_rng, Rng};
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

  let Cli { address, world_radius } = Cli::parse();
  let world_radius = NAUTICALMILES_TO_FEET * world_radius;

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
    tower: 118.6,
    ground: 118.6,
    center: 118.7,
  };

  // let player_two_frequencies = Frequencies {
  //   approach: 118.6,
  //   departure: 118.6,
  //   tower: 118.6,
  //   ground: 118.6,
  //   center: 118.6,
  // };

  // let mut airport_egll =
  //   Airport::new(airspace_egll.id.clone(), airspace_egll.pos);
  // airport::parallel::setup(
  //   &mut airport_egll,
  //   &mut engine.world.waypoints,
  //   &mut engine.world.waypoint_sets,
  // );
  // airport_egll.calculate_waypoints();
  // airspace_egll.airports.push(airport_egll);

  // engine.spawn_random_aircraft();

  const MANUAL_TOWER_AIRSPACE_RADIUS: f32 = NAUTICALMILES_TO_FEET * 30.0;
  const AUTO_TOWER_AIRSPACE_RADIUS: f32 = NAUTICALMILES_TO_FEET * 20.0;
  const TOWER_AIRSPACE_PADDING_RADIUS: f32 = NAUTICALMILES_TO_FEET * 20.0;

  let airport_setups = [
    airport::new_v_pattern::setup,
    airport::v_pattern::setup,
    airport::parallel::setup,
  ];

  let airspace_names = ["KLAX", "KPHL", "KJFK", "EGNX", "EGGW", "EGSH", "EGMC", "EGSS", "EGLL", "EGLC", "EGNV", "EGNT", "EGGP", "EGCC", "EGKK", "EGHI"];

  let mut rng = thread_rng();

  // Create a controlled KSFO airspace
  let mut airspace_ksfo = Airspace {
    id: "KSFO".into(),
    pos: Vec2::ZERO,
    size: MANUAL_TOWER_AIRSPACE_RADIUS,
    airports: vec![],
    auto: false,
    frequencies: player_one_frequencies.clone(),
  };

  let mut airport_ksfo = Airport {
    id: "KSFO".into(),
    center: airspace_ksfo.pos,
    ..Default::default()
  };

  (airport_setups.choose(&mut rng).unwrap())(
    &mut airport_ksfo,
    &mut engine.world.waypoints,
    &mut engine.world.waypoint_sets,
  );

  airport_ksfo.calculate_waypoints();
  airspace_ksfo.airports.push(airport_ksfo);
  engine.world.airspaces.push(airspace_ksfo);

  // Generate randomly positioned uncontrolled airspaces.
  for airspace_name in airspace_names {
    // TODO: This is a brute-force approach. A better solution would be to use
    //       some form of jitter or other, potentially, less infinite-loop-prone
    //       solution.

    let mut i = 0;

    let airspace_position = 'outer: loop {
      if i >= 1000 {
        tracing::error!("Unable to find a place for airspace '{airspace_name}'");
        std::process::exit(1);
      }

      i += 1;

      let position = Vec2::new((rng.gen::<f32>() - 0.5) * world_radius, (rng.gen::<f32>() - 0.5) * world_radius);

      for airspace in engine.world.airspaces.iter() {
        if circle_circle_intersection(position, airspace.pos, AUTO_TOWER_AIRSPACE_RADIUS + TOWER_AIRSPACE_PADDING_RADIUS, airspace.size + TOWER_AIRSPACE_PADDING_RADIUS) {
          continue 'outer;
        }
      }

      break position;
    };

    engine.world.airspaces.push(Airspace {
      id: airspace_name.into(),
      pos: airspace_position,
      size: AUTO_TOWER_AIRSPACE_RADIUS,
      airports: vec![],
      auto: true,
      frequencies: player_one_frequencies.clone(),
    });

    engine.world.waypoints.push(Node {
      name: airspace_name.into(),
      kind: NodeKind::Runway,
      behavior: NodeBehavior::GoTo,
      value: airspace_position,
    });
  }

  // Fill all gates with random aircraft.
  for airspace in engine.world.airspaces.iter() {
    if !airspace.auto {
      for airport in airspace.airports.iter() {
        let mut now = true;
        for gate in airport.terminals.iter().flat_map(|t| t.gates.iter()) {
          if rng.gen_bool(0.4) {
            let mut aircraft = Aircraft::random_parked(gate.clone());
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

  /// The radius of the entire world in nautical miles (NM).
  #[arg(long, default_value_t = 500.0)]
  world_radius: f32,
}
