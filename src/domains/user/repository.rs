use async_trait::async_trait;
use sqlx::PgPool;

use super::model::{User, VerificationToken};

#[async_trait]
pub trait UserRepository: Send + Sync {
  async fn create(&self, email: &str, display_name: &str, password: &str) -> Result<User, RepositoryError>;
  async fn find_by_email(&self, email: &str) -> Result<Option<User>, RepositoryError>;
  async fn find_by_id(&self, id: i32) -> Result<Option<User>, RepositoryError>;
  fn get_pool(&self) -> &PgPool;
}

#[async_trait]
pub trait VerificationTokenRepository: Send + Sync {
  async fn create_verification_token(
    &self,
    user_id: i32,
    token_type: &str,
    expires_at: chrono::DateTime<chrono::Utc>,
  ) -> Result<VerificationToken, RepositoryError>;
  async fn find_token_by_value(&self, token: &str) -> Result<Option<VerificationToken>, RepositoryError>;
  async fn mark_token_as_used(&self, token_id: i32) -> Result<VerificationToken, RepositoryError>;
}

#[derive(Debug)]
pub enum RepositoryError {
  DatabaseError(sqlx::Error),
  NotFound(String),
  Conflict(String),
}

impl std::fmt::Display for RepositoryError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      RepositoryError::DatabaseError(e) => write!(f, "Database error: {}", e),
      RepositoryError::NotFound(msg) => write!(f, "Not found: {}", msg),
      RepositoryError::Conflict(msg) => write!(f, "Conflict: {}", msg),
    }
  }
}

impl std::error::Error for RepositoryError {}

impl From<sqlx::Error> for RepositoryError {
  fn from(err: sqlx::Error) -> Self {
    RepositoryError::DatabaseError(err)
  }
}

pub struct SqlxUserRepository {
  pub pool: PgPool,
}

impl SqlxUserRepository {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

#[async_trait]
impl UserRepository for SqlxUserRepository {
  async fn create(&self, email: &str, display_name: &str, password: &str) -> Result<User, RepositoryError> {
    Ok(User::create(&self.pool, email, display_name, password).await?)
  }

  async fn find_by_email(&self, email: &str) -> Result<Option<User>, RepositoryError> {
    Ok(User::find_by_email(&self.pool, email).await?)
  }

  async fn find_by_id(&self, id: i32) -> Result<Option<User>, RepositoryError> {
    Ok(User::find_by_id(&self.pool, id).await?)
  }

  fn get_pool(&self) -> &PgPool {
    &self.pool
  }
}

pub struct SqlxVerificationTokenRepository {
  pub pool: PgPool,
}

impl SqlxVerificationTokenRepository {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

#[async_trait]
impl VerificationTokenRepository for SqlxVerificationTokenRepository {
  async fn create_verification_token(
    &self,
    user_id: i32,
    token_type: &str,
    expires_at: chrono::DateTime<chrono::Utc>,
  ) -> Result<VerificationToken, RepositoryError> {
    Ok(VerificationToken::create(&self.pool, user_id, token_type, expires_at).await?)
  }

  async fn find_token_by_value(&self, token: &str) -> Result<Option<VerificationToken>, RepositoryError> {
    Ok(VerificationToken::find_by_token(&self.pool, token).await?)
  }

  async fn mark_token_as_used(&self, token_id: i32) -> Result<VerificationToken, RepositoryError> {
    Ok(VerificationToken::mark_as_used(&self.pool, token_id).await?)
  }
}
