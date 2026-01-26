use tokio::signal;

use dotenvy::dotenv;

mod app;
mod db;
mod domains;
mod state;
mod utils;

use app::create_app;
use db::pool::create_pool;
use state::SharedAppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  dotenv().ok();

  tracing_subscriber::fmt::init();

  let pool = create_pool().await.expect("Failed to create database pool");

  sqlx::migrate!("./migrations").run(&pool).await?;

  println!("Database migrations applied successfully");

  let app_state = SharedAppState::new(pool).await;
  let app = create_app(app_state);

  let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();

  println!("Server running on http://0.0.0.0:8000");

  axum::serve(listener, app)
    .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap();

  Ok(())
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
