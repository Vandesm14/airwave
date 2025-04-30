use std::slice::Iter;

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

  let parsers = [parse_altitude, parse_heading];

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
  fn parse_heading() {
    assert_eq!(parse("turn 250"), vec![Task::Heading(250.0)]);
  }

  #[test]
  fn parse_heading_many() {
    assert_eq!(
      parse("turn 250; turn 123"),
      vec![Task::Heading(250.0), Task::Heading(123.0)]
    );
  }

  #[test]
  fn parse_altitude() {
    assert_eq!(parse("alt 250"), vec![Task::Altitude(25000.0)]);
    assert_eq!(parse("alt 040"), vec![Task::Altitude(4000.0)]);
  }
}
