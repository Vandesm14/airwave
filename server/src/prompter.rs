use std::{fs, path::PathBuf};

use async_openai::error::OpenAIError;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use thiserror::Error;

use engine::command::{Command, CommandReply, Tasks};

use crate::http::send_chatgpt_request;

fn deserialize_string_or_any<'de, D>(
  deserializer: D,
) -> Result<String, D::Error>
where
  D: Deserializer<'de>,
{
  #[derive(Debug, Clone, Default, Serialize, Deserialize)]
  struct AnyString(Value);

  let v = AnyString::deserialize(deserializer)?;
  let any_string = if v.0.is_string() {
    String::deserialize(v.0).unwrap()
  } else {
    v.0.to_string()
  };

  Ok(any_string)
}

fn deserialize_vec_of_strings<'de, D>(
  deserializer: D,
) -> Result<Vec<String>, D::Error>
where
  D: Deserializer<'de>,
{
  #[derive(Debug, Clone, Default, Serialize, Deserialize)]
  struct VecString(Vec<Value>);

  let vec_string = VecString::deserialize(deserializer)?;
  let vec: Vec<String> = vec_string
    .0
    .iter()
    .map(|v| {
      if v.is_string() {
        String::deserialize(v).unwrap()
      } else {
        v.to_string()
      }
    })
    .collect();

  Ok(vec)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Example {
  #[serde(deserialize_with = "deserialize_string_or_any")]
  pub user: String,
  #[serde(deserialize_with = "deserialize_string_or_any")]
  pub assistant: String,
}

impl core::fmt::Display for Example {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "User: {}\n Assistant:{}", self.user, self.assistant)?;

    Ok(())
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromptObject {
  #[serde(default)]
  pub imports: Vec<String>,
  #[serde(deserialize_with = "deserialize_vec_of_strings")]
  pub prompt: Vec<String>,
  #[serde(default)]
  pub examples: Vec<Example>,
}

#[derive(Error, Debug)]
pub enum LoadPromptError {
  #[error("failed to deserialize: {0}")]
  Deserialize(serde_json::Error, String),
  #[error("failed to load file: {0}")]
  FS(String),
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CallsignAndRequest {
  pub callsign: String,
  pub request: String,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TypeValue {
  command: String,
  value: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Prompter {
  pub message: String,
  pub tasks: Tasks,
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

  fn load_prompt(path: PathBuf) -> Result<Vec<String>, LoadPromptError> {
    let prompt = fs::read_to_string(path.clone())
      .map_err(|_| LoadPromptError::FS(path.to_str().unwrap().into()))?;
    let object: PromptObject = serde_json::from_str(&prompt)
      .map_err(|e| LoadPromptError::Deserialize(e, prompt))?;
    let mut full_prompt: Vec<String> = Vec::new();

    for path in object.imports {
      let lines = Self::load_prompt(path.into())?;
      full_prompt.extend(lines);
    }

    full_prompt.extend(object.prompt);
    full_prompt.extend(object.examples.iter().map(|e| e.to_string()));

    Ok(full_prompt)
  }

  fn load_prompt_as_string(path: PathBuf) -> Result<String, LoadPromptError> {
    let lines = Self::load_prompt(path)?;
    Ok(lines.join("\n"))
  }

  async fn split_message(&self) -> Result<CallsignAndRequest, Error> {
    let prompt =
      Self::load_prompt_as_string("server/prompts/splitter.json".into())?;
    let result =
      send_chatgpt_request(prompt.clone(), self.message.clone()).await?;
    if let Some(result) = result {
      let json: CallsignAndRequest = serde_json::from_str(&result)
        .map_err(|e| LoadPromptError::Deserialize(e, result))?;

      Ok(json)
    } else {
      Err(Error::NoResult(prompt))
    }
  }

  async fn parse_tasks(string: String) -> Result<Tasks, Error> {
    let prompt =
      Self::load_prompt_as_string("server/prompts/main.json".into())?;
    let result = send_chatgpt_request(prompt.clone(), string).await?;
    if let Some(result) = result {
      let json: Tasks = serde_json::from_str(&result)
        .map_err(|e| LoadPromptError::Deserialize(e, result))?;

      Ok(json)
    } else {
      Err(Error::NoResult(prompt))
    }
  }

  async fn execute_hidden(&self) -> Result<Command, Error> {
    let split = self.split_message().await?;
    let tasks = Self::parse_tasks(split.request.clone()).await?;

    let command = Command {
      id: split.callsign.clone(),
      reply: CommandReply::WithCallsign {
        text: split.request,
      },
      tasks,
    };

    Ok(command)
  }

  pub async fn execute(&self) -> Result<Command, Error> {
    let command = self.execute_hidden().await?;

    tracing::info!("prompt result: {:?}", command);

    Ok(command)
  }
}
