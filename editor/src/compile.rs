use std::{fs, path::PathBuf, sync::mpsc};

use clap::Parser;
use mlua::{Lua, LuaSerdeExt, Result, Value};
use notify::{Event, RecursiveMode, Watcher};

use engine::entities::airport::{Airport, Gate, Runway, Taxiway, Terminal};

/// View and edit an Airwave world file
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
  /// The file to load
  path: PathBuf,

  /// Watch for changes
  #[arg(short, long)]
  watch: bool,
}

pub fn compile_airport(lua: &Lua, path: &PathBuf) -> Result<()> {
  let script = if let Ok(script) = std::fs::read_to_string(path) {
    script
  } else {
    tracing::error!("Failed to load script file: {:?}", path);
    std::process::exit(1);
  };

  // This happens due to an issue with file watching. So I think it's fine if we
  // ignore blank files altogether anyway.
  if script.is_empty() {
    return Ok(());
  }

  let airport: Airport = lua.from_value(lua.load(script).eval()?)?;

  let json_path = path.to_str().unwrap().replace(".lua", ".json");
  let json_string = serde_json::to_string(&airport).unwrap();
  let json_size = json_string.len();
  fs::write(json_path.clone(), json_string)?;

  tracing::info!(
    "Wrote airport \"{}\" to {} ({} bytes)",
    airport.id,
    json_path,
    json_size
  );

  Ok(())
}

pub fn handle_compile_airport(lua: &Lua, path: &PathBuf) {
  match compile_airport(lua, path) {
    Ok(_) => {}
    Err(e) => tracing::error!("Error compiling: {:?}", e),
  };
}

pub fn main() -> Result<()> {
  tracing_subscriber::fmt::init();

  let args = Cli::parse();

  let lua = Lua::new();
  let globals = lua.globals();

  let assert_airport = lua.create_function(|lua, value: Value| {
    lua.from_value::<Airport>(value.clone()).map(|_| value)
  })?;
  let assert_runway = lua.create_function(|lua, value: Value| {
    lua.from_value::<Runway>(value.clone()).map(|_| value)
  })?;
  let assert_taxiway = lua.create_function(|lua, value: Value| {
    lua.from_value::<Taxiway>(value.clone()).map(|_| value)
  })?;
  let assert_gate = lua.create_function(|lua, value: Value| {
    lua.from_value::<Gate>(value.clone()).map(|_| value)
  })?;
  let assert_terminal = lua.create_function(|lua, value: Value| {
    lua.from_value::<Terminal>(value.clone()).map(|_| value)
  })?;

  globals.set("assert_airport", assert_airport)?;
  globals.set("assert_runway", assert_runway)?;
  globals.set("assert_taxiway", assert_taxiway)?;
  globals.set("assert_gate", assert_gate)?;
  globals.set("assert_terminal", assert_terminal)?;

  if args.watch {
    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();

    handle_compile_airport(&lua, &args.path);

    let mut watcher = notify::recommended_watcher(tx).unwrap();
    watcher.watch(&args.path, RecursiveMode::Recursive).unwrap();
    // Block forever, printing out events as they come in
    for res in rx {
      match res {
        Ok(event) => {
          if matches!(event.kind, notify::EventKind::Modify(..)) {
            handle_compile_airport(&lua, &args.path);
          }
        }
        Err(e) => tracing::error!("watch error: {:?}", e),
      }
    }
  } else {
    handle_compile_airport(&lua, &args.path);
  }

  Ok(())
}
