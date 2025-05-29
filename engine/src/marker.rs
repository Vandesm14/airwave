use std::{
  collections::HashMap,
  time::{Duration, Instant},
};

#[derive(Debug, Clone, Copy)]
pub enum Mark<T>
where
  T: std::hash::Hash + Clone + std::fmt::Debug + Eq,
{
  Start(T),
  End(T),
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

    let summary = marker.summarize_marks();
    let task1 = summary.get("task1").unwrap();
    let average = task1.iter().sum::<Duration>() / task1.len() as u32;

    assert_eq!(average, Duration::from_secs(2));
  }
}
