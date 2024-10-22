use core::{
  fmt,
  net::{IpAddr, Ipv4Addr, SocketAddr},
  str::FromStr,
};
use std::{
  path::PathBuf,
  sync::Arc,
};

use clap::Parser;
use engine::{
  entities::{
    airport::Airport,
    airspace::{Airspace, Frequencies},
  },
  NAUTICALMILES_TO_FEET,
};
use futures_util::StreamExt as _;
use glam::Vec2;
use internment::Intern;
use server::{
  airport::{self, AirportSetupFn},
  CompatAdapter, IncomingUpdate, OutgoingReply,
};
use tokio::net::TcpListener;
use turborand::{rng::Rng, SeededCore, TurboRand};

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
    world_radius,
    airport,
  } = Cli::parse();
  let world_radius = NAUTICALMILES_TO_FEET * world_radius;

  let (command_tx, command_rx) = async_channel::unbounded::<IncomingUpdate>();
  let (mut update_tx, update_rx) =
    async_broadcast::broadcast::<OutgoingReply>(16);

  update_tx.set_overflow(true);

  let rng = Rng::with_seed(0);
  let mut world_rng = Rng::with_seed(0);
  let mut engine = CompatAdapter::new(
    command_rx,
    update_tx.clone(),
    command_tx.clone(),
    Some(PathBuf::from_str("assets/world.json").unwrap()),
    rng,
  );

  let player_one_frequencies = Frequencies {
    clearance: 118.7,
    approach: 118.6,
    departure: 118.6,
    tower: 118.5,
    ground: 118.5,
    center: 118.7,
  };

  const MANUAL_TOWER_AIRSPACE_RADIUS: f32 = NAUTICALMILES_TO_FEET * 30.0;

  // Create a controlled KSFO airspace
  let mut airspace_ksfo = Airspace {
    id: Intern::from_ref("KSFO"),
    pos: Vec2::ZERO,
    radius: MANUAL_TOWER_AIRSPACE_RADIUS,
    airports: vec![],
    auto: false,
    altitude: 0.0..=10000.0,
    frequencies: player_one_frequencies.clone(),
  };

  let mut airport_ksfo = Airport {
    id: Intern::from_ref("KSFO"),
    center: airspace_ksfo.pos,
    ..Default::default()
  };

  airport.setup(&mut world_rng)(&mut airport_ksfo);

  airport_ksfo.calculate_waypoints();
  airspace_ksfo.airports.push(airport_ksfo);
  engine.world.airspace = airspace_ksfo;

  // // Generate randomly positioned uncontrolled airspaces.
  // for airspace_name in airspace_names {
  //   // TODO: This is a brute-force approach. A better solution would be to use
  //   //       some form of jitter or other, potentially, less infinite-loop-prone
  //   //       solution.

  //   let mut i = 0;

  //   let airspace_position = 'outer: loop {
  //     if i >= 1000 {
  //       tracing::error!(
  //         "Unable to find a place for airspace '{airspace_name}'"
  //       );
  //       std::process::exit(1);
  //     }

  //     i += 1;

  //     let position = Vec2::new(
  //       (world_rng.f32() - 0.5) * world_radius,
  //       (world_rng.f32() - 0.5) * world_radius,
  //     );

  //     for airspace in engine.world.airspaces.iter() {
  //       if circle_circle_intersection(
  //         position,
  //         airspace.pos,
  //         AUTO_TOWER_AIRSPACE_RADIUS + TOWER_AIRSPACE_PADDING_RADIUS,
  //         airspace.radius + TOWER_AIRSPACE_PADDING_RADIUS,
  //       ) {
  //         continue 'outer;
  //       }
  //     }

  //     break position;
  //   };

  //   engine.world.airspaces.push(Airspace {
  //     id: Intern::from_ref(airspace_name),
  //     pos: airspace_position,
  //     radius: AUTO_TOWER_AIRSPACE_RADIUS,
  //     airports: vec![],
  //     auto: true,
  //     altitude: 0.0..=10000.0,
  //     frequencies: player_one_frequencies.clone(),
  //   });

  //   engine.world.waypoints.push(Node {
  //     name: Intern::from_ref(airspace_name),
  //     kind: NodeKind::Runway,
  //     behavior: NodeBehavior::GoTo,
  //     value: NodeVORData {
  //       to: airspace_position,
  //       then: vec![],
  //     },
  //   });
  // }

  // let mut aircrafts: Vec<Aircraft> = Vec::new();
  // for airspace in engine.world.airspaces.iter() {
  //   if !airspace.auto {
  //     for airport in airspace.airports.iter() {
  //       let mut now = true;
  //       for gate in airport.terminals.iter().flat_map(|t| t.gates.iter()) {
  //         if engine.rng.chance(0.3) {
  //           let mut aircraft =
  //             Aircraft::random_parked(gate.clone(), &mut engine.rng, airspace);
  //           aircraft.flight_plan.arriving = airspace.id;
  //           aircraft
  //             .make_random_departure(&engine.world.airspaces, &mut engine.rng);
  //           aircraft.created = SystemTime::now()
  //             .duration_since(SystemTime::UNIX_EPOCH)
  //             .unwrap()
  //             .add(Duration::from_secs(
  //               engine.rng.sample_iter(120..=1800).unwrap(),
  //             ));

  //           if now {
  //             aircraft.created_now();
  //             now = false;
  //           }

  //           aircrafts.push(aircraft);
  //         }
  //       }
  //     }
  //   }
  // }

  //

  tracing::info!("Starting game loop...");
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

  /// The airport variation that should be used.
  #[arg(long, default_value_t = AirportChoice::Random)]
  airport: AirportChoice,
  /// The radius of the entire world in nautical miles (NM).
  #[arg(long, default_value_t = 500.0)]
  world_radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AirportChoice {
  Random,
  NewVPattern,
  Parallel,
}

impl AirportChoice {
  fn setup(&self, rng: &mut Rng) -> AirportSetupFn {
    static AIRPORT_SETUPS: &[AirportSetupFn] =
      &[airport::new_v_pattern::setup, airport::parallel::setup];

    match self {
      Self::Random => *rng.sample(AIRPORT_SETUPS).unwrap(),
      Self::NewVPattern => airport::new_v_pattern::setup,
      Self::Parallel => airport::parallel::setup,
    }
  }

  const fn as_str(&self) -> &str {
    match self {
      Self::Random => "random",
      Self::NewVPattern => "new_v_pattern",
      Self::Parallel => "parallel",
    }
  }
}

impl FromStr for AirportChoice {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "random" => Ok(Self::Random),
      "new_v_pattern" => Ok(Self::NewVPattern),
      "parallel" => Ok(Self::Parallel),
      _ => Err(format!("'{s}' is an invalid airport type")),
    }
  }
}

impl fmt::Display for AirportChoice {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.as_str())
  }
}
