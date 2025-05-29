use std::{
  collections::HashMap,
  time::{Duration, Instant},
};

use itertools::Itertools;

#[derive(Debug, Clone, Copy)]
pub enum Mark<T>
where
  T: std::hash::Hash + Clone + std::fmt::Debug + Eq,
{
  Start(T),
  End(T),
}

impl<T> Mark<T>
where
  T: std::hash::Hash + Clone + std::fmt::Debug + Eq,
{
  pub fn key(&self) -> &T {
    match self {
      Mark::Start(key) => key,
      Mark::End(key) => key,
    }
  }
}

#[derive(Debug, Clone, Default)]
pub struct Marker<T>
where
  T: std::hash::Hash + Clone + std::fmt::Debug + Eq,
{
  pub marks: Vec<Mark<T>>,
  pub times: Vec<Instant>,
}

impl<T> Marker<T>
where
  T: std::hash::Hash + Clone + std::fmt::Debug + Eq,
{
  pub fn new() -> Self {
    Self {
      marks: Vec::new(),
      times: Vec::new(),
    }
  }

  pub fn add_mark(&mut self, mark: Mark<T>, time: Instant) {
    self.marks.push(mark);
    self.times.push(time);
  }

  pub fn clear(&mut self) {
    self.marks.clear();
    self.times.clear();
  }

  pub fn summarize_marks(&self) -> HashMap<T, Vec<Duration>> {
    let mut times: HashMap<T, Vec<Duration>> = HashMap::new();
    let mut last: HashMap<T, Instant> = HashMap::new();
    for (mark, time) in self.marks.iter().zip(self.times.iter()) {
      match mark {
        Mark::Start(key) => {
          last.insert(key.clone(), *time);
        }
        Mark::End(key) => {
          if let Some(last_time) = last.get(key) {
            let diff = time.duration_since(*last_time);
            if let Some(vec) = times.get_mut(key) {
              vec.push(diff);
            } else {
              times.insert(key.clone(), vec![diff]);
            }
          }
        }
      }
    }

    times
  }

  pub fn average_duration(&self, key: &T) -> Option<Duration> {
    let durations = self.summarize_marks();
    let durations = durations.get(key)?;
    if durations.is_empty() {
      return None;
    }
    Some(durations.iter().sum::<Duration>() / durations.len() as u32)
  }

  pub fn min_duration(&self, key: &T) -> Option<Duration> {
    let durations = self.summarize_marks();
    let durations = durations.get(key)?;
    if durations.is_empty() {
      return None;
    }
    Some(*durations.iter().min()?)
  }

  pub fn max_duration(&self, key: &T) -> Option<Duration> {
    let durations = self.summarize_marks();
    let durations = durations.get(key)?;
    if durations.is_empty() {
      return None;
    }
    Some(*durations.iter().max()?)
  }

  pub fn total_duration(&self, key: &T) -> Option<Duration> {
    let durations = self.summarize_marks();
    let durations = durations.get(key)?;
    if durations.is_empty() {
      return None;
    }
    Some(durations.iter().sum())
  }

  pub fn keys(&self) -> impl Iterator<Item = &T> {
    self.marks.iter().map(|mark| mark.key()).dedup()
  }

  pub fn start(&mut self, key: T) -> T {
    let now = Instant::now();
    self.add_mark(Mark::Start(key.clone()), now);
    key
  }

  pub fn end(&mut self, key: T) {
    let now = Instant::now();
    self.add_mark(Mark::End(key), now);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_marker() {
    let mut marker = Marker::new();
    let start_time = Instant::now();

    marker.add_mark(Mark::Start("task1"), start_time);
    marker.add_mark(Mark::End("task1"), start_time + Duration::from_secs(2));

    marker.add_mark(Mark::Start("task2"), start_time + Duration::from_secs(3));
    marker.add_mark(Mark::End("task2"), start_time + Duration::from_secs(5));

    let summary = marker.summarize_marks();

    assert_eq!(summary.get("task1").unwrap().len(), 1);
    assert_eq!(summary.get("task2").unwrap().len(), 1);
    assert_eq!(summary.get("task1").unwrap()[0], Duration::from_secs(2));
    assert_eq!(summary.get("task2").unwrap()[0], Duration::from_secs(2));
  }

  #[test]
  fn test_average() {
    let mut marker = Marker::new();
    let next = Instant::now();

    marker.add_mark(Mark::Start("task1"), next);
    let next = next + Duration::from_secs(2);
    marker.add_mark(Mark::End("task1"), next);

    marker.add_mark(Mark::Start("task1"), next);
    let next = next + Duration::from_secs(2);
    marker.add_mark(Mark::End("task1"), next);

    marker.add_mark(Mark::Start("task1"), next);
    let next = next + Duration::from_secs(2);
    marker.add_mark(Mark::End("task1"), next);

    assert_eq!(
      marker.average_duration(&"task1"),
      Some(Duration::from_secs(2))
    );
  }
}
