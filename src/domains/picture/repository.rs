use sqlx::{Executor, PgPool, Postgres};

use super::model::Picture;

pub async fn find_all(db: &PgPool) -> Result<Vec<Picture>, sqlx::Error> {
  find_all_with_executor(db).await
}

pub async fn find_all_with_executor<'e, E>(executor: E) -> Result<Vec<Picture>, sqlx::Error>
where
  E: Executor<'e, Database = Postgres>,
{
  let pictures = sqlx::query_as!(
    Picture,
    r#"
      SELECT id, user_id, image_url, created_at
      FROM pictures
      ORDER BY created_at DESC
    "#
  )
  .fetch_all(executor)
  .await?;

  Ok(pictures)
}
