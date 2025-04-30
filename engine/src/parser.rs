use std::slice::Iter;

use internment::Intern;

use crate::command::Task;

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
  let aliases = ["f", "freq", "frequency", "tune"];
  if parts.next().map(|f| aliases.contains(f)) == Some(true) {
    return parts
      .next()
      .and_then(|a| a.parse::<f32>().ok())
      .map(Task::Frequency);
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
  let aliases = ["t", "turn", "heading", "h"];
  if parts.next().map(|f| aliases.contains(f)) == Some(true) {
    return parts
      .next()
      .and_then(|a| a.parse::<f32>().ok())
      .map(Task::Heading);
  }

  None
}

pub fn parse<T>(commands: T) -> Vec<Task>
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
  ];

  let commands = commands.as_ref().split(";");
  for command in commands {
    let parts = command.trim().split(" ").collect::<Vec<_>>();
    for parser in parsers {
      if let Some(t) = parser(parts.iter()) {
        tasks.push(t);
        break;
      }
    }
  }

  tasks
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_altitude() {
    assert_eq!(parse("alt 250"), vec![Task::Altitude(25000.0)]);
    assert_eq!(parse("alt 040"), vec![Task::Altitude(4000.0)]);
  }

  #[test]
  fn parse_altitude_many() {
    assert_eq!(
      parse("alt 250; alt 040; alt 40"),
      vec![
        Task::Altitude(25000.0),
        Task::Altitude(4000.0),
        Task::Altitude(4000.0)
      ]
    );
  }

  #[test]
  fn parse_direct() {
    assert_eq!(
      parse("direct ABCD"),
      vec![Task::Direct(Intern::from_ref("ABCD"))]
    );
    assert_eq!(
      parse("direct abcd"),
      vec![Task::Direct(Intern::from_ref("ABCD"))]
    );
  }

  #[test]
  fn parse_frequency() {
    assert_eq!(parse("frequency 123.4"), vec![Task::Frequency(123.4)]);
  }

  #[test]
  fn parse_go_around() {
    assert_eq!(parse("g"), vec![Task::GoAround]);
    assert_eq!(parse("ga"), vec![Task::GoAround]);
    assert_eq!(parse("go"), vec![Task::GoAround]);
    assert_eq!(parse("go around"), vec![Task::GoAround]);
  }

  #[test]
  fn parse_heading() {
    assert_eq!(parse("turn 250"), vec![Task::Heading(250.0)]);
  }
}
