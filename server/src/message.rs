use std::{
  collections::HashMap,
  process::{Command, Stdio},
};

use engine::command::CommandWithFreq;
use internment::Intern;
use turborand::{rng::Rng, TurboRand};

use crate::ring::RingBuffer;

#[derive(Debug, Clone, PartialEq)]
pub struct Messages {
  messages: RingBuffer<CommandWithFreq>,
  auto_generate: bool,

  aircraft_voices: HashMap<Intern<String>, Intern<String>>,
  available_voices: Vec<Intern<String>>,
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

      aircraft_voices: HashMap::new(),
      available_voices: Vec::new(),
    }
  }

  pub fn set_auto_generate(&mut self, auto_generate: bool) {
    self.auto_generate = auto_generate;
  }

  pub fn set_available_voices(&mut self, voices: Vec<Intern<String>>) {
    self.available_voices = voices;
  }

  pub fn add(&mut self, message: CommandWithFreq) {
    if self.auto_generate {
      self.generate(&message);
    }
    self.messages.push(message);
  }

  pub fn generate(&mut self, message: &CommandWithFreq) {
    let voice = if let Some(voice) =
      self.aircraft_voices.get(&Intern::from_ref(&message.id))
    {
      voice
    } else {
      let rng = Rng::new();
      let voice = rng.sample(&self.available_voices).unwrap();
      self
        .aircraft_voices
        .insert(Intern::from_ref(&message.id), *voice);

      voice
    };

    // Run `echo "message" | echo '{message.text}' | piper --model models/en_GB-vctk-medium.onnx --output_file {message.duration.seconds}.ogg`
    let mut echo = Command::new("echo")
      .arg(message.to_string())
      .stdout(Stdio::piped())
      .spawn()
      .unwrap();

    let echo_out = echo.stdout.take().unwrap();

    let _ = Command::new("piper")
      .arg("--model")
      .arg(format!("{}", voice))
      .arg("--output_file")
      .arg(format!("static/replies/{}.ogg", message.created.as_secs()))
      .stdin(echo_out)
      .stdout(Stdio::null())
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
