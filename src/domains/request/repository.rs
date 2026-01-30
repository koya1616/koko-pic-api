use sqlx::{Executor, PgPool, Postgres};

use super::model::{Request, RequestWithDistance};

pub async fn find_all(db: &PgPool) -> Result<Vec<Request>, sqlx::Error> {
  find_all_with_executor(db).await
}

pub async fn find_all_with_executor<'e, E>(executor: E) -> Result<Vec<Request>, sqlx::Error>
where
  E: Executor<'e, Database = Postgres>,
{
  let requests = sqlx::query_as!(
    Request,
    r#"
      SELECT id, user_id, lat, lng, status, place_name, description, created_at
      FROM requests
      ORDER BY created_at DESC
    "#
  )
  .fetch_all(executor)
  .await?;

  Ok(requests)
}

pub async fn find_all_with_distance(
  db: &PgPool,
  user_lat: f64,
  user_lng: f64,
) -> Result<Vec<RequestWithDistance>, sqlx::Error> {
  find_all_with_distance_with_executor(db, user_lat, user_lng).await
}

pub async fn find_all_with_distance_with_executor<'e, E>(
  executor: E,
  user_lat: f64,
  user_lng: f64,
) -> Result<Vec<RequestWithDistance>, sqlx::Error>
where
  E: Executor<'e, Database = Postgres>,
{
  // ハヴァサイン公式をSQLで実装
  let rows = sqlx::query!(
    r#"
      SELECT
        id,
        user_id,
        lat,
        lng,
        status,
        place_name,
        description,
        created_at,
        (
          6371000 * acos(
            cos(radians($1)) * cos(radians(lat)) *
            cos(radians(lng) - radians($2)) +
            sin(radians($1)) * sin(radians(lat))
          )
        ) as distance
      FROM requests
      ORDER BY distance ASC
    "#,
    user_lat,
    user_lng
  )
  .fetch_all(executor)
  .await?;

  let requests = rows
    .into_iter()
    .map(|row| RequestWithDistance {
      id: row.id,
      user_id: row.user_id,
      lat: row.lat,
      lng: row.lng,
      status: row.status,
      place_name: row.place_name,
      description: row.description,
      created_at: Some(row.created_at),
      distance: row.distance,
    })
    .collect();

  Ok(requests)
}

pub async fn create(
  db: &PgPool,
  user_id: i32,
  lat: f64,
  lng: f64,
  place_name: String,
  description: String,
) -> Result<Request, sqlx::Error> {
  create_with_executor(db, user_id, lat, lng, place_name, description).await
}

pub async fn create_with_executor<'e, E>(
  executor: E,
  user_id: i32,
  lat: f64,
  lng: f64,
  place_name: String,
  description: String,
) -> Result<Request, sqlx::Error>
where
  E: Executor<'e, Database = Postgres>,
{
  let request = sqlx::query_as!(
    Request,
    r#"
      INSERT INTO requests (user_id, lat, lng, place_name, description)
      VALUES ($1, $2, $3, $4, $5)
      RETURNING id, user_id, lat, lng, status, place_name, description, created_at
    "#,
    user_id,
    lat,
    lng,
    place_name,
    description
  )
  .fetch_one(executor)
  .await?;

  Ok(request)
}

pub async fn find_by_id(db: &PgPool, id: i32) -> Result<Option<Request>, sqlx::Error> {
  find_by_id_with_executor(db, id).await
}

pub async fn find_by_id_with_executor<'e, E>(executor: E, id: i32) -> Result<Option<Request>, sqlx::Error>
where
  E: Executor<'e, Database = Postgres>,
{
  let request = sqlx::query_as!(
    Request,
    r#"
      SELECT id, user_id, lat, lng, status, place_name, description, created_at
      FROM requests
      WHERE id = $1
    "#,
    id
  )
  .fetch_optional(executor)
  .await?;

  Ok(request)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[sqlx::test(migrations = "./migrations")]
  async fn create_and_find_request(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
    let user =
      crate::domains::user::model::User::create(&pool, "repo-test@example.com", "Repo Test", "password123").await?;

    let created = create(
      &pool,
      user.id,
      35.6812,
      139.7671,
      "東京タワー".to_string(),
      "写真をお願いします".to_string(),
    )
    .await?;

    assert_eq!(created.user_id, user.id);
    assert_eq!(created.lat, 35.6812);
    assert_eq!(created.lng, 139.7671);
    assert_eq!(created.place_name, "東京タワー");
    assert_eq!(created.description, "写真をお願いします");
    assert_eq!(created.status, "open");

    let found = find_by_id(&pool, created.id).await?;
    assert!(found.is_some());

    let found = found.unwrap();
    assert_eq!(created.id, found.id);
    assert_eq!(created.user_id, found.user_id);

    Ok(())
  }

  #[sqlx::test(migrations = "./migrations")]
  async fn find_by_id_returns_none(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
    let found = find_by_id(&pool, 99999).await?;
    assert!(found.is_none());
    Ok(())
  }

  #[sqlx::test(migrations = "./migrations")]
  async fn find_all_requests_ordered_by_created_at_desc(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
    let user =
      crate::domains::user::model::User::create(&pool, "find-all@example.com", "Find All", "password123").await?;

    let req1 = create(
      &pool,
      user.id,
      35.6812,
      139.7671,
      "東京".to_string(),
      "説明1".to_string(),
    )
    .await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    let req2 = create(
      &pool,
      user.id,
      34.6937,
      135.5023,
      "大阪".to_string(),
      "説明2".to_string(),
    )
    .await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    let req3 = create(
      &pool,
      user.id,
      43.0642,
      141.3469,
      "札幌".to_string(),
      "説明3".to_string(),
    )
    .await?;

    let requests = find_all(&pool).await?;

    assert!(requests.len() >= 3);

    let created_ids: Vec<i32> = requests.iter().take(3).map(|r| r.id).collect();
    assert_eq!(created_ids[0], req3.id);
    assert_eq!(created_ids[1], req2.id);
    assert_eq!(created_ids[2], req1.id);

    Ok(())
  }

  #[sqlx::test(migrations = "./migrations")]
  async fn create_with_different_coordinates(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
    let user =
      crate::domains::user::model::User::create(&pool, "coords-test@example.com", "Coords Test", "password123").await?;

    let req1 = create(
      &pool,
      user.id,
      -90.0,
      -180.0,
      "南極点".to_string(),
      "最南端".to_string(),
    )
    .await?;
    assert_eq!(req1.lat, -90.0);
    assert_eq!(req1.lng, -180.0);

    let req2 = create(&pool, user.id, 90.0, 180.0, "北極点".to_string(), "最北端".to_string()).await?;
    assert_eq!(req2.lat, 90.0);
    assert_eq!(req2.lng, 180.0);

    let req3 = create(&pool, user.id, 0.0, 0.0, "赤道".to_string(), "0度0度".to_string()).await?;
    assert_eq!(req3.lat, 0.0);
    assert_eq!(req3.lng, 0.0);

    Ok(())
  }

  #[sqlx::test(migrations = "./migrations")]
  async fn create_with_long_place_name(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
    let user =
      crate::domains::user::model::User::create(&pool, "long-name@example.com", "Long Name", "password123").await?;

    let long_name = "a".repeat(255);
    let created = create(
      &pool,
      user.id,
      35.6812,
      139.7671,
      long_name.clone(),
      "テスト".to_string(),
    )
    .await?;
    assert_eq!(created.place_name, long_name);

    Ok(())
  }

  #[sqlx::test(migrations = "./migrations")]
  async fn find_all_with_distance_sorted(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
    let user =
      crate::domains::user::model::User::create(&pool, "distance-sort@example.com", "Distance Sort", "password123")
        .await?;

    // 札幌（最も遠い）
    create(
      &pool,
      user.id,
      43.0642,
      141.3469,
      "札幌".to_string(),
      "説明1".to_string(),
    )
    .await?;
    // 東京（最も近い）
    create(
      &pool,
      user.id,
      35.6812,
      139.7671,
      "東京".to_string(),
      "説明2".to_string(),
    )
    .await?;
    // 大阪（中間）
    create(
      &pool,
      user.id,
      34.6937,
      135.5023,
      "大阪".to_string(),
      "説明3".to_string(),
    )
    .await?;

    // 東京タワーからの距離で取得
    let requests = find_all_with_distance(&pool, 35.6812, 139.7671).await?;

    assert!(requests.len() >= 3);

    // 全てのリクエストにdistanceがあることを確認
    for req in &requests {
      assert!(req.distance.is_some());
    }

    // 距離が昇順にソートされていることを確認
    for i in 0..requests.len() - 1 {
      let dist1 = requests[i].distance.unwrap();
      let dist2 = requests[i + 1].distance.unwrap();
      assert!(
        dist1 <= dist2,
        "距離が昇順にソートされていません: {} > {}",
        dist1,
        dist2
      );
    }

    // 最初のリクエストが最も近い（東京）
    assert!(requests[0].distance.unwrap() < 1000.0); // 1km以内

    Ok(())
  }
}
