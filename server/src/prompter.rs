use std::{fs, path::PathBuf};

use async_openai::{
  error::OpenAIError,
  types::{
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
    ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageContent,
    CreateChatCompletionRequest,
  },
};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use thiserror::Error;

use engine::{
  command::Tasks,
  entities::aircraft::{Aircraft, AircraftState},
};

pub async fn send_chatgpt_request(
  prompt: String,
  message: String,
) -> Result<Option<String>, OpenAIError> {
  let client = async_openai::Client::new();
  let request = CreateChatCompletionRequest {
    messages: vec![
      ChatCompletionRequestMessage::System(
        ChatCompletionRequestSystemMessage {
          content: prompt.clone(),
          name: None,
        },
      ),
      ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
        content: ChatCompletionRequestUserMessageContent::Text(message.clone()),
        name: None,
      }),
    ],
    model: "gpt-4o-mini".into(),
    ..Default::default()
  };

  let response = client.chat().create(request).await;
  match response {
    Ok(response) => Ok(response.choices.first().and_then(|c| {
      let c = c.message.content.clone();
      tracing::debug!(
        "**sent prompt:**\n{prompt}\n\n**message:**\n{message}\n\n**response:**\n{c:?}",
      );
      c
    })),
    Err(err) => Err(err),
  }
}

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
pub struct Prompter;

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

  pub async fn split_request(
    message: String,
  ) -> Result<Vec<CallsignAndRequest>, Error> {
    let prompt =
      Self::load_prompt_as_string("server/prompts/splitter.json".into())?;
    let result = send_chatgpt_request(prompt.clone(), message).await?;
    if let Some(result) = result {
      let json: Vec<CallsignAndRequest> = serde_json::from_str(&result)
        .map_err(|e| LoadPromptError::Deserialize(e, result))?;

      Ok(json)
    } else {
      Err(Error::NoResult(prompt))
    }
  }

  pub async fn generate_readback(message: String) -> Result<String, Error> {
    let prompt =
      Self::load_prompt_as_string("server/prompts/readback.json".into())?;
    let result = send_chatgpt_request(prompt.clone(), message).await?;
    if let Some(result) = result {
      Ok(result)
    } else {
      Err(Error::NoResult(prompt))
    }
  }

  pub async fn parse_into_tasks(
    split: CallsignAndRequest,
    aircraft: &Aircraft,
  ) -> Result<Tasks, Error> {
    let mode = if matches!(
      aircraft.state,
      AircraftState::Flying | AircraftState::Landing { .. }
    ) {
      "air"
    } else if matches!(
      aircraft.state,
      AircraftState::Taxiing { .. } | AircraftState::Parked { .. }
    ) {
      "ground"
    } else {
      return Err(Error::NoResult("Unknown aircraft state".into()));
    };

    let path = format!("server/prompts/{mode}.json");
    let prompt = Self::load_prompt_as_string(path.clone().into())?;

    tracing::info!("using prompt: {} for {}", path, aircraft.id);

    let result =
      send_chatgpt_request(prompt.clone(), split.request.clone()).await?;
    if let Some(result) = result {
      let json: Tasks = serde_json::from_str(&result)
        .map_err(|e| LoadPromptError::Deserialize(e, result))?;

      tracing::info!("prompt result ({}): {:?}", aircraft.id, json.clone());

      Ok(json.clone())
    } else {
      tracing::error!("no prompt result for: {}", aircraft.id);
      Err(Error::NoResult(prompt))
    }
  }

  /// This is a debug function to dump the prompts into a file for each mode.
  pub fn export_prompts() {
    use std::path::Path;

    let prompt = Prompter::load_prompt_as_string(
      Path::new("server/prompts/air.json").into(),
    )
    .unwrap();

    std::fs::write("prompt.air.txt", prompt).unwrap();

    let prompt = Prompter::load_prompt_as_string(
      Path::new("server/prompts/ground.json").into(),
    )
    .unwrap();

    std::fs::write("prompt.ground.txt", prompt).unwrap();
  }
}
