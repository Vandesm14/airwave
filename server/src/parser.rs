use std::slice::Iter;

use engine::{
  command::{self, CommandWithFreq, Task},
  pathfinder::{Node, NodeBehavior, NodeKind},
};
use internment::Intern;
use itertools::Itertools;
use regex::Regex;

fn runway_rgx() -> Regex {
  Regex::new(r"^[0-9]{2}[LCRlcr]?$").unwrap()
}

fn taxiway_rgx() -> Regex {
  Regex::new(r"^[a-zA-Z]{1}[0-9]{0,1}$").unwrap()
}

fn parse_altitude(mut parts: Iter<&str>) -> Option<Task> {
  let aliases = ["a", "alt", "altitude"];
  if parts.next().map(|f| aliases.contains(f)) == Some(true) {
    let alt = parts
      .next()
      .and_then(|a| a.parse::<f32>().ok())
      .map(|a| a * 100.0)
      .map(Task::Altitude);

    // End of input.
    if parts.next().is_none() {
      return alt;
    }
  }

  None
}

fn parse_direct(mut parts: Iter<&str>) -> Option<Task> {
  let aliases = ["d", "dt", "direct"];
  if parts.next().map(|f| aliases.contains(f)) == Some(true) {
    let direct = parts
      .next()
      .map(|a| Intern::from(a.to_owned().to_uppercase()))
      .map(Task::Direct);

    // End of input.
    if parts.next().is_none() {
      return direct;
    }
  }

  None
}

fn parse_frequency(mut parts: Iter<&str>) -> Option<Task> {
  let aliases = ["f", "freq", "frequency", "tune", "contact"];
  if parts.next().map(|f| aliases.contains(f)) == Some(true) {
    let arg = parts.next();
    let freq = arg.and_then(|a| a.parse::<f32>().ok());
    let name = arg.map(|a| a.to_lowercase());

    // End of input.
    if parts.next().is_none() {
      if freq.is_some() {
        return freq.map(Task::Frequency);
      } else if name.is_some() {
        return name.map(|a| a.to_owned()).map(Task::NamedFrequency);
      }
    }
  }

  None
}

fn parse_go_around(mut parts: Iter<&str>) -> Option<Task> {
  let aliases = ["g", "ga", "go"];
  if parts.next().map(|f| aliases.contains(f)) == Some(true) {
    let next = parts.next();
    // End of input.
    if next.is_none() || (next == Some(&"around") && parts.next().is_none()) {
      return Some(Task::GoAround);
    }
  }

  None
}

fn parse_heading(mut parts: Iter<&str>) -> Option<Task> {
  let aliases = ["t", "turn", "h", "heading"];
  if parts.next().map(|f| aliases.contains(f)) == Some(true) {
    let heading = parts
      .next()
      .and_then(|a| a.parse::<f32>().ok())
      .map(Task::Heading);

    // End of input.
    if parts.next().is_none() {
      return heading;
    }
  }

  None
}

fn parse_ident(mut parts: Iter<&str>) -> Option<Task> {
  let aliases = ["i", "id", "ident"];
  if parts.next().map(|f| aliases.contains(f)) == Some(true) {
    // End of input.
    if parts.next().is_none() {
      return Some(Task::Ident);
    }
  }

  None
}

fn parse_land(mut parts: Iter<&str>) -> Option<Task> {
  let aliases = ["l", "cl", "land"];
  if parts.next().map(|f| aliases.contains(f)) == Some(true) {
    let runway = parts.next();
    let land = runway
      .and_then(|a| {
        if runway_rgx().is_match(a) {
          Some(Intern::from(a.to_owned().to_uppercase()))
        } else {
          None
        }
      })
      .map(Task::Land);

    // End of input.
    if parts.next().is_none() {
      return land;
    }
  }

  None
}

fn parse_resume_own_navigation(mut parts: Iter<&str>) -> Option<Task> {
  let aliases = ["r", "raf", "resume", "own"];
  if parts.next().map(|f| aliases.contains(f)) == Some(true) {
    // End of input.
    if parts.next().is_none() {
      return Some(Task::ResumeOwnNavigation);
    }
  }

  None
}

fn parse_speed(mut parts: Iter<&str>) -> Option<Task> {
  let aliases = ["s", "spd", "speed"];
  if parts.next().map(|f| aliases.contains(f)) == Some(true) {
    let speed = parts
      .next()
      .and_then(|a| a.parse::<f32>().ok())
      .map(Task::Speed);

    // End of input.
    if parts.next().is_none() {
      return speed;
    }
  }

  None
}

fn parse_taxi(mut parts: Iter<&str>) -> Option<Task> {
  let aliases = ["tx", "taxi"];

  let runway_rgx = runway_rgx();
  let taxiway_rgx = taxiway_rgx();

  if parts.next().map(|f| aliases.contains(f)) == Some(true) {
    // Flags.
    let mut via = false;
    let mut gate = false;
    let mut short = false;

    let mut waypoints: Vec<Node<()>> = Vec::new();

    for part in parts {
      if part == &"via" {
        via = true;
        continue;
      } else if part == &"gate" {
        gate = true;
        continue;
      } else if part == &"short" {
        short = true;
        continue;
      }

      let behavior = if short {
        short = false;

        NodeBehavior::HoldShort
      } else {
        NodeBehavior::GoTo
      };

      if gate {
        gate = false;

        waypoints.push(Node::new(
          Intern::from(part.to_uppercase()),
          NodeKind::Gate,
          behavior,
          (),
        ));
      } else if runway_rgx.is_match(part) {
        waypoints.push(Node::new(
          Intern::from(part.to_uppercase()),
          NodeKind::Runway,
          behavior,
          (),
        ));
      } else if taxiway_rgx.is_match(part) {
        waypoints.push(Node::new(
          Intern::from(part.to_uppercase()),
          NodeKind::Taxiway,
          behavior,
          (),
        ));
      }
    }

    // Logic: A via B C = B C A.
    if via {
      let first = waypoints.remove(0);
      waypoints.push(first);
    }

    return Some(Task::Taxi(waypoints));
  }

  None
}

fn parse_taxi_continue(mut parts: Iter<&str>) -> Option<Task> {
  let aliases = ["c", "tc", "continue"];
  if parts.next().map(|f| aliases.contains(f)) == Some(true) {
    // End of input.
    if parts.next().is_none() {
      return Some(Task::TaxiContinue);
    }
  }

  None
}

fn parse_taxi_hold(mut parts: Iter<&str>) -> Option<Task> {
  let aliases = ["th", "hold", "stop"];
  if parts.next().map(|f| aliases.contains(f)) == Some(true) {
    // End of input.
    if parts.next().is_none() {
      return Some(Task::TaxiHold);
    }
  }

  None
}

fn parse_takeoff(mut parts: Iter<&str>) -> Option<Task> {
  let aliases = ["ct", "cto", "to", "takeoff"];
  if parts.next().map(|f| aliases.contains(f)) == Some(true) {
    let runway = parts.next();
    let takeoff = runway
      .and_then(|a| {
        if runway_rgx().is_match(a) {
          Some(Intern::from(a.to_owned().to_uppercase()))
        } else {
          None
        }
      })
      .map(Task::Takeoff);

    // End of input.
    if parts.next().is_none() {
      return takeoff;
    }
  }

  None
}

fn parse_line_up(mut parts: Iter<&str>) -> Option<Task> {
  let aliases = ["lu", "line", "wait"];
  if parts.next().map(|f| aliases.contains(f)) == Some(true) {
    let runway = parts.next();
    let line_up = runway
      .and_then(|a| {
        if runway_rgx().is_match(a) {
          Some(Intern::from(a.to_owned().to_uppercase()))
        } else {
          None
        }
      })
      .map(Task::LineUp);

    // End of input.
    if parts.next().is_none() {
      return line_up;
    }
  }

  None
}

fn parse_delete(mut parts: Iter<&str>) -> Option<Task> {
  let aliases = ["del", "delete"];
  if parts.next().map(|f| aliases.contains(f)) == Some(true) {
    // End of input.
    if parts.next().is_none() {
      return Some(Task::Delete);
    }
  }

  None
}

/// Parses a set of tasks.
pub fn parse_tasks<T>(tasks_str: T) -> Vec<Task>
where
  T: AsRef<str>,
{
  let mut tasks: Vec<Task> = Vec::new();

  let parsers = [
    parse_altitude,
    parse_direct,
    parse_frequency,
    parse_go_around,
    parse_heading,
    parse_ident,
    parse_land,
    parse_resume_own_navigation,
    parse_speed,
    parse_taxi,
    parse_taxi_continue,
    parse_taxi_hold,
    parse_takeoff,
    parse_line_up,
    parse_delete,
  ];

  let items = tasks_str.as_ref().split(',');
  for item in items {
    let parts = item.trim().split(' ').collect::<Vec<_>>();
    for parser in parsers {
      if let Some(t) = parser(parts.iter()) {
        tasks.push(t);
        break;
      }
    }
  }

  tasks
}

/// Parses full commands (callsign + tasks) addressed aircraft(s).
pub fn parse_commands<T>(
  commands_str: T,
  frequency: f32,
) -> Vec<CommandWithFreq>
where
  T: AsRef<str>,
{
  let mut commands: Vec<CommandWithFreq> = Vec::new();

  let items = commands_str.as_ref().split(';');
  for item in items {
    let mut parts = item.split(' ');
    if let Some(callsign) = parts.next() {
      let rest = parts.join(" ");
      let rest = rest.trim();
      let tasks = parse_tasks(rest);
      if !tasks.is_empty() {
        commands.push(CommandWithFreq::new(
          callsign.to_owned(),
          frequency,
          command::CommandReply::WithoutCallsign {
            text: rest.to_owned(),
          },
          tasks,
        ));
      }
    }
  }

  commands
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_altitude() {
    // Alias variants.
    assert_eq!(parse_tasks("a 250"), vec![Task::Altitude(25000.0)]);
    assert_eq!(parse_tasks("alt 250"), vec![Task::Altitude(25000.0)]);
    assert_eq!(parse_tasks("altitude 250"), vec![Task::Altitude(25000.0)]);

    // Argument variants.
    assert_eq!(parse_tasks("alt 250"), vec![Task::Altitude(25000.0)]);
    assert_eq!(parse_tasks("alt 040"), vec![Task::Altitude(4000.0)]);

    // Invalid.
    assert_eq!(parse_tasks("alt abcd"), vec![]);
    assert_eq!(parse_tasks("alt 250 abcd"), vec![]);
  }

  #[test]
  fn parse_many() {
    // Test multiple commands.
    assert_eq!(
      parse_tasks("alt 250, alt 040, alt 40"),
      vec![
        Task::Altitude(25000.0),
        Task::Altitude(4000.0),
        Task::Altitude(4000.0)
      ]
    );

    // Test multiple commands with different parsers.
    assert_eq!(
      parse_tasks("alt 250, direct ABCD, f 123.4"),
      vec![
        Task::Altitude(25000.0),
        Task::Direct(Intern::from_ref("ABCD")),
        Task::Frequency(123.4)
      ]
    );

    // Test trailing comma.
    assert_eq!(
      parse_tasks("alt 250, direct ABCD, f 123.4,"),
      vec![
        Task::Altitude(25000.0),
        Task::Direct(Intern::from_ref("ABCD")),
        Task::Frequency(123.4)
      ]
    );
  }

  #[test]
  fn parse_direct() {
    // Alias variants.
    assert_eq!(
      parse_tasks("d ABCD"),
      vec![Task::Direct(Intern::from_ref("ABCD"))]
    );
    assert_eq!(
      parse_tasks("dt ABCD"),
      vec![Task::Direct(Intern::from_ref("ABCD"))]
    );
    assert_eq!(
      parse_tasks("direct ABCD"),
      vec![Task::Direct(Intern::from_ref("ABCD"))]
    );

    // Argument variants.
    assert_eq!(
      parse_tasks("direct ABCD"),
      vec![Task::Direct(Intern::from_ref("ABCD"))]
    );
    assert_eq!(
      parse_tasks("direct abcd"),
      vec![Task::Direct(Intern::from_ref("ABCD"))]
    );

    // Invalid.
    assert_eq!(parse_tasks("direct"), vec![]);
    assert_eq!(parse_tasks("direct ABCD EFGH"), vec![]);
  }

  #[test]
  fn parse_frequency() {
    // Alias variants.
    assert_eq!(parse_tasks("f 123.4"), vec![Task::Frequency(123.4)]);
    assert_eq!(parse_tasks("freq 123.4"), vec![Task::Frequency(123.4)]);
    assert_eq!(parse_tasks("frequency 123.4"), vec![Task::Frequency(123.4)]);
    assert_eq!(parse_tasks("tune 123.4"), vec![Task::Frequency(123.4)]);
    assert_eq!(parse_tasks("contact 123.4"), vec![Task::Frequency(123.4)]);

    // Argument variants.
    assert_eq!(
      parse_tasks("contact departure"),
      vec![Task::NamedFrequency("departure".to_owned())]
    );
    assert_eq!(
      parse_tasks("contact DEPARTURE"),
      vec![Task::NamedFrequency("departure".to_owned())]
    );

    // Invalid.
    assert_eq!(parse_tasks("f"), vec![]);
    assert_eq!(parse_tasks("f 123.4 567.8"), vec![]);
    assert_eq!(parse_tasks("f 123.4 ABCD"), vec![]);
  }

  #[test]
  fn parse_go_around() {
    // Alias variants.
    assert_eq!(parse_tasks("g"), vec![Task::GoAround]);
    assert_eq!(parse_tasks("ga"), vec![Task::GoAround]);
    assert_eq!(parse_tasks("go"), vec![Task::GoAround]);
    assert_eq!(parse_tasks("go around"), vec![Task::GoAround]);

    // Invalid.
    assert_eq!(parse_tasks("go around 27L"), vec![]);
    assert_eq!(parse_tasks("go 27L"), vec![]);
  }

  #[test]
  fn parse_heading() {
    // Alias variants.
    assert_eq!(parse_tasks("t 250"), vec![Task::Heading(250.0)]);
    assert_eq!(parse_tasks("turn 250"), vec![Task::Heading(250.0)]);
    assert_eq!(parse_tasks("h 250"), vec![Task::Heading(250.0)]);
    assert_eq!(parse_tasks("heading 250"), vec![Task::Heading(250.0)]);

    // Argument variants.
    assert_eq!(parse_tasks("heading 250"), vec![Task::Heading(250.0)]);
    assert_eq!(parse_tasks("heading 040"), vec![Task::Heading(40.0)]);
    assert_eq!(parse_tasks("heading 040.0"), vec![Task::Heading(40.0)]);

    // Invalid.
    assert_eq!(parse_tasks("heading"), vec![]);
    assert_eq!(parse_tasks("heading 250 270"), vec![]);
    assert_eq!(parse_tasks("heading 250 ABCD"), vec![]);
    assert_eq!(parse_tasks("heading ABCD"), vec![]);
  }

  #[test]
  fn parse_ident() {
    // Alias variants.
    assert_eq!(parse_tasks("i"), vec![Task::Ident]);
    assert_eq!(parse_tasks("id"), vec![Task::Ident]);
    assert_eq!(parse_tasks("ident"), vec![Task::Ident]);

    // Invalid.
    assert_eq!(parse_tasks("id 27L"), vec![]);
    assert_eq!(parse_tasks("id 27L ABCD"), vec![]);
    assert_eq!(parse_tasks("id ABCD"), vec![]);
  }

  #[test]
  fn parse_land() {
    // Alias variants.
    assert_eq!(
      parse_tasks("l 27L"),
      vec![Task::Land(Intern::from_ref("27L"))]
    );
    assert_eq!(
      parse_tasks("cl 27L"),
      vec![Task::Land(Intern::from_ref("27L"))]
    );
    assert_eq!(
      parse_tasks("land 27L"),
      vec![Task::Land(Intern::from_ref("27L"))]
    );

    // Argument variants.
    assert_eq!(
      parse_tasks("land 27r"),
      vec![Task::Land(Intern::from_ref("27R"))]
    );
    assert_eq!(
      parse_tasks("land 27l"),
      vec![Task::Land(Intern::from_ref("27L"))]
    );

    // Invalid.
    assert_eq!(parse_tasks("land"), vec![]);
    assert_eq!(parse_tasks("land 27L 27R"), vec![]);
    assert_eq!(parse_tasks("land 27L ABCD"), vec![]);
    assert_eq!(parse_tasks("land ABCD"), vec![]);
  }

  #[test]
  fn parse_resume_own_navigation() {
    // Alias variants.
    assert_eq!(parse_tasks("r"), vec![Task::ResumeOwnNavigation]);
    assert_eq!(parse_tasks("raf"), vec![Task::ResumeOwnNavigation]);
    assert_eq!(parse_tasks("resume"), vec![Task::ResumeOwnNavigation]);
    assert_eq!(parse_tasks("own"), vec![Task::ResumeOwnNavigation]);

    // Invalid.
    assert_eq!(parse_tasks("resume 27L"), vec![]);
    assert_eq!(parse_tasks("resume 27L ABCD"), vec![]);
    assert_eq!(parse_tasks("resume ABCD"), vec![]);
  }

  #[test]
  fn parse_speed() {
    // Alias variants.
    assert_eq!(parse_tasks("s 250"), vec![Task::Speed(250.0)]);
    assert_eq!(parse_tasks("spd 250"), vec![Task::Speed(250.0)]);
    assert_eq!(parse_tasks("speed 250"), vec![Task::Speed(250.0)]);

    // Argument variants.
    assert_eq!(parse_tasks("speed 250"), vec![Task::Speed(250.0)]);
    assert_eq!(parse_tasks("speed 90"), vec![Task::Speed(90.0)]);

    // Invalid.
    assert_eq!(parse_tasks("speed"), vec![]);
    assert_eq!(parse_tasks("speed 250 270"), vec![]);
    assert_eq!(parse_tasks("speed 250 ABCD"), vec![]);
    assert_eq!(parse_tasks("speed ABCD"), vec![]);
  }

  #[test]
  fn parse_taxi() {
    // Argument variants.
    assert_eq!(
      parse_tasks("tx A"),
      vec![Task::Taxi(vec![
        Node::build(()).with_name(Intern::from_ref("A"))
      ])]
    );
    assert_eq!(
      parse_tasks("tx a"),
      vec![Task::Taxi(vec![
        Node::build(()).with_name(Intern::from_ref("A"))
      ])]
    );
    assert_eq!(
      parse_tasks("tx 27L"),
      vec![Task::Taxi(vec![
        Node::build(())
          .with_name(Intern::from_ref("27L"))
          .with_kind(NodeKind::Runway)
      ])]
    );
    assert_eq!(
      parse_tasks("tx 27r"),
      vec![Task::Taxi(vec![
        Node::build(())
          .with_name(Intern::from_ref("27R"))
          .with_kind(NodeKind::Runway)
      ])]
    );
    assert_eq!(
      parse_tasks("tx gate A1"),
      vec![Task::Taxi(vec![
        Node::build(())
          .with_name(Intern::from_ref("A1"))
          .with_kind(NodeKind::Gate)
      ])]
    );
    assert_eq!(
      parse_tasks("tx gate a1"),
      vec![Task::Taxi(vec![
        Node::build(())
          .with_name(Intern::from_ref("A1"))
          .with_kind(NodeKind::Gate),
      ])]
    );

    // Flags.
    // Flags - Gate.
    assert_eq!(
      parse_tasks("tx B gate A1"),
      vec![Task::Taxi(vec![
        Node::build(()).with_name(Intern::from_ref("B")),
        Node::build(())
          .with_name(Intern::from_ref("A1"))
          .with_kind(NodeKind::Gate),
      ])]
    );

    // Flags - Via.
    assert_eq!(
      parse_tasks("tx short 27L via A B"),
      vec![Task::Taxi(vec![
        Node::build(()).with_name(Intern::from_ref("A")),
        Node::build(()).with_name(Intern::from_ref("B")),
        Node::build(())
          .with_name(Intern::from_ref("27L"))
          .with_kind(NodeKind::Runway)
          .with_behavior(NodeBehavior::HoldShort)
      ])]
    );

    // Flags - Short.
    assert_eq!(
      parse_tasks("tx short 27L"),
      vec![Task::Taxi(vec![
        Node::build(())
          .with_name(Intern::from_ref("27L"))
          .with_kind(NodeKind::Runway)
          .with_behavior(NodeBehavior::HoldShort)
      ])]
    );
    assert_eq!(
      parse_tasks("tx A short B"),
      vec![Task::Taxi(vec![
        Node::build(()).with_name(Intern::from_ref("A")),
        Node::build(())
          .with_name(Intern::from_ref("B"))
          .with_behavior(NodeBehavior::HoldShort),
      ])]
    );

    // Integration.
    assert_eq!(
      parse_tasks("tx A via B C"),
      vec![Task::Taxi(vec![
        Node::build(()).with_name(Intern::from_ref("B")),
        Node::build(()).with_name(Intern::from_ref("C")),
        Node::build(()).with_name(Intern::from_ref("A")),
      ])]
    );
    assert_eq!(
      parse_tasks("tx A B C"),
      vec![Task::Taxi(vec![
        Node::build(()).with_name(Intern::from_ref("A")),
        Node::build(()).with_name(Intern::from_ref("B")),
        Node::build(()).with_name(Intern::from_ref("C")),
      ])]
    );
    assert_eq!(
      parse_tasks("tx short 27L via A1 B2 C"),
      vec![Task::Taxi(vec![
        Node::build(()).with_name(Intern::from_ref("A1")),
        Node::build(()).with_name(Intern::from_ref("B2")),
        Node::build(()).with_name(Intern::from_ref("C")),
        Node::build(())
          .with_name(Intern::from_ref("27L"))
          .with_kind(NodeKind::Runway)
          .with_behavior(NodeBehavior::HoldShort),
      ])]
    );

    // Invalid.
    assert_eq!(parse_tasks("tx ABC"), vec![Task::Taxi(vec![])]);
    assert_eq!(parse_tasks("tx A11"), vec![Task::Taxi(vec![])]);
    assert_eq!(parse_tasks("tx 23W"), vec![Task::Taxi(vec![])]);
    assert_eq!(parse_tasks("tx 2"), vec![Task::Taxi(vec![])]);
  }

  #[test]
  fn parse_taxi_continue() {
    // Alias variants.
    assert_eq!(parse_tasks("c"), vec![Task::TaxiContinue]);
    assert_eq!(parse_tasks("tc"), vec![Task::TaxiContinue]);
    assert_eq!(parse_tasks("continue"), vec![Task::TaxiContinue]);

    // Invalid.
    assert_eq!(parse_tasks("tc 27L"), vec![]);
    assert_eq!(parse_tasks("tc 27L ABCD"), vec![]);
    assert_eq!(parse_tasks("tc ABCD"), vec![]);
  }

  #[test]
  fn parse_taxi_hold() {
    // Alias variants.
    assert_eq!(parse_tasks("th"), vec![Task::TaxiHold]);
    assert_eq!(parse_tasks("hold"), vec![Task::TaxiHold]);
    assert_eq!(parse_tasks("stop"), vec![Task::TaxiHold]);

    // Invalid.
    assert_eq!(parse_tasks("th 27L"), vec![]);
    assert_eq!(parse_tasks("th 27L ABCD"), vec![]);
    assert_eq!(parse_tasks("th ABCD"), vec![]);
  }

  #[test]
  fn parse_takeoff() {
    // Alias variants.
    assert_eq!(
      parse_tasks("ct 27L"),
      vec![Task::Takeoff(Intern::from_ref("27L"))]
    );
    assert_eq!(
      parse_tasks("cto 27L"),
      vec![Task::Takeoff(Intern::from_ref("27L"))]
    );
    assert_eq!(
      parse_tasks("to 27L"),
      vec![Task::Takeoff(Intern::from_ref("27L"))]
    );
    assert_eq!(
      parse_tasks("takeoff 27L"),
      vec![Task::Takeoff(Intern::from_ref("27L"))]
    );

    // Argument variants.
    assert_eq!(
      parse_tasks("takeoff 27r"),
      vec![Task::Takeoff(Intern::from_ref("27R"))]
    );
    assert_eq!(
      parse_tasks("takeoff 27l"),
      vec![Task::Takeoff(Intern::from_ref("27L"))]
    );

    // Invalid.
    assert_eq!(parse_tasks("takeoff"), vec![]);
    assert_eq!(parse_tasks("takeoff 27L 27R"), vec![]);
    assert_eq!(parse_tasks("takeoff 27L ABCD"), vec![]);
    assert_eq!(parse_tasks("takeoff ABCD"), vec![]);
  }

  #[test]
  fn parse_line_up() {
    // Alias variants.
    assert_eq!(
      parse_tasks("lu 27L"),
      vec![Task::LineUp(Intern::from_ref("27L"))]
    );
    assert_eq!(
      parse_tasks("line 27L"),
      vec![Task::LineUp(Intern::from_ref("27L"))]
    );
    assert_eq!(
      parse_tasks("wait 27L"),
      vec![Task::LineUp(Intern::from_ref("27L"))]
    );

    // Argument variants.
    assert_eq!(
      parse_tasks("line 27r"),
      vec![Task::LineUp(Intern::from_ref("27R"))]
    );
    assert_eq!(
      parse_tasks("line 27l"),
      vec![Task::LineUp(Intern::from_ref("27L"))]
    );

    // Invalid.
    assert_eq!(parse_tasks("line"), vec![]);
    assert_eq!(parse_tasks("line 27L 27R"), vec![]);
    assert_eq!(parse_tasks("line 27L ABCD"), vec![]);
    assert_eq!(parse_tasks("line ABCD"), vec![]);
  }

  #[test]
  fn parse_delete() {
    // Alias variants.
    assert_eq!(parse_tasks("delete"), vec![Task::Delete]);
    assert_eq!(parse_tasks("del"), vec![Task::Delete]);

    // Invalid.
    assert_eq!(parse_tasks("delete 27L"), vec![]);
    assert_eq!(parse_tasks("delete 27L ABCD"), vec![]);
    assert_eq!(parse_tasks("delete ABCD"), vec![]);
  }
}
