use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;

use axum::{
  Router,
  extract::Path,
  http::{HeaderValue, StatusCode},
  response::{IntoResponse, Response},
  routing::get,
};
use clap::Parser;
use rust_embed::Embed;
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

#[derive(Embed)]
#[folder = "../client-web/dist"]
struct Asset;

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt::init();

  let Cli { address } = Cli::parse();

  let listener = match TcpListener::bind(address).await {
    Ok(listener) => listener,
    Err(e) => {
      tracing::error!("Unable to create a TCP listener: {e}");
      std::process::exit(1);
    }
  };

  tracing::info!("Serving on {address}");

  let serve = axum::serve(
    listener,
    Router::new()
      .layer(TraceLayer::new_for_http())
      .layer(CorsLayer::permissive())
      .route("/*path", get(my_file_server)),
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

enum MyResponse {
  Html(String),
  Css(String),
  Js(String),
}

impl IntoResponse for MyResponse {
  fn into_response(self) -> axum::response::Response {
    match self {
      MyResponse::Html(x) => {
        let mut response = Response::new(x.into());
        response
          .headers_mut()
          .append("content-type", HeaderValue::from_str("text/html").unwrap());

        response
      }
      MyResponse::Css(x) => {
        let mut response = Response::new(x.into());
        response
          .headers_mut()
          .append("content-type", HeaderValue::from_str("text/css").unwrap());

        response
      }
      MyResponse::Js(x) => {
        let mut response = Response::new(x.into());
        response.headers_mut().append(
          "content-type",
          HeaderValue::from_str("text/javascript").unwrap(),
        );

        response
      }
    }
  }
}

async fn my_file_server(Path(path): Path<String>) -> impl IntoResponse {
  match Asset::get(&path) {
    Some(file) => {
      // Bytes::copy_from_slice(&file.data)
      match String::from_utf8(file.data.to_vec()) {
        Ok(str) => {
          if let Some(ext) = PathBuf::from(path).extension() {
            match ext.to_str().unwrap() {
              "html" => Ok(MyResponse::Html(str)),
              "css" => Ok(MyResponse::Css(str)),
              "js" => Ok(MyResponse::Js(str)),
              _ => Err(StatusCode::IM_A_TEAPOT),
            }
          } else {
            Err(StatusCode::EXPECTATION_FAILED)
          }
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
      }
    }
    None => Err(StatusCode::NOT_FOUND),
  }
}

#[derive(Parser)]
struct Cli {
  /// The socket address to bind the HTTP server to.
  #[arg(short, long, default_value_t = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080))]
  address: SocketAddr,
}
