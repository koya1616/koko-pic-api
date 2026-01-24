use axum::{response::Html, routing::get, Router};
use sqlx::PgPool;
use tokio::signal;

use dotenvy::dotenv;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  dotenv().ok();

  tracing_subscriber::fmt::init();

  let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

  let pool = PgPool::connect(&database_url).await?;

  sqlx::migrate!("./migrations").run(&pool).await?;

  println!("Database migrations applied successfully");

  let app = Router::new()
    .route("/", get(hello_world_handler))
    .with_state(pool.clone());

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
