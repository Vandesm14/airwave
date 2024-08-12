use base64::{engine::general_purpose, Engine as _};
use core::str;
use dotenv::dotenv;
use reqwest::{header, Client};
use rocket::data::{Data, ToByteUnit};
use rocket::http::Status;
use rocket::response::content;
use rocket::{post, routes, State};
use std::env;

#[macro_use]
extern crate rocket;

struct AppState {
  client: Client,
  api_key: String,
}

#[post("/transcribe", data = "<body>")]
async fn transcribe(
  state: &State<AppState>,
  body: Data<'_>,
) -> Result<content::RawJson<String>, Status> {
  let body_bytes = body
    .open(512.mebibytes())
    .into_bytes()
    .await
    .map_err(|_| Status::InternalServerError)?;

  let mut req = reqwest::Request::new(
    reqwest::Method::POST,
    "https://api.openai.com/v1/audio/transcriptions"
      .parse()
      .unwrap(),
  );

  *req.body_mut() = Some(body_bytes.to_vec().into());
  req.headers_mut().insert(
    header::AUTHORIZATION,
    header::HeaderValue::from_str(&format!("Bearer {}", state.api_key))
      .unwrap(),
  );
  req.headers_mut().insert(
    header::CONTENT_TYPE,
    header::HeaderValue::from_str("multipart/form-data").unwrap(),
  );

  let response = state
    .client
    .execute(req)
    .await
    .map_err(|_| Status::BadGateway)?;
  let body = response
    .text()
    .await
    .map_err(|_| Status::InternalServerError)?;

  if is_base64(&body) {
    match general_purpose::STANDARD.decode(&body) {
      Ok(decoded) => Ok(content::RawJson(
        String::from_utf8_lossy(&decoded).into_owned(),
      )),
      Err(_) => Err(Status::InternalServerError),
    }
  } else {
    Ok(content::RawJson(body))
  }
}

fn is_base64(s: &str) -> bool {
  if s.contains(['\n', '\r', '\t', ' ']) {
    return false;
  }
  general_purpose::STANDARD.decode(s).is_ok()
}

#[launch]
fn rocket() -> _ {
  dotenv().ok();
  let api_key = env::var("API_KEY").expect("API_KEY must be set");
  let client = Client::new();

  rocket::build()
    .manage(AppState { client, api_key })
    .mount("/", routes![transcribe])
}
