use std::{net::SocketAddr, path::Path};

use engine::entities::{
  airport::Airport,
  airspace::{Airspace, Frequencies},
  world::{World, WorldOptions},
};
use glam::Vec2;
use internment::Intern;
use serde::{Deserialize, Serialize};

use crate::{airport::new_v_pattern, MANUAL_TOWER_AIRSPACE_RADIUS};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
  pub frequencies: Option<Frequencies>,
  pub world: Option<WorldConfig>,
  pub server: Option<ServerConfig>,
}

impl Config {
  pub fn from_path<T>(path: T) -> Result<Self, String>
  where
    T: AsRef<Path>,
  {
    let path = path.as_ref();
    let config = std::fs::read_to_string(path);
    match config {
      Ok(config) => match toml::from_str(&config) {
        Ok(config) => Ok(config),
        Err(err) => Err(format!("Failed to parse config file: {}", err)),
      },
      Err(err) => Err(format!("Failed to read config file: {}", err)),
    }
  }
}

impl From<Config> for World {
  fn from(value: Config) -> Self {
    let mut player_airspace = Airspace {
      id: Intern::from_ref("KSFO"),
      pos: Vec2::ZERO,
      radius: MANUAL_TOWER_AIRSPACE_RADIUS,
      airports: vec![],
      frequencies: value.frequencies.unwrap_or_default(),
    };

    let mut airport_ksfo = Airport {
      id: Intern::from_ref("KSFO"),
      center: player_airspace.pos,
      ..Default::default()
    };

    new_v_pattern::setup(&mut airport_ksfo);

    airport_ksfo.calculate_waypoints();
    player_airspace.airports.push(airport_ksfo);

    Self {
      airspace: player_airspace,
      connections: Vec::new(),
      options: WorldOptions {
        use_piper_tts: value
          .world
          .unwrap_or_default()
          .use_piper_tts
          .unwrap_or_default(),
      },
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct WorldConfig {
  pub seed: Option<u64>,
  pub use_piper_tts: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct ServerConfig {
  pub address: Option<SocketAddr>,
}
