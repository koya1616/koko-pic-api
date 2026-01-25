use axum::{
  extract::{Json, State},
  http::StatusCode,
  response::Json as JsonResponse,
  routing::{post, Router},
};
use sqlx::PgPool;

use crate::models::{CreateUserRequest, User};

pub fn user_routes() -> Router<PgPool> {
  Router::new().route("/users", post(create_user))
}

pub async fn create_user(
  State(pool): State<PgPool>,
  Json(payload): Json<CreateUserRequest>,
) -> Result<JsonResponse<User>, StatusCode> {
  // Hash the password before storing
  let hashed_password = hash_password(&payload.password);

  let user = sqlx::query_as!(
    User,
    r#"
        INSERT INTO users (email, display_name, password)
        VALUES ($1, $2, $3)
        RETURNING id, email, display_name, password, created_at
        "#,
    payload.email,
    payload.display_name,
    hashed_password
  )
  .fetch_one(&pool)
  .await
  .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

  Ok(JsonResponse(user))
}

// Simple password hashing function (in a real app, use a proper library like bcrypt)
fn hash_password(password: &str) -> String {
  use sha2::{Digest, Sha256};

  let mut hasher = Sha256::new();
  hasher.update(password.as_bytes());
  let result = hasher.finalize();
  format!("{:x}", result)
}
