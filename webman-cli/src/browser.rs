use anyhow::Result;

use serde::{Deserialize, Serialize};
use webman_core::{browser::*, node, Client, ToOk};

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
#[serde(transparent)]
pub struct Browsers(std::collections::HashMap<String, BrowserSetting>);

pub async fn provide(
    client: Client,
    provider_name: String,
    target: &node::Name,
    browsers: Browsers,
) -> Result<()> {
    let since = client.last_import_time(target, &provider_name).await?;
    log::debug!("browser settings: {:?}", browsers);
    let tasks = browsers
        .0
        .into_values()
        .flat_map(|b| b.url_insert(provider_name.clone(), since).to_ok().flatten())
        .map(|insert| client.insert_urls(target, insert));
    futures::future::try_join_all(tasks).await?;
    Ok(())
}
