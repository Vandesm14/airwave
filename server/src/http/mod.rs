pub mod methods;
pub mod shared;

use std::net::SocketAddr;

use axum::{
  Router,
  routing::{get, post},
};
use methods::{
  aircraft::{get_aircraft, get_one_aircraft},
  comms::{comms_text, comms_voice},
  misc::{ping_pong, post_pause},
  state::{
    get_airspace_status, get_messages, get_world, post_arrival_status,
    post_departure_status,
  },
};
use serde::{Deserialize, Serialize};
use shared::{AppState, GetSender, PostSender};

use engine::engine::UICommand;
use tower_http::{compression::CompressionLayer, cors::CorsLayer};

pub async fn run(
  address_ipv4: SocketAddr,
  address_ipv6: SocketAddr,
  get_sender: GetSender,
  post_sender: PostSender,
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
      .route("/game/aircraft/{id}", get(get_one_aircraft))
      // State
      .route("/messages", get(get_messages))
      .route("/world", get(get_world))
      .route("/status/{id}", get(get_airspace_status))
      .route("/status/arrival/{id}/{status}", post(post_arrival_status))
      .route(
        "/status/departure/{id}/{status}",
        post(post_departure_status),
      )
      .with_state(AppState::new(get_sender, post_sender))
      .layer(CompressionLayer::new())
      .layer(cors),
  );

  let listener4 = tokio::net::TcpListener::bind(address_ipv4).await.unwrap();
  let listener6 = tokio::net::TcpListener::bind(address_ipv6).await.unwrap();

  tracing::info!("Listening on {address_ipv4} and {address_ipv6}");

  tokio::try_join!(
    axum::serve(listener4, app.clone()),
    axum::serve(listener6, app),
  )
  .unwrap();
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
