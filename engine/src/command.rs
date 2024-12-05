use core::fmt;
use std::time::Duration;

use internment::Intern;
use serde::{Deserialize, Serialize};

use crate::{abbreviate_altitude, duration_now, pathfinder::Node};

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
  Altitude(f32),
  Frequency(f32),
  GoAround,
  Heading(f32),
  Ident,
  Land(Intern<String>),
  NamedFrequency(String),
  #[serde(rename = "resume")]
  ResumeOwnNavigation,
  Speed(f32),

  Taxi(Vec<Node<()>>),
  TaxiContinue,
  TaxiHold,
  Takeoff(Intern<String>),
  LineUp(Intern<String>),

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutgoingCommandReply {
  pub id: String,
  pub frequency: f32,
  pub reply: String,
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
}

impl fmt::Display for CommandWithFreq {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let decoded_callsign = decode_callsign(&self.id);

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
    }
  }
}
