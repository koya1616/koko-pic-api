use axum::{response::Html, routing::get, Router};

use crate::{
  domains::{picture::rest::picture_routes, user::rest::user_routes},
  state::SharedAppState,
};

pub fn create_app(state: SharedAppState) -> Router {
  Router::new()
    .route("/", get(hello_world_handler))
    .nest("/api/v1", user_routes().merge(picture_routes()))
    .with_state(state)
}

pub async fn hello_world_handler() -> Html<String> {
  Html("<h1>Hello, World!</h1>".to_string())
}
