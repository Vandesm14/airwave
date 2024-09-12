use std::{fs, path::PathBuf};

use async_openai::error::OpenAIError;
use engine::objects::command::Task;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::send_chatgpt_request;

fn deserialize_vec_of_strings<'de, D>(
  deserializer: D,
) -> Result<Vec<String>, D::Error>
where
  D: Deserializer<'de>,
{
  #[derive(Debug, Clone, Default, Serialize, Deserialize)]
  struct VecString(Vec<Value>);

  let vec_string = VecString::deserialize(deserializer)?;
  let vec: Vec<String> = vec_string.0.iter().map(|v| v.to_string()).collect();

  Ok(vec)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromptObject {
  #[serde(deserialize_with = "deserialize_vec_of_strings")]
  pub prompt: Vec<String>,
}

#[derive(Error, Debug)]
pub enum LoadPromptError {
  #[error("failed to serialize: {0}")]
  Serialize(#[from] serde_json::Error),
  #[error("failed to load file: {0}")]
  FS(#[from] std::io::Error),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CallsignAndRequest {
  pub callsign: String,
  pub request: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Prompter {
  pub message: String,
  pub tasks: Vec<Task>,
}

#[derive(Error, Debug)]
pub enum Error {
  #[error("{0}")]
  LoadPromptError(#[from] LoadPromptError),
  #[error("error from OpenAI: {0}")]
  OpenAI(#[from] OpenAIError),
  #[error("failed to complete prompt: {0}")]
  NoResult(String),
}

impl Prompter {
  pub fn new(message: String) -> Self {
    Self {
      message,
      ..Default::default()
    }
  }

  fn load_prompt(path: PathBuf) -> Result<String, LoadPromptError> {
    let prompt = fs::read_to_string(path)?;
    let object: PromptObject =
      serde_json::from_str(&prompt).map_err(LoadPromptError::Serialize)?;

    Ok(object.prompt.join("\n"))
  }

  async fn split_message(&self) -> Result<CallsignAndRequest, Error> {
    let prompt = Self::load_prompt("server/prompts/splitter.json".into())?;
    let result =
      send_chatgpt_request(prompt.clone(), self.message.clone()).await?;
    if let Some(result) = result {
      panic!("result: {}", result);
    } else {
      Err(Error::NoResult(prompt))
    }
  }

  pub async fn execute(&self) -> Result<(), Error> {
    self.split_message().await?;

    Ok(())
  }
}
