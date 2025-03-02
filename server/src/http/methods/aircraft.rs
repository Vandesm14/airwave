use axum::{
  extract::{Path, State},
  http,
};
use internment::Intern;

use crate::{
  http::shared::AppState,
  job::JobReq,
  runner::{ResKind, TinyReqKind},
};

pub async fn get_aircraft(
  State(mut state): State<AppState>,
) -> Result<String, http::StatusCode> {
  let res = JobReq::send(TinyReqKind::Aircraft, &mut state.tiny_sender)
    .recv()
    .await;
  if let Ok(ResKind::Aircraft(aircraft)) = res {
    if let Ok(string) = serde_json::to_string(&aircraft) {
      Ok(string)
    } else {
      Err(http::StatusCode::BAD_REQUEST)
    }
  } else {
    Err(http::StatusCode::INTERNAL_SERVER_ERROR)
  }
}

pub async fn get_one_aircraft(
  State(mut state): State<AppState>,
  Path(id): Path<String>,
) -> Result<String, http::StatusCode> {
  let res = JobReq::send(
    TinyReqKind::OneAircraft(Intern::from(id)),
    &mut state.tiny_sender,
  )
  .recv()
  .await;
  if let Ok(ResKind::OneAircraft(aircraft)) = res {
    if let Ok(string) = serde_json::to_string(&aircraft) {
      Ok(string)
    } else {
      Err(http::StatusCode::BAD_REQUEST)
    }
  } else {
    Err(http::StatusCode::INTERNAL_SERVER_ERROR)
  }
}

pub async fn accept_flight(
  State(mut state): State<AppState>,
  Path(id): Path<String>,
) -> Result<(), http::StatusCode> {
  let res = JobReq::send(
    TinyReqKind::AcceptFlight(Intern::from(id)),
    &mut state.tiny_sender,
  )
  .recv()
  .await;
  if let Ok(ResKind::Any) = res {
    Ok(())
  } else {
    Err(http::StatusCode::INTERNAL_SERVER_ERROR)
  }
}
