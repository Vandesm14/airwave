pub mod viewer;

use std::{path::PathBuf, sync::mpsc, thread};

use clap::Parser;
use notify::{Event, RecursiveMode, Watcher};

use engine::{
  compile::{compile_airport, setup_lua},
  entities::airport::Airport,
};
use viewer::start_app;

/// View and edit an Airwave world file
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
  /// The file to load
  path: PathBuf,

  /// Watch for changes
  #[arg(short, long)]
  watch: bool,

  /// View changes in a GUI
  #[arg(short, long)]
  view: bool,
}

pub fn main() -> mlua::Result<()> {
  let (sender, receiver) = mpsc::channel::<Airport>();

  let thread = thread::spawn(|| {
    let args = Cli::parse();
    let lua = setup_lua();

    if args.watch {
      let (tx, rx) = mpsc::channel::<notify::Result<Event>>();

      compile_airport(&lua, &args.path, args.view.then(|| sender.clone()));

      let mut watcher = notify::recommended_watcher(tx).unwrap();
      watcher.watch(&args.path, RecursiveMode::Recursive).unwrap();
      // Block forever, printing out events as they come in
      for res in rx {
        match res {
          Ok(event) => {
            if matches!(event.kind, notify::EventKind::Modify(..)) {
              compile_airport(
                &lua,
                &args.path,
                args.view.then(|| sender.clone()),
              );
            }
          }
          Err(e) => eprintln!("watch error: {:?}", e),
        }
      }
    } else {
      compile_airport(&lua, &args.path, args.view.then_some(sender));
    }
  });

  let args = Cli::parse();
  if args.view {
    start_app(receiver);
  } else {
    thread.join().unwrap();
  }

  Ok(())
}
