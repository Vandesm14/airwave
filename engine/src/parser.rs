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
  let aliases = ["t", "turn", "heading", "h"];
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
  let aliases = ["l", "land", "cl"];
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
  let aliases = ["tx"];
  if parts.next().map(|f| aliases.contains(f)) == Some(true) {
    todo!("parse taxi")
  }

  None
}

fn parse_taxi_continue(mut parts: Iter<&str>) -> Option<Task> {
  let aliases = ["tc", "c"];
  if parts.next().map(|f| aliases.contains(f)) == Some(true) {
    return Some(Task::TaxiContinue);
  }

  None
}

fn parse_taxi_hold(mut parts: Iter<&str>) -> Option<Task> {
  let aliases = ["th"];
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
  let aliases = ["delete", "del"];
  if parts.next().map(|f| aliases.contains(f)) == Some(true) {
    return Some(Task::Delete);
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
    assert_eq!(parse("f 123.4"), vec![Task::Frequency(123.4)]);
    assert_eq!(parse("frequency 123.4"), vec![Task::Frequency(123.4)]);

    assert_eq!(
      parse("contact departure"),
      vec![Task::NamedFrequency("departure".to_owned())]
    );
    assert_eq!(
      parse("contact DEPARTURE"),
      vec![Task::NamedFrequency("departure".to_owned())]
    );
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

  #[test]
  fn parse_ident() {
    assert_eq!(parse("i"), vec![Task::Ident]);
    assert_eq!(parse("id"), vec![Task::Ident]);
    assert_eq!(parse("ident"), vec![Task::Ident]);
  }

  #[test]
  fn parse_land() {
    assert_eq!(parse("land 27L"), vec![Task::Land(Intern::from_ref("27L"))]);
    assert_eq!(parse("land 27l"), vec![Task::Land(Intern::from_ref("27L"))]);
  }

  #[test]
  fn parse_resume_own_navigation() {
    assert_eq!(parse("r"), vec![Task::ResumeOwnNavigation]);
    assert_eq!(parse("raf"), vec![Task::ResumeOwnNavigation]);
    assert_eq!(parse("resume"), vec![Task::ResumeOwnNavigation]);
    assert_eq!(parse("own"), vec![Task::ResumeOwnNavigation]);
  }

  #[test]
  fn parse_speed() {
    assert_eq!(parse("s 250"), vec![Task::Speed(250.0)]);
    assert_eq!(parse("speed 250"), vec![Task::Speed(250.0)]);
  }

  #[test]
  fn parse_taxi() {}

  #[test]
  fn parse_taxi_continue() {
    assert_eq!(parse("tc"), vec![Task::TaxiContinue]);
    assert_eq!(parse("c"), vec![Task::TaxiContinue]);
  }

  #[test]
  fn parse_taxi_hold() {
    assert_eq!(parse("th"), vec![Task::TaxiHold]);
  }

  #[test]
  fn parse_takeoff() {
    assert_eq!(
      parse("takeoff 27L"),
      vec![Task::Takeoff(Intern::from_ref("27L"))]
    );
    assert_eq!(
      parse("takeoff 27l"),
      vec![Task::Takeoff(Intern::from_ref("27L"))]
    );
  }

  #[test]
  fn parse_line_up() {
    assert_eq!(
      parse("line 27L"),
      vec![Task::LineUp(Intern::from_ref("27L"))]
    );
    assert_eq!(
      parse("line 27l"),
      vec![Task::LineUp(Intern::from_ref("27L"))]
    );
  }

  #[test]
  fn parse_delete() {
    assert_eq!(parse("delete"), vec![Task::Delete]);
    assert_eq!(parse("del"), vec![Task::Delete]);
  }
}
