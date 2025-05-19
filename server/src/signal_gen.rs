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
      self.last_tick = now;
      return true;
    }

    if now - self.last_tick >= self.rate {
      self.last_tick = now;

      true
    } else {
      false
    }
  }

  pub fn set_first(&mut self) {
    self.first = true;
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn always_first() {
    let mut signal_gen = SignalGenerator::new(2);
    assert!(signal_gen.tick(0));
    assert!(!signal_gen.tick(1));
    assert!(signal_gen.tick(2));
    assert!(!signal_gen.tick(3));
    assert!(signal_gen.tick(4));
  }

  #[test]
  fn reset_first() {
    let mut signal_gen = SignalGenerator::new(3);
    assert!(signal_gen.tick(0));
    assert!(!signal_gen.tick(1));
    assert!(!signal_gen.tick(2));
    assert!(signal_gen.tick(3));
    signal_gen.set_first();
    assert!(signal_gen.tick(4));
    assert!(!signal_gen.tick(5));
    assert!(!signal_gen.tick(6));
    assert!(signal_gen.tick(7));
  }
}
