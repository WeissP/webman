mod joined;
pub mod prelude;
mod provider;
mod url;
mod visit;

#[cfg(test)]
mod tests;

use anyhow::{Context, Result};
use sqlx::migrate::Migrator;

type Pool = sqlx::pool::PoolConnection<sqlx::Postgres>;

static MIGRATOR: Migrator = sqlx::migrate!();
pub async fn migrate(pool: &sqlx::PgPool) -> Result<()> {
    MIGRATOR.run(pool).await.context("could not migrate")
}
