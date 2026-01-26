use tokio::signal;

use dotenvy::dotenv;

use koko_pic_api::app::create_app;
use koko_pic_api::db::pool::create_pool;
use koko_pic_api::state::SharedAppState;
use koko_pic_api::utils::init_email_service;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  dotenv().ok();

  tracing_subscriber::fmt::init();

  let pool = create_pool().await.expect("Failed to create database pool");

  sqlx::migrate!("./migrations").run(&pool).await?;

  println!("Database migrations applied successfully");

  let email_service = init_email_service().await?;
  let app_state = SharedAppState::new(pool, email_service).await;
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
