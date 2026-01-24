use axum::{response::Html, routing::get, Router};
use tokio::signal;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  tracing_subscriber::fmt::init();

  let app = Router::new().route("/", get(hello_world_handler));

  let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();

  println!("Server running on http://0.0.0.0:8000");

  axum::serve(listener, app)
    .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap();

  Ok(())
}

async fn hello_world_handler() -> Html<String> {
  Html("<h1>Hello, World!</h1>".to_string())
}

async fn shutdown_signal() {
  let ctrl_c = async {
    signal::ctrl_c().await.expect("Failed to install Ctrl+C handler");
  };

  #[cfg(unix)]
  let terminate = async {
    signal::unix::signal(signal::unix::SignalKind::terminate())
      .expect("Failed to install signal handler")
      .recv()
      .await;
  };

  #[cfg(not(unix))]
  let terminate = std::future::pending::<()>();

  tokio::select! {
      _ = ctrl_c => {},
      _ = terminate => {},
  }

  println!("Received termination signal, shutting down gracefully...");
}
