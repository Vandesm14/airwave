// "dev": "cd web-client && vite",
// "build": "cd web-client && vite build",
// "preview": "cd web-client && vite preview",
// "serve": "cd server && cargo run"

use core::{
  net::{IpAddr, Ipv4Addr, SocketAddr},
  str::FromStr,
};
use std::path::PathBuf;

use axum::Router;
use clap::Parser;
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt::init();

  let Cli { dir_path, address } = Cli::parse();

  let listener = match TcpListener::bind(address).await {
    Ok(listener) => listener,
    Err(e) => {
      tracing::error!("Unable to create a TCP listener: {e}");
      std::process::exit(1);
    }
  };

  tracing::info!("Serving '{}' on {address}", dir_path.display());

  let serve = axum::serve(
    listener,
    Router::new()
      .nest_service("/", ServeDir::new(&dir_path))
      .layer(TraceLayer::new_for_http())
      .layer(CorsLayer::permissive()),
  )
  .await;

  match serve {
    Ok(()) => {}
    Err(e) => {
      tracing::error!("{e}");
      std::process::exit(2);
    }
  }
}

#[derive(Parser)]
struct Cli {
  /// The directory to serve files out of.
  #[arg(value_parser = dir_path, default_value = "client-web/dist")]
  dir_path: PathBuf,
  /// The socket address to bind the HTTP server to.
  #[arg(short, long, default_value_t = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080))]
  address: SocketAddr,
}

fn dir_path(s: &str) -> Result<PathBuf, String> {
  // TODO: Use into_ok() instead of unwrap() once the conversion from Infallible
  //       to the never type becomes stable.
  let path = PathBuf::from_str(s).unwrap();
  let metadata = path.metadata().map_err(|e| e.to_string())?;

  if !metadata.is_dir() {
    return Err(format!("{} is not a directory", path.display()));
  }

  Ok(path)
}
