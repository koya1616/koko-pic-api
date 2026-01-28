use axum::{extract::State, response::Json as JsonResponse, routing::get, Router};

use crate::{
  state::{AppState, SharedAppState},
  AppError,
};

use super::model::PicturesResponse;

fn map_picture_service_error(e: super::service::PictureServiceError) -> AppError {
  match e {
    super::service::PictureServiceError::InternalServerError(msg) => AppError::internal_server_error(msg),
  }
}

pub fn picture_routes() -> Router<SharedAppState> {
  Router::new().route("/pictures", get(get_pictures_handler))
}

async fn get_pictures_handler(State(state): State<SharedAppState>) -> Result<JsonResponse<PicturesResponse>, AppError> {
  state
    .get_pictures()
    .await
    .map(JsonResponse)
    .map_err(map_picture_service_error)
}

#[cfg(test)]
mod tests {
  use crate::test_support::{app_with_pool, get};
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
}
