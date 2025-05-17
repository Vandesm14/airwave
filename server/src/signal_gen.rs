#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SignalGenerator {
  rate: usize,
  last_tick: usize,

  first: bool,
}

impl SignalGenerator {
  pub fn new(rate: usize) -> Self {
    Self {
      rate,
      last_tick: 0,
      first: true,
    }
  }

  pub fn tick(&mut self, now: usize) -> bool {
    if self.first {
      self.first = false;
      return true;
    }

    if now - self.last_tick >= self.rate {
      self.last_tick = now;

      true
    } else {
      false
    }
  }

  pub fn reset(&mut self) {
    self.last_tick = 0;
    self.first = true;
  }
}
