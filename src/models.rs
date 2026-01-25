use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Deserialize, Serialize)]
pub struct User {
  pub id: i32,
  pub email: String,
  pub display_name: String,
  pub password: String,
  pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateUserRequest {
  pub email: String,
  pub display_name: String,
  pub password: String,
}
