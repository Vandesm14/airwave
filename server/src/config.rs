use std::{
  net::{IpAddr, Ipv4Addr, SocketAddr},
  path::Path,
};

use engine::entities::{
  airport::Frequencies,
  world::{ArrivalStatus, DepartureStatus},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Config {
  #[serde(default)]
  frequencies: Option<Frequencies>,
  #[serde(default)]
  world: WorldConfig,
  #[serde(default)]
  server: ServerConfig,
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

  pub fn frequencies(&self) -> Option<&Frequencies> {
    self.frequencies.as_ref()
  }

  pub fn world(&self) -> &WorldConfig {
    &self.world
  }

  pub fn server(&self) -> &ServerConfig {
    &self.server
  }
}

#[derive(Clone, Serialize, Deserialize)]
struct WorldSeed(u64);

impl Default for WorldSeed {
  fn default() -> Self {
    let now = std::time::SystemTime::now();
    let since_epoch = now.duration_since(std::time::UNIX_EPOCH).unwrap();
    Self(since_epoch.as_secs())
  }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct WorldConfig {
  #[serde(default)]
  seed: WorldSeed,
  #[serde(default)]
  airport: Option<String>,
  #[serde(default)]
  paused: bool,
  #[serde(default)]
  arrivals: ArrivalStatus,
  #[serde(default)]
  departures: DepartureStatus,
}

impl WorldConfig {
  pub fn seed(&self) -> u64 {
    self.seed.0
  }

  pub fn airport(&self) -> Option<&str> {
    self.airport.as_deref()
  }

  pub fn paused(&self) -> bool {
    self.paused
  }

  pub fn arrivals(&self) -> &ArrivalStatus {
    &self.arrivals
  }

  pub fn departures(&self) -> &DepartureStatus {
    &self.departures
  }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct ServerAddress(SocketAddr);

impl Default for ServerAddress {
  fn default() -> Self {
    Self(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9001))
  }
}

#[derive(Clone, Copy, Default, Serialize, Deserialize)]
pub struct ServerConfig {
  #[serde(default)]
  address: ServerAddress,
}

impl ServerConfig {
  pub fn address(&self) -> SocketAddr {
    self.address.0
  }
}
