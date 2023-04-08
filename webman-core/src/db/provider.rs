use super::Pool;
use crate::node::Provider;
use anyhow::{Context, Result};
use chrono::NaiveDateTime;

pub async fn last_import_time(pool: &mut Pool, name: &str) -> Result<NaiveDateTime> {
    match sqlx::query!(
        r#"
SELECT last_import_time
FROM providers
WHERE provider_name = $1
"#,
        name
    )
    .fetch_optional(pool)
    .await
    .with_context(|| format!("could not find last_import_time by name {}", name))
    {
        Ok(Some(r)) => Ok(r.last_import_time),
        Ok(None) => Ok(NaiveDateTime::from_timestamp(0, 0)),
        Err(e) => Err(e),
    }
}

pub async fn all(pool: &mut Pool) -> Result<Vec<Provider>> {
    sqlx::query_as!(
        Provider,
        r#"
SELECT provider_name as name, last_import_time
FROM providers
"#,
    )
    .fetch_all(pool)
    .await
    .context("could not find all providers")
}

pub async fn try_find(pool: &mut Pool, name: &str) -> Result<Option<i16>> {
    let res = sqlx::query!(
        r#"
SELECT id
FROM providers
WHERE provider_name = $1
"#,
        name
    )
    .fetch_optional(&mut *pool)
    .await
    .context("could not find provider by name")?;
    Ok(res.map(|x| x.id))
}

pub async fn find(pool: &mut Pool, name: &str) -> Result<i16> {
    match try_find(pool, name).await {
        Ok(Some(id)) => Ok(id),
        Ok(None) => Err(anyhow::anyhow!("could not find provider by name: {}", name)),
        Err(e) => Err(e),
    }
}

pub async fn find_or_insert(pool: &mut Pool, name: &str) -> Result<i16> {
    match try_find(&mut *pool, name).await {
        Ok(Some(id)) => Ok(id),
        Err(e) => Err(e),
        _ => sqlx::query!(
            r#"
INSERT INTO providers(provider_name) VALUES ($1)
RETURNING id
"#,
            name
        )
        .fetch_one(pool)
        .await
        .context("could not insert provider by name")
        .map(|r| r.id),
    }
}

pub async fn update_last_import_time(
    pool: &mut Pool,
    id: i16,
    last_import_time: Option<NaiveDateTime>,
) -> Result<i16> {
    let last_import_time = match last_import_time {
        Some(t) => t,
        None => super::visit::last_visit_time(&mut *pool, id).await?,
    };

    sqlx::query!(
        r#"
UPDATE providers SET last_import_time = $1 WHERE id = $2
RETURNING id
"#,
        last_import_time,
        id
    )
    .fetch_one(pool)
    .await
    .with_context(|| {
        format!(
            "could not update last_import_time as {:?} of provider {}",
            last_import_time, id
        )
    })
    .map(|r| r.id)
}
