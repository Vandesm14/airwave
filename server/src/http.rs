use std::{net::SocketAddr, sync::Arc};

use async_openai::{
  error::OpenAIError,
  types::{
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
    ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageContent,
    CreateChatCompletionRequest,
  },
};
use axum::{
  body::Bytes,
  extract::{Query, State},
  http,
  routing::{get, post},
  Router,
};
use reqwest::{header, multipart::Part, Client};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use engine::{
  command::{CommandReply, CommandWithFreq},
  duration_now,
  engine::UICommand,
};
use tower_http::cors::{Any, CorsLayer};

use crate::{
  job::JobReq,
  prompter::Prompter,
  runner::{ArgReqKind, ResKind, TinyReqKind},
  CLI,
};

type GetSender = mpsc::UnboundedSender<JobReq<TinyReqKind, ResKind>>;
type PostSender = mpsc::UnboundedSender<JobReq<ArgReqKind, ResKind>>;

#[derive(Debug, Clone)]
pub struct AppState {
  pub tiny_sender: GetSender,
  pub big_sender: PostSender,
  pub openai_api_key: Arc<str>,
}

impl AppState {
  pub fn new(
    get_sender: GetSender,
    post_sender: PostSender,
    openai_api_key: Arc<str>,
  ) -> Self {
    Self {
      tiny_sender: get_sender,
      big_sender: post_sender,
      openai_api_key,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct CommsFrequencyQuery {
  frequency: f32,
}
async fn comms_text(
  State(mut state): State<AppState>,
  Query(query): Query<CommsFrequencyQuery>,
  text: String,
) {
  tracing::info!("Received comms text request: {} chars", text.len());

  let command = complete_atc_request(text.clone(), query.frequency).await;
  if let Some(command) = command {
    let _ = JobReq::send(
      ArgReqKind::Command {
        atc: CommandWithFreq::new(
          "ATC".to_string(),
          command.frequency,
          CommandReply::WithoutCallsign { text },
          Vec::new(),
        ),
        reply: command.clone(),
      },
      &mut state.big_sender,
    )
    .recv()
    .await;
  }

  tracing::info!("Replied to text request");
}

async fn comms_voice(
  State(mut state): State<AppState>,
  Query(query): Query<CommsFrequencyQuery>,
  bytes: Bytes,
) {
  tracing::info!("Received comms voice request: {} bytes", bytes.len());
  let now = duration_now();

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
    form.part("file", Part::bytes(bytes.to_vec()).file_name("audio.wav"));
  let form = form.text("model", "whisper-1".to_string());

  let response = client
    .post("https://api.openai.com/v1/audio/transcriptions")
    .multipart(form)
    .header(
      header::AUTHORIZATION,
      header::HeaderValue::from_str(&format!(
        "Bearer {}",
        &state.openai_api_key
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
    if let Some(command) =
      complete_atc_request(reply.text.clone(), query.frequency).await
    {
      if let Some(ref audio_path) = CLI.audio_path {
        let mut audio_path = audio_path.join(format!("{now:?}"));
        audio_path.set_extension("json");

        match std::fs::OpenOptions::new()
          .create_new(true)
          .write(true)
          .open(audio_path)
        {
          Ok(file) => match serde_json::to_writer(file, &command) {
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

      let _ = JobReq::send(
        ArgReqKind::Command {
          atc: CommandWithFreq::new(
            "ATC".to_string(),
            command.frequency,
            CommandReply::WithoutCallsign { text: reply.text },
            Vec::new(),
          ),
          reply: command.clone(),
        },
        &mut state.big_sender,
      )
      .recv()
      .await;
    }
  }

  tracing::info!("Replied to voice request");
}

async fn post_pause(
  State(mut state): State<AppState>,
) -> Result<(), http::StatusCode> {
  let res = JobReq::send(TinyReqKind::Pause, &mut state.tiny_sender)
    .recv()
    .await;
  if let Ok(ResKind::Any) = res {
    Ok(())
  } else {
    Err(http::StatusCode::INTERNAL_SERVER_ERROR)
  }
}

async fn get_messages(
  State(mut state): State<AppState>,
) -> Result<String, http::StatusCode> {
  let res = JobReq::send(TinyReqKind::Messages, &mut state.tiny_sender)
    .recv()
    .await;
  if let Ok(ResKind::Messages(messages)) = res {
    if let Ok(string) = serde_json::to_string(&messages) {
      Ok(string)
    } else {
      Err(http::StatusCode::BAD_REQUEST)
    }
  } else {
    Err(http::StatusCode::INTERNAL_SERVER_ERROR)
  }
}

async fn get_world(
  State(mut state): State<AppState>,
) -> Result<String, http::StatusCode> {
  let res = JobReq::send(TinyReqKind::World, &mut state.tiny_sender)
    .recv()
    .await;
  if let Ok(ResKind::World(world)) = res {
    if let Ok(string) = serde_json::to_string(&world) {
      Ok(string)
    } else {
      Err(http::StatusCode::BAD_REQUEST)
    }
  } else {
    Err(http::StatusCode::INTERNAL_SERVER_ERROR)
  }
}

async fn get_points(
  State(mut state): State<AppState>,
) -> Result<String, http::StatusCode> {
  let res = JobReq::send(TinyReqKind::Points, &mut state.tiny_sender)
    .recv()
    .await;
  if let Ok(ResKind::Points(points)) = res {
    if let Ok(string) = serde_json::to_string(&points) {
      Ok(string)
    } else {
      Err(http::StatusCode::BAD_REQUEST)
    }
  } else {
    Err(http::StatusCode::INTERNAL_SERVER_ERROR)
  }
}

async fn get_aircraft(
  State(mut state): State<AppState>,
) -> Result<String, http::StatusCode> {
  let res = JobReq::send(TinyReqKind::Aircraft, &mut state.tiny_sender)
    .recv()
    .await;
  if let Ok(ResKind::Aircraft(aircraft)) = res {
    if let Ok(string) = serde_json::to_string(&aircraft) {
      Ok(string)
    } else {
      Err(http::StatusCode::BAD_REQUEST)
    }
  } else {
    Err(http::StatusCode::INTERNAL_SERVER_ERROR)
  }
}

async fn ping_pong(
  State(mut state): State<AppState>,
) -> Result<String, http::StatusCode> {
  let res = JobReq::send(TinyReqKind::Ping, &mut state.tiny_sender)
    .recv()
    .await;

  if let Ok(ResKind::Pong) = res {
    Ok("pong".to_string())
  } else {
    Err(http::StatusCode::INTERNAL_SERVER_ERROR)
  }
}

pub async fn run(
  address: SocketAddr,
  get_sender: GetSender,
  post_sender: PostSender,
  openai_api_key: Arc<str>,
) {
  let cors = CorsLayer::very_permissive();
  let app = Router::new().nest(
    "/api",
    Router::new()
      .route("/", get(|| async { "Airwave API is active." }))
      .route("/comms/text", post(comms_text))
      .route("/comms/voice", post(comms_voice))
      .route("/messages", get(get_messages))
      .route("/world", get(get_world))
      .route("/game/points", get(get_points))
      .route("/game/aircraft", get(get_aircraft))
      .route("/pause", post(post_pause))
      .route("/ping", get(ping_pong))
      .with_state(AppState::new(get_sender, post_sender, openai_api_key))
      .layer(cors),
  );

  let listener = tokio::net::TcpListener::bind(address).await.unwrap();
  tracing::info!("Listening on {address}");
  axum::serve(listener, app).await.unwrap();
}

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
    Ok(command) => Some(CommandWithFreq::new(
      command.id,
      frequency,
      command.reply,
      command.tasks,
    )),
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
