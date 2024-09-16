use core::{
  fmt,
  net::{IpAddr, Ipv4Addr, SocketAddr},
  str::FromStr,
};
use std::{path::PathBuf, sync::Arc};

use clap::Parser;
use engine::{
  circle_circle_intersection,
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
use rand::{seq::SliceRandom, thread_rng, Rng};
use server::airport::{self, AirportSetupFn};
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

  let mut engine = Engine::new(
    command_rx,
    update_tx.clone(),
    Some(PathBuf::from_str("assets/world.json").unwrap()),
  );

  let player_one_frequencies = Frequencies {
    clearance: 118.6,
    approach: 118.5,
    departure: 118.5,
    tower: 118.5,
    ground: 118.5,
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

  engine.spawn_random_aircraft();

  const MANUAL_TOWER_AIRSPACE_RADIUS: f32 = NAUTICALMILES_TO_FEET * 30.0;
  const AUTO_TOWER_AIRSPACE_RADIUS: f32 = NAUTICALMILES_TO_FEET * 20.0;
  const TOWER_AIRSPACE_PADDING_RADIUS: f32 = NAUTICALMILES_TO_FEET * 20.0;
  const CENTER_WAYPOINT_RADIUS: f32 = NAUTICALMILES_TO_FEET * 15.0;

  let airspace_names = [
    "KLAX", "KPHL", "KJFK", "EGNX", "EGGW", "EGSH", "EGMC", "EGSS", "EGLL",
    "EGLC", "EGNV", "EGNT", "EGGP", "EGCC", "EGKK", "EGHI",
  ];

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

  airport.setup(&mut rng)(
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
        tracing::error!(
          "Unable to find a place for airspace '{airspace_name}'"
        );
        std::process::exit(1);
      }

      i += 1;

      let position = Vec2::new(
        (rng.gen::<f32>() - 0.5) * world_radius,
        (rng.gen::<f32>() - 0.5) * world_radius,
      );

      for airspace in engine.world.airspaces.iter() {
        if circle_circle_intersection(
          position,
          airspace.pos,
          AUTO_TOWER_AIRSPACE_RADIUS + TOWER_AIRSPACE_PADDING_RADIUS,
          airspace.size + TOWER_AIRSPACE_PADDING_RADIUS,
        ) {
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

  // Generating waypoints between sufficiently close airspaces.
  let mut i = 0;
  let mut iota = || {
    let t = i;
    i += 1;
    t
  };

  fn n_to_an(i: usize) -> String {
    let n = i % 9;
    let n = n + 1;

    let wraps = u8::try_from(i / 9).unwrap();

    if wraps > b'Z' {
      tracing::error!("Too many waypoints to generate a unique one");
      std::process::exit(1);
    }

    let l = (b'A' + wraps) as char;

    format!("{l}{n}")
  }

  for airspace in engine.world.airspaces.iter() {
    'second: for other_airspace in engine.world.airspaces.iter() {
      let waypoint = airspace.pos.lerp(other_airspace.pos, 0.5);

      for intersection_test_airspace in engine.world.airspaces.iter() {
        if circle_circle_intersection(
          intersection_test_airspace.pos,
          waypoint,
          intersection_test_airspace.size,
          CENTER_WAYPOINT_RADIUS,
        ) {
          tracing::trace!(
            "Skipping waypoint at {} between {} and {}",
            waypoint,
            airspace.id,
            other_airspace.id
          );
          continue 'second;
        }
      }

      for intersection_test_waypoint in engine.world.waypoints.iter() {
        if circle_circle_intersection(
          intersection_test_waypoint.value,
          waypoint,
          CENTER_WAYPOINT_RADIUS,
          CENTER_WAYPOINT_RADIUS,
        ) {
          tracing::trace!(
            "Skipping waypoint at {} between {} ({}) ({})",
            waypoint,
            intersection_test_waypoint.value,
            intersection_test_waypoint.name,
            waypoint == intersection_test_waypoint.value,
          );
          continue 'second;
        }
      }

      let waypoint_id = iota();

      engine.world.waypoints.push(Node {
        name: n_to_an(waypoint_id),
        kind: NodeKind::Runway,
        behavior: NodeBehavior::GoTo,
        value: waypoint,
      });
    }
  }

  //

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
  VPattern,
  Parallel,
}

impl AirportChoice {
  fn setup<R>(&self, rng: &mut R) -> AirportSetupFn
  where
    R: ?Sized + Rng,
  {
    static AIRPORT_SETUPS: &[AirportSetupFn] = &[
      airport::new_v_pattern::setup,
      airport::v_pattern::setup,
      airport::parallel::setup,
    ];

    match self {
      Self::Random => AIRPORT_SETUPS.choose(rng).copied().unwrap(),
      Self::NewVPattern => airport::new_v_pattern::setup,
      Self::VPattern => airport::v_pattern::setup,
      Self::Parallel => airport::parallel::setup,
    }
  }

  const fn as_str(&self) -> &str {
    match self {
      Self::Random => "random",
      Self::NewVPattern => "new_v_pattern",
      Self::VPattern => "v_pattern",
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
      "v_pattern" => Ok(Self::VPattern),
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
