use std::{fs, path::PathBuf};

use async_openai::error::OpenAIError;
use engine::objects::command::{Command, Task, Tasks};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::send_chatgpt_request;

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
  Deserialize(#[from] serde_json::Error),
  #[error("failed to load file: {0}")]
  FS(#[from] std::io::Error),
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
  pub tasks: Vec<Task>,
}

#[derive(Error, Debug)]
pub enum Error {
  #[error("failed to deserialize: {0}")]
  Deserialize(#[from] serde_json::Error),
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
    let prompt = fs::read_to_string(path)?;
    let object: PromptObject =
      serde_json::from_str(&prompt).map_err(LoadPromptError::Deserialize)?;
    let mut full_prompt: Vec<String> = object.prompt;

    for path in object.imports {
      let lines = Self::load_prompt(path.into())?;
      full_prompt.extend_from_slice(&lines);
    }

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
      let json: CallsignAndRequest =
        serde_json::from_str(&result).map_err(LoadPromptError::Deserialize)?;

      Ok(json)
    } else {
      Err(Error::NoResult(prompt))
    }
  }

  async fn classify_request(request: String) -> Result<Vec<TypeValue>, Error> {
    let prompt =
      Self::load_prompt_as_string("server/prompts/classifier.json".into())?;
    let result = send_chatgpt_request(prompt.clone(), request).await?;
    if let Some(result) = result {
      let json: Vec<TypeValue> =
        serde_json::from_str(&result).map_err(LoadPromptError::Deserialize)?;

      Ok(json)
    } else {
      Err(Error::NoResult(prompt))
    }
  }

  async fn parse_task(raw_command: TypeValue) -> Result<Tasks, Error> {
    let prompt = Self::load_prompt_as_string(
      format!("server/prompts/tasks/{}.json", raw_command.command).into(),
    )?
    .replace("{{type}}", &raw_command.command);
    let result =
      send_chatgpt_request(prompt.clone(), raw_command.value).await?;
    if let Some(result) = result {
      let json: Tasks =
        serde_json::from_str(&result).map_err(LoadPromptError::Deserialize)?;

      Ok(json)
    } else {
      Err(Error::NoResult(prompt))
    }
  }

  async fn parse_tasks(raw_commands: Vec<TypeValue>) -> Result<Tasks, Error> {
    let mut tasks: Tasks = Vec::new();
    for raw_command in raw_commands {
      let task_chunk = Self::parse_task(raw_command).await?;
      tasks.extend_from_slice(&task_chunk);
    }

    Ok(tasks)
  }

  pub async fn execute(&self) -> Result<Command, Error> {
    let split = self.split_message().await?;
    let raw_commands = Self::classify_request(split.request).await?;
    let tasks = Self::parse_tasks(raw_commands).await?;

    let command = Command {
      id: split.callsign,
      reply: "No reply.".into(),
      tasks,
    };

    tracing::debug!("execution result: {:?}", command);

    Ok(command)
  }
}
