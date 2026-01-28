use axum::{
  body::{Body, Bytes},
  http::{Request, StatusCode},
  Router,
};
use serde::Serialize;
use sqlx::PgPool;
use tower::ServiceExt;

use crate::{app::create_app, email::EmailService, state::SharedAppState, storage::S3Storage};

async fn create_test_email_service() -> EmailService {
  crate::utils::init_email_service()
    .await
    .expect("Failed to create test email service")
}

async fn create_test_storage() -> S3Storage {
  std::env::set_var("S3_ENDPOINT", "http://rustfs:9000");
  std::env::set_var("S3_PUBLIC_ENDPOINT", "http://127.0.0.1:9000");
  std::env::set_var("S3_ACCESS_KEY", "rustfs");
  std::env::set_var("S3_SECRET_KEY", "rustfssecret");
  std::env::set_var("S3_REGION", "us-east-1");
  std::env::set_var("S3_BUCKET", "test");
  S3Storage::new().await.expect("Failed to create test storage")
}

pub async fn app_with_pool(pool: PgPool) -> Router {
  let email_service = create_test_email_service().await;
  let storage = create_test_storage().await;
  let state = SharedAppState::new(pool, email_service, storage).await;
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

pub async fn get(app: Router, uri: &str) -> (StatusCode, Bytes) {
  let request = Request::builder()
    .method("GET")
    .uri(uri)
    .header("content-type", "application/json")
    .body(Body::empty())
    .expect("build request");

  let response = app.oneshot(request).await.expect("handle request");
  let status = response.status();
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .expect("read response body");
  (status, body)
}

pub async fn get_with_auth(app: Router, uri: &str, token: &str) -> (StatusCode, Bytes) {
  let request = Request::builder()
    .method("GET")
    .uri(uri)
    .header("content-type", "application/json")
    .header("authorization", format!("Bearer {}", token))
    .body(Body::empty())
    .expect("build request");

  let response = app.oneshot(request).await.expect("handle request");
  let status = response.status();
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .expect("read response body");
  (status, body)
}

pub async fn delete_with_auth(app: Router, uri: &str, token: &str) -> (StatusCode, Bytes) {
  let request = Request::builder()
    .method("DELETE")
    .uri(uri)
    .header("authorization", format!("Bearer {}", token))
    .body(Body::empty())
    .expect("build request");

  let response = app.oneshot(request).await.expect("handle request");
  let status = response.status();
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .expect("read response body");
  (status, body)
}
