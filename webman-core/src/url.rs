pub mod tag;

use crate::browser::Browser;
use anyhow::Result;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use strum::EnumString;

pub use self::tag::{Tags, UrlTag};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Deserialize, Serialize, Hash, EnumString)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "server", derive(sqlx::Type))]
#[cfg_attr(
    feature = "server",
    sqlx(rename_all = "lowercase"),
    sqlx(type_name = "privacy")
)]
pub enum UrlPrivacy {
    Normal,
    Private,
}

impl Default for UrlPrivacy {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Default)]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct Url {
    pub url: String,
    pub title: String,
    pub visit_count: i32,
    pub last_visit_time: NaiveDateTime,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct UrlResult {
    pub url: String,
    pub title: String,
    pub tag: UrlTag,
    pub privacy: UrlPrivacy,
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Filter {
    pub p_id: Option<i16>,
    pub provider_name: Option<String>,
    pub privacy: Option<UrlPrivacy>,
    pub tag: Option<UrlTag>,
    pub browser: Option<Browser>,
    pub time_from: Option<NaiveDateTime>,
    pub time_to: Option<NaiveDateTime>,
    pub url_segs: Vec<String>,
    /// url_segs without %% symbol
    pub url_segs_raw: Vec<String>,
    pub title_segs: Vec<String>,
    /// title_segs without %% symbol
    pub title_segs_raw: Vec<String>,
    pub limit: i64,
    /// filter can only be used if ready is true, to make sure fields are initialized
    pub ready: bool,
}

fn to_like(segs: &[String]) -> Vec<String> {
    segs.iter().map(|seg| format!("%{}%", seg)).collect()
}

impl Filter {
    /// parse a query to filter, which will be split by space, the segment start with / will be recognised as url_segs.
    pub fn parse(mut query: String) -> Self {
        let mut res = Self::default();

        if query.starts_with(",url ") {
            query.replace_range(..5, "");
            res.url_segs = vec![query];
            res.ready = true;
            return res;
        }

        for arg in query.split(' ') {
            if let Some(url_seg) = arg.strip_prefix('/') {
                res.url_segs.push(format!("%{}%", &url_seg));
            } else if let Some(pattern) = arg.strip_prefix(',') {
                match pattern {
                    "p" | "privacy" => res.privacy = Some(UrlPrivacy::Private),
                    "n" | "normal" => res.tag = Some(UrlTag::Normal),
                    "s" | "saved" => res.tag = Some(UrlTag::Saved),
                    "f" | "favorite" => res.tag = Some(UrlTag::Favorite),
                    "r" | "readlater" => res.tag = Some(UrlTag::ReadLater),
                    _ => (),
                }
            } else {
                res.title_segs.push(format!("%{}%", &arg));
            }
        }
        res
    }

    pub fn bulk_urls(name: String, browser: Browser, since: NaiveDateTime) -> Self {
        Self {
            provider_name: Some(name),
            browser: Some(browser),
            time_from: Some(since),
            ready: true,
            ..Default::default()
        }
    }

    pub fn ready(&self) -> Result<()> {
        if !self.ready {
            Err(anyhow::anyhow!("the filter is not ready!: {:?}", self))
        } else {
            Ok(())
        }
    }

    pub fn init(&mut self) {
        if self.ready {
            return;
        }
        if self.title_segs.is_empty() && !self.title_segs_raw.is_empty() {
            self.title_segs = to_like(&self.title_segs_raw);
        }
        if self.url_segs.is_empty() && !self.url_segs_raw.is_empty() {
            self.url_segs = to_like(&self.url_segs_raw);
        }
        if self.limit <= 0 {
            self.limit = 20
        }
        self.ready = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn filter_parse() {
        let f = Filter::parse(",url http://localhost:3000/".to_string());
        assert_eq!(
            f,
            Filter {
                url_segs: vec!["http://localhost:3000/".to_string()],
                ready: true,
                ..Default::default()
            }
        )
    }
}
