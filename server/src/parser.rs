use std::slice::Iter;

use engine::{
  command::{self, CommandWithFreq, Task},
  pathfinder::{Node, NodeBehavior, NodeKind},
};
use internment::Intern;
use itertools::Itertools;

fn parse_altitude(mut parts: Iter<&str>) -> Option<Task> {
  let aliases = ["a", "alt", "altitude"];
  if parts.next().map(|f| aliases.contains(f)) == Some(true) {
    return parts
      .next()
      .and_then(|a| a.parse::<f32>().ok())
      .map(|a| a * 100.0)
      .map(Task::Altitude);
  }

  None
}

fn parse_direct(mut parts: Iter<&str>) -> Option<Task> {
  let aliases = ["d", "dt", "direct"];
  if parts.next().map(|f| aliases.contains(f)) == Some(true) {
    return parts
      .next()
      .map(|a| Intern::from(a.to_owned().to_uppercase()))
      .map(Task::Direct);
  }

  None
}

fn parse_frequency(mut parts: Iter<&str>) -> Option<Task> {
  let aliases = ["f", "freq", "frequency", "tune", "contact"];
  if parts.next().map(|f| aliases.contains(f)) == Some(true) {
    let arg = parts.next();
    let freq = arg.and_then(|a| a.parse::<f32>().ok());
    let name = arg.map(|a| a.to_lowercase());

    if freq.is_some() {
      return freq.map(Task::Frequency);
    } else if name.is_some() {
      return name.map(|a| a.to_owned()).map(Task::NamedFrequency);
    }
  }

  None
}

fn parse_go_around(mut parts: Iter<&str>) -> Option<Task> {
  let aliases = ["g", "ga", "go"];
  if parts.next().map(|f| aliases.contains(f)) == Some(true) {
    return Some(Task::GoAround);
  }

  None
}

fn parse_heading(mut parts: Iter<&str>) -> Option<Task> {
  let aliases = ["t", "turn", "h", "heading"];
  if parts.next().map(|f| aliases.contains(f)) == Some(true) {
    return parts
      .next()
      .and_then(|a| a.parse::<f32>().ok())
      .map(Task::Heading);
  }

  None
}

fn parse_ident(mut parts: Iter<&str>) -> Option<Task> {
  let aliases = ["i", "id", "ident"];
  if parts.next().map(|f| aliases.contains(f)) == Some(true) {
    return Some(Task::Ident);
  }

  None
}

fn parse_land(mut parts: Iter<&str>) -> Option<Task> {
  let aliases = ["l", "cl", "land"];
  if parts.next().map(|f| aliases.contains(f)) == Some(true) {
    return parts
      .next()
      .map(|a| Intern::from(a.to_owned().to_uppercase()))
      .map(Task::Land);
  }

  None
}

fn parse_resume_own_navigation(mut parts: Iter<&str>) -> Option<Task> {
  let aliases = ["r", "raf", "resume", "own"];
  if parts.next().map(|f| aliases.contains(f)) == Some(true) {
    return Some(Task::ResumeOwnNavigation);
  }

  None
}

fn parse_speed(mut parts: Iter<&str>) -> Option<Task> {
  let aliases = ["s", "spd", "speed"];
  if parts.next().map(|f| aliases.contains(f)) == Some(true) {
    return parts
      .next()
      .and_then(|a| a.parse::<f32>().ok())
      .map(Task::Speed);
  }

  None
}

fn parse_taxi(mut parts: Iter<&str>) -> Option<Task> {
  let aliases = ["tx", "taxi"];
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
      } else {
        let runway = part.chars().next().and_then(|c| c.to_digit(10)).is_some();
        if runway {
          waypoints.push(Node::new(
            Intern::from(part.to_uppercase()),
            NodeKind::Runway,
            behavior,
            (),
          ));
        } else {
          waypoints.push(Node::new(
            Intern::from(part.to_uppercase()),
            NodeKind::Taxiway,
            behavior,
            (),
          ));
        }
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
    return Some(Task::TaxiContinue);
  }

  None
}

fn parse_taxi_hold(mut parts: Iter<&str>) -> Option<Task> {
  let aliases = ["th", "hold", "stop"];
  if parts.next().map(|f| aliases.contains(f)) == Some(true) {
    return Some(Task::TaxiHold);
  }

  None
}

fn parse_takeoff(mut parts: Iter<&str>) -> Option<Task> {
  let aliases = ["ct", "to", "takeoff"];
  if parts.next().map(|f| aliases.contains(f)) == Some(true) {
    return parts
      .next()
      .map(|a| Intern::from(a.to_owned().to_uppercase()))
      .map(Task::Takeoff);
  }

  None
}

fn parse_line_up(mut parts: Iter<&str>) -> Option<Task> {
  let aliases = ["lu", "line", "wait"];
  if parts.next().map(|f| aliases.contains(f)) == Some(true) {
    return parts
      .next()
      .map(|a| Intern::from(a.to_owned().to_uppercase()))
      .map(Task::LineUp);
  }

  None
}

fn parse_delete(mut parts: Iter<&str>) -> Option<Task> {
  let aliases = ["del", "delete"];
  if parts.next().map(|f| aliases.contains(f)) == Some(true) {
    return Some(Task::Delete);
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
  }

  #[test]
  fn parse_many() {
    // Test multiple commands
    assert_eq!(
      parse_tasks("alt 250, alt 040, alt 40"),
      vec![
        Task::Altitude(25000.0),
        Task::Altitude(4000.0),
        Task::Altitude(4000.0)
      ]
    );

    // Test multiple commands with different parsers
    assert_eq!(
      parse_tasks("alt 250, direct ABCD, f 123.4"),
      vec![
        Task::Altitude(25000.0),
        Task::Direct(Intern::from_ref("ABCD")),
        Task::Frequency(123.4)
      ]
    );

    // Test trailing comma
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
  }

  #[test]
  fn parse_go_around() {
    // Alias variants.
    assert_eq!(parse_tasks("g"), vec![Task::GoAround]);
    assert_eq!(parse_tasks("ga"), vec![Task::GoAround]);
    assert_eq!(parse_tasks("go"), vec![Task::GoAround]);
    assert_eq!(parse_tasks("go around"), vec![Task::GoAround]);
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
  }

  #[test]
  fn parse_ident() {
    // Alias variants.
    assert_eq!(parse_tasks("i"), vec![Task::Ident]);
    assert_eq!(parse_tasks("id"), vec![Task::Ident]);
    assert_eq!(parse_tasks("ident"), vec![Task::Ident]);
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
  }

  #[test]
  fn parse_resume_own_navigation() {
    // Alias variants.
    assert_eq!(parse_tasks("r"), vec![Task::ResumeOwnNavigation]);
    assert_eq!(parse_tasks("raf"), vec![Task::ResumeOwnNavigation]);
    assert_eq!(parse_tasks("resume"), vec![Task::ResumeOwnNavigation]);
    assert_eq!(parse_tasks("own"), vec![Task::ResumeOwnNavigation]);
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
  }

  #[test]
  fn parse_taxi() {
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
      vec![Task::Taxi(vec![Node::build(())
        .with_name(Intern::from_ref("27L"))
        .with_kind(NodeKind::Runway)])]
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
      vec![Task::Taxi(vec![Node::build(())
        .with_name(Intern::from_ref("27L"))
        .with_kind(NodeKind::Runway)
        .with_behavior(NodeBehavior::HoldShort)])]
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
  }

  #[test]
  fn parse_taxi_continue() {
    // Alias variants.
    assert_eq!(parse_tasks("c"), vec![Task::TaxiContinue]);
    assert_eq!(parse_tasks("tc"), vec![Task::TaxiContinue]);
    assert_eq!(parse_tasks("continue"), vec![Task::TaxiContinue]);
  }

  #[test]
  fn parse_taxi_hold() {
    // Alias variants.
    assert_eq!(parse_tasks("th"), vec![Task::TaxiHold]);
    assert_eq!(parse_tasks("hold"), vec![Task::TaxiHold]);
    assert_eq!(parse_tasks("stop"), vec![Task::TaxiHold]);
  }

  #[test]
  fn parse_takeoff() {
    // Alias variants.
    assert_eq!(
      parse_tasks("ct 27L"),
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
  }

  #[test]
  fn parse_delete() {
    // Alias variants.
    assert_eq!(parse_tasks("delete"), vec![Task::Delete]);
    assert_eq!(parse_tasks("del"), vec![Task::Delete]);
  }
}
