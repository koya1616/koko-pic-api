use async_trait::async_trait;
use sqlx::PgPool;

use super::model::User;

#[async_trait]
pub trait UserRepository: Send + Sync {
  async fn create(&self, email: &str, display_name: &str, password: &str) -> Result<User, sqlx::Error>;
  async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error>;
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
}
