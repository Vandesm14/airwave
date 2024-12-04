use std::{ops::Add, time::Duration};

use axum::{
  extract::{Path, State},
  http, Form,
};
use engine::{duration_now, entities::flight::FlightKind};
use serde::Deserialize;

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

#[derive(Debug, Clone, Deserialize)]
pub struct CreateFlightForm {
  pub kind: FlightKind,
  pub spawn_at: u64,
}

pub async fn create_flight(
  State(mut state): State<AppState>,
  Form(form): Form<CreateFlightForm>,
) -> Result<String, http::StatusCode> {
  let res = JobReq::send(
    TinyReqKind::CreateFlight {
      kind: form.kind,
      spawn_at: duration_now().add(Duration::from_secs(form.spawn_at)),
    },
    &mut state.tiny_sender,
  )
  .recv()
  .await;
  if let Ok(ResKind::OneFlight(flight)) = res {
    if let Ok(string) = serde_json::to_string(&flight) {
      Ok(string)
    } else {
      Err(http::StatusCode::BAD_REQUEST)
    }
  } else {
    Err(http::StatusCode::INTERNAL_SERVER_ERROR)
  }
}

pub async fn delete_flight(
  State(mut state): State<AppState>,
  Path(id): Path<usize>,
) -> Result<String, http::StatusCode> {
  let res = JobReq::send(TinyReqKind::DeleteFlight(id), &mut state.tiny_sender)
    .recv()
    .await;
  if let Ok(ResKind::OneFlight(flight)) = res {
    if let Ok(string) = serde_json::to_string(&flight) {
      Ok(string)
    } else {
      Err(http::StatusCode::BAD_REQUEST)
    }
  } else {
    Err(http::StatusCode::INTERNAL_SERVER_ERROR)
  }
}
