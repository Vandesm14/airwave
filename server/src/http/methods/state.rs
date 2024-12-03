use axum::{extract::State, http};

use crate::{
  http::shared::AppState,
  job::JobReq,
  runner::{ResKind, TinyReqKind},
};

pub async fn get_messages(
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

pub async fn get_world(
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

pub async fn get_points(
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
