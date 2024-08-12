#[cfg(feature = "browser")]
mod miner;
#[cfg(feature = "browser")]
pub use miner::BrowserSetting;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "server", derive(sqlx::Type))]
#[cfg_attr(
    feature = "server",
    sqlx(rename_all = "lowercase"),
    sqlx(type_name = "browser")
)]
pub enum Browser {
    Chromium,
    Chrome,
    Safari,
    Firefox,
    Vivaldi,
}

#[cfg(feature = "browser")]
impl Browser {
    pub fn default_location(&self) -> String {
        use std::env::consts::OS;
        let mut home = dirs::home_dir().expect("error detecting home dir");
        home.push(match (self, OS) {
            (Browser::Chromium, "linux") => ".config/chromium/Default/History",
            (Browser::Vivaldi, "linux") => ".config/vivaldi/Default/History",
            (Browser::Firefox, "linux") => ".mozilla/firefox/oqbprr8u.default/places.sqlite",
            (Browser::Chromium, "macos") => "Library/Application Support/Chromium/Default/History",
            (Browser::Safari, "macos") => "Library/Safari/History.db",
            (b, os) => panic!("Browser {:?} is not yet supported on {os}", b),
        });
        home.into_os_string()
            .into_string()
            .expect("could not detect browser's dir")
    }
}
