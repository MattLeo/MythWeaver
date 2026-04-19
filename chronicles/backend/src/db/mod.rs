use anyhow::Result;
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use std::path::Path;

pub mod campaign;
pub mod player;
pub mod world;
pub mod items;
pub mod companions;
pub mod session;
pub mod time;
pub mod events;

pub async fn connect(database_url: &str) -> Result<SqlitePool> {
    // Ensure the directory exists
    if let Some(parent) = Path::new(database_url.trim_start_matches("sqlite:")).parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    Ok(pool)
}

pub async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    let migration_sql = include_str!("../../migrations/001_initial_schema.sql");
    
    // Run each statement separately
    for statement in migration_sql.split(';') {
        let trimmed = statement.trim();
        if !trimmed.is_empty() && !trimmed.starts_with("--") {
            sqlx::query(trimmed)
                .execute(pool)
                .await
                .map_err(|e| anyhow::anyhow!("Migration error on statement '{}': {}", &trimmed[..50.min(trimmed.len())], e))?;
        }
    }

    tracing::info!("Database migrations complete");
    Ok(())
}