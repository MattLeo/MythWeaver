use anyhow::Result;
use sqlx::SqlitePool;
use uuid::Uuid;
use crate::models::*;

// ─── Locations ────────────────────────────────────────────────────────────────

pub async fn create_location(
    pool: &SqlitePool,
    campaign_id: &str,
    name: &str,
    location_type: &str,
    description: &str,
    notes: Option<&str>,
) -> Result<Location> {
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO locations (id, campaign_id, name, location_type, description, notes)
         VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(campaign_id)
    .bind(name)
    .bind(location_type)
    .bind(description)
    .bind(notes)
    .execute(pool)
    .await?;

    get_location(pool, &id).await?.ok_or_else(|| anyhow::anyhow!("Location not found after creation"))
}

pub async fn get_location(pool: &SqlitePool, id: &str) -> Result<Option<Location>> {
    Ok(sqlx::query_as::<_, Location>("SELECT * FROM locations WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

pub async fn get_location_by_name(
    pool: &SqlitePool,
    campaign_id: &str,
    name: &str,
) -> Result<Option<Location>> {
    Ok(sqlx::query_as::<_, Location>(
        "SELECT * FROM locations WHERE campaign_id = ? AND name LIKE ? LIMIT 1"
    )
    .bind(campaign_id)
    .bind(format!("%{}%", name))
    .fetch_optional(pool)
    .await?)
}

pub async fn update_location(
    pool: &SqlitePool,
    id: &str,
    description: Option<&str>,
    state: Option<&str>,
    notes: Option<&str>,
) -> Result<()> {
    sqlx::query(
        "UPDATE locations SET
            description = COALESCE(?, description),
            state = COALESCE(?, state),
            notes = COALESCE(?, notes),
            updated_at = datetime('now')
         WHERE id = ?"
    )
    .bind(description)
    .bind(state)
    .bind(notes)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_connected_locations(
    pool: &SqlitePool,
    location_id: &str,
) -> Result<Vec<Location>> {
    Ok(sqlx::query_as::<_, Location>(
        "SELECT l.* FROM locations l
         JOIN location_connections lc ON (lc.to_location = l.id OR lc.from_location = l.id)
         WHERE (lc.from_location = ? OR lc.to_location = ?)
           AND l.id != ?
           AND lc.is_hidden = 0"
    )
    .bind(location_id)
    .bind(location_id)
    .bind(location_id)
    .fetch_all(pool)
    .await?)
}

pub async fn connect_locations(
    pool: &SqlitePool,
    campaign_id: &str,
    from_id: &str,
    to_id: &str,
    travel_notes: Option<&str>,
) -> Result<()> {
    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO location_connections (id, campaign_id, from_location, to_location, travel_notes)
         VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(campaign_id)
    .bind(from_id)
    .bind(to_id)
    .bind(travel_notes)
    .execute(pool)
    .await?;

    Ok(())
}

// ─── NPCs ─────────────────────────────────────────────────────────────────────

pub async fn create_npc(
    pool: &SqlitePool,
    campaign_id: &str,
    name: &str,
    race: Option<&str>,
    occupation: Option<&str>,
    description: &str,
    personality: Option<&str>,
    disposition: &str,
    location_id: Option<&str>,
) -> Result<Npc> {
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO npcs (id, campaign_id, name, race, occupation, description, personality, disposition, current_location_id)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(campaign_id)
    .bind(name)
    .bind(race)
    .bind(occupation)
    .bind(description)
    .bind(personality)
    .bind(disposition)
    .bind(location_id)
    .execute(pool)
    .await?;

    get_npc(pool, &id).await?.ok_or_else(|| anyhow::anyhow!("NPC not found after creation"))
}

pub async fn get_npc(pool: &SqlitePool, id: &str) -> Result<Option<Npc>> {
    Ok(sqlx::query_as::<_, Npc>("SELECT * FROM npcs WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

pub async fn get_npc_by_name(
    pool: &SqlitePool,
    campaign_id: &str,
    name: &str,
) -> Result<Option<Npc>> {
    Ok(sqlx::query_as::<_, Npc>(
        "SELECT * FROM npcs WHERE campaign_id = ? AND name LIKE ? LIMIT 1"
    )
    .bind(campaign_id)
    .bind(format!("%{}%", name))
    .fetch_optional(pool)
    .await?)
}

pub async fn get_npcs_at_location(
    pool: &SqlitePool,
    location_id: &str,
) -> Result<Vec<Npc>> {
    Ok(sqlx::query_as::<_, Npc>(
        "SELECT * FROM npcs WHERE current_location_id = ? AND is_hidden = 0"
    )
    .bind(location_id)
    .fetch_all(pool)
    .await?)
}

pub async fn update_npc(
    pool: &SqlitePool,
    id: &str,
    disposition: Option<&str>,
    location_id: Option<&str>,
    is_alive: Option<bool>,
    notes: Option<&str>,
) -> Result<()> {
    sqlx::query(
        "UPDATE npcs SET
            disposition = COALESCE(?, disposition),
            current_location_id = COALESCE(?, current_location_id),
            is_alive = COALESCE(?, is_alive),
            notes = COALESCE(?, notes),
            updated_at = datetime('now')
         WHERE id = ?"
    )
    .bind(disposition)
    .bind(location_id)
    .bind(is_alive)
    .bind(notes)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

// ─── World Facts ──────────────────────────────────────────────────────────────

pub async fn add_world_fact(
    pool: &SqlitePool,
    campaign_id: &str,
    category: Option<&str>,
    title: &str,
    content: &str,
    tags: Option<&str>,
) -> Result<WorldFact> {
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO world_facts (id, campaign_id, category, title, content, tags)
         VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(campaign_id)
    .bind(category)
    .bind(title)
    .bind(content)
    .bind(tags)
    .execute(pool)
    .await?;

    get_world_fact(pool, &id).await?.ok_or_else(|| anyhow::anyhow!("World fact not found after creation"))
}

pub async fn get_world_fact(pool: &SqlitePool, id: &str) -> Result<Option<WorldFact>> {
    Ok(sqlx::query_as::<_, WorldFact>("SELECT * FROM world_facts WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

pub async fn search_world_facts(
    pool: &SqlitePool,
    campaign_id: &str,
    keyword: &str,
) -> Result<Vec<WorldFact>> {
    Ok(sqlx::query_as::<_, WorldFact>(
        "SELECT * FROM world_facts
         WHERE campaign_id = ?
           AND (title LIKE ? OR content LIKE ? OR tags LIKE ? OR category LIKE ?)
         ORDER BY created_at DESC
         LIMIT 10"
    )
    .bind(campaign_id)
    .bind(format!("%{}%", keyword))
    .bind(format!("%{}%", keyword))
    .bind(format!("%{}%", keyword))
    .bind(format!("%{}%", keyword))
    .fetch_all(pool)
    .await?)
}

// ─── Abilities ────────────────────────────────────────────────────────────────

pub async fn create_ability(
    pool: &SqlitePool,
    campaign_id: &str,
    owner_type: &str,
    owner_id: &str,
    name: &str,
    description: Option<&str>,
    max_uses: i64,
    refresh_type: &str,
) -> Result<Ability> {
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO abilities (id, campaign_id, owner_type, owner_id, name, description, current_uses, max_uses, refresh_type)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(campaign_id)
    .bind(owner_type)
    .bind(owner_id)
    .bind(name)
    .bind(description)
    .bind(max_uses)
    .bind(max_uses)
    .bind(refresh_type)
    .execute(pool)
    .await?;

    Ok(sqlx::query_as::<_, Ability>("SELECT * FROM abilities WHERE id = ?")
        .bind(&id)
        .fetch_one(pool)
        .await?)
}

pub async fn get_abilities(
    pool: &SqlitePool,
    owner_id: &str,
    owner_type: &str,
) -> Result<Vec<Ability>> {
    Ok(sqlx::query_as::<_, Ability>(
        "SELECT * FROM abilities WHERE owner_id = ? AND owner_type = ?"
    )
    .bind(owner_id)
    .bind(owner_type)
    .fetch_all(pool)
    .await?)
}

pub async fn use_ability(
    pool: &SqlitePool,
    ability_id: &str,
    uses: i64,
) -> Result<i64> {
    let ability = sqlx::query_as::<_, Ability>("SELECT * FROM abilities WHERE id = ?")
        .bind(ability_id)
        .fetch_one(pool)
        .await?;

    let new_uses = (ability.current_uses - uses).max(0);

    sqlx::query("UPDATE abilities SET current_uses = ? WHERE id = ?")
        .bind(new_uses)
        .bind(ability_id)
        .execute(pool)
        .await?;

    Ok(new_uses)
}

pub async fn refresh_abilities(
    pool: &SqlitePool,
    owner_id: &str,
    owner_type: &str,
    refresh_type: &str,
) -> Result<()> {
    if refresh_type == "long_rest" {
        // Long rest refreshes everything
        sqlx::query(
            "UPDATE abilities SET current_uses = max_uses
             WHERE owner_id = ? AND owner_type = ?
               AND refresh_type IN ('short_rest', 'long_rest')"
        )
        .bind(owner_id)
        .bind(owner_type)
        .execute(pool)
        .await?;
    } else {
        // Short rest only refreshes short_rest abilities
        sqlx::query(
            "UPDATE abilities SET current_uses = max_uses
             WHERE owner_id = ? AND owner_type = ?
               AND refresh_type = 'short_rest'"
        )
        .bind(owner_id)
        .bind(owner_type)
        .execute(pool)
        .await?;
    }

    Ok(())
}