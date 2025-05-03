use std::{fs, path::PathBuf, sync::mpsc, thread};

use clap::Parser;
use editor::viewer::start_app;
use glam::Vec2;
use mlua::{
  FromLua, Lua, LuaSerdeExt, MetaMethod, Result, UserData, UserDataMethods,
  Value,
};
use notify::{Event, RecursiveMode, Watcher};

use engine::{
  entities::airport::{Airport, Gate, Runway, Taxiway, Terminal},
  move_point,
};

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

pub fn try_compile_airport(
  lua: &Lua,
  path: &PathBuf,
  sender: Option<mpsc::Sender<Airport>>,
) -> Result<()> {
  let script = if let Ok(script) = std::fs::read_to_string(path) {
    script
  } else {
    eprint!("Failed to load script file: {:?}", path);
    std::process::exit(1);
  };

  // This happens due to an issue with file watching. So I think it's fine if we
  // ignore blank files altogether anyway.
  if script.is_empty() {
    return Ok(());
  }

  let airport: Airport = lua.from_value(lua.load(script).eval()?)?;
  if let Some(send) = sender {
    let _ = send.send(airport.clone());
  }

  let json_path = path.to_str().unwrap().replace(".lua", ".json");
  let json_string = serde_json::to_string(&airport).unwrap();
  let json_size = json_string.len();
  fs::write(json_path.clone(), json_string)?;

  println!(
    "Wrote airport \"{}\" to {} ({} bytes)",
    airport.id, json_path, json_size
  );

  Ok(())
}

pub fn compile_airport(
  lua: &Lua,
  path: &PathBuf,
  sender: Option<mpsc::Sender<Airport>>,
) {
  match try_compile_airport(lua, path, sender) {
    Ok(_) => {}
    Err(e) => eprintln!("Error compiling: {:?}", e),
  };
}

#[derive(Debug, Clone, Copy)]
pub struct LuaVec2 {
  inner: Vec2,
}

// We can implement `FromLua` trait for our `Vec2` to return a copy
impl FromLua for LuaVec2 {
  fn from_lua(value: Value, _: &Lua) -> Result<Self> {
    match value {
      Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
      _ => unreachable!(),
    }
  }
}

impl UserData for LuaVec2 {
  fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
    methods.add_method("lerp", |_, a, (b, s): (LuaVec2, f32)| {
      Ok(LuaVec2::from(a.inner.lerp(b.inner, s)))
    });
    methods
      .add_method("distance", |_, a, b: LuaVec2| Ok(a.inner.distance(b.inner)));
    methods.add_method("distance_squared", |_, a, b: LuaVec2| {
      Ok(a.inner.distance_squared(b.inner))
    });
    methods.add_method("length", |_, a, _: ()| Ok(a.inner.length()));
    methods.add_method("midpoint", |_, a, b: LuaVec2| {
      Ok(LuaVec2::from(a.inner.midpoint(b.inner)))
    });
    methods.add_method("angle_between", |_, a, b: LuaVec2| {
      Ok(a.inner.angle_to(b.inner))
    });
    methods.add_method("move", |_, a, (degrees, length): (f32, f32)| {
      Ok(LuaVec2::from(move_point(a.inner, degrees, length)))
    });
    methods.add_method("into", |_, a, _: ()| Ok(vec![a.inner.x, a.inner.y]));

    methods
      .add_meta_function(MetaMethod::Add, |_, (a, b): (LuaVec2, LuaVec2)| {
        Ok(LuaVec2::from(a.inner + b.inner))
      });
  }

  fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
    fields.add_field_method_get("x", |_, vec2: &LuaVec2| Ok(vec2.inner.x));
    fields.add_field_method_get("y", |_, vec2: &LuaVec2| Ok(vec2.inner.y));
  }
}

impl From<Vec2> for LuaVec2 {
  fn from(value: Vec2) -> Self {
    Self { inner: value }
  }
}

impl From<LuaVec2> for Vec2 {
  fn from(value: LuaVec2) -> Self {
    value.inner
  }
}

impl LuaVec2 {
  pub fn new(x: f32, y: f32) -> Self {
    Self {
      inner: Vec2::new(x, y),
    }
  }
}

pub fn main() -> Result<()> {
  let (sender, receiver) = mpsc::channel::<Airport>();

  thread::spawn(|| {
    let args = Cli::parse();

    let lua = Lua::new();
    let globals = lua.globals();

    let assert_airport = lua
      .create_function(|lua, value: Value| {
        lua.from_value::<Airport>(value.clone()).map(|_| value)
      })
      .unwrap();
    let assert_runway = lua
      .create_function(|lua, value: Value| {
        lua.from_value::<Runway>(value.clone()).map(|_| value)
      })
      .unwrap();
    let assert_taxiway = lua
      .create_function(|lua, value: Value| {
        lua.from_value::<Taxiway>(value.clone()).map(|_| value)
      })
      .unwrap();
    let assert_gate = lua
      .create_function(|lua, value: Value| {
        lua.from_value::<Gate>(value.clone()).map(|_| value)
      })
      .unwrap();
    let assert_terminal = lua
      .create_function(|lua, value: Value| {
        lua.from_value::<Terminal>(value.clone()).map(|_| value)
      })
      .unwrap();

    globals.set("airport", assert_airport).unwrap();
    globals.set("runway", assert_runway).unwrap();
    globals.set("taxiway", assert_taxiway).unwrap();
    globals.set("gate", assert_gate).unwrap();
    globals.set("terminal", assert_terminal).unwrap();

    let vec2_constructor = lua
      .create_function(|_, (x, y): (f32, f32)| Ok(LuaVec2::new(x, y)))
      .unwrap();
    globals.set("vec2", vec2_constructor).unwrap();

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

  start_app(receiver);

  Ok(())
}
