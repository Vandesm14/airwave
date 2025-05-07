use std::collections::{VecDeque, vec_deque};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RingBuffer<T> {
  capacity: usize,
  vec: VecDeque<T>,
}

impl<T> Extend<T> for RingBuffer<T> {
  fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
    self.vec.extend(iter);
  }
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
