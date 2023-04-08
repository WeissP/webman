mod chromium;
mod safari;

use super::Browser;
use crate::{url::Url, web::resp::UrlInsert, ToOk};
use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use log::info;
use rusqlite::{types::FromSql, Connection, OpenFlags, ToSql};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt::Debug};
const TEMP_DIR: &str = "/tmp";
use chromium::Chromium;
use safari::Safari;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct BrowserSetting {
    pub browser: Browser,
    pub location: Option<String>,
}

impl BrowserSetting {
    fn location(&self) -> Cow<'_, str> {
        match &self.location {
            Some(lo) => lo.into(),
            None => self.browser.default_location().into(),
        }
    }

    pub fn url_insert(&self, provider: String, since: NaiveDateTime) -> Result<Option<UrlInsert>> {
        let loc = self.location();
        let mut last_import_time = None;
        let urls = match self.browser {
            Browser::Chromium => {
                let conn = Chromium.establish_connection(&loc).with_context(|| {
                    format!("could not connect to {:?} browser db", self.browser)
                })?;
                Chromium.mine_urls(&conn, since)
            }
            Browser::Safari => {
                let conn = Safari.establish_connection(&loc).with_context(|| {
                    format!("could not connect to {:?} browser db", self.browser)
                })?;
                last_import_time = Some(Safari.last_import_time(&conn)?);
                Safari.mine_urls(&conn, since)
            }
            Browser::Chrome => todo!(),
            Browser::Firefox => todo!(),
        }?;
        if urls.is_empty() {
            info!("no new urls found");
            Ok(None)
        } else {
            info!("got urls with len: {}", urls.len());
            Ok(Some(UrlInsert {
                name: provider,
                browser: self.browser,
                urls,
                last_import_time,
            }))
        }
    }
}

pub fn establish_connection_in_place(location: &str) -> Result<Connection> {
    let uri = format!("file:{}?nolock=true", location);
    let flags = OpenFlags::SQLITE_OPEN_URI | OpenFlags::SQLITE_OPEN_READ_ONLY;
    Connection::open_with_flags(&uri, flags)
        .with_context(|| format!("could not connect browser database with {}", uri))
}

pub fn establish_connection_copy(location: &str, db_name: &str) -> Result<Connection> {
    let copied_path = format!("{}/{}", TEMP_DIR, db_name);
    let uri = &copied_path;
    std::fs::copy(location, &copied_path)?;
    std::fs::copy(
        format!("{}-wal", location),
        format!("{}/{}-wal", TEMP_DIR, db_name),
    )?;
    let flags = OpenFlags::SQLITE_OPEN_URI | OpenFlags::SQLITE_OPEN_READ_ONLY;
    Connection::open_with_flags(uri, flags)
        .with_context(|| format!("could not connect browser database with {}", uri))
}

impl Url {
    fn from_sql(
        url: Option<String>,
        title: Option<String>,
        visit_count: Option<i32>,
        last_visit_time: NaiveDateTime,
    ) -> Url {
        Url {
            url: url.unwrap_or_default(),
            title: title.unwrap_or_default(),
            visit_count: visit_count.unwrap_or_default(),
            last_visit_time,
        }
    }
}

pub trait BrowserMiner {
    type Timestamp: Sized + Debug + ToSql + FromSql;
    const BROWSER_TYPE: Browser;
    const QUERY: &'static str;

    fn ts_to_datetime(&self, ts: Self::Timestamp) -> NaiveDateTime;
    fn datetime_to_ts(&self, dt: NaiveDateTime) -> Self::Timestamp;
    fn establish_connection(&self, location: &str) -> Result<Connection>;

    fn mine_urls(
        &self,
        conn: &Connection,
        since: NaiveDateTime,
    ) -> anyhow::Result<Vec<crate::url::Url>> {
        let datetime = self.datetime_to_ts(since);
        log::info!(
            "trying to get urls since {}({:?}:{:?})",
            since,
            Self::BROWSER_TYPE,
            datetime
        );
        let mut stmt = conn.prepare(Self::QUERY)?;
        let rows = stmt
            .query_map([datetime], |row| {
                Ok(Url::from_sql(
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    self.ts_to_datetime(row.get(3)?),
                ))
            })
            .context("could not query browser db")?;
        Ok(rows.flat_map(|r| r.to_ok()).collect())
    }
}
