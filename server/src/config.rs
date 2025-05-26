use core::net::Ipv6Addr;
use std::{
  net::{IpAddr, Ipv4Addr, SocketAddr},
  path::Path,
};

use engine::entities::{airport::Frequencies, world::AirportStatus};
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
pub struct AirportStatusConfig {
  #[serde(default)]
  divert_arrivals: bool,
  #[serde(default)]
  delay_departures: bool,
  #[serde(default)]
  automate_air: bool,
  #[serde(default)]
  automate_ground: bool,
}

impl From<AirportStatusConfig> for AirportStatus {
  fn from(value: AirportStatusConfig) -> Self {
    Self {
      divert_arrivals: value.divert_arrivals,
      delay_departures: value.delay_departures,
      automate_air: value.automate_air,
      automate_ground: value.automate_ground,
    }
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
  status: AirportStatusConfig,
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

  pub fn status(&self) -> AirportStatus {
    self.status.clone().into()
  }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ServerConfig {
  #[serde(default = "default_ipv4")]
  pub address_ipv4: SocketAddr,
  #[serde(default = "default_ipv6")]
  pub address_ipv6: SocketAddr,
}

impl Default for ServerConfig {
  fn default() -> Self {
    Self {
      address_ipv4: default_ipv4(),
      address_ipv6: default_ipv6(),
    }
  }
}

fn default_ipv4() -> SocketAddr {
  SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080)
}

fn default_ipv6() -> SocketAddr {
  SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 8080)
}
