use std::{net::SocketAddr, path::Path};

use engine::entities::airport::Frequencies;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct WorldConfig {
  pub seed: Option<u64>,
  pub airport: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct ServerConfig {
  pub address: Option<SocketAddr>,
}
