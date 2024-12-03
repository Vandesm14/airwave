use axum::{extract::State, http};

use crate::{
  http::shared::AppState,
  job::JobReq,
  runner::{ResKind, TinyReqKind},
};

pub async fn get_flights(
  State(mut state): State<AppState>,
) -> Result<String, http::StatusCode> {
  let res = JobReq::send(TinyReqKind::Flights, &mut state.tiny_sender)
    .recv()
    .await;
  if let Ok(ResKind::Flights(flights)) = res {
    let mut sorted = flights;
    sorted.sort_by_key(|f| f.spawn_at);
    if let Ok(string) = serde_json::to_string(&sorted) {
      Ok(string)
    } else {
      Err(http::StatusCode::BAD_REQUEST)
    }
  } else {
    Err(http::StatusCode::INTERNAL_SERVER_ERROR)
  }
}
