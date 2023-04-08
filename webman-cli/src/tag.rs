use crate::config::TagConfig;
use anyhow::{Context, Result};
use webman_core::{node, Client};

pub async fn load(client: &Client, target: &node::Name, tag_config: TagConfig) -> Result<()> {
    futures::future::join_all(
        tag_config
            .tags
            .into_iter()
            .map(|(tag, urls)| client.set_tag(target, tag, urls)),
    )
    .await;
    Ok(())
}

pub async fn write(client: &Client, target: &node::Name, tag_config: &TagConfig) -> Result<()> {
    let mut backup_path = tag_config.location.clone();
    backup_path.set_file_name("tags.yaml.backup");
    let backup_file = std::fs::File::create(&backup_path)
        .with_context(|| format!("error creating file: {:?}", backup_path))?;
    serde_yaml::to_writer(backup_file, &tag_config.tags).context("error writing backup tags")?;

    let records = client
        .get_all_tags(target)
        .await
        .context("error getting all tags")?;

    let new_file = std::fs::File::create(&tag_config.location)
        .with_context(|| format!("error creating file: {:?}", &tag_config.location))?;
    serde_yaml::to_writer(new_file, &records).context("error writing tags")?;
    Ok(())
}
