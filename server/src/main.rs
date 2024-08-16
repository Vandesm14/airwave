use std::{
  env,
  sync::{mpsc, Arc},
};

use async_openai::types::{
  ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
  ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageContent,
  CreateChatCompletionRequest,
};
use axum::Router;
use dotenv::dotenv;
use futures_util::{stream::SplitSink, SinkExt, StreamExt, TryStreamExt};
use glam::Vec2;
use reqwest::{header, multipart::Part, Client};
use serde::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use tower_http::services::ServeDir;

use server::{
  engine::{Engine, IncomingUpdate, OutgoingReply},
  structs::{Command, CommandWithFreq, Runway},
  FEET_PER_UNIT, NAUTICALMILES_TO_FEET,
};

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

#[tokio::main]
async fn main() {
  dotenv().ok();
  let api_key: Arc<str> = env::var("OPENAI_API_KEY")
    .expect("OPENAI_API_KEY must be set")
    .into();

  let (command_sender, command_receiver) = mpsc::channel::<IncomingUpdate>();
  let (update_sender, update_receiver) = mpsc::channel::<OutgoingReply>();

  let app = Router::new().nest_service("/", ServeDir::new("../dist"));

  let airspace_size = NAUTICALMILES_TO_FEET * FEET_PER_UNIT * 40.0;

  let mut engine = Engine::new(
    command_receiver,
    update_sender.clone(),
    airspace_size,
    118.5,
  );
  let engine_handle = tokio::spawn(async move {
    engine.runways.push(Runway {
      id: "20".into(),
      pos: Vec2::new(airspace_size * 0.5, airspace_size * 0.5),
      heading: 200.0,
      length: 7000.0,
    });

    engine.runways.push(Runway {
      id: "29".into(),
      pos: Vec2::new(airspace_size * 0.5, airspace_size * 0.5),
      heading: 290.0,
      length: 7000.0,
    });

    engine.spawn_random_aircraft();
    engine.begin_loop();
  });

  let listener = TcpListener::bind("0.0.0.0:8000").await.unwrap();
  let http_handle = tokio::spawn(async move {
    axum::serve(listener, app).await.unwrap();
  });

  let (give_streams, take_streams) =
    mpsc::channel::<SplitSink<WebSocketStream<TcpStream>, Message>>();

  let ws_handle = tokio::spawn(async move {
    let try_socket = TcpListener::bind("0.0.0.0:9001").await;
    let listener = try_socket.expect("ws server failed to bind");

    while let Ok((stream, _)) = listener.accept().await {
      let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

      let (write, read) = ws_stream.split();
      give_streams.send(write).unwrap();

      let sender = command_sender.clone();
      let ws_sender = update_sender.clone();
      let api_key = api_key.clone();

      tokio::spawn(async move {
        read
          .try_for_each(|message| {
            let sender = sender.clone();
            let api_key = api_key.clone();

            let ws_sender = ws_sender.clone();
            async move {
              if let Message::Text(string) = message {
                dbg!("received incoming ws");
                let req =
                  serde_json::from_str::<FrontendRequest>(&string).unwrap();
                match req {
                  FrontendRequest::Voice {
                    data: bytes,
                    frequency,
                  } => {
                    dbg!("received transcription request", bytes.len());

                    let client = Client::new();
                    let form = reqwest::multipart::Form::new();
                    let form = form
                      .part("file", Part::bytes(bytes).file_name("audio.wav"));
                    let form = form.text("model", "whisper-1".to_string());

                    let response = client
                      .post("https://api.openai.com/v1/audio/transcriptions")
                      .multipart(form)
                      .header(
                        header::AUTHORIZATION,
                        header::HeaderValue::from_str(&format!(
                          "Bearer {}",
                          &api_key.clone()
                        ))
                        .unwrap(),
                      )
                      .header(
                        header::CONTENT_TYPE,
                        header::HeaderValue::from_str("multipart/form-data")
                          .unwrap(),
                      )
                      .send()
                      .await
                      .unwrap();

                    let text = response.text().await.unwrap();
                    if let Ok(reply) =
                      serde_json::from_str::<AudioResponse>(&text)
                    {
                      ws_sender
                        .send(OutgoingReply::ATCReply(CommandWithFreq {
                          id: "ATC".to_owned(),
                          frequency,
                          reply: reply.text.clone(),
                          tasks: Vec::new(),
                        }))
                        .unwrap();

                      if let Some(result) =
                        complete_atc_request(reply.text, frequency).await
                      {
                        sender.send(IncomingUpdate::Command(result)).unwrap();
                      }
                    }
                  }
                  FrontendRequest::Text {
                    text: string,
                    frequency,
                  } => {
                    if let Some(result) =
                      complete_atc_request(string, frequency).await
                    {
                      sender.send(IncomingUpdate::Command(result)).unwrap();
                    }
                  }
                  FrontendRequest::Connect => {
                    sender.send(IncomingUpdate::Connect).unwrap();
                  }
                }
              }

              Ok(())
            }
          })
          .await
          .unwrap();
      });
    }
  });

  let broadcast_handle = tokio::spawn(async move {
    let mut streams: Vec<SplitSink<WebSocketStream<TcpStream>, Message>> =
      Vec::new();

    loop {
      for stream in take_streams.try_iter() {
        streams.push(stream);
      }

      if let Ok(update) = update_receiver.try_recv() {
        for write in streams.iter_mut() {
          let _ = write
            .send(Message::Text(serde_json::to_string(&update).unwrap()))
            .await;
        }
      }
    }
  });

  tokio::select! {
    _ = engine_handle => println!("engine exit"),
    _ = http_handle => println!("http exit"),
    _ = broadcast_handle => println!("broadcast exit"),
    _ = ws_handle => println!("ws exit"),
  };
}

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
        if let Ok(reply) = serde_json::from_str::<Command>(text) {
          return Some(CommandWithFreq {
            id: reply.id,
            frequency: freq,
            reply: reply.reply,
            tasks: reply.tasks,
          });
        }
      }
    }
  }

  None
}
