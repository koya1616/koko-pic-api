use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Deserialize, Serialize)]
pub struct Picture {
  pub id: i32,
  pub user_id: i32,
  pub image_url: String,
  pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PicturesResponse {
  pub pictures: Vec<Picture>,
}
