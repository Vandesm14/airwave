use std::{fs, path::PathBuf};

use clap::Parser;
use mlua::{Lua, LuaSerdeExt, Result, Value};

use engine::entities::airport::{Airport, Gate, Runway, Taxiway, Terminal};

/// View and edit an Airwave world file
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
  /// The file to load
  file: PathBuf,
}

pub fn main() -> Result<()> {
  let args = Cli::parse();
  let script = if let Ok(script) = std::fs::read_to_string(&args.file) {
    script
  } else {
    eprintln!("Failed to load script file: {:?}", args.file);
    std::process::exit(1);
  };

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

  let airport: Airport = lua.from_value(lua.load(script).eval()?)?;
  println!(
    "Loaded airport \"{}\" ({} bytes)",
    airport.id,
    core::mem::size_of_val(&airport)
  );

  let json_path = args.file.to_str().unwrap().replace(".lua", ".json");
  fs::write(json_path.clone(), serde_json::to_string(&airport).unwrap())?;
  println!("Wrote airport \"{}\" to {}", airport.id, json_path);

  Ok(())
}
