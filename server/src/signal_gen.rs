use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SignalGenerator {
  rate: Duration,
  last_tick: Instant,
}

impl SignalGenerator {
  pub fn new(rate: Duration) -> Self {
    Self {
      rate,
      last_tick: Instant::now(),
    }
  }

  pub fn tick(&mut self) -> bool {
    let now = Instant::now();
    if now - self.last_tick >= self.rate {
      self.last_tick = now;

      true
    } else {
      false
    }
  }
}
