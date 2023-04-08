use super::{establish_connection_copy, Browser, BrowserMiner};

use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use rusqlite::Connection;
pub struct Safari;

const SAFARI_UNIX_DIFF: i64 = 978307200;

impl Safari {
    pub fn last_import_time(&self, conn: &Connection) -> Result<NaiveDateTime> {
        let mut stmt = conn.prepare(
            r#"
WITH visit_time_by_origin AS (
SELECT MAX(visit_time) as last_visit_time
FROM history_visits
GROUP BY origin
) SELECT MIN(last_visit_time)
FROM visit_time_by_origin
"#,
        )?;
        let timestamp: f64 = stmt
            .query_row([], |row| row.get(0))
            .context("could not get safari last import time")?;
        Ok(self.ts_to_datetime(timestamp))
    }
}

impl BrowserMiner for Safari {
    type Timestamp = f64;

    const BROWSER_TYPE: Browser = Browser::Safari;
    const QUERY: &'static str = r#"
WITH last_visits AS (
SELECT history_item, title, visit_time as last_visit_time
FROM history_visits
WHERE visit_time > ? 
GROUP BY history_item
HAVING MAX(visit_time)
) SELECT HI.url, LV.title, HI.visit_count , LV.last_visit_time
FROM history_items HI INNER JOIN last_visits LV ON HI.id = LV.history_item
WHERE length(url) < 2500
"#;

    fn ts_to_datetime(&self, dt: Self::Timestamp) -> NaiveDateTime {
        let integer = dt as i64;
        let sec = integer + SAFARI_UNIX_DIFF;

        let micro_sec = ((dt.fract()) * 1000000.).round() as u32;
        let nano_sec = micro_sec * 1000;

        NaiveDateTime::from_timestamp(sec, nano_sec)
    }

    fn datetime_to_ts(&self, dt: NaiveDateTime) -> Self::Timestamp {
        let sec = dt.timestamp() - SAFARI_UNIX_DIFF;
        let micro_sec = dt.timestamp_subsec_micros();
        sec as f64 + micro_sec as f64 / 1000000.
    }

    fn establish_connection(&self, location: &str) -> Result<Connection> {
        establish_connection_copy(location, "safari.db")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, NaiveTime};

    #[test]
    fn safari_datatime() {
        let ts = 683889614.050203;

        let d = NaiveDate::from_ymd(2022, 9, 3);
        let t = NaiveTime::from_hms_micro(9, 20, 14, 50203);
        let datetime = NaiveDateTime::new(d, t);

        assert_eq!(Safari.ts_to_datetime(ts), datetime);
        assert_eq!(Safari.datetime_to_ts(datetime), ts);
    }

    #[ignore]
    #[test]
    fn safari_last_import_time() {
        let conn = Safari
            .establish_connection(
                "/home/weiss/rust/webman/webman-core/src/browser/browser_db_sample/safari.db",
            )
            .unwrap();
        let d = NaiveDate::from_ymd(2022, 9, 25);
        let t = NaiveTime::from_hms_micro(9, 49, 14, 247482);
        let datetime = NaiveDateTime::new(d, t);
        assert_eq!(Safari.last_import_time(&conn).unwrap(), datetime)
    }
}
