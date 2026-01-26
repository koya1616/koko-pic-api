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
    Err(e) => match e {
      crate::domains::user::service::UserServiceError::ValidationError => Err(StatusCode::BAD_REQUEST),
      crate::domains::user::service::UserServiceError::InternalServerError => Err(StatusCode::INTERNAL_SERVER_ERROR),
      crate::domains::user::service::UserServiceError::Unauthorized => Err(StatusCode::UNAUTHORIZED),
    },
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
      crate::domains::user::service::UserServiceError::ValidationError => Err(StatusCode::BAD_REQUEST),
      crate::domains::user::service::UserServiceError::InternalServerError => Err(StatusCode::INTERNAL_SERVER_ERROR),
    },
  }
}

#[cfg(test)]
mod tests {
  use super::super::model::CreateUserRequest;
  use crate::test_support::{app_with_pool, post_json};
  use axum::http::StatusCode;

  #[sqlx::test(migrations = "./migrations")]
  async fn create_user_endpoint_returns_user(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
    let app = app_with_pool(pool).await;
    let payload = CreateUserRequest {
      email: "api-create@example.com".to_string(),
      display_name: "API Create".to_string(),
      password: "password123".to_string(),
    };
    let (status, body) = post_json(app, "/api/v1/users", &payload).await;
    assert_eq!(status, StatusCode::OK);

    let user: super::super::model::User = serde_json::from_slice(&body).expect("deserialize response");
    assert_eq!(user.email, payload.email);
    assert_eq!(user.display_name, payload.display_name);
    Ok(())
  }

  #[sqlx::test(migrations = "./migrations")]
  async fn create_user_endpoint_invalid_email(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
    let app = app_with_pool(pool).await;
    let payload = CreateUserRequest {
      email: "invalid-email".to_string(),
      display_name: "Test User".to_string(),
      password: "password123".to_string(),
    };
    let (status, _) = post_json(app, "/api/v1/users", &payload).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    Ok(())
  }

  #[sqlx::test(migrations = "./migrations")]
  async fn login_success(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
    let app = app_with_pool(pool.clone()).await;
    super::super::model::User::create(&pool, "api-login@example.com", "API Login", "password123").await?;

    let login_payload = super::super::model::LoginRequest {
      email: "api-login@example.com".to_string(),
      password: "password123".to_string(),
    };
    let (status, body) = post_json(app, "/api/v1/login", &login_payload).await;
    assert_eq!(status, StatusCode::OK);

    let response: super::super::model::LoginResponse = serde_json::from_slice(&body).expect("deserialize response");
    assert_eq!(response.email, "api-login@example.com");
    Ok(())
  }

  #[sqlx::test(migrations = "./migrations")]
  async fn login_unauthorized(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
    let app = app_with_pool(pool).await;
    let login_payload = super::super::model::LoginRequest {
      email: "missing@example.com".to_string(),
      password: "password123".to_string(),
    };
    let (status, _body) = post_json(app, "/api/v1/login", &login_payload).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    Ok(())
  }
}
