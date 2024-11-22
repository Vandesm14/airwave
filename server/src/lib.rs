use core::net::SocketAddr;
use std::{
  path::PathBuf,
  sync::{Arc, LazyLock},
  time::SystemTime,
};

use async_openai::{
  error::OpenAIError,
  types::{
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
    ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageContent,
    CreateChatCompletionRequest,
  },
};
use clap::Parser;
use futures_util::{
  stream::{SplitSink, SplitStream},
  SinkExt as _, StreamExt as _,
};
use reqwest::{header, multipart::Part, Client};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_tungstenite::{
  tungstenite::{self, Message},
  WebSocketStream,
};

use engine::{
  command::{CommandWithFreq, OutgoingCommandReply},
  engine::UICommand,
  NAUTICALMILES_TO_FEET,
};

use prompter::Prompter;
use runner::{IncomingUpdate, OutgoingReply};

pub const MANUAL_TOWER_AIRSPACE_RADIUS: f32 = NAUTICALMILES_TO_FEET * 30.0;
pub const AUTO_TOWER_AIRSPACE_RADIUS: f32 = NAUTICALMILES_TO_FEET * 30.0;
pub const TOWER_AIRSPACE_PADDING_RADIUS: f32 = NAUTICALMILES_TO_FEET * 20.0;
pub const WORLD_RADIUS: f32 = NAUTICALMILES_TO_FEET * 500.0;

pub mod airport;
pub mod config;
pub mod prompter;
pub mod runner;

pub static CLI: LazyLock<Cli> = LazyLock::new(Cli::parse);

#[derive(Parser)]
pub struct Cli {
  /// The socket address to bind the WebSocket server to.
  #[arg(short, long, default_value = None)]
  pub address: Option<SocketAddr>,

  /// The seed to use for the random number generator.
  #[arg(short, long)]
  pub seed: Option<u64>,

  /// Whether to and where to record incomming audio to.
  #[arg(long, default_value = None)]
  pub audio_path: Option<PathBuf>,

  /// The path to the config file.
  #[arg(short, long, default_value = None)]
  pub config_path: Option<PathBuf>,
}

pub async fn broadcast_updates_to(
  mut writer: SplitSink<WebSocketStream<TcpStream>, Message>,
  mut update_rx: async_broadcast::Receiver<OutgoingReply>,
) {
  loop {
    let update = match update_rx.recv().await {
      Ok(update) => update,
      Err(async_broadcast::RecvError::Overflowed(_)) => continue,
      Err(async_broadcast::RecvError::Closed) => break,
    };

    let ser = match serde_json::to_string(&update) {
      Ok(ser) => ser,
      Err(e) => {
        tracing::error!("Unable to serialise update: {e}");
        continue;
      }
    };

    if let Err(e) = writer.send(Message::Text(ser)).await {
      match e {
        tungstenite::Error::ConnectionClosed => break,
        tungstenite::Error::AlreadyClosed
        | tungstenite::Error::AttackAttempt => {
          tracing::error!("Unable to send update: {e}");
          break;
        }
        e => {
          tracing::error!("Unable to send update: {e}");
        }
      }
    }

    tracing::trace!("Sent update");
  }
}

pub async fn receive_commands_from(
  openai_api_key: Arc<str>,
  reader: SplitStream<WebSocketStream<TcpStream>>,
  update_tx: async_broadcast::Sender<OutgoingReply>,
  command_tx: async_channel::Sender<IncomingUpdate>,
) {
  reader
    .for_each(|message| {
      let openai_api_key = openai_api_key.clone();
      let update_tx = update_tx.clone();
      let command_tx = command_tx.clone();

      async move {
        let message = match message {
          Ok(message) => message,
          Err(e) => {
            tracing::error!("Unable to receive command: {e}");
            return;
          }
        };

        if let Message::Text(text) = message {
          let req: FrontendRequest = match serde_json::from_str(&text) {
            Ok(req) => req,
            Err(e) => {
              tracing::error!("Received malformed command: {e}");
              return;
            }
          };

          tracing::debug!("Received command message: length {}", text.len());

          match req {
            FrontendRequest::UI(ui_command) => {
              command_tx
                .send(IncomingUpdate::UICommand(ui_command))
                .await
                .unwrap();
            }
            FrontendRequest::Voice {
              data: bytes,
              frequency,
            } => {
              tracing::info!(
                "Received transcription request: {} bytes",
                bytes.len()
              );

              let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap();

              if let Some(ref audio_path) = CLI.audio_path {
                let mut audio_path = audio_path.join(format!("{now:?}"));
                audio_path.set_extension("wav");

                match std::fs::write(audio_path, bytes.clone()) {
                  Ok(_) => tracing::debug!("Wrote audio to file"),
                  Err(e) => tracing::error!("Unable to write path: {e}"),
                }
              }

              let client = Client::new();
              let form = reqwest::multipart::Form::new();
              let form =
                form.part("file", Part::bytes(bytes).file_name("audio.wav"));
              let form = form.text("model", "whisper-1".to_string());

              let response = client
                .post("https://api.openai.com/v1/audio/transcriptions")
                .multipart(form)
                .header(
                  header::AUTHORIZATION,
                  header::HeaderValue::from_str(&format!(
                    "Bearer {}",
                    &openai_api_key
                  ))
                  .unwrap(),
                )
                .header(
                  header::CONTENT_TYPE,
                  header::HeaderValue::from_str("multipart/form-data").unwrap(),
                )
                .send()
                .await
                .unwrap();

              let text = response.text().await.unwrap();
              tracing::info!("Transcribed request: {} chars", text.len());
              if let Ok(reply) = serde_json::from_str::<AudioResponse>(&text) {
                update_tx
                  .broadcast(OutgoingReply::ATCReply(OutgoingCommandReply {
                    id: "ATC".to_owned(),
                    frequency,
                    reply: reply.text.clone(),
                  }))
                  .await
                  .unwrap();

                if let Some(result) =
                  complete_atc_request(reply.text, frequency).await
                {
                  if let Some(ref audio_path) = CLI.audio_path {
                    let mut audio_path = audio_path.join(format!("{now:?}"));
                    audio_path.set_extension("json");

                    match std::fs::OpenOptions::new()
                      .create_new(true)
                      .write(true)
                      .open(audio_path)
                    {
                      Ok(file) => match serde_json::to_writer(file, &result) {
                        Ok(()) => {
                          tracing::debug!("Wrote associated audio command file")
                        }
                        Err(e) => tracing::warn!(
                          "Unable to write associated audio command file: {e}"
                        ),
                      },
                      Err(e) => tracing::warn!(
                        "Unable to create associated audio command file: {e}"
                      ),
                    }
                  }

                  command_tx
                    .send(IncomingUpdate::Command(result))
                    .await
                    .unwrap();
                }
              }
            }
            FrontendRequest::Text {
              text: string,
              frequency,
            } => {
              update_tx
                .broadcast(OutgoingReply::ATCReply(OutgoingCommandReply {
                  id: "ATC".to_owned(),
                  frequency,
                  reply: string.clone(),
                }))
                .await
                .unwrap();

              if let Some(result) =
                complete_atc_request(string, frequency).await
              {
                command_tx
                  .send(IncomingUpdate::Command(result))
                  .await
                  .unwrap();
              }
            }
            FrontendRequest::Connect => {
              command_tx.send(IncomingUpdate::Connect).await.unwrap();
            }
          }
        } else {
          tracing::debug!("Skipping non-text WebSocket message")
        }
      }
    })
    .await;
}

async fn send_chatgpt_request(
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

async fn complete_atc_request(
  message: String,
  frequency: f32,
) -> Option<CommandWithFreq> {
  let prompter = Prompter::new(message);
  let result = prompter.execute().await;
  match result {
    Ok(command) => Some(CommandWithFreq {
      id: command.id,
      frequency,
      reply: command.reply,
      tasks: command.tasks,
    }),
    Err(err) => {
      tracing::error!("Unable to parse command: {}", err);
      None
    }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type", content = "value")]
enum FrontendRequest {
  Voice { data: Vec<u8>, frequency: f32 },
  Text { text: String, frequency: f32 },
  UI(UICommand),
  Connect,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct AudioResponse {
  text: String,
}
