use async_trait::async_trait;
use sqlx::PgPool;
use std::error::Error;

use super::model::PicturesResponse;
use super::repository;

#[derive(Debug)]
pub enum PictureServiceError {
  InternalServerError(String),
}

impl Error for PictureServiceError {}

impl std::fmt::Display for PictureServiceError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      PictureServiceError::InternalServerError(msg) => write!(f, "Internal Server Error: {}", msg),
    }
  }
}

impl From<sqlx::Error> for PictureServiceError {
  fn from(err: sqlx::Error) -> Self {
    PictureServiceError::InternalServerError(format!("Database error: {}", err))
  }
}

#[async_trait]
pub trait PictureService: Send + Sync {
  async fn get_pictures(&self) -> Result<PicturesResponse, PictureServiceError>;
}

pub struct PictureServiceImpl {
  db: PgPool,
}

impl PictureServiceImpl {
  pub fn new(db: PgPool) -> Self {
    Self { db }
  }
}

#[async_trait]
impl PictureService for PictureServiceImpl {
  async fn get_pictures(&self) -> Result<PicturesResponse, PictureServiceError> {
    let pictures = repository::find_all(&self.db).await?;
    Ok(PicturesResponse { pictures })
  }
}
