use super::{firefox::Firefox, Browser, BrowserMiner};

use anyhow::Result;
use chrono::NaiveDateTime;
use rusqlite::Connection;

#[derive(Default, Debug)]
pub struct Floorp(Firefox);

impl BrowserMiner for Floorp {
    type Timestamp = i64;

    const BROWSER_TYPE: Browser = Browser::Floorp;

    const QUERY: &'static str = <Firefox as BrowserMiner>::QUERY;

    fn ts_to_datetime(&self, dt: Self::Timestamp) -> NaiveDateTime {
        self.0.ts_to_datetime(dt)
    }

    fn datetime_to_ts(&self, dt: NaiveDateTime) -> Self::Timestamp {
        self.0.datetime_to_ts(dt)
    }

    fn establish_connection(&self, location: &str) -> Result<Connection> {
        self.0.establish_connection(location)
    }
}
