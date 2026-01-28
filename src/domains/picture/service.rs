use async_trait::async_trait;
use sqlx::PgPool;
use std::error::Error;
use uuid::Uuid;

use crate::impl_service_error_conversions;
use crate::storage::S3Storage;

use super::model::{Picture, PicturesResponse};
use super::repository;

#[derive(Debug)]
pub enum PictureServiceError {
  InternalServerError(String),
  BadRequest(String),
}

impl Error for PictureServiceError {}

impl std::fmt::Display for PictureServiceError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      PictureServiceError::InternalServerError(msg) => write!(f, "Internal Server Error: {}", msg),
      PictureServiceError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
    }
  }
}

impl_service_error_conversions!(PictureServiceError, InternalServerError);

#[async_trait]
pub trait PictureService: Send + Sync {
  async fn get_pictures(&self) -> Result<PicturesResponse, PictureServiceError>;
  async fn create_picture(&self, user_id: i32, image_url: String) -> Result<Picture, PictureServiceError>;
  async fn upload_and_create_picture(
    &self,
    user_id: i32,
    file_data: Vec<u8>,
    file_name: String,
    content_type: String,
  ) -> Result<Picture, PictureServiceError>;
}

pub struct PictureServiceImpl {
  db: PgPool,
  storage: S3Storage,
}

impl PictureServiceImpl {
  pub fn new(db: PgPool, storage: S3Storage) -> Self {
    Self { db, storage }
  }
}

#[async_trait]
impl PictureService for PictureServiceImpl {
  async fn get_pictures(&self) -> Result<PicturesResponse, PictureServiceError> {
    let pictures = repository::find_all(&self.db).await?;
    Ok(PicturesResponse { pictures })
  }

  async fn create_picture(&self, user_id: i32, image_url: String) -> Result<Picture, PictureServiceError> {
    let picture = repository::create(&self.db, user_id, &image_url).await?;
    Ok(picture)
  }

  async fn upload_and_create_picture(
    &self,
    user_id: i32,
    file_data: Vec<u8>,
    file_name: String,
    content_type: String,
  ) -> Result<Picture, PictureServiceError> {
    let extension = file_name.split('.').next_back().unwrap_or("jpg");
    let unique_key = format!("pictures/{}/{}.{}", user_id, Uuid::new_v4(), extension);

    let image_url = self
      .storage
      .upload_file(&unique_key, file_data, &content_type)
      .await
      .map_err(|e| PictureServiceError::InternalServerError(format!("Failed to upload to S3: {}", e)))?;

    let picture = repository::create(&self.db, user_id, &image_url).await?;
    Ok(picture)
  }
}
