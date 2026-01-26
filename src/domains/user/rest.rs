use axum::{
  extract::{Json, State},
  http::StatusCode,
  response::Json as JsonResponse,
  routing::{post, Router},
};

use super::model::{CreateUserRequest, LoginRequest};
use crate::state::{AppState, SharedAppState};

pub fn user_routes() -> Router<SharedAppState> {
  Router::new()
    .route("/users", post(create_user_handler))
    .route("/login", post(login_handler))
}

pub async fn create_user_handler(
  State(state): State<SharedAppState>,
  Json(payload): Json<CreateUserRequest>,
) -> Result<JsonResponse<super::model::User>, StatusCode> {
  match state.create_user(payload).await {
    Ok(user) => Ok(JsonResponse(user)),
    Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
  }
}

pub async fn login_handler(
  State(state): State<SharedAppState>,
  Json(payload): Json<LoginRequest>,
) -> Result<JsonResponse<super::model::LoginResponse>, StatusCode> {
  match state.login(payload).await {
    Ok(response) => Ok(JsonResponse(response)),
    Err(e) => match e {
      crate::domains::user::service::UserServiceError::Unauthorized => Err(StatusCode::UNAUTHORIZED),
      crate::domains::user::service::UserServiceError::InternalServerError => Err(StatusCode::INTERNAL_SERVER_ERROR),
    },
  }
}
