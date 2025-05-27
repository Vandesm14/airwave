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
  state::{get_airport_status, get_messages, get_world, post_airport_status},
};
use serde::{Deserialize, Serialize};
use shared::{AppState, GetSender, PostSender};

use engine::engine::UICommand;
use tower_http::{
  compression::CompressionLayer, cors::CorsLayer, services::ServeDir,
};

pub async fn run(
  no_api: bool,
  no_client: bool,
  no_server: bool,
  address_ipv4: SocketAddr,
  address_ipv6: SocketAddr,
  get_sender: GetSender,
  post_sender: PostSender,
) {
  let cors = CorsLayer::very_permissive();

  let mut app = Router::new().layer(CompressionLayer::new()).layer(cors);
  if !no_server {
    let mut api = Router::new()
      // Misc
      .route("/ping", get(ping_pong))
      // Aircraft
      .route("/game/aircraft", get(get_aircraft))
      .route("/game/aircraft/{id}", get(get_one_aircraft))
      // State
      .route("/messages", get(get_messages))
      .route("/world", get(get_world))
      .route("/status/{id}", get(get_airport_status));

    if !no_api {
      api = api
        .route("/", get(|| async { "Airwave API is active." }))
        // Misc
        .route("/pause", post(post_pause))
        // Comms
        .route("/comms/text", post(comms_text))
        .route("/comms/voice", post(comms_voice))
        // State
        .route("/status/{id}", post(post_airport_status));
      tracing::info!("Serving API.");
    } else {
      api =
        api.route("/", get(|| async { "Airwave API is in readonly mode." }));
      tracing::info!("Serving API in readonly mode.");
    }

    app = app.nest(
      "/api",
      api.with_state(AppState::new(get_sender, post_sender)),
    );
  }

  if !no_client {
    app = app.fallback_service(ServeDir::new("assets/client-web"));
    tracing::info!("Serving web client.");
  }

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
