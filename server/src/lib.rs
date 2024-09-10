use std::sync::Arc;

use async_openai::types::{
  ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
  ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageContent,
  CreateChatCompletionRequest,
};
use engine::{
  engine::{IncomingUpdate, OutgoingReply},
  objects::command::{
    Command, CommandReply, CommandReplyKind, CommandWithFreq,
  },
};
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

pub mod airport;

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
            FrontendRequest::Voice {
              data: bytes,
              frequency,
            } => {
              tracing::info!(
                "Received transcription request: {} bytes",
                bytes.len()
              );

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
              if let Ok(reply) = serde_json::from_str::<AudioResponse>(&text) {
                update_tx
                  .broadcast(OutgoingReply::ATCReply(CommandWithFreq {
                    id: "ATC".to_owned(),
                    frequency,
                    reply: reply.text.clone(),
                    tasks: Vec::new(),
                  }))
                  .await
                  .unwrap();

                if let Some(result) =
                  complete_atc_request(reply.text, frequency).await
                {
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
                .broadcast(OutgoingReply::ATCReply(CommandWithFreq {
                  id: "ATC".to_owned(),
                  frequency,
                  reply: string.clone(),
                  tasks: Vec::new(),
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

// pub async fn speech_to_text() {}

async fn complete_atc_request(
  string: String,
  freq: f32,
) -> Option<CommandWithFreq> {
  let client = async_openai::Client::new();
  let request = CreateChatCompletionRequest {
    messages: vec![
      ChatCompletionRequestMessage::System(
        ChatCompletionRequestSystemMessage {
          content: include_str!("prompt.txt").to_owned(),
          name: None,
        },
      ),
      ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
        content: ChatCompletionRequestUserMessageContent::Text(string),
        name: None,
      }),
    ],
    model: "gpt-4o-mini".into(),
    ..Default::default()
  };

  let response = client.chat().create(request).await;
  if let Ok(response) = response {
    if let Some(choice) = response.choices.first() {
      if let Some(ref text) = choice.message.content {
        match serde_json::from_str::<Command>(text) {
          Ok(reply) => {
            return Some(CommandWithFreq {
              id: reply.id.clone(),
              frequency: freq,
              reply: CommandReply {
                callsign: reply.id,
                kind: CommandReplyKind::WithCallsign { text: reply.reply },
              }
              .to_string(),
              tasks: reply.tasks,
            })
          }
          Err(e) => {
            tracing::error!("Unable to parse command: {} (raw: {})", e, text);
          }
        }
      }
    }
  }

  None
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type", content = "value")]
enum FrontendRequest {
  Voice { data: Vec<u8>, frequency: f32 },
  Text { text: String, frequency: f32 },
  Connect,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct AudioResponse {
  text: String,
}
