mod browser;
mod config;
mod tag;

use clap::{Parser, Subcommand};
use simplelog::{SimpleLogger, WriteLogger};
use std::str::FromStr;
use webman_core::{config, init_fig, node, Client, ToOk};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(short, long, value_parser)]
    target: Option<String>,
    #[clap(long, value_parser)]
    log_level: Option<String>,
    #[clap(long, action)]
    log_file: bool,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[clap(subcommand)]
    Tag(Tag),

    Provide,
    SyncServer {
        host: String,
        remote: String,
    },
}

#[derive(Subcommand)]
enum Tag {
    Add {
        #[clap(value_parser, required = true)]
        tag: String,
        #[clap(value_parser, required = true)]
        url: String,
        #[clap(short, long, action)]
        out: bool,
    },
    Read,
    Write,
}

#[tokio::main]
async fn main() {
    let fig = init_fig().select("cli");

    let cli = Cli::parse();
    let target = cli.target.unwrap_or_else(|| config::target(&fig));
    let target = node::Name::new(&target);
    let log_level = cli
        .log_level
        .map(|s| simplelog::LevelFilter::from_str(&s).unwrap())
        .unwrap_or_else(|| {
            config::log_level(&fig)
                .to_ok()
                .unwrap_or(simplelog::LevelFilter::Info)
        });

    let log_file: Option<_> = if cli.log_file {
        Some(
            config::log_file(&fig)
                .expect("log_file option is set to true but the location is not set in config"),
        )
    } else {
        None
    };

    let log_config = simplelog::Config::default();
    match log_file {
        Some(file) => WriteLogger::init(
            log_level,
            log_config,
            std::fs::OpenOptions::new().append(true).open(file).unwrap(),
        ),
        None => SimpleLogger::init(log_level, log_config),
    }
    .unwrap();

    let client = Client::with_apikey(&config().api_key);

    match cli.command {
        Commands::Tag(tag) => {
            let tag_config = config::tags(&fig).unwrap();
            match tag {
                Tag::Add { tag, url, out } => {
                    client
                        .set_tag(&target, tag.as_str().try_into().unwrap(), vec![url])
                        .await
                        .unwrap();
                    if out {
                        tag::write(&client, &target, &tag_config).await.unwrap()
                    }
                }
                Tag::Read => tag::load(&client, &target, tag_config).await.unwrap(),
                Tag::Write => tag::write(&client, &target, &tag_config).await.unwrap(),
            }
        }
        Commands::Provide => browser::provide(
            client,
            config::provider_name(&fig),
            &target,
            config::browsers(&fig),
        )
        .await
        .unwrap(),
        Commands::SyncServer { host, remote } => client
            .sync_urls(&node::Name::from(host), &node::Name::from(remote))
            .await
            .unwrap(),
    }
}
