use std::collections::{vec_deque, VecDeque};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RingBuffer<T> {
  capacity: usize,
  vec: VecDeque<T>,
}

impl<T> RingBuffer<T> {
  pub fn new(capacity: usize) -> Self {
    Self {
      capacity,
      vec: VecDeque::with_capacity(capacity),
    }
  }

  pub fn push(&mut self, value: T) {
    if self.vec.len() == self.capacity {
      self.vec.pop_front();
    }

    self.vec.push_back(value);
  }

  pub fn trim(&mut self) {
    while self.vec.len() > self.capacity {
      self.vec.pop_front();
    }
  }

  pub fn iter(&self) -> vec_deque::Iter<'_, T> {
    self.vec.iter()
  }
}
