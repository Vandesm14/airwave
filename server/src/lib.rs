use std::{
  path::PathBuf,
  sync::Arc,
  time::{Duration, Instant},
};

use async_channel::TryRecvError;
use async_openai::{
  error::OpenAIError,
  types::{
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
    ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageContent,
    CreateChatCompletionRequest,
  },
};
use futures_util::{
  stream::{SplitSink, SplitStream},
  SinkExt as _, StreamExt as _,
};
use internment::Intern;
use prompter::Prompter;
use reqwest::{header, multipart::Part, Client};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_tungstenite::{
  tungstenite::{self, Message},
  WebSocketStream,
};
use turborand::rng::Rng;

use engine::{
  command::{CommandWithFreq, OutgoingCommandReply, Task},
  engine::{Engine, Event, UICommand, UIEvent},
  entities::{
    aircraft::{
      events::{AircraftEvent, EventKind},
      Aircraft,
    },
    world::{Game, Points, World},
  },
};

pub mod airport;
pub mod prompter;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type", content = "value")]
pub enum OutgoingReply {
  // Partial/Small Updates
  ATCReply(OutgoingCommandReply),
  Reply(OutgoingCommandReply),

  // Full State Updates
  Aircraft(Vec<Aircraft>),
  World(World),
  Size(f32),
  Points(Points),
  Funds(usize),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IncomingUpdate {
  Command(CommandWithFreq),
  UICommand(UICommand),
  Connect,
}

#[derive(Debug, Clone)]
pub struct Runner {
  pub world: World,
  pub game: Game,
  pub engine: Engine,

  pub receiver: async_channel::Receiver<IncomingUpdate>,
  pub outgoing_sender: async_broadcast::Sender<OutgoingReply>,
  pub incoming_sender: async_channel::Sender<IncomingUpdate>,

  pub save_to: Option<PathBuf>,
  pub rng: Rng,

  last_tick: Instant,
  rate: usize,
}

impl Runner {
  pub fn new(
    receiver: async_channel::Receiver<IncomingUpdate>,
    outgoing_sender: async_broadcast::Sender<OutgoingReply>,
    incoming_sender: async_channel::Sender<IncomingUpdate>,
    save_to: Option<PathBuf>,
    rng: Rng,
  ) -> Self {
    Self {
      world: World::default(),
      game: Game::default(),
      engine: Engine::default(),

      receiver,
      outgoing_sender,
      incoming_sender,

      save_to,
      rng,

      last_tick: Instant::now(),
      rate: 10,
    }
  }

  pub fn add_aircraft(&mut self, mut aircraft: Aircraft) {
    while self.game.aircraft.iter().any(|a| a.id == aircraft.id) {
      aircraft.id = Intern::from(Aircraft::random_callsign(&mut self.rng));
    }

    if aircraft.flight_plan.departing == aircraft.flight_plan.arriving {
      tracing::warn!(
        "deleted a flight departing and arriving at the same airspace"
      );
      return;
    }

    self.game.aircraft.push(aircraft);
  }

  pub fn begin_loop(&mut self) {
    'main_loop: loop {
      if Instant::now() - self.last_tick
        >= Duration::from_secs_f32(1.0 / self.rate as f32)
      {
        self.last_tick = Instant::now();

        let mut commands: Vec<CommandWithFreq> = Vec::new();
        let mut ui_commands: Vec<UICommand> = Vec::new();

        loop {
          let incoming = match self.receiver.try_recv() {
            Ok(incoming) => incoming,
            Err(TryRecvError::Closed) => break 'main_loop,
            Err(TryRecvError::Empty) => break,
          };

          match incoming {
            IncomingUpdate::Command(command) => commands.push(command),
            IncomingUpdate::UICommand(ui_command) => {
              ui_commands.push(ui_command)
            }
            IncomingUpdate::Connect => self.broadcast_for_new_client(),
          }
        }

        for command in commands {
          self.execute_command(command);
        }

        for ui_command in ui_commands {
          self.engine.events.push(Event::UiEvent(ui_command.into()));
        }

        let dt = 1.0 / self.rate as f32;
        let events =
          self
            .engine
            .tick(&self.world, &mut self.game, &mut self.rng, dt);

        // Run through all callout events and broadcast them
        for event in events.iter().filter_map(|e| match e {
          Event::Aircraft(aircraft_event) => Some(aircraft_event),
          Event::UiEvent(_) => None,
        }) {
          if let EventKind::Callout(command) = &event.kind {
            if let Err(e) = self
              .outgoing_sender
              .try_broadcast(OutgoingReply::Reply(command.clone().into()))
            {
              tracing::error!("error sending outgoing reply: {e}")
            }
          }
        }

        for event in events.iter().filter_map(|e| match e {
          Event::Aircraft(_) => None,
          Event::UiEvent(ui_event) => Some(ui_event),
        }) {
          if let UIEvent::Funds(_) = &event {
            self.broadcast_funds();
          }
        }

        self.cleanup(events.iter().filter_map(|e| match e {
          Event::Aircraft(aircraft_event) => Some(aircraft_event),
          Event::UiEvent(_) => None,
        }));
        self.broadcast_aircraft();
        // TODO: self.save_world();
      }
    }
  }

  fn cleanup<'a, T>(&mut self, events: T)
  where
    T: Iterator<Item = &'a AircraftEvent>,
  {
    for event in events {
      if let AircraftEvent {
        id,
        kind: EventKind::Delete,
      } = event
      {
        let index = self
          .game
          .aircraft
          .iter()
          .enumerate()
          .find_map(|(i, a)| (a.id == *id).then_some(i));
        if let Some(index) = index {
          self.game.aircraft.swap_remove(index);
        }
      }
    }
  }

  fn execute_command(&mut self, command: CommandWithFreq) {
    let id = Intern::from_ref(&command.id);
    if self
      .game
      .aircraft
      .iter()
      .any(|a| a.id == id && a.frequency == command.frequency)
    {
      self.engine.events.extend(
        command
          .tasks
          .iter()
          .cloned()
          .map(|t| AircraftEvent { id, kind: t.into() }.into()),
      );

      let mut callout = true;
      for task in command.tasks.iter() {
        match task {
          Task::Ident => {
            // Don't generate a callout for these commands
            callout = command.tasks.len() > 1;
          }

          _ => {
            // Generate a callout from the command
            callout = true;
          }
        }
      }

      if callout {
        self
          .outgoing_sender
          .try_broadcast(OutgoingReply::Reply(command.clone().into()))
          .unwrap();
      }
    }
  }

  fn broadcast_aircraft(&self) {
    let _ = self
      .outgoing_sender
      .try_broadcast(OutgoingReply::Aircraft(self.game.aircraft.clone()))
      .inspect_err(|e| tracing::warn!("failed to broadcast aircraft: {}", e));
  }

  fn broadcast_points(&self) {
    let _ = self
      .outgoing_sender
      .try_broadcast(OutgoingReply::Points(self.game.points.clone()))
      .inspect_err(|e| tracing::warn!("failed to broadcast points: {}", e));
  }

  fn broadcast_funds(&self) {
    let _ = self
      .outgoing_sender
      .try_broadcast(OutgoingReply::Funds(self.game.funds))
      .inspect_err(|e| tracing::warn!("failed to broadcast funds: {}", e));
  }

  fn broadcast_world(&self) {
    let _ = self
      .outgoing_sender
      .try_broadcast(OutgoingReply::World(self.world.clone()))
      .inspect_err(|e| tracing::warn!("failed to broadcast world: {}", e));
  }

  fn broadcast_for_new_client(&self) {
    self.broadcast_world();
    self.broadcast_points();
  }
}

pub async fn broadcast_updates_to(
  mut writer: SplitSink<WebSocketStream<TcpStream>, Message>,
  mut update_rx: async_broadcast::Receiver<OutgoingReply>,
) {
  loop {
    let update = match update_rx.recv().await {
      Ok(update) => update,
      Err(async_broadcast::RecvError::Overflowed(_)) => continue,
      Err(async_broadcast::RecvError::Closed) => break,
    };

    let ser = match serde_json::to_string(&update) {
      Ok(ser) => ser,
      Err(e) => {
        tracing::error!("Unable to serialise update: {e}");
        continue;
      }
    };

    if let Err(e) = writer.send(Message::Text(ser)).await {
      match e {
        tungstenite::Error::ConnectionClosed => break,
        tungstenite::Error::AlreadyClosed
        | tungstenite::Error::AttackAttempt => {
          tracing::error!("Unable to send update: {e}");
          break;
        }
        e => {
          tracing::error!("Unable to send update: {e}");
        }
      }
    }

    tracing::trace!("Sent update");
  }
}

pub async fn receive_commands_from(
  openai_api_key: Arc<str>,
  reader: SplitStream<WebSocketStream<TcpStream>>,
  update_tx: async_broadcast::Sender<OutgoingReply>,
  command_tx: async_channel::Sender<IncomingUpdate>,
) {
  reader
    .for_each(|message| {
      let openai_api_key = openai_api_key.clone();
      let update_tx = update_tx.clone();
      let command_tx = command_tx.clone();

      async move {
        let message = match message {
          Ok(message) => message,
          Err(e) => {
            tracing::error!("Unable to receive command: {e}");
            return;
          }
        };

        if let Message::Text(text) = message {
          let req: FrontendRequest = match serde_json::from_str(&text) {
            Ok(req) => req,
            Err(e) => {
              tracing::error!("Received malformed command: {e}");
              return;
            }
          };

          tracing::debug!("Received command message: length {}", text.len());

          match req {
            FrontendRequest::UI(ui_command) => {
              command_tx
                .send(IncomingUpdate::UICommand(ui_command))
                .await
                .unwrap();
            }
            FrontendRequest::Voice {
              data: bytes,
              frequency,
            } => {
              tracing::info!(
                "Received transcription request: {} bytes",
                bytes.len()
              );

              let client = Client::new();
              let form = reqwest::multipart::Form::new();
              let form =
                form.part("file", Part::bytes(bytes).file_name("audio.wav"));
              let form = form.text("model", "whisper-1".to_string());

              let response = client
                .post("https://api.openai.com/v1/audio/transcriptions")
                .multipart(form)
                .header(
                  header::AUTHORIZATION,
                  header::HeaderValue::from_str(&format!(
                    "Bearer {}",
                    &openai_api_key
                  ))
                  .unwrap(),
                )
                .header(
                  header::CONTENT_TYPE,
                  header::HeaderValue::from_str("multipart/form-data").unwrap(),
                )
                .send()
                .await
                .unwrap();

              let text = response.text().await.unwrap();
              tracing::info!("Transcribed request: {} chars", text.len());
              if let Ok(reply) = serde_json::from_str::<AudioResponse>(&text) {
                update_tx
                  .broadcast(OutgoingReply::ATCReply(OutgoingCommandReply {
                    id: "ATC".to_owned(),
                    frequency,
                    reply: reply.text.clone(),
                  }))
                  .await
                  .unwrap();

                if let Some(result) =
                  complete_atc_request(reply.text, frequency).await
                {
                  command_tx
                    .send(IncomingUpdate::Command(result))
                    .await
                    .unwrap();
                }
              }
            }
            FrontendRequest::Text {
              text: string,
              frequency,
            } => {
              update_tx
                .broadcast(OutgoingReply::ATCReply(OutgoingCommandReply {
                  id: "ATC".to_owned(),
                  frequency,
                  reply: string.clone(),
                }))
                .await
                .unwrap();

              if let Some(result) =
                complete_atc_request(string, frequency).await
              {
                command_tx
                  .send(IncomingUpdate::Command(result))
                  .await
                  .unwrap();
              }
            }
            FrontendRequest::Connect => {
              command_tx.send(IncomingUpdate::Connect).await.unwrap();
            }
          }
        } else {
          tracing::debug!("Skipping non-text WebSocket message")
        }
      }
    })
    .await;
}

async fn send_chatgpt_request(
  prompt: String,
  message: String,
) -> Result<Option<String>, OpenAIError> {
  let client = async_openai::Client::new();
  let request = CreateChatCompletionRequest {
    messages: vec![
      ChatCompletionRequestMessage::System(
        ChatCompletionRequestSystemMessage {
          content: prompt.clone(),
          name: None,
        },
      ),
      ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
        content: ChatCompletionRequestUserMessageContent::Text(message.clone()),
        name: None,
      }),
    ],
    model: "gpt-4o-mini".into(),
    ..Default::default()
  };

  let response = client.chat().create(request).await;
  match response {
    Ok(response) => Ok(response.choices.first().and_then(|c| {
      let c = c.message.content.clone();
      tracing::debug!(
        "**sent prompt:**\n{prompt}\n\n**message:**\n{message}\n\n**response:**\n{c:?}",
      );
      c
    })),
    Err(err) => Err(err),
  }
}

async fn complete_atc_request(
  message: String,
  frequency: f32,
) -> Option<CommandWithFreq> {
  let prompter = Prompter::new(message);
  let result = prompter.execute().await;
  match result {
    Ok(command) => Some(CommandWithFreq {
      id: command.id,
      frequency,
      reply: command.reply,
      tasks: command.tasks,
    }),
    Err(err) => {
      tracing::error!("Unable to parse command: {}", err);
      None
    }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type", content = "value")]
enum FrontendRequest {
  Voice { data: Vec<u8>, frequency: f32 },
  Text { text: String, frequency: f32 },
  UI(UICommand),
  Connect,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct AudioResponse {
  text: String,
}
