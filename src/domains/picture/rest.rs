use axum::{
  extract::{Multipart, Path, State},
  http::HeaderMap,
  response::Json as JsonResponse,
  routing::{delete, get},
  Router,
};

use crate::{
  middleware::auth::auth_middleware,
  state::{AppState, SharedAppState},
  AppError,
};

use super::model::{Picture, PicturesResponse};

pub fn picture_routes() -> Router<SharedAppState> {
  Router::new()
    .route("/pictures", get(get_pictures_handler).post(create_picture_handler))
    .route("/pictures/{picture_id}", delete(delete_picture_handler))
}

async fn get_pictures_handler(State(state): State<SharedAppState>) -> Result<JsonResponse<PicturesResponse>, AppError> {
  state.get_pictures().await.map(JsonResponse).map_err(Into::into)
}

async fn create_picture_handler(
  State(state): State<SharedAppState>,
  headers: HeaderMap,
  mut multipart: Multipart,
) -> Result<JsonResponse<Picture>, AppError> {
  let claims = auth_middleware(headers).await?;
  let user_id = claims.user_id;

  let mut file_data: Option<Vec<u8>> = None;
  let mut file_name: Option<String> = None;
  let mut content_type: Option<String> = None;

  while let Some(field) = multipart
    .next_field()
    .await
    .map_err(|e| AppError::bad_request(format!("Failed to read multipart field: {}", e)))?
  {
    let name = field.name().unwrap_or("").to_string();

    if name == "file" {
      file_name = field.file_name().map(|s| s.to_string());
      content_type = field.content_type().map(|s| s.to_string());

      let data = field
        .bytes()
        .await
        .map_err(|e| AppError::bad_request(format!("Failed to read file data: {}", e)))?;
      file_data = Some(data.to_vec());
    }
  }

  let file_data = file_data.ok_or_else(|| AppError::bad_request("No file provided".to_string()))?;
  let file_name = file_name.ok_or_else(|| AppError::bad_request("No file name provided".to_string()))?;
  let content_type = content_type.unwrap_or_else(|| "application/octet-stream".to_string());

  state
    .upload_and_create_picture(user_id, file_data, file_name, content_type)
    .await
    .map(JsonResponse)
    .map_err(Into::into)
}

async fn delete_picture_handler(
  State(state): State<SharedAppState>,
  headers: HeaderMap,
  Path(picture_id): Path<i32>,
) -> Result<(), AppError> {
  let claims = auth_middleware(headers).await?;
  let user_id = claims.user_id;

  state.delete_picture(picture_id, user_id).await?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use crate::test_support::{app_with_pool, delete_with_auth, get, post_json};
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

  #[sqlx::test(migrations = "./migrations")]
  async fn delete_picture_unauthorized(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
    let app = app_with_pool(pool.clone()).await;

    let user =
      crate::domains::user::model::User::create(&pool, "delete-test@example.com", "Delete Test", "password123").await?;

    let picture_id = sqlx::query_scalar!(
      "INSERT INTO pictures (user_id, image_url) VALUES ($1, $2) RETURNING id",
      user.id,
      "https://example.com/test.jpg"
    )
    .fetch_one(&pool)
    .await?;

    let (status, _) = delete_with_auth(app, &format!("/api/v1/pictures/{}", picture_id), "invalid-token").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    Ok(())
  }

  #[sqlx::test(migrations = "./migrations")]
  async fn delete_picture_not_found(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
    let app = app_with_pool(pool.clone()).await;

    let _user =
      crate::domains::user::model::User::create(&pool, "delete-test@example.com", "Delete Test", "password123").await?;

    sqlx::query!(
      "UPDATE users SET email_verified = true WHERE email = $1",
      "delete-test@example.com"
    )
    .execute(&pool)
    .await?;

    let login_payload = crate::domains::user::model::LoginRequest {
      email: "delete-test@example.com".to_string(),
      password: "password123".to_string(),
    };
    let (login_status, login_body) = post_json(app.clone(), "/api/v1/login", &login_payload).await;
    assert_eq!(login_status, StatusCode::OK);

    let login_response: crate::domains::user::model::LoginResponse =
      serde_json::from_slice(&login_body).expect("deserialize login response");
    let token = login_response.token;

    let (status, _) = delete_with_auth(app, "/api/v1/pictures/99999", &token).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    Ok(())
  }

  #[sqlx::test(migrations = "./migrations")]
  async fn delete_picture_forbidden(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
    let app = app_with_pool(pool.clone()).await;

    let owner = crate::domains::user::model::User::create(&pool, "owner@example.com", "Owner", "password123").await?;
    let _other_user =
      crate::domains::user::model::User::create(&pool, "other@example.com", "Other", "password123").await?;

    sqlx::query!(
      "UPDATE users SET email_verified = true WHERE email = $1",
      "other@example.com"
    )
    .execute(&pool)
    .await?;

    let picture_id = sqlx::query_scalar!(
      "INSERT INTO pictures (user_id, image_url) VALUES ($1, $2) RETURNING id",
      owner.id,
      "https://example.com/test.jpg"
    )
    .fetch_one(&pool)
    .await?;

    let login_payload = crate::domains::user::model::LoginRequest {
      email: "other@example.com".to_string(),
      password: "password123".to_string(),
    };
    let (login_status, login_body) = post_json(app.clone(), "/api/v1/login", &login_payload).await;
    assert_eq!(login_status, StatusCode::OK);

    let login_response: crate::domains::user::model::LoginResponse =
      serde_json::from_slice(&login_body).expect("deserialize login response");
    let token = login_response.token;

    let (status, _) = delete_with_auth(app, &format!("/api/v1/pictures/{}", picture_id), &token).await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    Ok(())
  }

  #[sqlx::test(migrations = "./migrations")]
  async fn delete_picture_success(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
    let app = app_with_pool(pool.clone()).await;

    let user =
      crate::domains::user::model::User::create(&pool, "delete-success@example.com", "Delete Success", "password123")
        .await?;

    sqlx::query!(
      "UPDATE users SET email_verified = true WHERE email = $1",
      "delete-success@example.com"
    )
    .execute(&pool)
    .await?;

    let picture_id = sqlx::query_scalar!(
      "INSERT INTO pictures (user_id, image_url) VALUES ($1, $2) RETURNING id",
      user.id,
      "http://127.0.0.1:9000/test/pictures/1/test.jpg"
    )
    .fetch_one(&pool)
    .await?;

    let login_payload = crate::domains::user::model::LoginRequest {
      email: "delete-success@example.com".to_string(),
      password: "password123".to_string(),
    };
    let (login_status, login_body) = post_json(app.clone(), "/api/v1/login", &login_payload).await;
    assert_eq!(login_status, StatusCode::OK);

    let login_response: crate::domains::user::model::LoginResponse =
      serde_json::from_slice(&login_body).expect("deserialize login response");
    let token = login_response.token;

    let (status, _) = delete_with_auth(app, &format!("/api/v1/pictures/{}", picture_id), &token).await;
    assert_eq!(status, StatusCode::OK);

    let picture = sqlx::query!("SELECT * FROM pictures WHERE id = $1", picture_id)
      .fetch_optional(&pool)
      .await?;
    assert!(picture.is_none());

    Ok(())
  }
}
