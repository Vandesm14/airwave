use std::{
  env,
  sync::{mpsc, Arc},
  vec,
};

use async_openai::types::{
  ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
  ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageContent,
  CreateChatCompletionRequest,
};
use axum::Router;
use dotenv::dotenv;
use futures_util::{stream::SplitSink, SinkExt, StreamExt, TryStreamExt};
use glam::Vec2;
use reqwest::{header, multipart::Part, Client};
use serde::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use tower_http::services::ServeDir;

use server::{
  add_degrees,
  engine::{Engine, IncomingUpdate, OutgoingReply},
  heading_to_degrees, move_point,
  structs::{
    Command, CommandWithFreq, Gate, Runway, Task, Taxiway, TaxiwayKind,
    Terminal,
  },
  FEET_PER_UNIT, NAUTICALMILES_TO_FEET,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type", content = "value")]
enum FrontendRequest {
  Voice { data: Vec<u8>, frequency: f32 },
  Text { text: String, frequency: f32 },
  Connect,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct AudioResponse {
  text: String,
}

#[tokio::main]
async fn main() {
  dotenv().ok();
  let api_key: Arc<str> = env::var("OPENAI_API_KEY")
    .expect("OPENAI_API_KEY must be set")
    .into();

  let (command_sender, command_receiver) = mpsc::channel::<IncomingUpdate>();
  let (update_sender, update_receiver) = mpsc::channel::<OutgoingReply>();

  let app = Router::new().nest_service("/", ServeDir::new("../dist"));

  let airspace_size = NAUTICALMILES_TO_FEET * FEET_PER_UNIT * 40.0;

  let mut engine = Engine::new(
    command_receiver,
    update_sender.clone(),
    airspace_size,
    118.5,
  );
  let engine_handle = tokio::spawn(async move {
    let runway_20 = Runway {
      id: "20".into(),
      pos: Vec2::new(airspace_size * 0.5, airspace_size * 0.5),
      heading: 200.0,
      length: 7000.0,
    };

    let runway_27: Runway = Runway {
      id: "27".into(),
      pos: Vec2::new(
        airspace_size * 0.5 - FEET_PER_UNIT * 1000.0,
        airspace_size * 0.5 - FEET_PER_UNIT * 2400.0,
      ),
      heading: 270.0,
      length: 7000.0,
    };

    let taxiway_b = Taxiway {
      id: "B".into(),
      a: move_point(
        runway_27.start(),
        add_degrees(heading_to_degrees(runway_27.heading), 90.0),
        FEET_PER_UNIT * 500.0,
      ),
      b: move_point(
        runway_27.end(),
        add_degrees(heading_to_degrees(runway_27.heading), 90.0),
        FEET_PER_UNIT * 500.0,
      ),
      kind: TaxiwayKind::Normal,
    };

    let taxiway_c = Taxiway {
      id: "C".into(),
      a: move_point(
        runway_20.start(),
        add_degrees(heading_to_degrees(runway_20.heading), 90.0),
        FEET_PER_UNIT * 500.0,
      ),
      b: move_point(
        runway_20.end(),
        add_degrees(heading_to_degrees(runway_20.heading), 90.0),
        FEET_PER_UNIT * 500.0,
      ),
      kind: TaxiwayKind::Normal,
    };

    let taxiway_hs_20 = Taxiway {
      id: "HS20".into(),
      a: runway_20.start(),
      b: taxiway_c.a,
      kind: TaxiwayKind::HoldShort("20".into()),
    };

    let taxiway_hs_27 = Taxiway {
      id: "HS27".into(),
      a: runway_27.start(),
      b: move_point(
        runway_27.start(),
        add_degrees(heading_to_degrees(runway_27.heading), 90.0),
        FEET_PER_UNIT * 500.0,
      ),
      kind: TaxiwayKind::HoldShort("27".into()),
    };

    let taxiway_exit_27 = Taxiway {
      id: "E27-1".into(),
      a: runway_27.end(),
      b: move_point(
        runway_27.end(),
        add_degrees(heading_to_degrees(runway_27.heading), 90.0),
        FEET_PER_UNIT * 500.0,
      ),
      kind: TaxiwayKind::HoldShort("09".into()),
    };

    let a = move_point(taxiway_b.b, 270.0, FEET_PER_UNIT * 500.0);
    let b = move_point(a, 0.0, FEET_PER_UNIT * 4000.0);
    let c = move_point(b, 270.0, FEET_PER_UNIT * 1500.0);
    let d = move_point(c, 180.0, FEET_PER_UNIT * 4000.0);
    let mut terminal_a = Terminal {
      id: 'A',
      a,
      b,
      c,
      d,
      gates: Vec::new(),
    };

    let gates_line_start =
      move_point(terminal_a.a, 270.0, FEET_PER_UNIT * 1200.0);
    let gates = 5;
    let padding = 400.0;
    let spacing = 4000.0 / gates as f32;
    for i in 0..gates {
      let gate = Gate {
        id: format!("A{i}"),
        pos: move_point(
          gates_line_start,
          0.0,
          spacing * i as f32 * FEET_PER_UNIT + padding * FEET_PER_UNIT,
        ),
        heading: 0.0,
      };
      terminal_a.gates.push(gate);
    }

    let tw_a = move_point(a, 0.0, FEET_PER_UNIT * 200.0);
    let taxiway_a1 = Taxiway {
      id: "A1".into(),
      a: tw_a,
      b: move_point(tw_a, 90.0, FEET_PER_UNIT * 500.0),
      kind: TaxiwayKind::Normal,
    };

    let tw_a = move_point(a, 0.0, FEET_PER_UNIT * 2000.0);
    let taxiway_a2 = Taxiway {
      id: "A2".into(),
      a: tw_a,
      b: move_point(tw_a, 90.0, FEET_PER_UNIT * 500.0),
      kind: TaxiwayKind::Normal,
    };

    let tw_a = move_point(a, 0.0, FEET_PER_UNIT * 3800.0);
    let taxiway_a3 = Taxiway {
      id: "A3".into(),
      a: tw_a,
      b: move_point(tw_a, 90.0, FEET_PER_UNIT * 500.0),
      kind: TaxiwayKind::Normal,
    };

    engine.runways.push(runway_20);
    engine.runways.push(runway_27);

    engine.taxiways.push(taxiway_a1);
    engine.taxiways.push(taxiway_a2);
    engine.taxiways.push(taxiway_a3);
    engine.taxiways.push(taxiway_b);
    engine.taxiways.push(taxiway_c);
    engine.taxiways.push(taxiway_hs_20);
    engine.taxiways.push(taxiway_hs_27);
    engine.taxiways.push(taxiway_exit_27);

    engine.terminals.push(terminal_a);

    engine.spawn_random_aircraft();
    let aircraft = engine.aircraft.last().unwrap();
    engine.execute_command(CommandWithFreq {
      id: aircraft.callsign.clone(),
      frequency: aircraft.frequency,
      reply: "Doing the thing.".into(),
      tasks: vec![Task::TaxiRunway {
        runway: "20".into(),
        waypoints: vec![
          ("A3".into(), false),
          ("B".into(), false),
          ("C".into(), true),
        ],
      }],
    });
    engine.begin_loop();
  });

  let listener = TcpListener::bind("0.0.0.0:8000").await.unwrap();
  let http_handle = tokio::spawn(async move {
    axum::serve(listener, app).await.unwrap();
  });

  let (give_streams, take_streams) =
    mpsc::channel::<SplitSink<WebSocketStream<TcpStream>, Message>>();

  let ws_handle = tokio::spawn(async move {
    let try_socket = TcpListener::bind("0.0.0.0:9001").await;
    let listener = try_socket.expect("ws server failed to bind");

    while let Ok((stream, _)) = listener.accept().await {
      let ws_stream = tokio_tungstenite::accept_async(stream).await;
      if let Err(e) = ws_stream {
        eprintln!("Error during the websocket handshake occurred: {e}");
        return;
      }

      let ws_stream = ws_stream.unwrap();

      let (write, read) = ws_stream.split();
      give_streams.send(write).unwrap();

      let sender = command_sender.clone();
      let ws_sender = update_sender.clone();
      let api_key = api_key.clone();

      tokio::spawn(async move {
        read
          .try_for_each(|message| {
            let sender = sender.clone();
            let api_key = api_key.clone();

            let ws_sender = ws_sender.clone();
            async move {
              if let Message::Text(string) = message {
                dbg!("received incoming ws");
                let req =
                  serde_json::from_str::<FrontendRequest>(&string).unwrap();
                match req {
                  FrontendRequest::Voice {
                    data: bytes,
                    frequency,
                  } => {
                    dbg!("received transcription request", bytes.len());

                    let client = Client::new();
                    let form = reqwest::multipart::Form::new();
                    let form = form
                      .part("file", Part::bytes(bytes).file_name("audio.wav"));
                    let form = form.text("model", "whisper-1".to_string());

                    let response = client
                      .post("https://api.openai.com/v1/audio/transcriptions")
                      .multipart(form)
                      .header(
                        header::AUTHORIZATION,
                        header::HeaderValue::from_str(&format!(
                          "Bearer {}",
                          &api_key.clone()
                        ))
                        .unwrap(),
                      )
                      .header(
                        header::CONTENT_TYPE,
                        header::HeaderValue::from_str("multipart/form-data")
                          .unwrap(),
                      )
                      .send()
                      .await
                      .unwrap();

                    let text = response.text().await.unwrap();
                    if let Ok(reply) =
                      serde_json::from_str::<AudioResponse>(&text)
                    {
                      ws_sender
                        .send(OutgoingReply::ATCReply(CommandWithFreq {
                          id: "ATC".to_owned(),
                          frequency,
                          reply: reply.text.clone(),
                          tasks: Vec::new(),
                        }))
                        .unwrap();

                      if let Some(result) =
                        complete_atc_request(reply.text, frequency).await
                      {
                        sender.send(IncomingUpdate::Command(result)).unwrap();
                      }
                    }
                  }
                  FrontendRequest::Text {
                    text: string,
                    frequency,
                  } => {
                    if let Some(result) =
                      complete_atc_request(string, frequency).await
                    {
                      sender.send(IncomingUpdate::Command(result)).unwrap();
                    }
                  }
                  FrontendRequest::Connect => {
                    sender.send(IncomingUpdate::Connect).unwrap();
                  }
                }
              }

              Ok(())
            }
          })
          .await
          .unwrap();
      });
    }
  });

  let broadcast_handle = tokio::spawn(async move {
    let mut streams: Vec<SplitSink<WebSocketStream<TcpStream>, Message>> =
      Vec::new();

    loop {
      for stream in take_streams.try_iter() {
        streams.push(stream);
      }

      if let Ok(update) = update_receiver.try_recv() {
        for write in streams.iter_mut() {
          let _ = write
            .send(Message::Text(serde_json::to_string(&update).unwrap()))
            .await;
        }
      }
    }
  });

  tokio::select! {
    _ = engine_handle => println!("engine exit"),
    _ = http_handle => println!("http exit"),
    _ = broadcast_handle => println!("broadcast exit"),
    _ = ws_handle => println!("ws exit"),
  };
}

async fn complete_atc_request(
  string: String,
  freq: f32,
) -> Option<CommandWithFreq> {
  let client = async_openai::Client::new();
  let request = CreateChatCompletionRequest {
    messages: vec![
      ChatCompletionRequestMessage::System(
        ChatCompletionRequestSystemMessage {
          content: include_str!("prompt.txt").to_owned(),
          name: None,
        },
      ),
      ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
        content: ChatCompletionRequestUserMessageContent::Text(string),
        name: None,
      }),
    ],
    model: "gpt-4o-mini".into(),
    ..Default::default()
  };

  let response = client.chat().create(request).await;
  if let Ok(response) = response {
    if let Some(choice) = response.choices.first() {
      if let Some(ref text) = choice.message.content {
        if let Ok(reply) = serde_json::from_str::<Command>(text) {
          return Some(CommandWithFreq {
            id: reply.id,
            frequency: freq,
            reply: reply.reply,
            tasks: reply.tasks,
          });
        }
      }
    }
  }

  None
}
