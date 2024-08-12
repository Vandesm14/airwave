use axum::body::Bytes;
use axum::Router;
use axum::{extract::State, routing::post};
use dotenv::dotenv;
use reqwest::multipart::Part;
use reqwest::{header, Client};
use std::{env, sync::Arc};

struct AppState {
  client: Client,
  api_key: String,
}

#[tokio::main]
async fn main() {
  dotenv().ok();
  let api_key = env::var("API_KEY").expect("API_KEY must be set");
  let client = Client::new();

  let shared_state = Arc::new(AppState { api_key, client });

  let app = Router::new()
    .route("/transcribe", post(transcribe))
    .with_state(shared_state);

  // run our app with hyper, listening globally on port 3000
  let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
  axum::serve(listener, app).await.unwrap();
}

async fn transcribe(State(state): State<Arc<AppState>>, body: Bytes) -> String {
  let form = reqwest::multipart::Form::new();
  let form =
    form.part("file", Part::bytes(body.to_vec()).file_name("audio.wav"));
  let form = form.text("model", "whisper-1".to_string());

  let response = state
    .client
    .post("https://api.openai.com/v1/audio/transcriptions")
    .multipart(form)
    .header(
      header::AUTHORIZATION,
      header::HeaderValue::from_str(&format!("Bearer {}", state.api_key))
        .unwrap(),
    )
    .header(
      header::CONTENT_TYPE,
      header::HeaderValue::from_str("multipart/form-data").unwrap(),
    )
    .send()
    .await
    .unwrap();

  response.text().await.unwrap()
}
