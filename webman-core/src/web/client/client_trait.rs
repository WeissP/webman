use crate::{
    browser::Browser,
    node::{Node, Provider},
    url::{self, tag, Url},
    web::resp::{UrlInsert, UrlTagSetter},
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::NaiveDateTime;
use log::info;
use reqwest::RequestBuilder;
use serde::Deserialize;

async fn get_json<C, T>(
    c: &C,
    node: &Node,
    end_point: &str,
    params: Option<&[(&str, &str)]>,
) -> Result<T>
where
    C: ClientTrait + ?Sized,
    for<'de> T: Deserialize<'de>,
{
    let req = c.get(node, end_point);
    if let Some(p) = params {
        req.query(&p)
    } else {
        req
    }
    .send()
    .await
    .with_context(|| format!("error getting to {}", end_point))?
    .json()
    .await
    .with_context(|| {
        format!(
            "error converting body of type {}",
            std::any::type_name::<T>()
        )
    })
}

#[async_trait]
pub trait ClientTrait: 'static + Send + Sync {
    fn post(&self, node: &Node, end_point: &str) -> RequestBuilder;
    fn get(&self, node: &Node, end_point: &str) -> RequestBuilder;

    async fn last_import_time(&self, node: &Node, name: &str) -> Result<NaiveDateTime> {
        get_json(
            self,
            node,
            "/provider/last_import_time",
            Some(&[("name", name)]),
        )
        .await
    }

    async fn get_urls(&self, node: &Node, filter: url::Filter) -> Result<Vec<Url>> {
        let body = self
            .post(node, "/urls/filter")
            .json(&filter)
            .send()
            .await
            .context("error posting to /urls/filter")?
            .bytes()
            .await?;

        rmp_serde::from_slice(&body).context("could not convert bytes to Vec<Url>")
    }

    async fn insert_urls(&self, node: &Node, url: UrlInsert) -> Result<()> {
        log::debug!(
            "trying to send urls with length {} to {:?}",
            url.urls.len(),
            node
        );
        self.post(node, "/urls/insert")
            .body(rmp_serde::to_vec(&url)?)
            .send()
            .await
            .context("error posting to /urls/insert")?;
        Ok(())
    }

    async fn all_browsers(&self, node: &Node, name: &str) -> Result<Vec<Browser>> {
        get_json(self, node, "/provider/browsers", Some(&[("name", name)])).await
    }

    async fn provider_info(&self, node: &Node) -> Result<Vec<Provider>> {
        get_json(self, node, "/provider/info", None).await
    }

    async fn set_tag(&self, node: &Node, tag_setter: &UrlTagSetter) -> Result<()> {
        self.post(node, "/urls/tag")
            .json(&tag_setter)
            .send()
            .await
            .context("error posting to /urls/tag")?;
        Ok(())
    }

    async fn get_all_tags(&self, node: &Node) -> Result<url::Tags> {
        self.get(node, "/urls/tags")
            .send()
            .await
            .context("error getting to /urls/tags")?
            .json::<url::Tags>()
            .await
            .context("error converting body to url::Tags")
    }

    async fn get_tag_log(&self, node: &Node) -> Result<tag::History> {
        get_json(self, node, "/memory/tag_log", None).await
    }

    async fn update_tag_log(&self, node: &Node, log: tag::History) -> Result<()> {
        self.post(node, "/memory/tag_log")
            .json(&log)
            .send()
            .await
            .context("error posting to /memory/tag_log")?;
        Ok(())
    }

    async fn sync_tag_logs_one_side(&self, from: &Node, to: &Node) -> Result<()> {
        let from_log = self.get_tag_log(from).await?;
        self.update_tag_log(to, from_log).await?;
        Ok(())
    }

    async fn sync_tag_logs(&self, host: &Node, remote: &Node) -> Result<()> {
        info!("start to sync tags between {:?} and {:?}", host, remote);

        let host_task = self.sync_tag_logs_one_side(remote, host);
        let remote_task = self.sync_tag_logs_one_side(host, remote);

        futures::try_join!(host_task, remote_task)?;
        Ok(())
    }

    async fn sync_urls(&self, host: &Node, remote: &Node) -> Result<()> {
        info!("start to sync urls between {:?} and {:?}", host, remote);
        use std::cmp::Ordering;
        let unix = NaiveDateTime::from_timestamp(0, 0);
        // (host import time, remote import time)
        let mut hm = std::collections::HashMap::<String, (NaiveDateTime, NaiveDateTime)>::new();

        // insert host to hashmap
        for Provider {
            name,
            last_import_time,
        } in self.provider_info(host).await?
        {
            hm.insert(name, (last_import_time, unix));
        }

        // update remote import time to hashmap
        for Provider {
            name,
            last_import_time,
        } in self.provider_info(remote).await?
        {
            let ent = hm.entry(name).or_insert((unix, unix));
            ent.1 = last_import_time;
        }

        for (name, (host_time, remote_time)) in hm.into_iter() {
            let (since, older, newer) = match host_time.cmp(&remote_time) {
                Ordering::Less => (host_time, host, remote),
                Ordering::Equal => continue,
                Ordering::Greater => (remote_time, remote, host),
            };
            log::info!(
                "trying to sync urls since {} from {:?} to {:?}",
                since,
                newer,
                older
            );
            let browsers = self.all_browsers(newer, &name).await?;
            for browser in browsers {
                let urls = self
                    .get_urls(
                        newer,
                        url::Filter::bulk_urls(name.to_owned(), browser, since),
                    )
                    .await?;
                let insert = UrlInsert {
                    name: name.to_owned(),
                    browser,
                    urls,
                    last_import_time: None,
                };
                self.insert_urls(older, insert).await?;
            }
        }

        Ok(())
    }
}
