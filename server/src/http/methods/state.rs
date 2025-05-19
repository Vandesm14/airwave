use axum::{
  extract::{Path, State},
  http,
};
use engine::entities::world::{ArrivalStatus, DepartureStatus};
use internment::Intern;

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

pub async fn get_airport_status(
  State(mut state): State<AppState>,
  Path(id): Path<String>,
) -> Result<String, http::StatusCode> {
  let res = JobReq::send(
    TinyReqKind::AirspaceStatus(Intern::from(id)),
    &mut state.tiny_sender,
  )
  .recv()
  .await;
  if let Ok(ResKind::AirspaceStatus(status)) = res {
    if let Ok(string) = serde_json::to_string(&status) {
      Ok(string)
    } else {
      Err(http::StatusCode::BAD_REQUEST)
    }
  } else {
    Err(http::StatusCode::INTERNAL_SERVER_ERROR)
  }
}

pub async fn post_arrival_status(
  State(mut state): State<AppState>,
  Path((id, status)): Path<(String, ArrivalStatus)>,
) -> Result<(), http::StatusCode> {
  let res = JobReq::send(
    TinyReqKind::ArrivalStatus(Intern::from(id), status),
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

pub async fn post_departure_status(
  State(mut state): State<AppState>,
  Path((id, status)): Path<(String, DepartureStatus)>,
) -> Result<(), http::StatusCode> {
  let res = JobReq::send(
    TinyReqKind::DepartureStatus(Intern::from(id), status),
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
