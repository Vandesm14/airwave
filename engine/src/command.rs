use core::fmt;

use serde::{Deserialize, Serialize};

use crate::pathfinder::Node;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "type", content = "value")]
pub enum Task {
  Altitude(f32),
  Approach(String),
  Arrival(String),

  Clearance {
    departure: Option<String>,
    altitude: Option<f32>,
    speed: Option<f32>,
  },

  Depart(String),
  Direct(Vec<String>),
  DirectionOfTravel,
  Frequency(f32),
  GoAround,
  Heading(f32),
  Ident,
  Land(String),
  NamedFrequency(String),
  #[serde(rename = "resume")]
  ResumeOwnNavigation,
  Speed(f32),
  Takeoff(String),

  Taxi(Vec<Node<()>>),
  TaxiContinue,
  TaxiHold,
}

pub type Tasks = Vec<Task>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Command {
  pub id: String,
  pub reply: String,
  pub tasks: Tasks,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandWithFreq {
  pub id: String,
  pub frequency: f32,
  // TODO: Should this be converted to CommandReply so that the front-end can
  //       handle formatting?
  pub reply: String,
  pub tasks: Tasks,
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
pub struct CommandReply {
  pub callsign: String,
  pub kind: CommandReplyKind,
}

impl CommandReply {
  pub fn new(callsign: String, kind: CommandReplyKind) -> Self {
    Self { callsign, kind }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CommandReplyKind {
  WithCallsign { text: String },
  WithoutCallsign { text: String },
  ArriveInAirspace { direction: String },
  HoldShortRunway { runway: String },
  ReadyForDeparture { airport: String },
  TaxiToGates { runway: String },
  DirectionOfDeparture { direction: String },
}

impl fmt::Display for CommandReply {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let decoded_callsign = decode_callsign(&self.callsign);

    match &self.kind {
      CommandReplyKind::WithCallsign { text } => {
        write!(f, "{text}, {}.", decoded_callsign)
      }
      CommandReplyKind::WithoutCallsign { text } => {
        write!(f, "{text}.")
      }
      CommandReplyKind::ArriveInAirspace { direction } => {
        write!(
          f,
          "Approach, {} is {direction} of the airport, with you.",
          decoded_callsign,
        )
      }
      CommandReplyKind::HoldShortRunway { runway } => {
        write!(
          f,
          "Tower, {} is holding short at {}.",
          decoded_callsign, runway
        )
      }
      CommandReplyKind::ReadyForDeparture { airport } => {
        write!(
          f,
          "Clearence, {} ready for departure to {}, as filed.",
          decoded_callsign, airport
        )
      }
      CommandReplyKind::TaxiToGates { runway } => {
        write!(
          f,
          "Ground, {} is at {}, requesting taxi to the gates.",
          decoded_callsign, runway
        )
      }
      CommandReplyKind::DirectionOfDeparture { direction } => {
        write!(
          f,
          "Clearence, {} is departing to the {direction}.",
          decoded_callsign,
        )
      }
    }
  }
}
