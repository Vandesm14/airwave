use std::{
  env,
  path::PathBuf,
  str::FromStr,
  sync::{mpsc, Arc},
  thread,
  vec,
  // sync::OnceLock,
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

use engine::{
  add_degrees,
  engine::{Engine, IncomingUpdate, OutgoingReply},
  inverse_degrees, move_point,
  pathfinder::{Node, NodeBehavior, NodeKind},
  structs::{
    Aircraft, AircraftState, Airport, Airspace, Command, CommandWithFreq, Gate,
    Line, Runway, Taxiway, TaxiwayKind, Terminal,
  },
  DOWN, LEFT, NAUTICALMILES_TO_FEET, RIGHT, UP,
};
use tracing::{error, info};

// fn runtime() -> &'static tokio::runtime::Runtime {
//     static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

//     RUNTIME.get_or_init(|| tokio::runtime::Builder::new_multi_thread()
//       .enable_all()
//       .build()
//       .unwrap())
// }

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

fn main() {
  tracing_subscriber::fmt::init();

  dotenv().ok();
  let api_key: Arc<str> = env::var("OPENAI_API_KEY")
    .expect("OPENAI_API_KEY must be set")
    .into();

  let (command_sender, command_receiver) = mpsc::channel::<IncomingUpdate>();
  let (update_sender, update_receiver) = mpsc::channel::<OutgoingReply>();

  let app = Router::new().nest_service("/", ServeDir::new("../dist"));

  let airspace_size = NAUTICALMILES_TO_FEET * 30.0;

  let mut engine = Engine::new(
    command_receiver,
    update_sender.clone(),
    Some(PathBuf::from_str("assets/world.json").unwrap()),
  );

  let mut airport = Airport::new("KSFO".into(), Vec2::new(0.0, 0.0));
  v_pattern_airport(&mut airport);
  airport.cache_waypoints();

  let airspace = Airspace {
    id: "KSFO".into(),
    pos: airport.center,
    size: airspace_size,
    // TODO: remove clone after debugging
    airports: vec![airport.clone()],
  };

  let runway = airport.runways.first().unwrap().clone();
  let mut aircraft = Aircraft::random_to_land(&airspace, 118.5);
  aircraft.state = AircraftState::Taxiing {
    current: Node {
      name: runway.id.clone(),
      kind: NodeKind::Runway,
      behavior: NodeBehavior::GoTo,
      value: runway.pos,
    },
    waypoints: Vec::new(),
  };
  // aircraft.pos = move_point(
  //   runway.start(),
  //   inverse_degrees(runway.heading),
  //   NAUTICALMILES_TO_FEET * 5.0,
  // );
  aircraft.pos = runway.pos;

  aircraft.speed = 0.0;
  aircraft.altitude = 0.0;
  aircraft.heading = runway.heading;

  aircraft.target.speed = aircraft.speed;
  aircraft.target.altitude = aircraft.altitude;
  aircraft.target.heading = aircraft.heading;

  engine.world.aircraft.push(aircraft);

  engine.world.airspaces.push(airspace);
  // engine.spawn_random_aircraft();

  thread::spawn(move || {
    // let runtime = runtime();

    let runtime = tokio::runtime::Builder::new_multi_thread()
      .enable_all()
      .build()
      .unwrap();

    // Without a non-underscore identifier, this would be dropped and hence
    // useless. This ties tokio::spawn, and other executor related stuff, to
    // this runtime.
    let _runtime_guard = runtime.enter();

    runtime.block_on(async move {
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
            tracing::error!(
              "Error during the websocket handshake occurred: {e}"
            );
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
                    info!("received incoming ws");
                    let req =
                      serde_json::from_str::<FrontendRequest>(&string).unwrap();
                    match req {
                      FrontendRequest::Voice {
                        data: bytes,
                        frequency,
                      } => {
                        info!(
                          "received transcription request: {} bytes",
                          bytes.len()
                        );

                        let client = Client::new();
                        let form = reqwest::multipart::Form::new();
                        let form = form.part(
                          "file",
                          Part::bytes(bytes).file_name("audio.wav"),
                        );
                        let form = form.text("model", "whisper-1".to_string());

                        let response = client
                          .post(
                            "https://api.openai.com/v1/audio/transcriptions",
                          )
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
                            header::HeaderValue::from_str(
                              "multipart/form-data",
                            )
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
                            sender
                              .send(IncomingUpdate::Command(result))
                              .unwrap();
                          }
                        }
                      }
                      FrontendRequest::Text {
                        text: string,
                        frequency,
                      } => {
                        ws_sender
                          .send(OutgoingReply::ATCReply(CommandWithFreq {
                            id: "ATC".to_owned(),
                            frequency,
                            reply: string.clone(),
                            tasks: Vec::new(),
                          }))
                          .unwrap();

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
        _ = http_handle => tracing::debug!("http exit"),
        _ = broadcast_handle => tracing::debug!("broadcast exit"),
        _ = ws_handle => tracing::debug!("ws exit"),
      };
    });
  });

  engine.begin_loop();
}

#[allow(dead_code)]
fn cross_roads_airport(airport: &mut Airport, airspace_size: f32) {
  let runway_01 = Runway {
    id: "01".into(),
    pos: Vec2::new(airspace_size * 0.5, airspace_size * 0.5)
      + Vec2::new(750.0, 750.0),
    heading: 10.0,
    length: 7000.0,
  };

  let runway_14 = Runway {
    id: "14".into(),
    pos: Vec2::new(airspace_size * 0.5, airspace_size * 0.5),
    heading: 140.0,
    length: 9000.0,
  };

  let taxiway_b = Taxiway {
    id: "B".into(),
    a: move_point(
      runway_14.start(),
      add_degrees(runway_14.heading, 90.0),
      -500.0,
    ),
    b: move_point(
      runway_14.end(),
      add_degrees(runway_14.heading, 90.0),
      -500.0,
    ),
    kind: TaxiwayKind::Normal,
  };

  let taxiway_c = Taxiway {
    id: "C".into(),
    a: runway_01.end(),
    b: move_point(runway_01.end(), 180.0, 3600.0),
    kind: TaxiwayKind::Normal,
  };

  let taxiway_hs14 = Taxiway {
    id: "HS14".into(),
    a: runway_14.start(),
    b: taxiway_b.a,
    kind: TaxiwayKind::HoldShort("14".into()),
  };

  let taxiway_a1 = Taxiway {
    id: "A1".into(),
    a: move_point(runway_14.start(), runway_14.heading - 90.0, 3250.0),
    b: move_point(taxiway_b.a, runway_14.heading - 90.0, 3250.0),
    kind: TaxiwayKind::Normal,
  };

  let taxiway_a2 = Taxiway {
    id: "A2".into(),
    a: move_point(runway_14.end(), runway_14.heading + 90.0, 2750.0),
    b: move_point(taxiway_b.b, runway_14.heading + 90.0, 2750.0),
    kind: TaxiwayKind::Normal,
  };

  let taxiway_a3 = Taxiway {
    id: "A3".into(),
    a: runway_14.end(),
    b: taxiway_b.b,
    kind: TaxiwayKind::Normal,
  };

  let taxiway_hs01 = Taxiway {
    id: "HS01".into(),
    a: runway_01.start(),
    b: runway_14.end(),
    kind: TaxiwayKind::HoldShort("01".into()),
  };

  let mut terminal_a = Terminal {
    id: 'A',
    a: move_point(taxiway_b.b, runway_14.heading + 90.0, 2750.0),
    b: taxiway_b.b,
    c: move_point(taxiway_b.b, runway_14.heading + 180.0, 1000.0),
    d: move_point(
      move_point(taxiway_b.b, runway_14.heading + 180.0, 1000.0),
      runway_14.heading + 90.0,
      2750.0,
    ),
    gates: Vec::new(),
    apron: Line::default(),
  };
  terminal_a.apron = Line::new(terminal_a.a, terminal_a.b);

  let gate_count = 8;

  for i in 1..=gate_count {
    terminal_a.gates.push(Gate {
      id: format!("A{i}"),
      heading: 0.0,
      pos: move_point(
        move_point(taxiway_b.b, runway_14.heading + 180.0, 1000.0),
        runway_14.heading + 90.0,
        2400.0 / gate_count as f32 * i as f32,
      ),
    });
  }

  airport.add_taxiway(taxiway_a1);
  airport.add_taxiway(taxiway_a2);
  airport.add_taxiway(taxiway_a3);

  airport.add_taxiway(taxiway_b);
  airport.add_taxiway(taxiway_c);

  airport.add_taxiway(taxiway_hs14);
  airport.add_taxiway(taxiway_hs01);

  airport.add_runway(runway_01);
  airport.add_runway(runway_14);

  airport.terminals.push(terminal_a);
}

#[allow(dead_code)]
fn v_pattern_airport(airport: &mut Airport) {
  let runway_20 = Runway {
    id: "20".into(),
    pos: Vec2::new(0.0, 0.0),
    heading: 200.0,
    length: 7000.0,
  };

  let runway_27: Runway = Runway {
    id: "27".into(),
    pos: Vec2::new(-1000.0, 2400.0),
    heading: 270.0,
    length: 7000.0,
  };

  let taxiway_b = Taxiway {
    id: "B".into(),
    a: move_point(
      runway_27.start(),
      add_degrees(runway_27.heading, 90.0),
      500.0,
    ),
    b: move_point(runway_27.end(), add_degrees(runway_27.heading, 90.0), 500.0),
    kind: TaxiwayKind::Normal,
  };

  let taxiway_c = Taxiway {
    id: "C".into(),
    a: move_point(
      runway_20.start(),
      add_degrees(runway_20.heading, 90.0),
      500.0,
    ),
    b: move_point(runway_20.end(), add_degrees(runway_20.heading, 90.0), 500.0),
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
      add_degrees(runway_27.heading, 90.0),
      500.0,
    ),
    kind: TaxiwayKind::HoldShort("27".into()),
  };

  let a = move_point(taxiway_b.b, UP, 500.0);
  let b = move_point(a, RIGHT, 4000.0);
  let c = move_point(b, UP, 1500.0);
  let d = move_point(c, LEFT, 4000.0);
  let mut terminal_a = Terminal {
    id: 'A',
    a,
    b,
    c,
    d,
    gates: Vec::new(),
    apron: Line::default(),
  };
  terminal_a.apron = Line::new(terminal_a.a, terminal_a.b);

  let gates_line_start = move_point(terminal_a.a, UP, 1200.0);
  let gates = 5;
  let padding = 400.0;
  let spacing = 4000.0 / gates as f32;
  for i in 0..gates {
    let gate = Gate {
      id: format!("A{}", i + 1),
      pos: move_point(gates_line_start, RIGHT, spacing * i as f32 + padding),
      heading: 0.0,
    };
    terminal_a.gates.push(gate);
  }

  let tw_a = move_point(a, RIGHT, 200.0);
  let taxiway_a1 = Taxiway {
    id: "A1".into(),
    a: tw_a,
    b: move_point(tw_a, DOWN, 1000.0),
    kind: TaxiwayKind::Normal,
  };

  let tw_a = move_point(a, RIGHT, 2000.0);
  let taxiway_a2 = Taxiway {
    id: "A2".into(),
    a: tw_a,
    b: move_point(tw_a, DOWN, 1000.0),
    kind: TaxiwayKind::Normal,
  };

  let tw_a = move_point(a, RIGHT, 3800.0);
  let taxiway_a3 = Taxiway {
    id: "A3".into(),
    a: tw_a,
    b: move_point(tw_a, DOWN, 1000.0),
    kind: TaxiwayKind::Normal,
  };

  let taxiway_d1 = Taxiway {
    id: "D1".into(),
    a: taxiway_c.b,
    b: runway_20.end(),
    kind: TaxiwayKind::Normal,
  };

  let taxiway_d2 = Taxiway {
    id: "D2".into(),
    a: move_point(taxiway_c.b, inverse_degrees(runway_20.heading), 1000.0),
    b: move_point(runway_20.end(), inverse_degrees(runway_20.heading), 1000.0),
    kind: TaxiwayKind::Normal,
  };

  let taxiway_d3 = Taxiway {
    id: "D3".into(),
    a: move_point(taxiway_c.b, inverse_degrees(runway_20.heading), 2500.0),
    b: move_point(runway_20.end(), inverse_degrees(runway_20.heading), 2500.0),
    kind: TaxiwayKind::Normal,
  };

  airport.add_runway(runway_20);
  airport.add_runway(runway_27);

  airport.add_taxiway(taxiway_a1);
  airport.add_taxiway(taxiway_a2);
  airport.add_taxiway(taxiway_a3);
  airport.add_taxiway(taxiway_b);
  airport.add_taxiway(taxiway_c);
  airport.add_taxiway(taxiway_d1);
  airport.add_taxiway(taxiway_d2);
  airport.add_taxiway(taxiway_d3);
  airport.add_taxiway(taxiway_hs_20);
  airport.add_taxiway(taxiway_hs_27);

  airport.terminals.push(terminal_a);
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
        match serde_json::from_str::<Command>(text) {
          Ok(reply) => {
            return Some(CommandWithFreq {
              id: reply.id,
              frequency: freq,
              reply: reply.reply,
              tasks: reply.tasks,
            })
          }
          Err(e) => {
            error!("failed to parse command: {} (raw: {})", e, text);
          }
        }
      }
    }
  }

  None
}
