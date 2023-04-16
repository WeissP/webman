use super::config::config;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::Display,
    net::{Ipv4Addr, Ipv6Addr},
    ops::Deref,
};

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub enum Host {
    #[serde(alias = "domain")]
    Domain(String),
    #[serde(alias = "ipv4")]
    Ipv4(Ipv4Addr),
    #[serde(alias = "ipv6")]
    Ipv6(Ipv6Addr),
}

impl Display for Host {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Host::Domain(d) => d.fmt(f),
            Host::Ipv4(i) => i.fmt(f),
            Host::Ipv6(i) => i.fmt(f),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(from = "String")]
#[cfg_attr(feature = "server", derive(sqlx::Type))]
#[cfg_attr(feature = "server", sqlx(transparent))]
pub struct Name(String);

impl Name {
    pub fn new<S: ToString>(s: S) -> Name {
        Self(s.to_string())
    }
}

impl From<String> for Name {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Deref for Name {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<Node> for Name {
    fn as_ref(&self) -> &Node {
        config().nodes.get(self.0.as_str())
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct Provider {
    #[cfg_attr(feature = "server", sqlx(rename = "provider_name"))]
    pub name: String,
    pub last_import_time: chrono::NaiveDateTime,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct Node {
    pub host: Host,
    pub tls: bool,
    pub port: Option<u16>,
}

impl Node {
    pub fn api_url(&self, end_point: &str) -> String {
        format!(
            "{}://{}{}{}/api{}",
            if self.tls { "https" } else { "http" },
            self.host,
            if self.port.is_some() { ":" } else { "" },
            if let Some(p) = self.port {
                p.to_string()
            } else {
                String::new()
            },
            end_point
        )
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
#[serde(transparent)]
pub struct Nodes(HashMap<String, Node>);

impl Nodes {
    pub fn get(&self, name: &str) -> &Node {
        let c = self.0.get(name);
        assert!(
            c.is_some(),
            "could not find node by name {}, possibile nodes are {:?}",
            name,
            self
        );
        c.unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn api_url() {
        let node1 = Node {
            host: Host::Ipv4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
            tls: false,
            port: Some(7777),
        };
        assert_eq!(node1.api_url(""), "http://127.0.0.1:7777/api");

        let node2 = Node {
            host: Host::Domain("www.myserver.com".to_owned()),
            tls: true,
            port: None,
        };
        assert_eq!(node2.api_url(""), "https://www.myserver.com/api");
    }
}
