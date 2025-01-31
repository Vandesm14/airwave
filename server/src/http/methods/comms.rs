use std::sync::Arc;

use async_openai::error::OpenAIError;
use axum::{
  body::Bytes,
  extract::{Query, State},
};
use engine::{
  command::{CommandReply, CommandWithFreq},
  duration_now,
};
use internment::Intern;
use reqwest::{header, multipart::Part, Client};
use serde::{Deserialize, Serialize};

use crate::{
  http::{
    shared::{AppState, GetSender},
    AudioResponse,
  },
  job::JobReq,
  prompter::Prompter,
  runner::{ArgReqKind, ResKind, TinyReqKind},
  CLI,
};

async fn complete_atc_request(
  tiny_sender: &mut GetSender,
  message: String,
  frequency: f32,
) -> Option<CommandWithFreq> {
  let split = Prompter::split_request(message).await;

  // Split the request into the callsign and the rest of the message.
  match split {
    Ok(split) => {
      // Find the aircraft associated with the request.
      let res = JobReq::send(
        TinyReqKind::OneAircraft(Intern::from_ref(&split.callsign)),
        tiny_sender,
      )
      .recv()
      .await;
      match res {
        Ok(ResKind::OneAircraft(Some(aircraft))) => {
          if !aircraft.active() {
            // Prevent inactive aircraft from receiving commands.
            tracing::warn!(
              "Inactive aircraft \"{}\" received command",
              aircraft.id
            );
            return None;
          }

          // Parse the command from the message.
          let readback =
            Prompter::generate_readback(split.request.clone(), &aircraft).await;
          let tasks = Prompter::parse_into_tasks(split, &aircraft).await;
          match (tasks, readback) {
            // Return the command.
            (Ok(tasks), Ok(readback)) => Some(CommandWithFreq::new(
              aircraft.id.to_string(),
              frequency,
              CommandReply::Blank { text: readback },
              tasks,
            )),
            (Err(err), _) => {
              tracing::error!("Unable to parse command: {}", err);
              None
            }
            (_, Err(err)) => {
              tracing::error!("Unable to generate readback: {}", err);
              None
            }
            _ => {
              tracing::error!("Unknown error");
              None
            }
          }
        }
        _ => {
          tracing::error!("Unable to find aircraft for command");
          None
        }
      }
    }
    Err(_) => todo!(),
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommsFrequencyQuery {
  frequency: f32,
}
pub async fn comms_text(
  State(mut state): State<AppState>,
  Query(query): Query<CommsFrequencyQuery>,
  text: String,
) {
  tracing::info!("Received comms text request: {} chars", text.len());

  let _ = JobReq::send(
    ArgReqKind::CommandATC(CommandWithFreq::new(
      "ATC".to_string(),
      query.frequency,
      CommandReply::Blank { text: text.clone() },
      Vec::new(),
    )),
    &mut state.big_sender,
  )
  .recv()
  .await;

  let command =
    complete_atc_request(&mut state.tiny_sender, text.clone(), query.frequency)
      .await;
  if let Some(command) = command {
    let _ = JobReq::send(
      ArgReqKind::CommandReply(command.clone()),
      &mut state.big_sender,
    )
    .recv()
    .await;
  }

  tracing::info!("Replied to text request");
}

fn write_wav_data(bytes: &Bytes) {
  if let Some(ref audio_path) = CLI.audio_path {
    let now = duration_now();
    let mut audio_path = audio_path.join(format!("{now:?}"));
    audio_path.set_extension("wav");

    match std::fs::write(audio_path, bytes.clone()) {
      Ok(_) => tracing::debug!("Wrote audio to file"),
      Err(e) => tracing::error!("Unable to write path: {e}"),
    }
  }
}

async fn transcribe_voice(
  bytes: Bytes,
  openai_api_key: Arc<str>,
) -> Result<String, OpenAIError> {
  write_wav_data(&bytes);

  let client = Client::new();
  let form = reqwest::multipart::Form::new()
    .part("file", Part::bytes(bytes.to_vec()).file_name("audio.wav"))
    .text("model", "whisper-1".to_string());

  let response = client
    .post("https://api.openai.com/v1/audio/transcriptions")
    .multipart(form)
    .header(
      header::AUTHORIZATION,
      header::HeaderValue::from_str(&format!("Bearer {}", openai_api_key))
        .unwrap(),
    )
    .header(
      header::CONTENT_TYPE,
      header::HeaderValue::from_str("multipart/form-data").unwrap(),
    )
    .send()
    .await?;

  let text = response.text().await?;
  Ok(text)
}

fn write_json_data(command: &CommandWithFreq) {
  if let Some(ref audio_path) = CLI.audio_path {
    let now = duration_now();
    let mut audio_path = audio_path.join(format!("{now:?}"));
    audio_path.set_extension("json");

    match std::fs::OpenOptions::new()
      .create_new(true)
      .write(true)
      .open(audio_path)
    {
      Ok(file) => match serde_json::to_writer(file, command) {
        Ok(()) => {
          tracing::debug!("Wrote associated audio command file")
        }
        Err(e) => {
          tracing::warn!("Unable to write associated audio command file: {e}")
        }
      },
      Err(e) => {
        tracing::warn!("Unable to create associated audio command file: {e}")
      }
    }
  }
}

pub async fn comms_voice(
  State(mut state): State<AppState>,
  Query(query): Query<CommsFrequencyQuery>,
  bytes: Bytes,
) {
  tracing::info!("Received comms voice request: {} bytes", bytes.len());

  match transcribe_voice(bytes, state.openai_api_key.clone()).await {
    Ok(text) => {
      tracing::info!("Transcribed request: {} chars", text.len());
      if let Ok(reply) = serde_json::from_str::<AudioResponse>(&text) {
        let _ = JobReq::send(
          ArgReqKind::CommandATC(CommandWithFreq::new(
            "ATC".to_string(),
            query.frequency,
            CommandReply::Blank {
              text: reply.text.clone(),
            },
            Vec::new(),
          )),
          &mut state.big_sender,
        )
        .recv()
        .await;

        if let Some(command) = complete_atc_request(
          &mut state.tiny_sender,
          reply.text.clone(),
          query.frequency,
        )
        .await
        {
          write_json_data(&command);

          let _ = JobReq::send(
            ArgReqKind::CommandReply(command.clone()),
            &mut state.big_sender,
          )
          .recv()
          .await;
        }
      }
    }
    Err(e) => tracing::error!("Transcription failed: {}", e),
  }

  tracing::info!("Replied to voice request");
}
