use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// The kind of flight scheduled: Inbound or Outbound.
pub enum FlightKind {
  Inbound,
  Outbound,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// A scheduled flight.
pub struct Flight {
  /// The unique identifier of the flight.
  pub id: usize,
  /// The kind of flight.
  pub kind: FlightKind,
  /// The time at which the flight is scheduled to spawn.
  pub spawn_at: Duration,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
/// A collection of scheduled flights.
pub struct Flights {
  flights: Vec<Flight>,
  id_seq: usize,
}

impl Flights {
  pub fn new() -> Self {
    Flights {
      flights: Vec::new(),
      id_seq: 0,
    }
  }

  pub fn flights(&self) -> &[Flight] {
    &self.flights
  }

  pub fn add(&mut self, kind: FlightKind, spawn_at: Duration) -> usize {
    let id = self.id_seq;
    let flight = Flight { id, kind, spawn_at };
    self.id_seq += 1;
    self.flights.push(flight);

    id
  }

  pub fn remove(&mut self, id: usize) -> bool {
    let index = self.flights.iter().position(|flight| flight.id == id);
    if let Some(index) = index {
      self.flights.swap_remove(index);
      true
    } else {
      false
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
/// A frontend order for a set of flights.
pub struct FrontendOrder {
  /// The quanity of flights to schedule.
  pub quantity: usize,
  /// The kind of flight to schedule.
  pub kind: FlightKind,
  /// The time at which the first flight is scheduled to spawn.
  pub spawn_at: Duration,
  /// The time between each flight spawn.
  pub stagger_by: Duration,
}
