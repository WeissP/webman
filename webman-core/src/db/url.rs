use super::Pool;
use crate::url::{self, UrlTag};
use anyhow::{Context, Result};

pub async fn upsert_urls<'a>(
    pool: &mut Pool,
    urls: Vec<String>,
    titles: Vec<String>,
) -> Result<Vec<i32>> {
    let data = sqlx::query!(
        r#"
INSERT INTO urls(url,title)
SELECT * FROM UNNEST($1::text[],$2::text[])
ON CONFLICT (url) DO UPDATE SET title = EXCLUDED.title
RETURNING id
"#,
        &urls[..],
        &titles[..]
    )
    .fetch_all(pool)
    .await
    .context("could not insert urls")?;

    Ok(data.into_iter().map(|r| r.id).collect())
}

pub async fn set_tag<S: AsRef<[String]>>(pool: &mut Pool, urls: S, tag: UrlTag) -> Result<u64> {
    sqlx::query!(
        r#"
UPDATE urls SET tag = $1 WHERE url = ANY ($2::text[])
"#,
        tag as UrlTag,
        urls.as_ref()
    )
    .execute(pool)
    .await
    .with_context(|| format!("could not set tag {:?}", tag))
    .map(|r| r.rows_affected())
}

pub async fn get_all_tags(pool: &mut Pool) -> Result<url::Tags> {
    Ok(sqlx::query!(
        r#"
SELECT tag as "tag!:UrlTag", array_agg(url)
FROM urls
WHERE tag != 'normal'
GROUP BY tag
"#,
    )
    .fetch_all(pool)
    .await
    .context("could not get all url tags")?
    .into_iter()
    .map(|r| (r.tag, r.array_agg.unwrap_or_default()))
    .collect())
}
