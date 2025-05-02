use std::path::PathBuf;

use clap::Parser;
use rhai::{Engine, EvalAltResult};

/// View and edit an Airwave world file
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
  /// The file to load
  file: PathBuf,
}

pub fn main() -> Result<(), Box<EvalAltResult>> {
  let args = Cli::parse();
  let script = if let Ok(script) = std::fs::read_to_string(&args.file) {
    script
  } else {
    eprintln!("Failed to load script file: {:?}", args.file);
    std::process::exit(1);
  };

  // Create an 'Engine'
  let engine = Engine::new();

  // Run the script - prints "42"
  engine.run(&script)?;

  // Done!
  Ok(())
}
