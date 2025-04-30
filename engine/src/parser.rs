use std::slice::Iter;

use crate::command::Task;

fn parse_heading(mut parts: Iter<&str>) -> Option<Task> {
  let first = parts.next();
  if first == Some(&"turn") || first == Some(&"t") {
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

  let parsers = [parse_heading];

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
  fn test_parse_heading() {
    assert_eq!(parse("turn 250"), vec![Task::Heading(250.0)]);
  }

  #[test]
  fn test_parse_heading_many() {
    assert_eq!(
      parse("turn 250; turn 123"),
      vec![Task::Heading(250.0), Task::Heading(123.0)]
    );
  }
}
