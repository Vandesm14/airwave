use std::time::Instant;

use async_openai::{
  Audio,
  error::OpenAIError,
  types::{AudioInput, CreateTranscriptionRequest},
};
use axum::{
  body::Bytes,
  extract::{Query, State},
};
use engine::{
  command::{CommandReply, CommandWithFreq, Task},
  duration_now,
};
use internment::Intern;
use serde::{Deserialize, Serialize};

use crate::{
  CLI,
  http::shared::{AppState, GetSender},
  job::JobReq,
  parser::parse_commands,
  prompter::Prompter,
  runner::{ArgReqKind, ResKind, TinyReqKind},
};

async fn complete_atc_request(
  tiny_sender: &mut GetSender,
  message: String,
  frequency: f32,
) -> Vec<CommandWithFreq> {
  tracing::info!("Parsing request: {}", message);

  let split = Prompter::split_request(message).await;

  // Split the request into the callsign and the rest of the message.
  match split {
    Ok(split) => {
      if split.is_empty() {
        tracing::warn!("Received empty request");
        return Vec::new();
      } else {
        tracing::info!("Split request for {} aircraft", split.len());
      }

      let mut messages: Vec<CommandWithFreq> = Vec::new();

      for req in split {
        // Find the aircraft associated with the request.
        let res = JobReq::send(
          TinyReqKind::OneAircraft(Intern::from_ref(&req.callsign)),
          tiny_sender,
        )
        .recv()
        .await;
        match res {
          Ok(ResKind::OneAircraft(Some(aircraft))) => {
            // Parse the command from the message.
            let (tasks, readback) = tokio::join!(
              Prompter::parse_into_tasks(req.clone(), &aircraft),
              Prompter::generate_readback(req.request)
            );
            match (tasks, readback) {
              // Return the command.
              (Ok(mut tasks), Ok(readback)) => {
                tracing::info!(
                  "Completed request for aircraft {}",
                  aircraft.id
                );
                // Fill in the frequencies of custom events.
                let tasks: Vec<_> = tasks
                  .drain(..)
                  .map(|t| {
                    if let Task::Custom(_, e, a) = t {
                      Task::Custom(frequency, e, a)
                    } else {
                      t
                    }
                  })
                  .collect();
                messages.push(CommandWithFreq::new(
                  aircraft.id.to_string(),
                  frequency,
                  CommandReply::WithCallsign { text: readback },
                  tasks,
                ))
              }
              (Err(err), _) => {
                tracing::error!("Unable to parse tasks: {}", err);
              }
              (_, Err(err)) => {
                tracing::error!("Unable to generate readback: {}", err);
              }
            }
          }
          _ => {
            tracing::error!("Unable to find aircraft \"{}\"", req.callsign);
          }
        }
      }

      messages
    }
    Err(e) => {
      tracing::error!("Unable to split command: {}", e);
      Vec::new()
    }
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
  let time = Instant::now();

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

  let commands = parse_commands(text.clone(), query.frequency);
  let commands = if commands.is_empty() {
    if std::env::var("OPENAI_API_KEY").is_err() {
      let _ = JobReq::send(
          ArgReqKind::CommandATC(CommandWithFreq::new(
            "ATC".to_string(),
            query.frequency,
            CommandReply::Blank { text: "Failed to parse shorthand. Unable to use AI features: OpenAI API key not provided.".to_owned() },
            Vec::new(),
          )),
          &mut state.big_sender,
        )
        .recv()
        .await;

      return;
    } else {
      complete_atc_request(
        &mut state.tiny_sender,
        text.clone(),
        query.frequency,
      )
      .await
    }
  } else {
    tracing::info!(
      "Parsing shorthand: {} into {} commands",
      text,
      commands.len()
    );
    commands
  };

  for command in commands.iter() {
    let _ = JobReq::send(
      ArgReqKind::CommandReply(command.clone()),
      &mut state.big_sender,
    )
    .recv()
    .await;
  }

  let duration = time.elapsed();
  tracing::info!(
    "Completed text request in {:.2} seconds",
    duration.as_secs_f32()
  );
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

async fn transcribe_voice(bytes: Bytes) -> Result<String, OpenAIError> {
  write_wav_data(&bytes);

  let client = async_openai::Client::new();
  let audio = Audio::new(&client);

  let response = audio
    .transcribe(CreateTranscriptionRequest {
      file: AudioInput::from_bytes("audio.wav".to_owned(), bytes),
      model: "whisper-1".to_owned(),
      ..Default::default()
    })
    .await?;

  Ok(response.text)
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
  let time = Instant::now();

  tracing::info!("Received comms voice request: {} bytes", bytes.len());

  if std::env::var("OPENAI_API_KEY").is_err() {
    let _ = JobReq::send(
      ArgReqKind::CommandATC(CommandWithFreq::new(
        "ATC".to_string(),
        query.frequency,
        CommandReply::Blank {
          text: "Failed to transcribe voice. Unable to use AI features: OpenAI API key not provided."
            .to_owned(),
        },
        Vec::new(),
      )),
      &mut state.big_sender,
    )
    .recv()
    .await;
  } else {
    match transcribe_voice(bytes).await {
      Ok(text) => {
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

        let commands = complete_atc_request(
          &mut state.tiny_sender,
          text.clone(),
          query.frequency,
        )
        .await;

        for command in commands.iter() {
          write_json_data(command);

          let _ = JobReq::send(
            ArgReqKind::CommandReply(command.clone()),
            &mut state.big_sender,
          )
          .recv()
          .await;
        }
      }
      Err(e) => tracing::error!("Transcription failed: {}", e),
    }
  }

  let duration = time.elapsed();
  tracing::info!(
    "Completed voice request in {:.2} seconds",
    duration.as_secs_f32()
  );
}
