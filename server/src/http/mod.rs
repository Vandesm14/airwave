pub mod methods;
pub mod shared;

use std::{net::SocketAddr, sync::Arc};

use axum::{
  routing::{delete, get, post},
  Router,
};
use methods::{
  aircraft::{get_aircraft, get_one_aircraft},
  comms::{comms_text, comms_voice},
  flights::{create_flight, delete_flight, get_flights},
  misc::{ping_pong, post_pause},
  state::{get_messages, get_points, get_world},
};
use serde::{Deserialize, Serialize};
use shared::{AppState, GetSender, PostSender};

use engine::engine::UICommand;
use tower_http::cors::CorsLayer;

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
      // Misc
      .route("/pause", post(post_pause))
      .route("/ping", get(ping_pong))
      // Comms
      .route("/comms/text", post(comms_text))
      .route("/comms/voice", post(comms_voice))
      // Aircraft
      .route("/game/aircraft", get(get_aircraft))
      .route("/game/aircraft/:id", get(get_one_aircraft))
      // Flights
      .route("/game/flights", get(get_flights))
      .route("/game/flight", post(create_flight))
      .route("/game/flight/:id", delete(delete_flight))
      // State
      .route("/messages", get(get_messages))
      .route("/world", get(get_world))
      .route("/game/points", get(get_points))
      .with_state(AppState::new(get_sender, post_sender, openai_api_key))
      .layer(cors),
  );

  let listener = tokio::net::TcpListener::bind(address).await.unwrap();
  tracing::info!("Listening on {address}");
  axum::serve(listener, app).await.unwrap();
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
