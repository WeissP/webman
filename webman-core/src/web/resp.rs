use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::{
    browser::Browser,
    url::{Url, UrlTag},
};

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct Auth {
    pub api_key: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct UrlInsert {
    pub name: String,
    pub browser: Browser,
    pub urls: Vec<Url>,
    pub last_import_time: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct UrlTagSetter {
    pub tag: UrlTag,
    pub urls: Vec<String>,
}

impl UrlTagSetter {
    pub fn new(tag: UrlTag, urls: Vec<String>) -> Self {
        Self { tag, urls }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FakeUrl {
    pub url: String,
    pub browser: Browser,
}

impl From<FakeUrl> for UrlInsert {
    fn from(fu: FakeUrl) -> Self {
        let url = Url {
            url: fu.url,
            ..Default::default()
        };
        Self {
            name: String::new(),
            browser: fu.browser,
            urls: vec![url],
            last_import_time: None,
        }
    }
}
