use std::process::{Command, Stdio};

use engine::command::CommandWithFreq;

use crate::ring::RingBuffer;

#[derive(Debug, Clone, PartialEq)]
pub struct Messages {
  messages: RingBuffer<CommandWithFreq>,
  auto_generate: bool,
}

impl Default for Messages {
  fn default() -> Self {
    Self::new(30, false)
  }
}

impl Messages {
  pub fn new(capacity: usize, auto_generate: bool) -> Self {
    Self {
      messages: RingBuffer::new(capacity),
      auto_generate,
    }
  }

  pub fn add(&mut self, message: CommandWithFreq) {
    if self.auto_generate {
      self.generate(&message);
    }
    self.messages.push(message);
  }

  pub fn generate(&self, message: &CommandWithFreq) {
    // Run `echo "message" | echo '{message.text}' | piper --model models/en_GB-vctk-medium.onnx --output_file {message.duration.seconds}.ogg`
    let mut echo = Command::new("echo")
      .arg(message.to_string())
      .stdout(Stdio::piped())
      .spawn()
      .unwrap();

    let echo_out = echo.stdout.take().unwrap();

    let _ = Command::new("piper")
      .arg("--model")
      .arg("models/en_GB-vctk-medium.onnx")
      .arg("--output_file")
      .arg(format!("static/replies/{}.ogg", message.created.as_secs()))
      .stdin(echo_out)
      .spawn()
      .unwrap()
      .wait();

    let _ = echo.wait();
  }

  pub fn iter(&self) -> impl Iterator<Item = &CommandWithFreq> {
    self.messages.iter()
  }
}

impl Extend<CommandWithFreq> for Messages {
  fn extend<T: IntoIterator<Item = CommandWithFreq>>(&mut self, iter: T) {
    for message in iter {
      self.add(message);
    }
  }
}
