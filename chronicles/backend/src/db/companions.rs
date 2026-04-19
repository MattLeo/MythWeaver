// companions.rs
use anyhow::Result;
use sqlx::SqlitePool;
use uuid::Uuid;
use crate::models::*;

pub async fn create_companion(
    pool: &SqlitePool,
    campaign_id: &str,
    data: &serde_json::Value,
) -> Result<Companion> {
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO companions (
            id, campaign_id, name, companion_type, description,
            personality, disposition, current_hp, max_hp, armor_class,
            current_location_id, notes
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(campaign_id)
    .bind(data["name"].as_str().unwrap_or("Unknown"))
    .bind(data["companion_type"].as_str().unwrap_or("ally"))
    .bind(data["description"].as_str().unwrap_or(""))
    .bind(data["personality"].as_str())
    .bind(data["disposition"].as_str().unwrap_or("friendly"))
    .bind(data["current_hp"].as_i64().unwrap_or(10))
    .bind(data["max_hp"].as_i64().unwrap_or(10))
    .bind(data["armor_class"].as_i64().unwrap_or(10))
    .bind(data["location_id"].as_str())
    .bind(data["notes"].as_str())
    .execute(pool)
    .await?;

    get_companion(pool, &id).await?.ok_or_else(|| anyhow::anyhow!("Companion not found after creation"))
}

pub async fn get_companion(pool: &SqlitePool, id: &str) -> Result<Option<Companion>> {
    Ok(sqlx::query_as::<_, Companion>("SELECT * FROM companions WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

pub async fn get_active_companions(pool: &SqlitePool, campaign_id: &str) -> Result<Vec<Companion>> {
    Ok(sqlx::query_as::<_, Companion>(
        "SELECT * FROM companions WHERE campaign_id = ? AND is_active = 1"
    )
    .bind(campaign_id)
    .fetch_all(pool)
    .await?)
}

pub async fn update_companion(pool: &SqlitePool, id: &str, data: &serde_json::Value) -> Result<()> {
    sqlx::query(
        "UPDATE companions SET
            disposition = COALESCE(?, disposition),
            current_location_id = COALESCE(?, current_location_id),
            is_active = COALESCE(?, is_active),
            notes = COALESCE(?, notes),
            updated_at = datetime('now')
         WHERE id = ?"
    )
    .bind(data["disposition"].as_str())
    .bind(data["location_id"].as_str())
    .bind(data["is_active"].as_bool())
    .bind(data["notes"].as_str())
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn apply_companion_damage(
    pool: &SqlitePool,
    companion_id: &str,
    damage: i64,
) -> Result<(i64, bool)> {
    let companion = get_companion(pool, companion_id).await?
        .ok_or_else(|| anyhow::anyhow!("Companion not found"))?;

    let new_hp = (companion.current_hp - damage).max(0);
    let is_dead = new_hp == 0;

    sqlx::query(
        "UPDATE companions SET current_hp = ?, is_alive = ?, updated_at = datetime('now') WHERE id = ?"
    )
    .bind(new_hp)
    .bind(!is_dead)
    .bind(companion_id)
    .execute(pool)
    .await?;

    Ok((new_hp, is_dead))
}

pub async fn apply_companion_healing(
    pool: &SqlitePool,
    companion_id: &str,
    healing: i64,
) -> Result<i64> {
    let companion = get_companion(pool, companion_id).await?
        .ok_or_else(|| anyhow::anyhow!("Companion not found"))?;

    let new_hp = (companion.current_hp + healing).min(companion.max_hp);

    sqlx::query(
        "UPDATE companions SET current_hp = ?, updated_at = datetime('now') WHERE id = ?"
    )
    .bind(new_hp)
    .bind(companion_id)
    .execute(pool)
    .await?;

    Ok(new_hp)
}