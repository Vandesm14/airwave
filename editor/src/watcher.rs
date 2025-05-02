use std::path::PathBuf;

use clap::Parser;
use mlua::{Lua, LuaSerdeExt, Result};

use engine::entities::airport::{Gate, Runway, Taxiway, Terminal};

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

  // Create Taxiway struct from a Lua table
  let taxiway: Taxiway = lua.from_value(lua.load(script).eval()?)?;

  println!("{taxiway:?}");

  Ok(())
}
