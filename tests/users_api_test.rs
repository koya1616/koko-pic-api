use axum::{
  body::Body,
  http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use koko_pic_api::{app_with_state, models::CreateUserRequest};
use sqlx::{PgPool, Row};
use tower::ServiceExt;

#[tokio::test]
async fn test_create_user() {
  let database_url = std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");
  let pool = PgPool::connect(&database_url).await.unwrap();

  sqlx::migrate!("./migrations").run(&pool).await.unwrap();

  sqlx::query("TRUNCATE TABLE users RESTART IDENTITY CASCADE")
    .execute(&pool)
    .await
    .unwrap();

  let app = app_with_state(pool.clone());

  let create_user_request = CreateUserRequest {
    email: "test@example.com".to_string(),
    display_name: "Test User".to_string(),
    password: "securepassword123".to_string(),
  };

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/v1/users")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&create_user_request).unwrap()))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  let body = response.into_body().collect().await.unwrap().to_bytes();
  let response_str = String::from_utf8(body.to_vec()).unwrap();

  let row = sqlx::query("SELECT id, email, display_name FROM users WHERE email = $1")
    .bind("test@example.com")
    .fetch_one(&pool)
    .await
    .unwrap();

  assert_eq!(row.get::<i32, _>("id"), 1);
  assert_eq!(row.get::<String, _>("email"), "test@example.com");
  assert_eq!(row.get::<String, _>("display_name"), "Test User");
}
