use axum::{
  extract::{Json, Path, Query, State},
  http::HeaderMap,
  response::Json as JsonResponse,
  routing::{get, post},
  Router,
};
use serde::Deserialize;
use validator::Validate;

use super::model::{CreateRequestRequest, Request, RequestsResponse};
use crate::{
  middleware::auth::auth_middleware,
  state::{AppState, SharedAppState},
  AppError,
};

#[derive(Debug, Deserialize)]
pub struct GetRequestsQuery {
  pub lat: Option<f64>,
  pub lng: Option<f64>,
}

pub fn request_routes() -> Router<SharedAppState> {
  Router::new()
    .route("/requests", get(get_requests_handler))
    .route("/requests", post(create_request_handler))
    .route("/requests/{request_id}", get(get_request_by_id_handler))
}

pub async fn get_requests_handler(
  State(state): State<SharedAppState>,
  Query(query): Query<GetRequestsQuery>,
) -> Result<JsonResponse<RequestsResponse>, AppError> {
  state
    .get_requests(query.lat, query.lng)
    .await
    .map(JsonResponse)
    .map_err(Into::into)
}

pub async fn create_request_handler(
  State(state): State<SharedAppState>,
  headers: HeaderMap,
  Json(payload): Json<CreateRequestRequest>,
) -> Result<JsonResponse<Request>, AppError> {
  payload
    .validate()
    .map_err(|e| AppError::bad_request(format!("Validation failed: {}", e)))?;

  let claims = auth_middleware(headers).await?;
  let user_id = claims.user_id;

  state
    .create_request(user_id, payload)
    .await
    .map(JsonResponse)
    .map_err(Into::into)
}

pub async fn get_request_by_id_handler(
  State(state): State<SharedAppState>,
  Path(request_id): Path<i32>,
) -> Result<JsonResponse<Request>, AppError> {
  state
    .get_request_by_id(request_id)
    .await
    .map(JsonResponse)
    .map_err(Into::into)
}

#[cfg(test)]
mod tests {
  use super::super::model::CreateRequestRequest;
  use crate::test_support::{app_with_pool, get, post_json};
  use axum::http::StatusCode;

  #[sqlx::test(migrations = "./migrations")]
  async fn get_requests_success(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
    let app = app_with_pool(pool.clone()).await;

    let user =
      crate::domains::user::model::User::create(&pool, "get-req@example.com", "Get Req", "password123").await?;
    super::super::repository::create(
      &pool,
      user.id,
      35.6812,
      139.7671,
      "東京".to_string(),
      "テスト".to_string(),
    )
    .await?;

    let (status, body) = get(app, "/api/v1/requests").await;
    assert_eq!(status, StatusCode::OK);

    let response: super::super::model::RequestsResponse = serde_json::from_slice(&body).expect("deserialize response");
    assert!(response.requests.len() >= 1);

    Ok(())
  }

  #[sqlx::test(migrations = "./migrations")]
  async fn create_request_unauthorized(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
    let app = app_with_pool(pool).await;

    let payload = CreateRequestRequest {
      lat: 35.6812,
      lng: 139.7671,
      place_name: "東京".to_string(),
      description: "テスト説明".to_string(),
    };

    let request = axum::http::Request::builder()
      .method("POST")
      .uri("/api/v1/requests")
      .header("content-type", "application/json")
      .body(axum::body::Body::from(serde_json::to_string(&payload).unwrap()))
      .unwrap();

    let response = tower::ServiceExt::oneshot(app, request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    Ok(())
  }

  #[sqlx::test(migrations = "./migrations")]
  async fn create_request_success(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
    let app = app_with_pool(pool.clone()).await;

    let _user =
      crate::domains::user::model::User::create(&pool, "create-req@example.com", "Create Req", "password123").await?;

    sqlx::query!(
      "UPDATE users SET email_verified = true WHERE email = $1",
      "create-req@example.com"
    )
    .execute(&pool)
    .await?;

    let login_payload = crate::domains::user::model::LoginRequest {
      email: "create-req@example.com".to_string(),
      password: "password123".to_string(),
    };
    let (login_status, login_body) = post_json(app.clone(), "/api/v1/login", &login_payload).await;
    assert_eq!(login_status, StatusCode::OK);

    let login_response: crate::domains::user::model::LoginResponse =
      serde_json::from_slice(&login_body).expect("deserialize login response");
    let token = login_response.token;

    let payload = CreateRequestRequest {
      lat: 35.6812,
      lng: 139.7671,
      place_name: "東京".to_string(),
      description: "テスト説明".to_string(),
    };

    let request = axum::http::Request::builder()
      .method("POST")
      .uri("/api/v1/requests")
      .header("authorization", format!("Bearer {}", token))
      .header("content-type", "application/json")
      .body(axum::body::Body::from(serde_json::to_string(&payload).unwrap()))
      .unwrap();

    let response = tower::ServiceExt::oneshot(app, request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created_request: super::super::model::Request =
      serde_json::from_slice(&body_bytes).expect("deserialize response");
    assert_eq!(created_request.lat, 35.6812);
    assert_eq!(created_request.lng, 139.7671);
    assert_eq!(created_request.place_name, "東京");
    assert_eq!(created_request.status, "open");

    Ok(())
  }

  #[sqlx::test(migrations = "./migrations")]
  async fn create_request_invalid_lat(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
    let app = app_with_pool(pool.clone()).await;

    let _user =
      crate::domains::user::model::User::create(&pool, "invalid-lat@example.com", "Invalid Lat", "password123").await?;

    sqlx::query!(
      "UPDATE users SET email_verified = true WHERE email = $1",
      "invalid-lat@example.com"
    )
    .execute(&pool)
    .await?;

    let login_payload = crate::domains::user::model::LoginRequest {
      email: "invalid-lat@example.com".to_string(),
      password: "password123".to_string(),
    };
    let (login_status, login_body) = post_json(app.clone(), "/api/v1/login", &login_payload).await;
    assert_eq!(login_status, StatusCode::OK);

    let login_response: crate::domains::user::model::LoginResponse =
      serde_json::from_slice(&login_body).expect("deserialize login response");
    let token = login_response.token;

    let payload = CreateRequestRequest {
      lat: 91.0, // Invalid: > 90
      lng: 139.7671,
      place_name: "東京".to_string(),
      description: "テスト説明".to_string(),
    };

    let request = axum::http::Request::builder()
      .method("POST")
      .uri("/api/v1/requests")
      .header("authorization", format!("Bearer {}", token))
      .header("content-type", "application/json")
      .body(axum::body::Body::from(serde_json::to_string(&payload).unwrap()))
      .unwrap();

    let response = tower::ServiceExt::oneshot(app, request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    Ok(())
  }

  #[sqlx::test(migrations = "./migrations")]
  async fn create_request_invalid_lng(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
    let app = app_with_pool(pool.clone()).await;

    let _user =
      crate::domains::user::model::User::create(&pool, "invalid-lng@example.com", "Invalid Lng", "password123").await?;

    sqlx::query!(
      "UPDATE users SET email_verified = true WHERE email = $1",
      "invalid-lng@example.com"
    )
    .execute(&pool)
    .await?;

    let login_payload = crate::domains::user::model::LoginRequest {
      email: "invalid-lng@example.com".to_string(),
      password: "password123".to_string(),
    };
    let (login_status, login_body) = post_json(app.clone(), "/api/v1/login", &login_payload).await;
    assert_eq!(login_status, StatusCode::OK);

    let login_response: crate::domains::user::model::LoginResponse =
      serde_json::from_slice(&login_body).expect("deserialize login response");
    let token = login_response.token;

    let payload = CreateRequestRequest {
      lat: 35.6812,
      lng: 181.0, // Invalid: > 180
      place_name: "東京".to_string(),
      description: "テスト説明".to_string(),
    };

    let request = axum::http::Request::builder()
      .method("POST")
      .uri("/api/v1/requests")
      .header("authorization", format!("Bearer {}", token))
      .header("content-type", "application/json")
      .body(axum::body::Body::from(serde_json::to_string(&payload).unwrap()))
      .unwrap();

    let response = tower::ServiceExt::oneshot(app, request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    Ok(())
  }

  #[sqlx::test(migrations = "./migrations")]
  async fn create_request_empty_place_name(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
    let app = app_with_pool(pool.clone()).await;

    let _user =
      crate::domains::user::model::User::create(&pool, "empty-place@example.com", "Empty Place", "password123").await?;

    sqlx::query!(
      "UPDATE users SET email_verified = true WHERE email = $1",
      "empty-place@example.com"
    )
    .execute(&pool)
    .await?;

    let login_payload = crate::domains::user::model::LoginRequest {
      email: "empty-place@example.com".to_string(),
      password: "password123".to_string(),
    };
    let (login_status, login_body) = post_json(app.clone(), "/api/v1/login", &login_payload).await;
    assert_eq!(login_status, StatusCode::OK);

    let login_response: crate::domains::user::model::LoginResponse =
      serde_json::from_slice(&login_body).expect("deserialize login response");
    let token = login_response.token;

    let payload = CreateRequestRequest {
      lat: 35.6812,
      lng: 139.7671,
      place_name: "".to_string(), // Invalid: empty
      description: "テスト説明".to_string(),
    };

    let request = axum::http::Request::builder()
      .method("POST")
      .uri("/api/v1/requests")
      .header("authorization", format!("Bearer {}", token))
      .header("content-type", "application/json")
      .body(axum::body::Body::from(serde_json::to_string(&payload).unwrap()))
      .unwrap();

    let response = tower::ServiceExt::oneshot(app, request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    Ok(())
  }

  #[sqlx::test(migrations = "./migrations")]
  async fn create_request_empty_description(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
    let app = app_with_pool(pool.clone()).await;

    let _user =
      crate::domains::user::model::User::create(&pool, "empty-desc@example.com", "Empty Desc", "password123").await?;

    sqlx::query!(
      "UPDATE users SET email_verified = true WHERE email = $1",
      "empty-desc@example.com"
    )
    .execute(&pool)
    .await?;

    let login_payload = crate::domains::user::model::LoginRequest {
      email: "empty-desc@example.com".to_string(),
      password: "password123".to_string(),
    };
    let (login_status, login_body) = post_json(app.clone(), "/api/v1/login", &login_payload).await;
    assert_eq!(login_status, StatusCode::OK);

    let login_response: crate::domains::user::model::LoginResponse =
      serde_json::from_slice(&login_body).expect("deserialize login response");
    let token = login_response.token;

    let payload = CreateRequestRequest {
      lat: 35.6812,
      lng: 139.7671,
      place_name: "東京".to_string(),
      description: "".to_string(), // Invalid: empty
    };

    let request = axum::http::Request::builder()
      .method("POST")
      .uri("/api/v1/requests")
      .header("authorization", format!("Bearer {}", token))
      .header("content-type", "application/json")
      .body(axum::body::Body::from(serde_json::to_string(&payload).unwrap()))
      .unwrap();

    let response = tower::ServiceExt::oneshot(app, request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    Ok(())
  }

  #[sqlx::test(migrations = "./migrations")]
  async fn create_request_place_name_too_long(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
    let app = app_with_pool(pool.clone()).await;

    let _user =
      crate::domains::user::model::User::create(&pool, "long-place@example.com", "Long Place", "password123").await?;

    sqlx::query!(
      "UPDATE users SET email_verified = true WHERE email = $1",
      "long-place@example.com"
    )
    .execute(&pool)
    .await?;

    let login_payload = crate::domains::user::model::LoginRequest {
      email: "long-place@example.com".to_string(),
      password: "password123".to_string(),
    };
    let (login_status, login_body) = post_json(app.clone(), "/api/v1/login", &login_payload).await;
    assert_eq!(login_status, StatusCode::OK);

    let login_response: crate::domains::user::model::LoginResponse =
      serde_json::from_slice(&login_body).expect("deserialize login response");
    let token = login_response.token;

    let payload = CreateRequestRequest {
      lat: 35.6812,
      lng: 139.7671,
      place_name: "a".repeat(256), // Invalid: > 255
      description: "テスト説明".to_string(),
    };

    let request = axum::http::Request::builder()
      .method("POST")
      .uri("/api/v1/requests")
      .header("authorization", format!("Bearer {}", token))
      .header("content-type", "application/json")
      .body(axum::body::Body::from(serde_json::to_string(&payload).unwrap()))
      .unwrap();

    let response = tower::ServiceExt::oneshot(app, request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    Ok(())
  }

  #[sqlx::test(migrations = "./migrations")]
  async fn get_requests_with_distance(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
    let app = app_with_pool(pool.clone()).await;

    let user =
      crate::domains::user::model::User::create(&pool, "distance-test@example.com", "Distance Test", "password123")
        .await?;

    // 東京タワー周辺のリクエスト
    super::super::repository::create(
      &pool,
      user.id,
      35.6812,
      139.7671,
      "東京タワー".to_string(),
      "写真1".to_string(),
    )
    .await?;
    // 大阪城周辺のリクエスト
    super::super::repository::create(
      &pool,
      user.id,
      34.6937,
      135.5023,
      "大阪城".to_string(),
      "写真2".to_string(),
    )
    .await?;
    // 札幌周辺のリクエスト
    super::super::repository::create(
      &pool,
      user.id,
      43.0642,
      141.3469,
      "札幌".to_string(),
      "写真3".to_string(),
    )
    .await?;

    // 東京タワーからの距離を計算
    let (status, body) = get(app, "/api/v1/requests?lat=35.6812&lng=139.7671").await;
    assert_eq!(status, StatusCode::OK);

    let response: super::super::model::RequestsResponse = serde_json::from_slice(&body).expect("deserialize response");
    assert!(response.requests.len() >= 3);

    // 全てのリクエストに distance フィールドがあることを確認
    for req in &response.requests {
      assert!(req.distance.is_some());
    }

    // 最初のリクエストが最も近い（東京タワー自身）
    assert!(response.requests[0].distance.unwrap() < 1000.0); // 1km以内

    Ok(())
  }

  #[sqlx::test(migrations = "./migrations")]
  async fn get_requests_without_location_no_distance(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
    let app = app_with_pool(pool.clone()).await;

    let user = crate::domains::user::model::User::create(&pool, "no-loc@example.com", "No Loc", "password123").await?;
    super::super::repository::create(
      &pool,
      user.id,
      35.6812,
      139.7671,
      "東京".to_string(),
      "テスト".to_string(),
    )
    .await?;

    let (status, body) = get(app, "/api/v1/requests").await;
    assert_eq!(status, StatusCode::OK);

    let response: super::super::model::RequestsResponse = serde_json::from_slice(&body).expect("deserialize response");

    // distance フィールドが None であることを確認
    for req in &response.requests {
      assert!(req.distance.is_none());
    }

    Ok(())
  }

  #[sqlx::test(migrations = "./migrations")]
  async fn get_requests_with_only_lat_no_distance(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
    let app = app_with_pool(pool.clone()).await;

    let user =
      crate::domains::user::model::User::create(&pool, "only-lat@example.com", "Only Lat", "password123").await?;
    super::super::repository::create(
      &pool,
      user.id,
      35.6812,
      139.7671,
      "東京".to_string(),
      "テスト".to_string(),
    )
    .await?;

    // lat のみ指定（lng なし）
    let (status, body) = get(app, "/api/v1/requests?lat=35.6812").await;
    assert_eq!(status, StatusCode::OK);

    let response: super::super::model::RequestsResponse = serde_json::from_slice(&body).expect("deserialize response");

    // distance フィールドが None であることを確認
    for req in &response.requests {
      assert!(req.distance.is_none());
    }

    Ok(())
  }

  #[sqlx::test(migrations = "./migrations")]
  async fn get_requests_with_only_lng_no_distance(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
    let app = app_with_pool(pool.clone()).await;

    let user =
      crate::domains::user::model::User::create(&pool, "only-lng@example.com", "Only Lng", "password123").await?;
    super::super::repository::create(
      &pool,
      user.id,
      35.6812,
      139.7671,
      "東京".to_string(),
      "テスト".to_string(),
    )
    .await?;

    // lng のみ指定（lat なし）
    let (status, body) = get(app, "/api/v1/requests?lng=139.7671").await;
    assert_eq!(status, StatusCode::OK);

    let response: super::super::model::RequestsResponse = serde_json::from_slice(&body).expect("deserialize response");

    // distance フィールドが None であることを確認
    for req in &response.requests {
      assert!(req.distance.is_none());
    }

    Ok(())
  }

  #[sqlx::test(migrations = "./migrations")]
  async fn get_requests_sorted_by_distance(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
    let app = app_with_pool(pool.clone()).await;

    let user = crate::domains::user::model::User::create(&pool, "sorted@example.com", "Sorted", "password123").await?;

    // 札幌（最も遠い）
    super::super::repository::create(
      &pool,
      user.id,
      43.0642,
      141.3469,
      "札幌".to_string(),
      "写真1".to_string(),
    )
    .await?;
    // 東京（最も近い）
    super::super::repository::create(
      &pool,
      user.id,
      35.6812,
      139.7671,
      "東京".to_string(),
      "写真2".to_string(),
    )
    .await?;
    // 大阪（中間）
    super::super::repository::create(
      &pool,
      user.id,
      34.6937,
      135.5023,
      "大阪".to_string(),
      "写真3".to_string(),
    )
    .await?;

    // 東京タワーからの距離順
    let (status, body) = get(app, "/api/v1/requests?lat=35.6812&lng=139.7671").await;
    assert_eq!(status, StatusCode::OK);

    let response: super::super::model::RequestsResponse = serde_json::from_slice(&body).expect("deserialize response");

    // 最初が最も近く、最後が最も遠いことを確認
    let distances: Vec<f64> = response.requests.iter().filter_map(|r| r.distance).collect();

    // 距離が昇順にソートされていることを確認
    for i in 0..distances.len() - 1 {
      assert!(distances[i] <= distances[i + 1]);
    }

    Ok(())
  }

  #[sqlx::test(migrations = "./migrations")]
  async fn get_request_by_id_success(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
    let app = app_with_pool(pool.clone()).await;

    let user =
      crate::domains::user::model::User::create(&pool, "get-by-id@example.com", "Get By ID", "password123").await?;
    let created = super::super::repository::create(
      &pool,
      user.id,
      35.6812,
      139.7671,
      "東京タワー".to_string(),
      "テスト説明".to_string(),
    )
    .await?;

    let (status, body) = get(app, &format!("/api/v1/requests/{}", created.id)).await;
    assert_eq!(status, StatusCode::OK);

    let response: super::super::model::Request = serde_json::from_slice(&body).expect("deserialize response");
    assert_eq!(response.id, created.id);
    assert_eq!(response.user_id, user.id);
    assert_eq!(response.lat, 35.6812);
    assert_eq!(response.lng, 139.7671);
    assert_eq!(response.place_name, "東京タワー");
    assert_eq!(response.description, "テスト説明");
    assert_eq!(response.status, "open");

    Ok(())
  }

  #[sqlx::test(migrations = "./migrations")]
  async fn get_request_by_id_not_found(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
    let app = app_with_pool(pool).await;

    let (status, _body) = get(app, "/api/v1/requests/99999").await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    Ok(())
  }
}
