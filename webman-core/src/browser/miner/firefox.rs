use super::{establish_connection_copy, establish_connection_in_place, Browser, BrowserMiner};

use anyhow::Result;
use chrono::NaiveDateTime;
use rusqlite::Connection;

#[derive(Default, Debug)]
pub struct Firefox;

impl BrowserMiner for Firefox {
    type Timestamp = i64;

    const BROWSER_TYPE: Browser = Browser::Firefox;

    const QUERY: &'static str = r#"
SELECT url,title, visit_count,last_visit_date
FROM moz_places
WHERE last_visit_date > ? AND length(url) < 2500
"#;

    fn ts_to_datetime(&self, dt: Self::Timestamp) -> NaiveDateTime {
        let sec = dt / 1000000;
        NaiveDateTime::from_timestamp(sec, 0)
    }

    fn datetime_to_ts(&self, dt: NaiveDateTime) -> Self::Timestamp {
        dt.timestamp_micros()
    }

    fn establish_connection(&self, location: &str) -> Result<Connection> {
        establish_connection_copy(location, "firefox.db")
    }
}

#[cfg(test)]
mod tests {
    use crate::browser::BrowserSetting;

    use super::*;
    #[test]
    fn connect_firefox() {
        BrowserSetting {
            browser: Browser::Firefox,
            location: None,
        }
        .url_insert("firefox".to_string(), NaiveDateTime::from_timestamp(0, 0))
        .unwrap();
    }
}
