use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, Clone, FromRow, Deserialize, Serialize)]
pub struct Request {
  pub id: i32,
  pub user_id: i32,
  pub lat: f64,
  pub lng: f64,
  pub status: String,
  pub place_name: String,
  pub description: String,
  pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RequestWithDistance {
  pub id: i32,
  pub user_id: i32,
  pub lat: f64,
  pub lng: f64,
  pub status: String,
  pub place_name: String,
  pub description: String,
  pub created_at: Option<DateTime<Utc>>,
  pub distance: Option<f64>,
}

impl From<Request> for RequestWithDistance {
  fn from(req: Request) -> Self {
    Self {
      id: req.id,
      user_id: req.user_id,
      lat: req.lat,
      lng: req.lng,
      status: req.status,
      place_name: req.place_name,
      description: req.description,
      created_at: req.created_at,
      distance: None,
    }
  }
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct CreateRequestRequest {
  #[validate(range(min = -90.0, max = 90.0, message = "緯度は-90から90の範囲である必要があります"))]
  pub lat: f64,
  #[validate(range(min = -180.0, max = 180.0, message = "経度は-180から180の範囲である必要があります"))]
  pub lng: f64,
  #[validate(length(min = 1, max = 255, message = "場所名は1文字以上255文字以内である必要があります"))]
  pub place_name: String,
  #[validate(length(min = 1, message = "説明が必要です"))]
  pub description: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RequestsResponse {
  pub requests: Vec<RequestWithDistance>,
}
