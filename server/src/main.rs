use axum::Router;
use dotenv::dotenv;
use futures_util::{
  stream::SplitSink,
  {SinkExt, StreamExt},
};
use glam::Vec2;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use server::engine::{Engine, IncomingUpdate, StateUpdate};
use std::{
  sync::mpsc,
  {env, sync::Arc},
};
use tokio::{
  net::{TcpListener, TcpStream},
  sync::Mutex,
};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use tower_http::services::ServeDir;

use server::structs::{Aircraft, AircraftTargets};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum FrontendRequest {
  Transcribe(String),
  Complete(String),
}

#[tokio::main]
async fn main() {
  dotenv().ok();
  let api_key = env::var("API_KEY").expect("API_KEY must be set");
  let client = Client::new();

  let (command_sender, command_receiver) = mpsc::channel::<IncomingUpdate>();
  let (update_sender, update_receiver) = mpsc::channel::<StateUpdate>();

  let app = Router::new().nest_service("/", ServeDir::new("../dist"));

  let mut engine = Engine::new(command_receiver, update_sender);
  let engine_handle = tokio::spawn(async move {
    engine.aircraft.push(Aircraft {
      callsign: "SKW1234".into(),
      is_colliding: false,
      is_active: true,
      pos: Vec2::new(0.0, 0.0),
      heading: 135.0,
      speed: 250.0,
      altitude: 8000.0,
      target: AircraftTargets {
        heading: 135.0,
        speed: 250.0,
        altitude: 8000.0,
        runway: None,
      },
    });
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

      let (write, _) = ws_stream.split();
      give_streams.send(write).unwrap();
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
          write
            .send(Message::Text(serde_json::to_string(&update).unwrap()))
            .await
            .expect("failed to send message");
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

// async fn transcribe(State(state): State<Arc<AppState>>, body: Bytes) -> String {
//   let form = reqwest::multipart::Form::new();
//   let form =
//     form.part("file", Part::bytes(body.to_vec()).file_name("audio.wav"));
//   let form = form.text("model", "whisper-1".to_string());

//   let response = state
//     .client
//     .post("https://api.openai.com/v1/audio/transcriptions")
//     .multipart(form)
//     .header(
//       header::AUTHORIZATION,
//       header::HeaderValue::from_str(&format!("Bearer {}", state.api_key))
//         .unwrap(),
//     )
//     .header(
//       header::CONTENT_TYPE,
//       header::HeaderValue::from_str("multipart/form-data").unwrap(),
//     )
//     .send()
//     .await
//     .unwrap();

//   response.text().await.unwrap()
// }

// async fn complete(State(state): State<Arc<AppState>>, body: String) -> String {
//   let response = state
//     .client
//     .post("https://api.openai.com/v1/chat/completions")
//     .header(
//       header::AUTHORIZATION,
//       header::HeaderValue::from_str(&format!("Bearer {}", state.api_key))
//         .unwrap(),
//     )
//     .header(
//       header::CONTENT_TYPE,
//       header::HeaderValue::from_str("application/json").unwrap(),
//     )
//     .body(body)
//     .send()
//     .await
//     .unwrap();

//   response.text().await.unwrap()
// }
