use axum::{
  extract::{Multipart, State},
  http::HeaderMap,
  response::Json as JsonResponse,
  routing::get,
  Router,
};

use crate::{
  state::{AppState, SharedAppState},
  AppError,
};

use super::model::{Picture, PicturesResponse};

fn map_picture_service_error(e: super::service::PictureServiceError) -> AppError {
  match e {
    super::service::PictureServiceError::InternalServerError(msg) => AppError::internal_server_error(msg),
    super::service::PictureServiceError::BadRequest(msg) => AppError::bad_request(msg),
  }
}

async fn auth_middleware(headers: HeaderMap) -> Result<crate::utils::jwt::Claims, AppError> {
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

pub fn picture_routes() -> Router<SharedAppState> {
  Router::new().route("/pictures", get(get_pictures_handler).post(create_picture_handler))
}

async fn get_pictures_handler(State(state): State<SharedAppState>) -> Result<JsonResponse<PicturesResponse>, AppError> {
  state
    .get_pictures()
    .await
    .map(JsonResponse)
    .map_err(map_picture_service_error)
}

async fn create_picture_handler(
  State(state): State<SharedAppState>,
  headers: HeaderMap,
  mut multipart: Multipart,
) -> Result<JsonResponse<Picture>, AppError> {
  let claims = auth_middleware(headers).await?;
  let user_id = claims.user_id;

  let mut image_url: Option<String> = None;

  while let Some(field) = multipart
    .next_field()
    .await
    .map_err(|e| AppError::bad_request(format!("Failed to read multipart field: {}", e)))?
  {
    let name = field.name().unwrap_or("").to_string();

    if name == "file" {
      let _data = field
        .bytes()
        .await
        .map_err(|e| AppError::bad_request(format!("Failed to read file data: {}", e)))?;
      image_url = Some("https://example.com/uploaded-image.jpg".to_string());
    }
  }

  let image_url = image_url.ok_or_else(|| AppError::bad_request("No file provided".to_string()))?;

  state
    .create_picture(user_id, image_url)
    .await
    .map(JsonResponse)
    .map_err(map_picture_service_error)
}

#[cfg(test)]
mod tests {
  use crate::test_support::{app_with_pool, get, post_json};
  use axum::http::StatusCode;

  #[sqlx::test(migrations = "./migrations")]
  async fn get_pictures_returns_empty_list(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
    let app = app_with_pool(pool).await;
    let (status, body) = get(app, "/api/v1/pictures").await;
    assert_eq!(status, StatusCode::OK);

    let response: super::super::model::PicturesResponse = serde_json::from_slice(&body).expect("deserialize response");
    assert_eq!(response.pictures.len(), 0);
    Ok(())
  }

  #[sqlx::test(migrations = "./migrations")]
  async fn get_pictures_returns_pictures(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
    let app = app_with_pool(pool.clone()).await;

    let user =
      crate::domains::user::model::User::create(&pool, "pic-test@example.com", "Pic Test", "password123").await?;

    sqlx::query!(
      "INSERT INTO pictures (user_id, image_url) VALUES ($1, $2)",
      user.id,
      "https://example.com/image1.jpg"
    )
    .execute(&pool)
    .await?;

    sqlx::query!(
      "INSERT INTO pictures (user_id, image_url) VALUES ($1, $2)",
      user.id,
      "https://example.com/image2.jpg"
    )
    .execute(&pool)
    .await?;

    let (status, body) = get(app, "/api/v1/pictures").await;
    assert_eq!(status, StatusCode::OK);

    let response: super::super::model::PicturesResponse = serde_json::from_slice(&body).expect("deserialize response");
    assert_eq!(response.pictures.len(), 2);
    assert_eq!(response.pictures[0].image_url, "https://example.com/image2.jpg");
    assert_eq!(response.pictures[1].image_url, "https://example.com/image1.jpg");

    Ok(())
  }

  #[sqlx::test(migrations = "./migrations")]
  async fn create_picture_unauthorized(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
    let app = app_with_pool(pool).await;

    let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
    let body_content = format!(
      "------WebKitFormBoundary7MA4YWxkTrZu0gW\r\nContent-Disposition: form-data; name=\"file\"; filename=\"test.jpg\"\r\nContent-Type: image/jpeg\r\n\r\nfake-image-data\r\n------WebKitFormBoundary7MA4YWxkTrZu0gW--\r\n"
    );

    let request = axum::http::Request::builder()
      .method("POST")
      .uri("/api/v1/pictures")
      .header("content-type", format!("multipart/form-data; boundary={}", boundary))
      .body(axum::body::Body::from(body_content))
      .unwrap();

    let response = tower::ServiceExt::oneshot(app, request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    Ok(())
  }

  #[sqlx::test(migrations = "./migrations")]
  async fn create_picture_success(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
    let app = app_with_pool(pool.clone()).await;

    let user =
      crate::domains::user::model::User::create(&pool, "pic-upload@example.com", "Pic Upload", "password123").await?;

    sqlx::query!(
      "UPDATE users SET email_verified = true WHERE email = $1",
      "pic-upload@example.com"
    )
    .execute(&pool)
    .await?;

    let login_payload = crate::domains::user::model::LoginRequest {
      email: "pic-upload@example.com".to_string(),
      password: "password123".to_string(),
    };
    let (login_status, login_body) = post_json(app.clone(), "/api/v1/login", &login_payload).await;
    assert_eq!(login_status, StatusCode::OK);

    let login_response: crate::domains::user::model::LoginResponse =
      serde_json::from_slice(&login_body).expect("deserialize login response");
    let token = login_response.token;

    let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
    let body_content = format!(
      "------WebKitFormBoundary7MA4YWxkTrZu0gW\r\nContent-Disposition: form-data; name=\"file\"; filename=\"test.jpg\"\r\nContent-Type: image/jpeg\r\n\r\nfake-image-data\r\n------WebKitFormBoundary7MA4YWxkTrZu0gW--\r\n"
    );

    let request = axum::http::Request::builder()
      .method("POST")
      .uri("/api/v1/pictures")
      .header("authorization", format!("Bearer {}", token))
      .header("content-type", format!("multipart/form-data; boundary={}", boundary))
      .body(axum::body::Body::from(body_content))
      .unwrap();

    let response = tower::ServiceExt::oneshot(app, request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let picture: super::super::model::Picture = serde_json::from_slice(&body_bytes).expect("deserialize response");
    assert_eq!(picture.user_id, user.id);

    Ok(())
  }
}
