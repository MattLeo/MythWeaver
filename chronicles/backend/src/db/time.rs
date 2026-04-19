use anyhow::Result;
use sqlx::SqlitePool;
use uuid::Uuid;
use crate::models::*;

// ─── Time ─────────────────────────────────────────────────────────────────────

pub async fn init_campaign_time(pool: &SqlitePool, campaign_id: &str) -> Result<CampaignTime> {
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT OR IGNORE INTO campaign_time (id, campaign_id) VALUES (?, ?)"
    )
    .bind(&id)
    .bind(campaign_id)
    .execute(pool)
    .await?;

    get_campaign_time(pool, campaign_id).await?
        .ok_or_else(|| anyhow::anyhow!("Failed to init campaign time"))
}

pub async fn get_campaign_time(pool: &SqlitePool, campaign_id: &str) -> Result<Option<CampaignTime>> {
    Ok(sqlx::query_as::<_, CampaignTime>(
        "SELECT * FROM campaign_time WHERE campaign_id = ?"
    )
    .bind(campaign_id)
    .fetch_optional(pool)
    .await?)
}

pub async fn advance_time(
    pool: &SqlitePool,
    campaign_id: &str,
    steps: i64,
    reason: &str,
) -> Result<CampaignTime> {
    let current = get_campaign_time(pool, campaign_id).await?
        .ok_or_else(|| anyhow::anyhow!("No time record for campaign"))?;

    let time_of_day_order = [
        "dawn", "morning", "midday", "afternoon",
        "dusk", "evening", "night", "deep_night"
    ];

    let current_idx = time_of_day_order
        .iter()
        .position(|&t| t == current.time_of_day)
        .unwrap_or(1);

    let total_steps = current_idx as i64 + steps;
    let new_idx = (total_steps % 8) as usize;
    let days_passed = total_steps / 8;
    let new_day = current.current_day + days_passed;
    let new_time = time_of_day_order[new_idx];

    // Advance season every 90 days
    let new_season = match (new_day / 90) % 4 {
        0 => "spring",
        1 => "summer",
        2 => "autumn",
        _ => "winter",
    };

    tracing::info!("Time advanced by {} steps ({}): Day {} {}", steps, reason, new_day, new_time);

    sqlx::query(
        "UPDATE campaign_time SET
            time_of_day = ?,
            current_day = ?,
            season = ?,
            updated_at = datetime('now')
         WHERE campaign_id = ?"
    )
    .bind(new_time)
    .bind(new_day)
    .bind(new_season)
    .bind(campaign_id)
    .execute(pool)
    .await?;

    get_campaign_time(pool, campaign_id).await?
        .ok_or_else(|| anyhow::anyhow!("Failed to get updated time"))
}

// ─── Events ───────────────────────────────────────────────────────────────────

pub async fn create_event_table(
    pool: &SqlitePool,
    campaign_id: &str,
    name: &str,
    location_type: Option<&str>,
    trigger_type: &str,
    trigger_chance: i64,
) -> Result<EventTable> {
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO event_tables (id, campaign_id, name, location_type, trigger_type, trigger_chance)
         VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(campaign_id)
    .bind(name)
    .bind(location_type)
    .bind(trigger_type)
    .bind(trigger_chance)
    .execute(pool)
    .await?;

    get_event_table(pool, &id).await?
        .ok_or_else(|| anyhow::anyhow!("Event table not found after creation"))
}

pub async fn get_event_table(pool: &SqlitePool, id: &str) -> Result<Option<EventTable>> {
    Ok(sqlx::query_as::<_, EventTable>("SELECT * FROM event_tables WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

pub async fn add_event_entry(
    pool: &SqlitePool,
    table_id: &str,
    campaign_id: &str,
    data: &serde_json::Value,
) -> Result<EventEntry> {
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO event_entries (id, table_id, campaign_id, weight, event_type, title, description, conditions, is_repeatable)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(table_id)
    .bind(campaign_id)
    .bind(data["weight"].as_i64().unwrap_or(10))
    .bind(data["event_type"].as_str().unwrap_or("encounter"))
    .bind(data["title"].as_str().unwrap_or(""))
    .bind(data["description"].as_str().unwrap_or(""))
    .bind(data["conditions"].as_str())
    .bind(data["is_repeatable"].as_bool().unwrap_or(true))
    .execute(pool)
    .await?;

    get_event_entry(pool, &id).await?
        .ok_or_else(|| anyhow::anyhow!("Event entry not found after creation"))
}

pub async fn get_event_entry(pool: &SqlitePool, id: &str) -> Result<Option<EventEntry>> {
    Ok(sqlx::query_as::<_, EventEntry>("SELECT * FROM event_entries WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

/// Roll for a random event, weighted by entry weight
pub async fn roll_random_event(
    pool: &SqlitePool,
    campaign_id: &str,
    trigger_type: &str,
) -> Result<Option<EventEntry>> {
    use rand::Rng;

    // Get active table for this trigger type
    let table = sqlx::query_as::<_, EventTable>(
        "SELECT * FROM event_tables
         WHERE campaign_id = ? AND trigger_type = ? AND is_active = 1
         LIMIT 1"
    )
    .bind(campaign_id)
    .bind(trigger_type)
    .fetch_optional(pool)
    .await?;

    let table = match table {
        Some(t) => t,
        None => return Ok(None),
    };

    // Check trigger chance
    let roll: i64 = rand::thread_rng().gen_range(1..=100);
    if roll > table.trigger_chance {
        return Ok(None);
    }

    // Get eligible entries
    let entries = sqlx::query_as::<_, EventEntry>(
        "SELECT * FROM event_entries
         WHERE table_id = ?
           AND (is_repeatable = 1 OR times_triggered = 0)"
    )
    .bind(&table.id)
    .fetch_all(pool)
    .await?;

    if entries.is_empty() {
        return Ok(None);
    }

    // Weighted random selection
    let total_weight: i64 = entries.iter().map(|e| e.weight).sum();
    let mut roll = rand::thread_rng().gen_range(0..total_weight);

    for entry in &entries {
        roll -= entry.weight;
        if roll < 0 {
            // Mark as triggered
            sqlx::query(
                "UPDATE event_entries SET times_triggered = times_triggered + 1 WHERE id = ?"
            )
            .bind(&entry.id)
            .execute(pool)
            .await?;

            return Ok(Some(entry.clone()));
        }
    }

    Ok(entries.first().cloned())
}

pub async fn get_event_tables(
    pool: &SqlitePool,
    campaign_id: &str,
) -> Result<Vec<EventTable>> {
    Ok(sqlx::query_as::<_, EventTable>(
        "SELECT * FROM event_tables WHERE campaign_id = ?"
    )
    .bind(campaign_id)
    .fetch_all(pool)
    .await?)
}