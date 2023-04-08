use super::*;
use crate::{
    browser::Browser,
    node,
    node::Provider,
    url::{Filter, Url, UrlPrivacy},
};
use chrono::{NaiveDateTime, TimeZone, Utc};

fn mock_provider() -> String {
    "test".to_owned()
}

fn mock_node() -> node::Name {
    node::Name::new("test")
}

fn mock_browser() -> crate::browser::Browser {
    Browser::Chrome
}

fn mock_urls(n: i32) -> Vec<Url> {
    vec![
        Url {
            url: "u1".to_owned(),
            title: "t1".to_owned(),
            visit_count: 1 + n,
            last_visit_time: mock_time(1 + n),
        },
        Url {
            url: "u2".to_owned(),
            title: "t2".to_owned(),
            visit_count: 2 + n,
            last_visit_time: mock_time(2 + n),
        },
        Url {
            url: "u3".to_owned(),
            title: "t3".to_owned(),
            visit_count: 3 + n,
            last_visit_time: mock_time(3 + n),
        },
    ]
}

fn mock_time(i: i32) -> NaiveDateTime {
    NaiveDateTime::from_timestamp(i.into(), 0)
}

async fn conn(p: sqlx::PgPool) -> Pool {
    p.acquire().await.expect("could not acquire Pool")
}

#[sqlx_database_tester::test(pool(variable = "pool"))]
async fn search() -> () {
    let mut conn = conn(pool).await;

    let p_id = provider::find_or_insert(&mut conn, &mock_node())
        .await
        .unwrap();

    joined::insert_urls_by_id(&mut conn, p_id, mock_browser(), mock_urls(0).to_vec(), None)
        .await
        .unwrap();

    let f = Filter {
        privacy: Some(UrlPrivacy::Normal),
        limit: 8,
        ..Default::default()
    };
    let res = joined::fuzzy_search(&mut conn, f).await.unwrap();
    assert_eq!(res.len(), 3);
    assert_eq!(res[0].title, "t3");

    joined::insert_urls_by_id(
        &mut conn,
        p_id,
        Browser::Safari,
        mock_urls(99).to_vec(),
        None,
    )
    .await
    .unwrap();

    let f = Filter {
        browser: Some(Browser::Safari),
        ..Default::default()
    };

    let res = joined::fuzzy_search(&mut conn, f).await.unwrap();
    assert_eq!(res.len(), 3);
    assert_eq!(res[0].title, "t3");

    let f = Filter {
        title_segs: vec!["%1%".to_owned()],
        url_segs: vec!["%1%".to_owned()],
        ..Default::default()
    };

    let res = joined::fuzzy_search(&mut conn, f).await.unwrap();
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].title, "t1");

    let res = joined::fuzzy_search(&mut conn, Filter::default())
        .await
        .unwrap();
    assert_eq!(res.len(), 3);
}

#[sqlx_database_tester::test(pool(variable = "pool"))]
async fn providers() {
    let mut conn = conn(pool).await;
    let all = provider::all(&mut conn).await.unwrap();
    assert_eq!(all.len(), 0);

    let _test_node = mock_node();
    provider::find_or_insert(&mut conn, &mock_provider())
        .await
        .unwrap();
    let all = provider::all(&mut conn).await.unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(
        all[0],
        Provider {
            name: mock_provider(),
            last_import_time: Utc.timestamp(0, 0).naive_utc()
        }
    )
}

#[sqlx_database_tester::test(pool(variable = "pool"))]
async fn find_or_insert_provider() {
    let mut conn = conn(pool).await;
    let test_node = mock_node();
    provider::find_or_insert(&mut conn, &test_node)
        .await
        .unwrap();
    let ps = provider::all(&mut conn).await.unwrap();
    assert_eq!(
        Provider {
            name: mock_provider(),
            last_import_time: Utc.timestamp(0, 0).naive_utc()
        },
        ps[0]
    );
    provider::find_or_insert(&mut conn, &test_node)
        .await
        .unwrap();
    assert_eq!(
        provider::find_or_insert(&mut conn, &test_node)
            .await
            .unwrap(),
        1
    )
}

async fn check(conn: &mut Pool, want_urls: Vec<Url>) {
    let f = Filter::bulk_urls(mock_node().to_string(), mock_browser(), mock_time(-1));
    let got = joined::get_urls(&mut *conn, f).await.unwrap();
    assert_eq!(want_urls, got);
}

#[sqlx_database_tester::test(pool(variable = "pool"))]
async fn insert_joined_urls() {
    let mut conn = conn(pool).await;

    let p_id = provider::find_or_insert(&mut conn, &mock_node())
        .await
        .unwrap();

    joined::insert_urls_by_id(
        &mut conn,
        p_id,
        mock_browser(),
        mock_urls(0)[..2].to_vec(),
        None,
    )
    .await
    .unwrap();
    check(&mut conn, mock_urls(0)[..2].to_vec()).await;

    joined::insert_urls_by_id(
        &mut conn,
        p_id,
        mock_browser(),
        mock_urls(0)[1..].to_vec(),
        None,
    )
    .await
    .unwrap();
    check(&mut conn, mock_urls(0)).await;

    joined::insert_urls_by_id(&mut conn, p_id, mock_browser(), mock_urls(99), None)
        .await
        .unwrap();
    check(&mut conn, mock_urls(99)).await;

    assert_eq!(
        provider::last_import_time(&mut conn, mock_node().as_str())
            .await
            .unwrap(),
        mock_time(102)
    )
}

#[sqlx_database_tester::test(pool(variable = "pool"))]
async fn upsert_urls() {
    let mut conn = conn(pool).await;
    let (urls, titles, _, _) = joined::unpack_urls(mock_urls(0)[..2].to_vec());
    let url_ids = url::upsert_urls(&mut conn, urls, titles).await.unwrap();
    assert_eq!(vec![1, 2], url_ids);

    let (urls, titles, _, _) = joined::unpack_urls(mock_urls(0)[1..].to_vec());
    let url_ids = url::upsert_urls(&mut conn, urls, titles).await.unwrap();
    assert_eq!(vec![2, 4], url_ids);
}

// #[ignore]
// #[sqlx_database_tester::test(pool(variable = "pool"))]
// async fn upsert_safari_urls() {
//     let mut conn = conn(pool).await;
//     let (urls, titles, _, _) = joined::unpack_urls(mock_urls(0)[..2].to_vec());
//     let url_ids = url::upsert_urls(&mut conn, urls, titles).await.unwrap();
//     assert_eq!(vec![1, 2], url_ids);

//     let (urls, titles, _, _) = joined::unpack_urls(mock_urls(0)[1..].to_vec());
//     let url_ids = url::upsert_urls(&mut conn, urls, titles).await.unwrap();
//     assert_eq!(vec![2, 4], url_ids);
// }

#[ignore]
#[sqlx_database_tester::test(pool(variable = "pool"))]
async fn upsert_bulk_urls() {
    let mut conn = conn(pool).await;
    let range = 1..90000;
    let bulk_urls: Vec<_> = range
        .clone()
        .map(|n| Url {
            url: format!("url{} {:?}", n, (1..100).collect::<Vec<_>>()),
            title: format!("title {}", n),
            visit_count: 1 + n,
            last_visit_time: mock_time(1 + n),
        })
        .collect();

    let (urls, titles, _, _) = joined::unpack_urls(bulk_urls);
    let url_ids = url::upsert_urls(&mut conn, urls, titles).await.unwrap();
    assert_eq!(range.clone().collect::<Vec<_>>(), url_ids);
}

// #[sqlx_database_tester::test(pool(variable = "pool"))]
// async fn upsert_long_url() {
//     let mut conn = conn(pool).await;
//     let url = format!("url {:?}", (1..10000).collect::<Vec<_>>());
//     let url_ids = url::upsert_urls(&mut conn, vec![url], vec!["titles".to_owned()])
//         .await
//         .unwrap();
//     assert_eq!(vec![1], url_ids);
// }
