use crate::browser::Browsers;
use anyhow::{Context, Result};
use figment::{
    providers::{Format, Yaml},
    Figment,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;
use webman_core::{config, url};

pub fn target(fig: &Figment) -> String {
    fig.extract_inner("target").unwrap_or_else(|_| {
        config()
            .name
            .clone()
            .expect("neither target nor name is set in config")
    })
}

pub fn log_level(fig: &Figment) -> Result<simplelog::LevelFilter> {
    Ok(simplelog::LevelFilter::from_str(
        fig.find_value("log_level")
            .context("could not find log_level in [cli]")?
            .as_str()
            .unwrap(),
    )
    .expect("invalid log level in [cli]"))
}

pub fn log_file(fig: &Figment) -> Result<PathBuf> {
    fig.extract_inner("log_file")
        .context("could not find log_file in [cli]")
}

pub fn provider_name(fig: &Figment) -> String {
    fig.extract_inner("provider.provider_name")
        .unwrap_or_else(|_| {
            config()
                .name
                .clone()
                .expect("neither provider_name nor name is set in config")
        })
}

pub fn browsers(fig: &Figment) -> Browsers {
    fig.extract_inner("provider.browsers")
        .expect("could not construct Browsers in [cli]")
}

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct TagConfig {
    pub location: PathBuf,
    pub tags: url::Tags,
}

pub fn tags(fig: &Figment) -> Result<TagConfig> {
    let tag_path: String = fig
        .extract_inner("tags_file")
        .context("cound not find config tags_file in [cli]")?;
    let fig = Figment::from(Yaml::file(tag_path));
    let tags: url::Tags = fig.extract().context("could not read tags in tags.yaml")?;
    let source = fig
        .metadata()
        .next()
        .context("error reading tags.yaml")?
        .source
        .clone();
    let location = match source {
        Some(figment::Source::File(p)) => p,
        any => Err(anyhow::anyhow!(
            "tags.yaml is not from file but from {:?}",
            any
        ))?,
    };
    Ok(TagConfig { location, tags })
}
