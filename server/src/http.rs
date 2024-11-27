use std::net::SocketAddr;

use async_openai::{
  error::OpenAIError,
  types::{
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
    ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageContent,
    CreateChatCompletionRequest,
  },
};
use axum::{
  extract::State,
  routing::{get, post},
  Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use engine::{command::CommandWithFreq, engine::UICommand};

use crate::{
  job::{JobReq, JobReqKind, JobResKind},
  prompter::Prompter,
};

#[derive(Debug, Clone)]
pub struct AppState {
  pub sender: mpsc::UnboundedSender<JobReq>,
}

impl AppState {
  pub fn new(sender: mpsc::UnboundedSender<JobReq>) -> Self {
    Self { sender }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct CommsText {
  text: String,
  frequency: f32,
}
async fn comms_text(
  State(mut state): State<AppState>,
  Json(payload): Json<CommsText>,
) -> Result<(), String> {
  let command = complete_atc_request(payload.text, payload.frequency).await;
  if let Some(command) = command {
    let x = JobReq::send(JobReqKind::Command(command), &mut state.sender)
      .try_recv()
      .await;

    Ok(())
  } else {
    Err("Failed to parse ATC message.".to_string())
  }
}

async fn get_messages(State(mut state): State<AppState>) -> String {
  let res = JobReq::send(JobReqKind::Messages, &mut state.sender)
    .try_recv()
    .await;
  if let Ok(JobResKind::Messages(messages)) = res {
    if let Ok(string) = serde_json::to_string(&messages) {
      string
    } else {
      todo!("failed to serialize")
    }
  } else {
    todo!("failed to get messages: {res:?}")
  }
}

async fn ping_pong(State(mut state): State<AppState>) -> String {
  let res = JobReq::send(JobReqKind::Ping, &mut state.sender)
    .try_recv()
    .await;

  if let Ok(JobResKind::Pong) = res {
    "Pong".to_string()
  } else {
    todo!("failed to ping: {res:?}")
  }
}

pub async fn run(address: SocketAddr, sender: mpsc::UnboundedSender<JobReq>) {
  let app = Router::new()
    .route("/", get(|| async { "Hello, World!" }))
    .route("/comms/text", post(comms_text))
    .route("/messages", get(get_messages))
    .route("/ping", get(ping_pong))
    .with_state(AppState::new(sender));

  let listener = tokio::net::TcpListener::bind(address).await.unwrap();
  tracing::info!("Listening on {address}");
  axum::serve(listener, app).await.unwrap();
}

// pub async fn broadcast_updates_to(
//   mut writer: SplitSink<WebSocketStream<TcpStream>, Message>,
//   mut update_rx: async_broadcast::Receiver<OutgoingReply>,
// ) {
//   loop {
//     let update = match update_rx.recv().await {
//       Ok(update) => update,
//       Err(async_broadcast::RecvError::Overflowed(_)) => continue,
//       Err(async_broadcast::RecvError::Closed) => break,
//     };

//     let ser = match serde_json::to_string(&update) {
//       Ok(ser) => ser,
//       Err(e) => {
//         tracing::error!("Unable to serialise update: {e}");
//         continue;
//       }
//     };

//     if let Err(e) = writer.send(Message::Text(ser)).await {
//       match e {
//         tungstenite::Error::ConnectionClosed => break,
//         tungstenite::Error::AlreadyClosed
//         | tungstenite::Error::AttackAttempt => {
//           tracing::error!("Unable to send update: {e}");
//           break;
//         }
//         e => {
//           tracing::error!("Unable to send update: {e}");
//         }
//       }
//     }

//     tracing::trace!("Sent update");
//   }
// }

// pub async fn receive_commands_from(
//   openai_api_key: Arc<str>,
//   reader: SplitStream<WebSocketStream<TcpStream>>,
//   update_tx: async_broadcast::Sender<OutgoingReply>,
//   command_tx: async_channel::Sender<IncomingUpdate>,
// ) {
//   reader
//     .for_each(|message| {
//       let openai_api_key = openai_api_key.clone();
//       let update_tx = update_tx.clone();
//       let command_tx = command_tx.clone();

//       async move {
//         let message = match message {
//           Ok(message) => message,
//           Err(e) => {
//             tracing::error!("Unable to receive command: {e}");
//             return;
//           }
//         };

//         if let Message::Text(text) = message {
//           let req: FrontendRequest = match serde_json::from_str(&text) {
//             Ok(req) => req,
//             Err(e) => {
//               tracing::error!("Received malformed command: {e}");
//               return;
//             }
//           };

//           tracing::debug!("Received command message: length {}", text.len());

//           match req {
//             FrontendRequest::UI(ui_command) => {
//               command_tx
//                 .send(IncomingUpdate::UICommand(ui_command))
//                 .await
//                 .unwrap();
//             }
//             FrontendRequest::Voice {
//               data: bytes,
//               frequency,
//             } => {
//               tracing::info!(
//                 "Received transcription request: {} bytes",
//                 bytes.len()
//               );

//               let now = SystemTime::now()
//                 .duration_since(SystemTime::UNIX_EPOCH)
//                 .unwrap();

//               if let Some(ref audio_path) = CLI.audio_path {
//                 let mut audio_path = audio_path.join(format!("{now:?}"));
//                 audio_path.set_extension("wav");

//                 match std::fs::write(audio_path, bytes.clone()) {
//                   Ok(_) => tracing::debug!("Wrote audio to file"),
//                   Err(e) => tracing::error!("Unable to write path: {e}"),
//                 }
//               }

//               let client = Client::new();
//               let form = reqwest::multipart::Form::new();
//               let form =
//                 form.part("file", Part::bytes(bytes).file_name("audio.wav"));
//               let form = form.text("model", "whisper-1".to_string());

//               let response = client
//                 .post("https://api.openai.com/v1/audio/transcriptions")
//                 .multipart(form)
//                 .header(
//                   header::AUTHORIZATION,
//                   header::HeaderValue::from_str(&format!(
//                     "Bearer {}",
//                     &openai_api_key
//                   ))
//                   .unwrap(),
//                 )
//                 .header(
//                   header::CONTENT_TYPE,
//                   header::HeaderValue::from_str("multipart/form-data").unwrap(),
//                 )
//                 .send()
//                 .await
//                 .unwrap();

//               let text = response.text().await.unwrap();
//               tracing::info!("Transcribed request: {} chars", text.len());
//               if let Ok(reply) = serde_json::from_str::<AudioResponse>(&text) {
//                 update_tx
//                   .broadcast(OutgoingReply::ATCReply(OutgoingCommandReply {
//                     id: "ATC".to_owned(),
//                     frequency,
//                     reply: reply.text.clone(),
//                   }))
//                   .await
//                   .unwrap();

//                 if let Some(result) =
//                   complete_atc_request(reply.text, frequency).await
//                 {
//                   if let Some(ref audio_path) = CLI.audio_path {
//                     let mut audio_path = audio_path.join(format!("{now:?}"));
//                     audio_path.set_extension("json");

//                     match std::fs::OpenOptions::new()
//                       .create_new(true)
//                       .write(true)
//                       .open(audio_path)
//                     {
//                       Ok(file) => match serde_json::to_writer(file, &result) {
//                         Ok(()) => {
//                           tracing::debug!("Wrote associated audio command file")
//                         }
//                         Err(e) => tracing::warn!(
//                           "Unable to write associated audio command file: {e}"
//                         ),
//                       },
//                       Err(e) => tracing::warn!(
//                         "Unable to create associated audio command file: {e}"
//                       ),
//                     }
//                   }

//                   command_tx
//                     .send(IncomingUpdate::Command(result))
//                     .await
//                     .unwrap();
//                 }
//               }
//             }
//             FrontendRequest::Text {
//               text: string,
//               frequency,
//             } => {
//               update_tx
//                 .broadcast(OutgoingReply::ATCReply(OutgoingCommandReply {
//                   id: "ATC".to_owned(),
//                   frequency,
//                   reply: string.clone(),
//                 }))
//                 .await
//                 .unwrap();

//               if let Some(result) =
//                 complete_atc_request(string, frequency).await
//               {
//                 command_tx
//                   .send(IncomingUpdate::Command(result))
//                   .await
//                   .unwrap();
//               }
//             }
//             FrontendRequest::Connect => {
//               command_tx.send(IncomingUpdate::Connect).await.unwrap();
//             }
//           }
//         } else {
//           tracing::debug!("Skipping non-text WebSocket message")
//         }
//       }
//     })
//     .await;
// }

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
