use axum::{response::Html, routing::get, Router};
use sqlx::PgPool;

pub fn router() -> Router<()> {
  Router::new().route("/", get(hello_world_handler))
}

pub fn app_with_state(pool: PgPool) -> Router {
  Router::new().route("/", get(hello_world_handler)).with_state(pool)
}

pub async fn hello_world_handler() -> Html<String> {
  Html("<h1>Hello, World!!</h1>".to_string())
}
