use axum::{
  extract::{Json, State},
  http::HeaderMap,
  response::Json as JsonResponse,
  routing::{get, post, Router},
};
use validator::Validate;

use super::model::{CreateUserRequest, LoginRequest, ResendVerificationRequest};
use crate::{
  middleware::auth::auth_middleware,
  state::{AppState, SharedAppState},
  AppError,
};

pub fn user_routes() -> Router<SharedAppState> {
  Router::new()
    .route("/users", post(create_user_handler))
    .route("/users/me", get(get_current_user_handler))
    .route("/login", post(login_handler))
    .route("/verify-email/{token}", get(verify_email_handler))
    .route("/resend-verification", post(resend_verification_handler))
}

pub async fn create_user_handler(
  State(state): State<SharedAppState>,
  Json(payload): Json<CreateUserRequest>,
) -> Result<JsonResponse<super::model::User>, AppError> {
  state.create_user(payload).await.map(JsonResponse).map_err(Into::into)
}

pub async fn login_handler(
  State(state): State<SharedAppState>,
  Json(payload): Json<LoginRequest>,
) -> Result<JsonResponse<super::model::LoginResponse>, AppError> {
  state.login(payload).await.map(JsonResponse).map_err(Into::into)
}

pub async fn verify_email_handler(
  State(state): State<SharedAppState>,
  axum::extract::Path(token): axum::extract::Path<String>,
) -> Result<JsonResponse<super::model::VerifyEmailResponse>, AppError> {
  state.verify_email(token).await.map(JsonResponse).map_err(Into::into)
}

pub async fn get_current_user_handler(
  State(state): State<SharedAppState>,
  headers: HeaderMap,
) -> Result<JsonResponse<super::model::User>, AppError> {
  let claims = auth_middleware(headers).await?;
  let user_id = claims.user_id;

  state
    .get_user_by_id(user_id)
    .await
    .map(JsonResponse)
    .map_err(Into::into)
}

pub async fn resend_verification_handler(
  State(state): State<SharedAppState>,
  Json(payload): Json<ResendVerificationRequest>,
) -> Result<(), AppError> {
  payload
    .validate()
    .map_err(|e| AppError::bad_request(format!("Validation failed: {}", e)))?;

  state
    .send_verification_email_by_email(payload.email)
    .await
    .map(|_| ())
    .map_err(Into::into)
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
    sqlx::query!(
      "UPDATE users SET email_verified = true WHERE email = $1",
      "api-login@example.com"
    )
    .execute(&pool)
    .await?;

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

  #[sqlx::test(migrations = "./migrations")]
  async fn get_current_user_success(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
    let app = app_with_pool(pool.clone()).await;

    let user = super::super::model::User::create(&pool, "me-user@example.com", "Me User", "password123").await?;

    sqlx::query!(
      "UPDATE users SET email_verified = true WHERE email = $1",
      "me-user@example.com"
    )
    .execute(&pool)
    .await?;

    let login_payload = super::super::model::LoginRequest {
      email: "me-user@example.com".to_string(),
      password: "password123".to_string(),
    };
    let (login_status, login_body) = crate::test_support::post_json(app.clone(), "/api/v1/login", &login_payload).await;
    assert_eq!(login_status, StatusCode::OK);

    let login_response: super::super::model::LoginResponse =
      serde_json::from_slice(&login_body).expect("deserialize login response");
    let token = login_response.token;

    let (status, body) = crate::test_support::get_with_auth(app, "/api/v1/users/me", &token).await;

    assert_eq!(status, StatusCode::OK);

    let retrieved_user: super::super::model::User = serde_json::from_slice(&body).expect("deserialize response");
    assert_eq!(retrieved_user.id, user.id);
    assert_eq!(retrieved_user.email, user.email);
    assert_eq!(retrieved_user.display_name, user.display_name);

    Ok(())
  }

  #[sqlx::test(migrations = "./migrations")]
  async fn get_current_user_unauthorized_no_token(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
    let app = app_with_pool(pool).await;

    let (status, _) = crate::test_support::get(app, "/api/v1/users/me").await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);

    Ok(())
  }

  #[sqlx::test(migrations = "./migrations")]
  async fn get_current_user_unauthorized_invalid_token(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
    let app = app_with_pool(pool).await;

    let (status, _) = crate::test_support::get_with_auth(app, "/api/v1/users/me", "invalid-token").await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);

    Ok(())
  }
}
