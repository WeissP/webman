use super::{establish_connection_in_place, Browser, BrowserMiner};

use anyhow::Result;
use chrono::NaiveDateTime;
use rusqlite::Connection;

#[derive(Default, Debug)]
pub struct Chromium;

const CHROME_UNIX_DIFF: i64 = 11644473600;

impl BrowserMiner for Chromium {
    type Timestamp = i64;

    const BROWSER_TYPE: Browser = Browser::Chromium;

    const QUERY: &'static str = r#"
SELECT url,title, visit_count, last_visit_time
FROM urls
WHERE last_visit_time > ? AND length(url) < 2500
"#;

    fn ts_to_datetime(&self, dt: Self::Timestamp) -> NaiveDateTime {
        let sec = dt / 1000000 - CHROME_UNIX_DIFF;
        let nano_sec = (dt % 1000000) * 1000;
        NaiveDateTime::from_timestamp(
            sec,
            nano_sec
                .try_into()
                .expect("could not convert chromium micro seconds to u32"),
        )
    }

    fn datetime_to_ts(&self, dt: NaiveDateTime) -> Self::Timestamp {
        dt.timestamp_micros() + CHROME_UNIX_DIFF * 1000000
    }

    fn establish_connection(&self, location: &str) -> Result<Connection> {
        establish_connection_in_place(location)
    }
}
