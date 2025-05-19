use std::{collections::HashMap, fs, path::Path};

use crate::{
  compile::{setup_lua, try_compile_airport},
  entities::airport::Airport,
  geometry::Translate,
};

pub fn airport_asset_path() -> &'static Path {
  Path::new("assets/airports")
}

#[derive(Debug, Clone, Default)]
pub struct Assets {
  pub airports: HashMap<String, Airport>,
}

pub fn load_assets() -> Assets {
  let mut assets = Assets::default();

  // Compile any assets that don't have a matching json file.
  let lua_files: Vec<_> = fs::read_dir(airport_asset_path())
    .unwrap()
    .flatten()
    .filter(|f| f.file_name().to_str().unwrap().ends_with(".lua"))
    .map(|f| {
      (
        f.path(),
        f.file_name().to_string_lossy().replace(".lua", ""),
      )
    })
    .collect();

  let json_files: Vec<_> = fs::read_dir(airport_asset_path())
    .unwrap()
    .flatten()
    .filter(|f| f.file_name().to_str().unwrap().ends_with(".json"))
    .map(|f| f.file_name().to_string_lossy().replace(".json", ""))
    .collect();

  let lua = setup_lua();
  for (path, lua_filename) in lua_files {
    if !json_files.contains(&lua_filename) {
      match try_compile_airport(&lua, &path) {
        Ok(_) => {
          tracing::info!("Compiled: {:?}", path);
        }
        Err(e) => tracing::error!("Failed to compile: {:?}: {:?}", path, e),
      }
    }
  }

  // Gather all compiled assets.
  let json_files: Vec<_> = fs::read_dir(airport_asset_path())
    .unwrap()
    .flatten()
    .filter(|f| f.file_name().to_str().unwrap().ends_with(".json"))
    .collect();
  for path in json_files {
    match fs::read_to_string(path.path()) {
      Ok(content) => {
        match serde_json::from_str::<Airport>(&content) {
          Ok(mut airport) => {
            airport.translate(airport.center * -1.0);
            airport.extend_all();
            airport.calculate_waypoints();

            let name = path.file_name();
            let name = name.to_str().unwrap().replace(".json", "");
            tracing::info!(
              "Loaded airport \"{}\" from {}",
              airport.id,
              path.file_name().to_str().unwrap()
            );
            assets.airports.insert(name.to_owned(), airport);
          }
          Err(e) => {
            tracing::error!("Failed to read {:?}: {:?}", path.file_name(), e);
          }
        };
      }
      Err(e) => {
        tracing::error!(
          "Failed to read airport file {:?}: {:?}",
          path.file_name(),
          e
        );
      }
    }
  }

  assets
}
