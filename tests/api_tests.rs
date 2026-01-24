use axum::{
  body::Body,
  http::{self, Request, StatusCode},
};
use http_body_util::BodyExt;
use tower::ServiceExt; // for `app.oneshot()`

#[tokio::test]
async fn hello_world_test() {
  let app = koko_pic_api::router();

  let response = app
    .oneshot(
      Request::builder()
        .method(http::Method::GET)
        .uri("/")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  let body = response.into_body().collect().await.unwrap().to_bytes();

  assert_eq!(&body[..], b"<h1>Hello, World!!</h1>");
}

#[tokio::test]
async fn hello_world_handler_test() {
  let response = koko_pic_api::hello_world_handler().await;
  let html_string: String = response.0;
  assert_eq!(html_string, "<h1>Hello, World!!</h1>".to_string());
}

#[tokio::test]
async fn test_root_route_status_ok() {
  let app = koko_pic_api::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/")
        .method(http::Method::GET)
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
}
