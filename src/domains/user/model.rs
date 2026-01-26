use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgExecutor, PgPool};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, FromRow, Deserialize, Serialize)]
pub struct User {
  pub id: i32,
  pub email: String,
  pub display_name: String,
  pub password: String,
  pub email_verified: bool,
  pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow, Deserialize, Serialize)]
pub struct VerificationToken {
  pub id: i32,
  pub user_id: i32,
  pub token: String,
  pub token_type: String,
  pub expires_at: DateTime<Utc>,
  pub used_at: Option<DateTime<Utc>>,
  pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct CreateUserRequest {
  #[validate(email(message = "メールアドレスが無効です"))]
  pub email: String,
  #[validate(length(min = 1, message = "表示名が必要です"))]
  pub display_name: String,
  #[validate(length(min = 8, message = "パスワードは8文字以上である必要があります"), custom(function = crate::utils::validate_password))]
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
            INSERT INTO users (email, display_name, password, email_verified)
            VALUES ($1, $2, $3, FALSE)
            RETURNING id, email, display_name, password, email_verified, created_at
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
      r#"SELECT id, email, display_name, password, email_verified, created_at FROM users WHERE email = $1"#,
      email
    )
    .fetch_optional(executor)
    .await?;

    Ok(user)
  }

  pub async fn find_by_id<'e, E>(executor: E, id: i32) -> Result<Option<User>, sqlx::Error>
  where
    E: PgExecutor<'e>,
  {
    let user = sqlx::query_as!(
      User,
      r#"SELECT id, email, display_name, password, email_verified, created_at FROM users WHERE id = $1"#,
      id
    )
    .fetch_optional(executor)
    .await?;

    Ok(user)
  }

  pub async fn verify_email(db: &PgPool, user_id: i32) -> Result<User, sqlx::Error> {
    let user = sqlx::query_as!(
      User,
      r#"
          UPDATE users
          SET email_verified = TRUE
          WHERE id = $1
          RETURNING id, email, display_name, password, email_verified, created_at
      "#,
      user_id
    )
    .fetch_one(db)
    .await?;

    Ok(user)
  }
}

impl VerificationToken {
  pub async fn create(
    db: &PgPool,
    user_id: i32,
    token_type: &str,
    expires_at: DateTime<Utc>,
  ) -> Result<VerificationToken, sqlx::Error> {
    let token = Uuid::new_v4().to_string(); // UUIDを使用して安全なトークンを生成

    let verification_token = sqlx::query_as!(
      VerificationToken,
      r#"
          INSERT INTO verification_tokens (user_id, token, token_type, expires_at)
          VALUES ($1, $2, $3, $4)
          RETURNING id, user_id, token, token_type, expires_at, used_at, created_at
      "#,
      user_id,
      token,
      token_type,
      expires_at
    )
    .fetch_one(db)
    .await?;

    Ok(verification_token)
  }

  pub async fn find_by_token<'e, E>(executor: E, token: &str) -> Result<Option<VerificationToken>, sqlx::Error>
  where
    E: PgExecutor<'e>,
  {
    let verification_token = sqlx::query_as!(
      VerificationToken,
      r#"
          SELECT id, user_id, token, token_type, expires_at, used_at, created_at
          FROM verification_tokens
          WHERE token = $1
      "#,
      token
    )
    .fetch_optional(executor)
    .await?;

    Ok(verification_token)
  }

  pub async fn mark_as_used(db: &PgPool, token_id: i32) -> Result<VerificationToken, sqlx::Error> {
    let verification_token = sqlx::query_as!(
      VerificationToken,
      r#"
          UPDATE verification_tokens
          SET used_at = NOW()
          WHERE id = $1
          RETURNING id, user_id, token, token_type, expires_at, used_at, created_at
      "#,
      token_id
    )
    .fetch_one(db)
    .await?;

    Ok(verification_token)
  }
}

#[cfg(test)]
mod tests {
  use super::User;

  #[sqlx::test(migrations = "./migrations")]
  async fn create_and_find_user(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
    let created = User::create(&pool, "db-test@example.com", "DB Test", "password123").await?;
    let found = User::find_by_email(&pool, "db-test@example.com").await?;
    let found = found.expect("user should exist");
    assert_eq!(created.id, found.id);
    assert_eq!(created.email, found.email);
    assert_eq!(created.email_verified, found.email_verified); // 新しく追加
    Ok(())
  }

  #[sqlx::test(migrations = "./migrations")]
  async fn find_user_returns_none(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
    let found = User::find_by_email(&pool, "missing@example.com").await?;
    assert!(found.is_none());
    Ok(())
  }

  #[sqlx::test(migrations = "./migrations")]
  async fn verify_user_email(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
    let created = User::create(&pool, "verify-test@example.com", "Verify Test", "password123").await?;
    assert!(!created.email_verified);

    let verified = User::verify_email(&pool, created.id).await?;
    assert!(verified.email_verified);

    Ok(())
  }
}
