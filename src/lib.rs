use axum::{response::Html, routing::get, Router};
use sqlx::PgPool;

pub mod handlers;
pub mod models;

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
    .nest("/api", api_routes())
    .with_state(AppState { db: pool })
}

fn api_routes() -> Router<AppState> {
  Router::new().nest("/v1", v1_routes())
}

fn v1_routes() -> Router<AppState> {
  Router::new().merge(handlers::user_routes())
}

pub async fn hello_world_handler() -> Html<String> {
  Html("<h1>Hello, World!!</h1>".to_string())
}
