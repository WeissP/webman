use super::{provider, Pool};
use crate::browser::Browser;
use anyhow::{Context, Result};
use chrono::NaiveDateTime;

async fn all_browsers_by_p_id(pool: &mut Pool, p_id: i16) -> Result<Vec<Browser>> {
    let rows = sqlx::query!(
        r#"
SELECT DISTINCT browser_type as "browser: Browser"
FROM visits
WHERE provider_id = $1
"#,
        p_id
    )
    .fetch_all(pool)
    .await
    .with_context(|| format!("could not find all browsers for p_id {:?}", p_id))?;
    Ok(rows.into_iter().map(|r| r.browser).collect())
}

pub async fn all_browsers(pool: &mut Pool, name: &str) -> Result<Vec<Browser>> {
    let p_id = provider::find(&mut *pool, name).await?;
    all_browsers_by_p_id(pool, p_id).await
}

pub async fn last_visit_time(pool: &mut Pool, p_id: i16) -> Result<NaiveDateTime> {
    sqlx::query!(
        r#"
SELECT MAX(last_visit_time) as time
FROM visits
WHERE provider_id = $1
"#,
        p_id
    )
    .fetch_one(pool)
    .await
    .with_context(|| format!("could not find last visit time of p_id: {:?}", p_id))
    .map(|r| {
        r.time
            .unwrap_or_else(|| NaiveDateTime::from_timestamp(0, 0))
    })
}

pub async fn upsert_visits(
    pool: &mut Pool,
    p_id: i16,
    browser: Browser,
    url_ids: Vec<i32>,
    visit_counts: Vec<i32>,
    last_visit_times: Vec<NaiveDateTime>,
) -> Result<i32> {
    sqlx::query!(
        r#"
SELECT upsert_visits($1,$2::browser, $3,$4,$5) as number;
"#,
        p_id,
        browser as Browser,
        url_ids.as_slice(),
        visit_counts.as_slice(),
        last_visit_times.as_slice()
    )
    .fetch_one(pool)
    .await
    .context("could not upsert visits")
    .map(|r| r.number.unwrap_or(0))
}
