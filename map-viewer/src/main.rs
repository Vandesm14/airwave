use std::{
  io::Write,
  path::{Path, PathBuf},
  sync::mpsc::Receiver,
  thread,
};

use clap::Parser;
use engine::structs::World;
use map_viewer::Draw;
use nannou::{
  color::*,
  prelude::{App, Frame, Update},
};

use nannou_egui::{egui, Egui};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
  path: String,
}

fn main() {
  nannou::app(model).update(update).run();
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Settings {
  pub scale: f32,
  pub pos: f32,
}

impl Settings {
  pub fn new() -> Self {
    Self {
      scale: 1.0,
      pos: 0.0,
    }
  }
}

fn read_world_file(path: &Path) -> Result<World, String> {
  match std::fs::File::open(path) {
    Ok(file) => match serde_json::de::from_reader::<_, World>(file) {
      Ok(world) => Ok(world),
      Err(e) => {
        return Err(format!(
          "Failed to parse world file at {}: {}",
          path.display(),
          e
        ))
      }
    },
    Err(e) => Err(format!(
      "Failed to open world file at {}: {}",
      path.display(),
      e
    )),
  }
}

pub struct Model {
  path: PathBuf,
  settings: Settings,
  world: World,
  egui: Egui,

  update_receiver: Receiver<Result<notify::Event, notify::Error>>,
}

impl Model {
  fn load_world(&mut self) {
    match read_world_file(&self.path) {
      Ok(w) => {
        self.world = w;
      }
      Err(e) => {
        eprintln!("{}", e);
      }
    }
  }
}

fn model(app: &App) -> Model {
  let window_id = app
    .new_window()
    .view(view)
    .raw_event(raw_window_event)
    .build()
    .unwrap();
  let window = app.window(window_id).unwrap();
  let egui = Egui::from_window(&window);

  let args = Args::parse();

  let world = World::default();
  let path = PathBuf::from(args.path);

  let (tx, rx) = std::sync::mpsc::channel();
  let thread_path = path.clone();

  let mut model = Model {
    path,
    settings: Settings::new(),
    world,
    egui,
    update_receiver: rx,
  };

  model.load_world();

  thread::spawn(move || {
    let mut watcher = RecommendedWatcher::new(tx, Config::default())
      .expect("Failed to create file watcher");

    watcher
      .watch(&thread_path, RecursiveMode::Recursive)
      .expect("failed to watch");

    println!("Watching for changes in {:?}", thread_path);

    loop {
      std::thread::sleep(std::time::Duration::from_secs(1));
    }
  });

  model
}

fn update(_app: &App, model: &mut Model, update: Update) {
  if let Ok(Ok(notify::Event {
    kind: notify::EventKind::Modify(..),
    ..
  })) = model.update_receiver.try_recv()
  {
    model.load_world();
  }

  let egui = &mut model.egui;

  egui.set_elapsed_time(update.since_start);
  let ctx = egui.begin_frame();

  let x = egui::TopBottomPanel::top("my_top_panel");
  x.show(&ctx, |ui| {
    if ui.button("Save").clicked() {
      let path = model.path.clone();
      let world = model.world.clone();
      std::thread::spawn(move || {
        let string = serde_json::ser::to_string_pretty(&world).unwrap();
        match std::fs::File::create(&path) {
          Ok(mut file) => {
            println!("Saved world file to {}", path.display());
            file.write_all(string.as_bytes()).unwrap();
          }
          Err(e) => {
            eprintln!("Failed to save world file: {}", e);
          }
        }
      });
    }
  });

  egui::SidePanel::new(egui::panel::Side::Left, "Settings").show(&ctx, |ui| {
    ui.label("Scale:");
    ui.add(
      egui::widgets::DragValue::new(&mut model.settings.scale).speed(0.05),
    );

    ui.collapsing("Objects", |ui| {
      ui.collapsing("Airspaces", |ui| {
        for airspace in model.world.airspaces.iter_mut() {
          ui.collapsing("Airports", |ui| {
            for airport in airspace.airports.iter_mut() {
              ui.collapsing(airport.id.as_str(), |ui| {
                ui.collapsing("Runways", |ui| {
                  airport.runways.iter_mut().for_each(|runway| {
                    ui.label("Runway:");
                    ui.add(egui::widgets::TextEdit::singleline(&mut runway.id));
                    ui.label("X:");
                    ui.add(egui::widgets::DragValue::new(&mut runway.pos.x));
                    ui.label("Y:");
                    ui.add(egui::widgets::DragValue::new(&mut runway.pos.y));
                    ui.label("Heading:");
                    ui.add(egui::widgets::DragValue::new(&mut runway.heading));
                  });
                });
              });
            }
          });
        }
      });
    });
  });
}

fn raw_window_event(
  _app: &App,
  model: &mut Model,
  event: &nannou::winit::event::WindowEvent,
) {
  // Let egui handle things like keyboard and mouse input.
  model.egui.handle_raw_event(event);
}

fn view(app: &App, model: &Model, frame: Frame) {
  let draw = app.draw();

  draw.background().color(BLACK);
  const FEET_TO_PIXELS: f32 = 0.003;
  let scale = model.settings.scale * FEET_TO_PIXELS;

  for airspace in model.world.airspaces.iter() {
    for airport in airspace.airports.iter() {
      airport.taxiways.iter().for_each(|taxiway| {
        taxiway.draw(&draw, scale);
      });

      airport.terminals.iter().for_each(|terminal| {
        terminal.draw(&draw, scale);
      });

      airport.runways.iter().for_each(|taxiway| {
        taxiway.draw(&draw, scale);
      });
    }
  }

  // TODO: draw a scale for 1, 10, 100, and 1000 feet

  draw.to_frame(app, &frame).unwrap();
  model.egui.draw_to_frame(&frame).unwrap();
}
