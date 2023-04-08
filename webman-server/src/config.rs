use anyhow::{Context, Result};
use figment::Figment;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Deserializer, Serialize};
use std::time::Duration;
use webman_core::{config, init_fig, node};

pub static HOST: OnceCell<node::Name> = OnceCell::new();
pub static SYNC_NODES: OnceCell<Vec<SyncNode>> = OnceCell::new();

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct SyncNode {
    pub name: node::Name,
    #[serde(deserialize_with = "deserialize_duration")]
    pub interval: Duration,
}

fn deserialize_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    use parse_duration::parse;
    let buf = String::deserialize(deserializer)?;

    parse(&buf)
        .context("could not parse interval in config as duration")
        .map_err(serde::de::Error::custom)
}

pub fn rocket_figment() -> Figment {
    let rocket_fig = init_fig().select("server");
    let host_name: String = rocket_fig.extract_inner("host_node").unwrap_or_else(|_| {
        config()
            .name
            .clone()
            .expect("neither host_node nor name is set in config")
    });

    SYNC_NODES
        .set(match rocket_fig.extract_inner("sync") {
            Ok(nodes) => nodes,
            Err(e) => {
                log::warn!("sync nodes are empty: {}", e);
                vec![]
            }
        })
        .unwrap();

    HOST.set(node::Name::new(&host_name))
        .expect("could not set HOST");
    let host_node = config().nodes.get(&host_name);

    if let node::Host::Ipv4(ip) = host_node.host {
        Figment::from(rocket::Config::default())
            .merge(rocket_fig)
            .merge(("address", ip))
            .merge(("port", host_node.port))
    } else {
        panic!(
            "host node [{}] must have ipv4 address, but it was: {:?}",
            host_name, host_node.host
        )
    }
}
