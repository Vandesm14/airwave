use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FlightKind {
  Inbound,
  Outbound,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Flight {
  pub id: usize,
  pub kind: FlightKind,
  pub spawn_at: Duration,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Flights {
  pub flights: Vec<Flight>,
  pub id_seq: usize,
}

impl Flights {
  pub fn new() -> Self {
    Flights {
      flights: Vec::new(),
      id_seq: 0,
    }
  }

  pub fn add(&mut self, kind: FlightKind, spawn_at: Duration) -> usize {
    let id = self.id_seq;
    let flight = Flight { id, kind, spawn_at };
    self.id_seq += 1;
    self.flights.push(flight);

    id
  }

  pub fn remove(&mut self, id: usize) {
    let index = self.flights.iter().position(|flight| flight.id == id);
    if let Some(index) = index {
      self.flights.swap_remove(index);
    }
  }

  pub fn get(&self, id: usize) -> Option<&Flight> {
    self.flights.iter().find(|flight| flight.id == id)
  }

  pub fn get_mut(&mut self, id: usize) -> Option<&mut Flight> {
    self.flights.iter_mut().find(|flight| flight.id == id)
  }

  pub fn iter(&self) -> impl Iterator<Item = &Flight> {
    self.flights.iter()
  }

  pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Flight> {
    self.flights.iter_mut()
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrontendOrder {
  pub quantity: usize,
  pub kind: FlightKind,
  pub spawn_at: Duration,
  pub stagger_by: Duration,
}
