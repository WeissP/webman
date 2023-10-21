use super::{chromium::Chromium, establish_connection_in_place, Browser, BrowserMiner};

use anyhow::Result;
use chrono::NaiveDateTime;
use rusqlite::Connection;

#[derive(Default, Debug)]
pub struct Vivaldi(Chromium);

impl BrowserMiner for Vivaldi {
    type Timestamp = <Chromium as BrowserMiner>::Timestamp;

    const BROWSER_TYPE: Browser = Browser::Vivaldi;

    const QUERY: &'static str = <Chromium as BrowserMiner>::QUERY;

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
