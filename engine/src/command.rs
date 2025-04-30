use core::fmt;
use std::time::Duration;

use internment::Intern;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
  abbreviate_altitude, duration_now, pathfinder::Node, wordify::wordify,
  ExportedDuration,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskWaypoint {
  Approach(Intern<String>),
  Arrival(Intern<String>),
  Departure(Intern<String>),
  Direct(Intern<String>),
  Destination,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "type", content = "value")]
pub enum Task {
  // a, alt, altitude
  Altitude(f32),
  // d, dt, direct
  Direct(Intern<String>),
  // f, freq, frequency, tune
  Frequency(f32),
  // g, ga, go
  GoAround,
  // h, heading, t, turn
  Heading(f32),
  // i
  Ident,
  // l, land, cl
  Land(Intern<String>),
  // fn
  NamedFrequency(String),
  // r, raf
  #[serde(rename = "resume")]
  ResumeOwnNavigation,
  // s, spd, speed
  Speed(f32),

  // tx
  Taxi(Vec<Node<()>>),
  // tc, c
  TaxiContinue,
  // th
  TaxiHold,
  // ct, to, takeoff
  Takeoff(Intern<String>),
  // lu. line
  LineUp(Intern<String>),

  // delete ,del
  Delete,
}

pub type Tasks = Vec<Task>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Command {
  pub id: String,
  pub reply: CommandReply,
  pub tasks: Tasks,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandWithFreq {
  pub id: String,
  pub frequency: f32,
  pub reply: CommandReply,
  pub tasks: Tasks,
  pub created: Duration,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct OutgoingCommandReply {
  pub id: String,
  pub frequency: f32,
  pub reply: String,
  #[ts(as = "ExportedDuration")]
  pub created: Duration,
}

impl From<CommandWithFreq> for OutgoingCommandReply {
  fn from(value: CommandWithFreq) -> Self {
    Self {
      id: value.id.clone(),
      frequency: value.frequency,
      reply: value.to_string(),
      created: value.created,
    }
  }
}

impl CommandWithFreq {
  pub fn new(
    id: String,
    frequency: f32,
    reply: CommandReply,
    tasks: Tasks,
  ) -> Self {
    Self {
      id,
      frequency,
      reply,
      tasks,
      created: duration_now(),
    }
  }
}

pub fn decode_callsign(callsign: &str) -> String {
  let airline = callsign.chars().take(3).collect::<String>();
  let fnumber = callsign.chars().skip(3).collect::<String>();

  let airline_str = match airline.as_str() {
    "AAL" => "American Airlines",
    "SKW" => "Skywest",
    "JBU" => "JetBlue",
    _ => "Unknown",
  };

  format!("{airline_str} {fnumber}")
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CommandReply {
  Empty,
  Blank { text: String },
  WithoutCallsign { text: String },
  WithCallsign { text: String },

  GoAround { runway: String },
  HoldShortRunway { runway: String },
  ReadyForDeparture { airport: String },
  TaxiToGates { runway: String },
  ArriveInAirspace { direction: String, altitude: f32 },
  TARAResolved { assigned_alt: f32 },
}

impl fmt::Display for CommandWithFreq {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let decoded_callsign = wordify(&self.id);

    match &self.reply {
      CommandReply::Empty => {
        write!(f, "")
      }
      CommandReply::Blank { text } => {
        write!(f, "{text}")
      }
      CommandReply::WithoutCallsign { text } => {
        write!(f, "{text}.")
      }
      CommandReply::WithCallsign { text } => {
        write!(f, "{text}, {}.", decoded_callsign)
      }

      CommandReply::GoAround { runway } => {
        write!(f, "{decoded_callsign}, going around, missed approach for runway {runway}.")
      }
      CommandReply::ArriveInAirspace {
        direction,
        altitude,
      } => {
        write!(
          f,
          "Approach, {} is {direction} of the airport at {}, with you.",
          decoded_callsign,
          abbreviate_altitude(*altitude)
        )
      }
      CommandReply::HoldShortRunway { runway } => {
        write!(
          f,
          "Tower, {} is holding short at {}.",
          decoded_callsign, runway
        )
      }
      CommandReply::ReadyForDeparture { airport } => {
        write!(
          f,
          "Ground, {} ready for departure to {}.",
          decoded_callsign, airport
        )
      }
      CommandReply::TaxiToGates { runway } => {
        write!(
          f,
          "Ground, {} is on runway {}, requesting taxi to the gates.",
          decoded_callsign, runway
        )
      }
      CommandReply::TARAResolved { assigned_alt } => {
        write!(
          f,
          "Center, {} TCAS RA, returning to {}.",
          decoded_callsign,
          abbreviate_altitude(*assigned_alt)
        )
      }
    }
  }
}
