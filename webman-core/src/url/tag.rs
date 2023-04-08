use serde::{Deserialize, Serialize};
use strum::EnumString;

use std::{borrow::Cow, collections::HashMap};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Deserialize, Serialize, Hash, EnumString)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "server", derive(sqlx::Type))]
#[cfg_attr(
    feature = "server",
    sqlx(rename_all = "lowercase"),
    sqlx(type_name = "tag")
)]
pub enum UrlTag {
    Normal,
    Saved,
    Favorite,
    ReadLater,
}

impl Default for UrlTag {
    fn default() -> Self {
        Self::Normal
    }
}

pub type Tags = std::collections::HashMap<UrlTag, Vec<String>>;

#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize)]
pub struct Record {
    pub tag: UrlTag,
    pub unix: u64,
}

impl Record {
    pub fn new(tag: UrlTag, unix: u64) -> Self {
        Self { tag, unix }
    }
}

fn cur_unix() -> u64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("SystemTime before UNIX EPOCH!")
        .as_secs()
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct History(HashMap<String, Record>);

impl History {
    /// If updated, return the updated tag
    fn upsert<'a, U: Into<Cow<'a, str>>>(&mut self, url: U, new_r: Record) -> Option<UrlTag> {
        let url = url.into();
        match self.0.get_mut(url.as_ref()) {
            Some(old_r) if new_r.unix >= old_r.unix => {
                *old_r = new_r;
            }
            Some(_) => return None,
            None => {
                self.0.insert(url.into_owned(), new_r);
            }
        }
        Some(new_r.tag)
    }

    pub fn insert(&mut self, url: String, tag: UrlTag) {
        self.0.insert(url, Record::new(tag, cur_unix()));
    }

    pub fn batch_insert(&mut self, urls: Vec<String>, tag: UrlTag) {
        let r = Record::new(tag, cur_unix());
        for url in urls {
            self.0.insert(url, r);
        }
    }

    pub fn merge<'a>(&mut self, o: &'a Self) -> Vec<(&'a str, UrlTag)> {
        o.0.iter()
            .flat_map(|(url, r)| self.upsert(url, *r).map(|t| (url.as_str(), t)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::{future::join_all, stream::FuturesUnordered};
    use once_cell::sync::Lazy;
    use rand::Rng;
    use std::{sync::Mutex, thread, time::Duration};

    static LEN: usize = 30;
    static HIS: Lazy<Mutex<History>> = Lazy::new(|| Mutex::new(History::default()));
    static RANDOMS: Lazy<Vec<usize>> = Lazy::new(|| {
        let mut rng = rand::thread_rng();
        (0..LEN).map(|_| rng.gen_range(0..3)).collect()
    });
    static WAIT_TIMES: Lazy<Vec<Duration>> = Lazy::new(|| {
        let mut rng = rand::thread_rng();
        (0..LEN)
            .map(|_| Duration::from_millis(100 * rng.gen_range(0..3)))
            .collect()
    });

    #[test]
    fn insert_tags() {
        let url = "insert".to_string();
        {
            let mut his = HIS.lock().unwrap();
            (*his).insert(url.clone(), UrlTag::Saved);
        }
        // thread::sleep(Duration::from_millis(1100));
        {
            let mut his = HIS.lock().unwrap();
            (*his).insert(url.clone(), UrlTag::Favorite);
        }
        assert_eq!(
            HIS.lock().unwrap().0.get(&url).unwrap().tag,
            UrlTag::Favorite
        )
    }

    #[test]
    fn upsert_tags() {
        let url = "upsert".to_string();
        {
            let mut his = HIS.lock().unwrap();
            (*his).insert(url.clone(), UrlTag::Saved);
        }
        {
            let mut his = HIS.lock().unwrap();
            (*his).upsert(url.clone(), Record::new(UrlTag::Favorite, cur_unix()));
        }
        assert_eq!(
            HIS.lock().unwrap().0.get(&url).unwrap().tag,
            UrlTag::Favorite
        )
    }

    #[tokio::test]
    async fn upsert_async() {
        let tasks = (0..LEN)
            .map(|i| {
                tokio::spawn(async move {
                    let url_tag = match RANDOMS.get(i).unwrap() {
                        0 => UrlTag::Normal,
                        1 => UrlTag::Saved,
                        2 => UrlTag::Favorite,
                        3 => UrlTag::ReadLater,
                        _ => panic!("invalid index for UrlTag"),
                    };
                    tokio::time::sleep(WAIT_TIMES.get(i).unwrap().to_owned()).await;
                    HIS.lock()
                        .unwrap()
                        .upsert("upsert_async".to_string(), Record::new(url_tag, cur_unix()))
                })
            })
            .collect::<FuturesUnordered<_>>();
        join_all(tasks).await;
    }

    #[test]
    #[ignore]
    fn merge_tags() {
        let mut his_other = History::default();
        his_other.insert("a".to_string(), UrlTag::ReadLater);

        // make sure the following tags are newer
        thread::sleep(Duration::from_millis(1100));
        {
            let mut his = HIS.lock().unwrap();
            (*his).insert("a".to_string(), UrlTag::Saved);
            (*his).insert("c".to_string(), UrlTag::Saved);
        }

        his_other.insert("b".to_string(), UrlTag::ReadLater);
        his_other.insert("c".to_string(), UrlTag::ReadLater);

        {
            let mut his = HIS.lock().unwrap();
            (*his).insert("d".to_string(), UrlTag::Saved);
            his.merge(&his_other);
        }

        for (url, tag) in [
            ("a", UrlTag::Saved),
            ("b", UrlTag::ReadLater),
            ("c", UrlTag::ReadLater),
            ("d", UrlTag::Saved),
        ] {
            assert_eq!(
                HIS.lock().unwrap().0.get(url).unwrap().tag,
                tag,
                "url:{}",
                url
            );
        }
    }
}
