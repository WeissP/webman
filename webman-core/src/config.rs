use std::{env, path::PathBuf};

use crate::node::Nodes;
use figment::{
    providers::{Format, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};

use once_cell::sync::OnceCell;

static CONFIG: OnceCell<Config> = OnceCell::new();

fn config_path() -> PathBuf {
    let mut p = PathBuf::new();
    match env::var("XDG_CONFIG_HOME") {
        Ok(c) => p.push(c),
        Err(_) => {
            let home = env::var("HOME").expect("neither env xdg_config_home nor home is set");
            p.push(home);
            p.push(".config");
        }
    };
    p.push("webman");
    p.push("webman.toml");
    if !p.exists() {
        panic!(
            "config file does not exist, please write your config in {:?}",
            p
        )
    }

    p
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct Config {
    pub name: Option<String>,
    pub api_key: String,
    pub nodes: Nodes,
}

pub fn config() -> &'static Config {
    CONFIG
        .get()
        .expect("config is not initialized, init_fig should be called first!")
}

pub fn init_fig() -> Figment {
    let fig = Figment::from(Toml::file(config_path()).nested());
    CONFIG
        .set(fig.extract().expect("could not construct CONFIG in config"))
        .expect("could not init CONFIG");
    fig
}
