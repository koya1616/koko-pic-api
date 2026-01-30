use axum::http::HeaderMap;

use crate::utils::error::AppError;
use crate::utils::jwt::Claims;

pub async fn auth_middleware(headers: HeaderMap) -> Result<Claims, AppError> {
  let auth_header = headers
    .get(axum::http::header::AUTHORIZATION)
    .ok_or_else(|| AppError::unauthorized("Authorization header missing"))?
    .to_str()
    .map_err(|_| AppError::unauthorized("Invalid authorization header"))?;

  let token = auth_header
    .strip_prefix("Bearer ")
    .ok_or_else(|| AppError::unauthorized("Invalid authorization format"))?;

  let claims = crate::utils::jwt::decode_jwt(token).map_err(|_| AppError::unauthorized("Invalid token"))?;

  Ok(claims)
}
