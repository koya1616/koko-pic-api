use axum::{response::Html, routing::get, Router};
use sqlx::PgPool;

pub mod handlers;
pub mod models;
pub mod utils;

#[derive(Clone)]
pub struct AppState {
  pub db: PgPool,
}

pub fn router() -> Router<()> {
  Router::new().route("/", get(hello_world_handler))
}

pub fn app_with_state(pool: PgPool) -> Router {
  Router::new()
    .route("/", get(hello_world_handler))
    .nest("/api/v1", handlers::user_routes())
    .with_state(AppState { db: pool })
}

pub async fn hello_world_handler() -> Html<String> {
  Html("<h1>Hello, World!!</h1>".to_string())
}
