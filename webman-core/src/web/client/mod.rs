mod apikey_client;
mod client_trait;
use crate::{node::Node, url::UrlTag};

use chrono::NaiveDateTime;

use crate::{node, url};
use anyhow::Result;

use client_trait::ClientTrait;

use super::resp::{UrlInsert, UrlTagSetter};

pub struct Client(Box<dyn ClientTrait>);

impl Client {
    pub async fn insert_urls(&self, name: &node::Name, url: UrlInsert) -> Result<()> {
        self.0.insert_urls(name.as_ref(), url).await
    }

    pub async fn sync_tags<N: AsRef<Node>>(&self, host: &N, remote: &N) -> Result<()> {
        self.0.sync_tag_logs(host.as_ref(), remote.as_ref()).await
    }

    pub async fn sync_urls<N: AsRef<Node>>(&self, host: &N, remote: &N) -> Result<()> {
        self.0.sync_urls(host.as_ref(), remote.as_ref()).await
    }

    pub async fn sync_all<N: AsRef<Node>>(&self, host: &N, remote: &N) -> Result<()> {
        self.sync_urls(host, remote).await?;
        // futures::try_join!(self.sync_urls(host, remote), self.sync_tags(host, remote))?;
        Ok(())
    }

    pub async fn last_import_time(
        &self,
        node: &node::Name,
        provider_name: &str,
    ) -> Result<NaiveDateTime> {
        self.0.last_import_time(node.as_ref(), provider_name).await
    }

    pub async fn set_tag(&self, node: &node::Name, tag: UrlTag, urls: Vec<String>) -> Result<()> {
        self.0
            .set_tag(node.as_ref(), &UrlTagSetter::new(tag, urls))
            .await
    }

    pub async fn get_all_tags(&self, node: &node::Name) -> Result<url::Tags> {
        self.0.get_all_tags(node.as_ref()).await
    }
}
