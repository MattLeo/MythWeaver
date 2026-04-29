use anyhow::Result;
use sqlx::SqlitePool;
use uuid::Uuid;
use crate::models::*;

// ─── Campaign ─────────────────────────────────────────────────────────────────

pub async fn create_campaign(pool: &SqlitePool, name: &str) -> Result<Campaign> {
    let id = Uuid::new_v4().to_string();
    
    sqlx::query(
        "INSERT INTO campaigns (id, name) VALUES (?, ?)"
    )
    .bind(&id)
    .bind(name)
    .execute(pool)
    .await?;

    Ok(Campaign {
        id,
        name: name.to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    })
}

pub async fn get_campaign(pool: &SqlitePool, id: &str) -> Result<Option<Campaign>> {
    let campaign = sqlx::query_as::<_, Campaign>(
        "SELECT * FROM campaigns WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(campaign)
}

pub async fn list_campaigns(pool: &SqlitePool) -> Result<Vec<Campaign>> {
    Ok(sqlx::query_as::<_, Campaign>(
        "SELECT * FROM campaigns ORDER BY updated_at DESC"
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_story_journal(pool:&SqlitePool, campaign_id: &str) -> Result<Option<String>> {
    Ok(sqlx::query_scalar(
        "SELECT story_journal FROM campaigns WHERE id = ?"
    )
    .bind(campaign_id)
    .fetch_optional(pool)
    .await?
    .flatten())
}

pub async fn update_story_journal(pool: &SqlitePool, campaign_id: &str, journal: &str
) -> Result<()> {
    sqlx::query(
        "UPDATE campagins SET story_journal = ?, updated_at = datetime('now') WHERE id = ?"
    )
    .bind(journal)
    .bind(campaign_id)
    .execute(pool)
    .await?;
    Ok(())
}



// ─── Session ──────────────────────────────────────────────────────────────────

pub async fn create_session(pool: &SqlitePool, campaign_id: &str) -> Result<Session> {
    // Mark any existing active sessions as inactive
    sqlx::query(
        "UPDATE sessions SET is_active = 0 WHERE campaign_id = ? AND is_active = 1"
    )
    .bind(campaign_id)
    .execute(pool)
    .await?;

    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO sessions (id, campaign_id) VALUES (?, ?)"
    )
    .bind(&id)
    .bind(campaign_id)
    .execute(pool)
    .await?;

    Ok(Session {
        id,
        campaign_id: campaign_id.to_string(),
        started_at: chrono::Utc::now().to_rfc3339(),
        ended_at: None,
        is_active: true,
    })
}

pub async fn end_session(pool: &SqlitePool, session_id: &str) -> Result<()> {
    sqlx::query(
        "UPDATE sessions SET is_active = 0, ended_at = datetime('now') WHERE id = ?"
    )
    .bind(session_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_active_session(pool: &SqlitePool, campaign_id: &str) -> Result<Option<Session>> {
    let session = sqlx::query_as::<_, Session>(
        "SELECT * FROM sessions WHERE campaign_id = ? AND is_active = 1 ORDER BY started_at DESC LIMIT 1"
    )
    .bind(campaign_id)
    .fetch_optional(pool)
    .await?;

    Ok(session)
}

// ─── Messages ─────────────────────────────────────────────────────────────────

pub async fn save_message(
    pool: &SqlitePool,
    session_id: &str,
    campaign_id: &str,
    role: &str,
    content: &str,
    tool_calls: Option<&str>,
) -> Result<Message> {
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO messages (id, session_id, campaign_id, role, content, tool_calls)
         VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(session_id)
    .bind(campaign_id)
    .bind(role)
    .bind(content)
    .bind(tool_calls)
    .execute(pool)
    .await?;

    Ok(Message {
        id,
        session_id: session_id.to_string(),
        campaign_id: campaign_id.to_string(),
        role: role.to_string(),
        content: content.to_string(),
        tool_calls: tool_calls.map(|s| s.to_string()),
        created_at: chrono::Utc::now().to_rfc3339(),
    })
}

pub async fn get_session_messages(pool: &SqlitePool, session_id: &str) -> Result<Vec<Message>> {
    let messages = sqlx::query_as::<_, Message>(
        "SELECT * FROM messages WHERE session_id = ? ORDER BY created_at ASC"
    )
    .bind(session_id)
    .fetch_all(pool)
    .await?;

    Ok(messages)
}

pub async fn get_recent_messages(pool: &SqlitePool, campaign_id: &str, limit: i64) -> Result<Vec<crate::models::Message>> {
    Ok(sqlx::query_as::<_, crate::models::Message>(
        "SELECT * FROM messages WHERE campaign_id = ?
         ORDER BY created_at DESC LIMIT ?"
    )
    .bind(campaign_id)
    .bind(limit)
    .fetch_all(pool)
    .await?)
}

// ─── Session Summaries ────────────────────────────────────────────────────────

pub async fn save_session_summary(
    pool: &SqlitePool,
    campaign_id: &str,
    session_id: &str,
    summary: &str,
) -> Result<SessionSummary> {
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO session_summaries (id, campaign_id, session_id, summary)
         VALUES (?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(campaign_id)
    .bind(session_id)
    .bind(summary)
    .execute(pool)
    .await?;

    Ok(SessionSummary {
        id,
        campaign_id: campaign_id.to_string(),
        session_id: session_id.to_string(),
        summary: summary.to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    })
}

pub async fn get_session_summaries(
    pool: &SqlitePool,
    campaign_id: &str,
) -> Result<Vec<SessionSummary>> {
    let summaries = sqlx::query_as::<_, SessionSummary>(
        "SELECT * FROM session_summaries WHERE campaign_id = ? ORDER BY created_at ASC"
    )
    .bind(campaign_id)
    .fetch_all(pool)
    .await?;

    Ok(summaries)
}