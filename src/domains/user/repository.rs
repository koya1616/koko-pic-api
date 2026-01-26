use async_trait::async_trait;
use sqlx::PgPool;

use super::model::{User, VerificationToken};

#[async_trait]
pub trait UserRepository: Send + Sync {
  async fn create(&self, email: &str, display_name: &str, password: &str) -> Result<User, sqlx::Error>;
  async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error>;
  async fn find_by_id(&self, id: i32) -> Result<Option<User>, sqlx::Error>;
  fn get_pool(&self) -> &PgPool;
}

#[async_trait]
pub trait VerificationTokenRepository: Send + Sync {
  async fn create_verification_token(
    &self,
    user_id: i32,
    token_type: &str,
    expires_at: chrono::DateTime<chrono::Utc>,
  ) -> Result<VerificationToken, sqlx::Error>;
  async fn find_token_by_value(&self, token: &str) -> Result<Option<VerificationToken>, sqlx::Error>;
  async fn mark_token_as_used(&self, token_id: i32) -> Result<VerificationToken, sqlx::Error>;
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
  async fn create(&self, email: &str, display_name: &str, password: &str) -> Result<User, sqlx::Error> {
    User::create(&self.pool, email, display_name, password).await
  }

  async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
    User::find_by_email(&self.pool, email).await
  }

  async fn find_by_id(&self, id: i32) -> Result<Option<User>, sqlx::Error> {
    User::find_by_id(&self.pool, id).await
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
  ) -> Result<VerificationToken, sqlx::Error> {
    VerificationToken::create(&self.pool, user_id, token_type, expires_at).await
  }

  async fn find_token_by_value(&self, token: &str) -> Result<Option<VerificationToken>, sqlx::Error> {
    VerificationToken::find_by_token(&self.pool, token).await
  }

  async fn mark_token_as_used(&self, token_id: i32) -> Result<VerificationToken, sqlx::Error> {
    VerificationToken::mark_as_used(&self.pool, token_id).await
  }
}
