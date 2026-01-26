use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgExecutor, PgPool};

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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoginRequest {
  pub email: String,
  pub password: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoginResponse {
  pub token: String,
  pub user_id: i32,
  pub email: String,
  pub display_name: String,
}

impl User {
  pub async fn create(db: &PgPool, email: &str, display_name: &str, password: &str) -> Result<User, sqlx::Error> {
    Self::create_with_executor(db, email, display_name, password).await
  }

  pub async fn create_with_executor<'e, E>(
    executor: E,
    email: &str,
    display_name: &str,
    password: &str,
  ) -> Result<User, sqlx::Error>
  where
    E: PgExecutor<'e>,
  {
    let hashed_password = crate::utils::hash_password(password);

    let user = sqlx::query_as!(
      User,
      r#"
            INSERT INTO users (email, display_name, password)
            VALUES ($1, $2, $3)
            RETURNING id, email, display_name, password, created_at
            "#,
      email,
      display_name,
      hashed_password
    )
    .fetch_one(executor)
    .await?;

    Ok(user)
  }

  pub async fn find_by_email<'e, E>(executor: E, email: &str) -> Result<Option<User>, sqlx::Error>
  where
    E: PgExecutor<'e>,
  {
    let user = sqlx::query_as!(
      User,
      r#"SELECT id, email, display_name, password, created_at FROM users WHERE email = $1"#,
      email
    )
    .fetch_optional(executor)
    .await?;

    Ok(user)
  }
}
