use axum::{
  body::{Body, Bytes},
  http::{Request, StatusCode},
  Router,
};
use serde::Serialize;
use sqlx::PgPool;
use tower::ServiceExt;

use crate::{
  app::create_app,
  email::{EmailService, SmtpConfig},
  state::SharedAppState,
};

fn create_test_email_service() -> EmailService {
  let smtp_config = SmtpConfig {
    host: "localhost".to_string(),
    port: 1025,
    username: "test".to_string(),
    password: "test".to_string(),
    from_email: "noreply@test.com".to_string(),
  };
  EmailService::new(smtp_config).expect("Failed to create test email service")
}

pub async fn app_with_pool(pool: PgPool) -> Router {
  let email_service = create_test_email_service();
  let state = SharedAppState::new(pool, email_service).await;
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
