use axum::http::StatusCode;
use koko_pic_api::{
  handlers::{create_user_logic, login_logic},
  models::{CreateUserRequest, LoginRequest, User},
};
use sqlx::PgPool;
use tokio::sync::OnceCell;

static DB_POOL: OnceCell<PgPool> = OnceCell::const_new();

async fn get_test_pool() -> &'static PgPool {
  DB_POOL
    .get_or_init(|| async {
      let database_url = std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");
      let pool = PgPool::connect(&database_url).await.unwrap();
      sqlx::migrate!("./migrations").run(&pool).await.unwrap();
      pool
    })
    .await
}

#[tokio::test]
async fn test_create_user() {
  let pool = get_test_pool().await;
  let mut tx = pool.begin().await.unwrap();

  let email = "create-user-test@example.com".to_string();

  let payload = CreateUserRequest {
    email: email.clone(),
    display_name: "Test User".to_string(),
    password: "securepassword123".to_string(),
  };

  let result = create_user_logic(&mut *tx, payload).await;
  assert!(result.is_ok());

  let user = result.unwrap();
  assert!(user.id > 0);
  assert_eq!(user.email, email);
  assert_eq!(user.display_name, "Test User");

  tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_create_user_duplicate_email() {
  let pool = get_test_pool().await;
  let mut tx = pool.begin().await.unwrap();

  let email = "duplicate-test@example.com".to_string();

  let payload1 = CreateUserRequest {
    email: email.clone(),
    display_name: "First User".to_string(),
    password: "password1".to_string(),
  };

  let result1 = create_user_logic(&mut *tx, payload1).await;
  assert!(result1.is_ok());

  let payload2 = CreateUserRequest {
    email: email.clone(),
    display_name: "Second User".to_string(),
    password: "password2".to_string(),
  };

  let result2 = create_user_logic(&mut *tx, payload2).await;
  assert_eq!(result2.unwrap_err(), StatusCode::INTERNAL_SERVER_ERROR);

  tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_login_success() {
  let pool = get_test_pool().await;
  let mut tx = pool.begin().await.unwrap();

  let email = "login-success@example.com".to_string();
  let password = "securepassword123".to_string();

  User::create_with_executor(&mut *tx, &email, "Login Test User", &password)
    .await
    .unwrap();

  let payload = LoginRequest {
    email: email.clone(),
    password,
  };

  let result = login_logic(&mut *tx, payload).await;
  assert!(result.is_ok());

  let response = result.unwrap();
  assert!(!response.token.is_empty());
  assert_eq!(response.email, email);
  assert_eq!(response.display_name, "Login Test User");

  tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_login_invalid_password() {
  let pool = get_test_pool().await;
  let mut tx = pool.begin().await.unwrap();

  let email = "login-invalid-pw@example.com".to_string();

  User::create_with_executor(&mut *tx, &email, "Test User", "correctpassword")
    .await
    .unwrap();

  let payload = LoginRequest {
    email,
    password: "wrongpassword".to_string(),
  };

  let result = login_logic(&mut *tx, payload).await;
  assert_eq!(result.unwrap_err(), StatusCode::UNAUTHORIZED);

  tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_login_user_not_found() {
  let pool = get_test_pool().await;
  let mut tx = pool.begin().await.unwrap();

  let payload = LoginRequest {
    email: "nonexistent@example.com".to_string(),
    password: "anypassword".to_string(),
  };

  let result = login_logic(&mut *tx, payload).await;
  assert_eq!(result.unwrap_err(), StatusCode::UNAUTHORIZED);

  tx.rollback().await.unwrap();
}
