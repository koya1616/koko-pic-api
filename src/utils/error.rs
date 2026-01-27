use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use serde_json::json;

#[derive(Debug)]
pub struct AppError {
  pub status_code: StatusCode,
  pub message: String,
}

impl AppError {
  pub fn new(status_code: StatusCode, message: impl Into<String>) -> Self {
    Self {
      status_code,
      message: message.into(),
    }
  }

  pub fn bad_request(message: impl Into<String>) -> Self {
    Self::new(StatusCode::BAD_REQUEST, message)
  }

  pub fn unauthorized(message: impl Into<String>) -> Self {
    Self::new(StatusCode::UNAUTHORIZED, message)
  }

  pub fn not_found(message: impl Into<String>) -> Self {
    Self::new(StatusCode::NOT_FOUND, message)
  }

  pub fn internal_server_error(message: impl Into<String>) -> Self {
    Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
  }
}

impl IntoResponse for AppError {
  fn into_response(self) -> Response {
    let body = Json(json!({
      "error": self.message,
      "status_code": self.status_code.as_u16(),
    }));

    (self.status_code, body).into_response()
  }
}

impl From<AppError> for StatusCode {
  fn from(err: AppError) -> Self {
    err.status_code
  }
}

// Implement From for common error types
impl From<sqlx::Error> for AppError {
  fn from(error: sqlx::Error) -> Self {
    tracing::error!("Database error: {:?}", error);
    AppError::internal_server_error("Internal server error occurred")
  }
}

impl From<serde_json::Error> for AppError {
  fn from(error: serde_json::Error) -> Self {
    tracing::error!("JSON error: {:?}", error);
    AppError::bad_request("Invalid JSON format")
  }
}

impl From<std::string::FromUtf8Error> for AppError {
  fn from(error: std::string::FromUtf8Error) -> Self {
    tracing::error!("UTF8 error: {:?}", error);
    AppError::bad_request("Invalid UTF8 encoding")
  }
}
