use axum::{
  body::{Body, Bytes},
  http::{Request, StatusCode},
  Router,
};
use serde::Serialize;
use sqlx::PgPool;
use tower::ServiceExt;

use crate::{app::create_app, state::SharedAppState};

pub async fn app_with_pool(pool: PgPool) -> Router {
  let state = SharedAppState::new(pool).await;
  create_app(state)
}

pub async fn post_json<T: Serialize>(app: Router, uri: &str, body: &T) -> (StatusCode, Bytes) {
  let request = Request::builder()
    .method("POST")
    .uri(uri)
    .header("content-type", "application/json")
    .body(Body::from(serde_json::to_vec(body).expect("serialize request body")))
    .expect("build request");

  let response = app.oneshot(request).await.expect("handle request");
  let status = response.status();
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .expect("read response body");
  (status, body)
}
