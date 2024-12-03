use axum::{extract::State, http};

use crate::{
  http::shared::AppState,
  job::JobReq,
  runner::{ResKind, TinyReqKind},
};

pub async fn ping_pong(
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

pub async fn post_pause(
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
