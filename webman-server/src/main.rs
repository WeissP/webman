#[macro_use]
extern crate rocket;

mod config;
mod server;

use config::{HOST, SYNC_NODES};
use tokio::{task, time};
use webman_core::{config, Client, ToOk};

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let fig = config::rocket_figment();
    // webman_core::db::migrate(&db_url).await.expect("");
    for n in SYNC_NODES.get().unwrap() {
        task::spawn(async move {
            let host = HOST.get().unwrap();
            let client = Client::with_apikey(&config().api_key);
            let mut interval = time::interval(n.interval);
            loop {
                interval.tick().await;
                log::debug!(
                    "Syncing nodes between host {} and {}",
                    host.as_str(),
                    &n.name
                );
                client.sync_all(host, &n.name).await.to_ok();
            }
        });
    }

    if let Err(e) = server::launch(fig).launch().await {
        println!("Whoops! Rocket didn't launch!");
        // We drop the error to get a Rocket-formatted panic.
        drop(e);
    };

    Ok(())
}
