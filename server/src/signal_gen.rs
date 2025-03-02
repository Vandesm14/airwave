use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SignalGenerator {
  rate: Duration,
  last_tick: Instant,

  first: bool,
}

impl SignalGenerator {
  pub fn new(rate: Duration) -> Self {
    Self {
      rate,
      last_tick: Instant::now(),

      first: true,
    }
  }

  pub fn tick(&mut self) -> bool {
    if self.first {
      self.first = false;
      return true;
    }

    let now = Instant::now();
    if now - self.last_tick >= self.rate {
      self.last_tick = now;

      true
    } else {
      false
    }
  }
}
