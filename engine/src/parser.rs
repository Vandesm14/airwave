use std::slice::Iter;

use crate::command::Task;

fn parse_heading(mut parts: Iter<&str>) -> Option<Task> {
  let first = parts.next();
  if first == Some(&"turn") || first == Some(&"t") {
    let arg = parts.next();
    if let Some(arg) = arg.and_then(|a| a.parse::<f32>().ok()) {
      return Some(Task::Heading(arg));
    }
  }

  None
}

pub fn parse<T>(commands: T) -> Vec<Task>
where
  T: AsRef<str>,
{
  let mut tasks: Vec<Task> = Vec::new();

  let commands = commands.as_ref().split(";");
  for command in commands {
    let parts = command.trim().split(" ").collect::<Vec<_>>();
    if let Some(t) = parse_heading(parts.iter()) {
      tasks.push(t)
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
}
