use axum::{
  extract::{Json, State},
  http::StatusCode,
  response::Json as JsonResponse,
  routing::{post, Router},
};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use sqlx::PgExecutor;

use crate::{
  models::{CreateUserRequest, LoginRequest, LoginResponse, User},
  utils, AppState,
};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
  sub: String,
  exp: usize,
  user_id: i32,
}

pub fn user_routes() -> Router<AppState> {
  Router::new()
    .route("/users", post(create_user))
    .route("/login", post(login))
}

async fn create_user(
  State(state): State<AppState>,
  Json(payload): Json<CreateUserRequest>,
) -> Result<JsonResponse<User>, StatusCode> {
  let user = create_user_logic(&state.db, payload).await?;
  Ok(JsonResponse(user))
}

pub async fn create_user_logic<'e, E>(executor: E, payload: CreateUserRequest) -> Result<User, StatusCode>
where
  E: PgExecutor<'e>,
{
  User::create_with_executor(executor, &payload.email, &payload.display_name, &payload.password)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn login(
  State(state): State<AppState>,
  Json(payload): Json<LoginRequest>,
) -> Result<JsonResponse<LoginResponse>, StatusCode> {
  let response = login_logic(&state.db, payload).await?;
  Ok(JsonResponse(response))
}

pub async fn login_logic<'e, E>(executor: E, payload: LoginRequest) -> Result<LoginResponse, StatusCode>
where
  E: PgExecutor<'e>,
{
  let user = User::find_by_email(executor, &payload.email)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

  let user = match user {
    Some(user) => user,
    None => return Err(StatusCode::UNAUTHORIZED),
  };

  let hashed_input_password = utils::hash_password(&payload.password);
  if user.password != hashed_input_password {
    return Err(StatusCode::UNAUTHORIZED);
  }

  let expiration = chrono::Utc::now()
    .checked_add_signed(chrono::Duration::hours(24))
    .expect("Valid timestamp")
    .timestamp() as usize;

  let claims = Claims {
    sub: user.email.clone(),
    exp: expiration,
    user_id: user.id,
  };

  let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret_key".to_string());

  let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

  Ok(LoginResponse {
    token,
    user_id: user.id,
    email: user.email,
    display_name: user.display_name,
  })
}
