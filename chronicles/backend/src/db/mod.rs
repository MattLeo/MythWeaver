use anyhow::Result;
use sqlx::{SqlitePool, sqlite::{SqlitePoolOptions, SqliteConnectOptions}};
use std::str::FromStr;

pub mod campaign;
pub mod player;
pub mod world;
pub mod items;
pub mod companions;
pub mod combat;
pub mod session;
pub mod time;
pub mod events;
pub mod fighter;
pub mod shop;
pub mod spells;

pub async fn connect(database_url: &str) -> Result<SqlitePool> {
    std::fs::create_dir_all("data")?;

    let options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    Ok(pool)
}

pub async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    let migration_sql = include_str!("../../migrations/001_initial_schema.sql");

    sqlx::query(migration_sql)
        .execute(pool)
        .await
        .map_err(|e| anyhow::anyhow!("Migration error: {}", e))?;

    tracing::info!("Database migrations complete");
    Ok(())
}