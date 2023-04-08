use super::{provider, url, visit, Pool};
use crate::{
    browser::Browser,
    url::{Filter, Url, UrlPrivacy, UrlResult, UrlTag},
};
use anyhow::{Context, Result};
use chrono::NaiveDateTime;

impl Filter {
    pub async fn check_pid(&mut self, pool: &mut Pool) -> Result<()> {
        if self.p_id.is_none() {
            if let Some(name) = &self.provider_name {
                self.p_id = provider::try_find(&mut *pool, name).await?;
            }
        }
        Ok(())
    }
}

pub async fn get_urls(pool: &mut Pool, mut filter: Filter) -> Result<Vec<Url>> {
    filter.ready()?;
    filter.check_pid(&mut *pool).await?;

    if let Filter {
        p_id: Some(id),
        time_from: Some(since),
        browser: Some(browser),
        ..
    } = filter
    {
        sqlx::query_as!(
            Url,
            r#"
SELECT url, title, visit_count, last_visit_time
FROM urls INNER JOIN visits ON urls.id = visits.url_id  
WHERE provider_id = $1 AND last_visit_time > $2 AND browser_type = $3
"#,
            id,
            since,
            browser as Browser
        )
        .fetch_all(pool)
        .await
        .with_context(|| format!("could not get urls by p_id {:?} since {:?}", id, since))
    } else {
        Err(anyhow::anyhow!(
            "the required values in filter are missing: {:?}",
            filter
        ))
    }
}

pub async fn fuzzy_search(pool: &mut Pool, mut f: Filter) -> Result<Vec<UrlResult>> {
    f.init();
    f.check_pid(&mut *pool).await?;

    sqlx::query_as!(
        UrlResult,
        r#"
WITH grouped_visits AS (
SELECT url_id, SUM(visit_count) as visit_count, MAX(last_visit_time) as last_visit_time
FROM visits
WHERE ($1::smallint is null OR provider_id = $1)
  AND ($2::browser is null OR browser_type = $2) 
GROUP BY url_id
HAVING ($4::timestamp is null OR MAX(last_visit_time) >= $4) 
  AND ($5::timestamp is null OR MAX(last_visit_time) <= $5) 
) SELECT url as "url!", title as "title!", tag as "tag!:_", privacy as "privacy!:_"
FROM urls INNER JOIN grouped_visits ON urls.id = grouped_visits.url_id 
WHERE ($3::privacy is null OR privacy = $3)
  AND url ILIKE ALL ($6::text[]) 
  AND title ILIKE ALL ($7::text[]) 
  AND ($8::tag is null OR tag = $8)
ORDER BY tag DESC, last_visit_time DESC, visit_count DESC
limit $9
"#,
        f.p_id,
        f.browser as Option<Browser>,
        f.privacy as Option<UrlPrivacy>,
        f.time_from,
        f.time_to,
        f.url_segs.as_slice(),
        f.title_segs.as_slice(),
        f.tag as Option<UrlTag>,
        f.limit,
    )
    .fetch_all(pool)
    .await
    .with_context(|| format!("could not fuzzy search by {:?}", f))
}

pub fn unpack_urls(urls: Vec<Url>) -> (Vec<String>, Vec<String>, Vec<i32>, Vec<NaiveDateTime>) {
    let n = urls.len();
    urls.into_iter().fold(
        (
            Vec::with_capacity(n),
            Vec::with_capacity(n),
            Vec::with_capacity(n),
            Vec::with_capacity(n),
        ),
        |(mut url, mut title, mut count, mut time), u| {
            url.push(u.url);
            title.push(u.title);
            count.push(u.visit_count);
            time.push(u.last_visit_time);
            (url, title, count, time)
        },
    )
}

pub async fn insert_urls(
    pool: &mut Pool,
    name: &str,
    browser: Browser,
    urls: Vec<Url>,
    last_import_time: Option<NaiveDateTime>,
) -> Result<i32> {
    let p_id = provider::find_or_insert(&mut *pool, name).await?;
    insert_urls_by_id(pool, p_id, browser, urls, last_import_time).await
}

pub async fn insert_urls_by_id(
    pool: &mut Pool,
    p_id: i16,
    browser: Browser,
    urls: Vec<Url>,
    last_import_time: Option<NaiveDateTime>,
) -> Result<i32> {
    let (urls, titles, counts, times) = unpack_urls(urls);
    let url_ids = url::upsert_urls(&mut *pool, urls, titles).await?;
    assert_eq!(
        url_ids.len(),
        times.len(),
        "length of url_ids and urls mismatch"
    );
    let num = visit::upsert_visits(&mut *pool, p_id, browser, url_ids, counts, times).await?;
    provider::update_last_import_time(pool, p_id, last_import_time).await?;
    Ok(num)
}
