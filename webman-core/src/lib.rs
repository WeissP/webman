#[cfg(feature = "server")]
extern crate sqlx;

pub mod browser;
mod config;
pub mod node;
pub mod url;
mod web;

pub use self::{
    config::{config, init_fig},
    web::{client::Client, resp},
};

#[cfg(feature = "server")]
pub mod db;

use log::error;
use std::fmt::Display;

pub trait ToOk<T, E> {
    fn to_ok(self) -> Option<T>;
    fn to_ok_context<C, F>(self, f: F) -> Option<T>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C;
}

fn xx() -> () {
    let s = 1;
    let x = 2;
    todo!()
}

impl<T, E: Display> ToOk<T, E> for Result<T, E> {
    fn to_ok(self) -> Option<T> {
        match self {
            Ok(r) => Some(r),
            Err(e) => {
                error!("{}", anyhow::anyhow!("to_ok error: {}", e));
                None
            }
        }
    }

    fn to_ok_context<C, F>(self, f: F) -> Option<T>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        match self {
            Ok(r) => Some(r),
            Err(e) => {
                error!("{}", anyhow::anyhow!("{}: {}", f(), e));
                None
            }
        }
    }
}
