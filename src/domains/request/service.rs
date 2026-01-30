use sqlx::PgPool;

use crate::domains::request::{
  model::{CreateRequestRequest, RequestWithDistance, RequestsResponse},
  repository,
};

pub struct RequestService {
  pool: PgPool,
}

impl RequestService {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }

  pub async fn get_requests(
    &self,
    user_lat: Option<f64>,
    user_lng: Option<f64>,
  ) -> Result<RequestsResponse, sqlx::Error> {
    let requests = if let (Some(lat), Some(lng)) = (user_lat, user_lng) {
      repository::find_all_with_distance(&self.pool, lat, lng).await?
    } else {
      let reqs = repository::find_all(&self.pool).await?;
      reqs.into_iter().map(RequestWithDistance::from).collect()
    };

    Ok(RequestsResponse { requests })
  }

  pub async fn create_request(
    &self,
    user_id: i32,
    req: CreateRequestRequest,
  ) -> Result<crate::domains::request::model::Request, sqlx::Error> {
    repository::create(&self.pool, user_id, req.lat, req.lng, req.place_name, req.description).await
  }
}
