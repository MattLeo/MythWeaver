use anyhow::Result;
use serde_json::{json, Value};
use sqlx::SqlitePool;
use uuid::Uuid;

// ─── Spell Queries ────────────────────────────────────────────────────────────

pub async fn get_spell(pool: &SqlitePool, spell_id: &str) -> Result<Option<Value>> {
    let row = sqlx::query!(
        "SELECT * FROM spells WHERE id = ?",
        spell_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| json!({
        "id": r.id,
        "name": r.name,
        "level": r.level,
        "school": r.school,
        "casting_time": r.casting_time,
        "range_type": r.range_type,
        "range_feet": r.range_feet,
        "has_verbal": r.has_verbal,
        "has_somatic": r.has_somatic,
        "has_material": r.has_material,
        "material_component": r.material_component,
        "duration": r.duration,
        "concentration": r.concentration,
        "ritual": r.ritual,
        "description": r.description,
        "damage_die": r.damage_die,
        "damage_die_count": r.damage_die_count,
        "damage_type": r.damage_type,
        "scales_with_level": r.scales_with_level,
        "cantrip_dice_5": r.cantrip_dice_5,
        "cantrip_dice_11": r.cantrip_dice_11,
        "cantrip_dice_17": r.cantrip_dice_17,
        "slot_scale_dice": r.slot_scale_dice,
        "save_type": r.save_type,
        "attack_type": r.attack_type,
        "target_type": r.target_type,
        "area_shape": r.area_shape,
        "area_size_feet": r.area_size_feet,
        "has_backend_resolver": r.has_backend_resolver,
        "is_wizard_spell": r.is_wizard_spell,
    })))
}

pub async fn get_spell_by_name(pool: &SqlitePool, name: &str) -> Result<Option<Value>> {
    let row = sqlx::query!(
        "SELECT * FROM spells WHERE LOWER(name) = LOWER(?)",
        name
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| json!({
        "id": r.id,
        "name": r.name,
        "level": r.level,
        "school": r.school,
        "casting_time": r.casting_time,
        "range_type": r.range_type,
        "duration": r.duration,
        "concentration": r.concentration,
        "description": r.description,
        "damage_die": r.damage_die,
        "damage_die_count": r.damage_die_count,
        "damage_type": r.damage_type,
        "scales_with_level": r.scales_with_level,
        "cantrip_dice_5": r.cantrip_dice_5,
        "cantrip_dice_11": r.cantrip_dice_11,
        "cantrip_dice_17": r.cantrip_dice_17,
        "slot_scale_dice": r.slot_scale_dice,
        "save_type": r.save_type,
        "attack_type": r.attack_type,
        "target_type": r.target_type,
        "has_backend_resolver": r.has_backend_resolver,
    })))
}

pub async fn search_spells(pool: &SqlitePool, query: &str, wizard_only: bool) -> Result<Vec<Value>> {
    let pattern = format!("%{}%", query);
    let wizard_filter: i64 = if wizard_only { 1 } else { 0 };

    let rows = sqlx::query!(
        "SELECT id, name, level, school, casting_time, duration, concentration, description
         FROM spells
         WHERE (LOWER(name) LIKE LOWER(?) OR LOWER(description) LIKE LOWER(?))
           AND (? = 0 OR is_wizard_spell = 1)
         ORDER BY level, name
         LIMIT 20",
        pattern, pattern, wizard_filter
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.iter().map(|r| json!({
        "id": r.id,
        "name": r.name,
        "level": r.level,
        "school": r.school,
        "casting_time": r.casting_time,
        "duration": r.duration,
        "concentration": r.concentration,
        "description": r.description,
    })).collect())
}

// ─── Known Spells ─────────────────────────────────────────────────────────────

pub async fn get_known_spells(pool: &SqlitePool, player_id: &str) -> Result<Vec<Value>> {
    let rows = sqlx::query!(
        r#"SELECT ks.id, ks.spell_id, ks.spell_type, ks.source,
                  s.name, s.level, s.school, s.casting_time, s.range_type,
                  s.duration, s.concentration, s.description,
                  s.damage_die, s.damage_die_count, s.damage_type,
                  s.save_type, s.attack_type, s.target_type,
                  s.has_backend_resolver, s.scales_with_level,
                  s.cantrip_dice_5, s.cantrip_dice_11, s.cantrip_dice_17,
                  s.slot_scale_dice
           FROM known_spells ks
           JOIN spells s ON ks.spell_id = s.id
           WHERE ks.player_id = ?
           ORDER BY s.level, s.name"#,
        player_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.iter().map(|r| json!({
        "known_spell_id": r.id,
        "spell_id": r.spell_id,
        "spell_type": r.spell_type,
        "source": r.source,
        "name": r.name,
        "level": r.level,
        "school": r.school,
        "casting_time": r.casting_time,
        "range_type": r.range_type,
        "duration": r.duration,
        "concentration": r.concentration,
        "description": r.description,
        "damage_die": r.damage_die,
        "damage_die_count": r.damage_die_count,
        "damage_type": r.damage_type,
        "save_type": r.save_type,
        "attack_type": r.attack_type,
        "target_type": r.target_type,
        "has_backend_resolver": r.has_backend_resolver,
        "scales_with_level": r.scales_with_level,
        "cantrip_dice_5": r.cantrip_dice_5,
        "cantrip_dice_11": r.cantrip_dice_11,
        "cantrip_dice_17": r.cantrip_dice_17,
        "slot_scale_dice": r.slot_scale_dice,
    })).collect())
}

pub async fn get_cantrips(pool: &SqlitePool, player_id: &str) -> Result<Vec<Value>> {
    let rows = sqlx::query!(
        r#"SELECT ks.spell_id, s.name, s.school, s.casting_time, s.range_type,
                  s.duration, s.concentration, s.description,
                  s.damage_die, s.damage_die_count, s.damage_type,
                  s.save_type, s.attack_type, s.target_type,
                  s.has_backend_resolver,
                  s.cantrip_dice_5, s.cantrip_dice_11, s.cantrip_dice_17
           FROM known_spells ks
           JOIN spells s ON ks.spell_id = s.id
           WHERE ks.player_id = ? AND ks.spell_type = 'cantrip'
           ORDER BY s.name"#,
        player_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.iter().map(|r| json!({
        "spell_id": r.spell_id,
        "name": r.name,
        "school": r.school,
        "casting_time": r.casting_time,
        "range_type": r.range_type,
        "duration": r.duration,
        "concentration": r.concentration,
        "description": r.description,
        "damage_die": r.damage_die,
        "damage_die_count": r.damage_die_count,
        "damage_type": r.damage_type,
        "save_type": r.save_type,
        "attack_type": r.attack_type,
        "target_type": r.target_type,
        "has_backend_resolver": r.has_backend_resolver,
        "cantrip_dice_5": r.cantrip_dice_5,
        "cantrip_dice_11": r.cantrip_dice_11,
        "cantrip_dice_17": r.cantrip_dice_17,
    })).collect())
}

pub async fn learn_spell(
    pool: &SqlitePool,
    campaign_id: &str,
    player_id: &str,
    spell_id: &str,
    spell_type: &str, // 'cantrip', 'prepared', 'ritual', 'always_prepared'
    source: &str,     // 'eldritch_knight'
) -> Result<String> {
    // Check if already known
    let existing = sqlx::query!(
        "SELECT id FROM known_spells WHERE player_id = ? AND spell_id = ?",
        player_id, spell_id
    )
    .fetch_optional(pool)
    .await?;

    if existing.is_some() {
        return Ok("already_known".to_string());
    }

    let id = Uuid::new_v4().to_string();
    sqlx::query!(
        "INSERT INTO known_spells (id, campaign_id, player_id, spell_id, spell_type, source)
         VALUES (?, ?, ?, ?, ?, ?)",
        id, campaign_id, player_id, spell_id, spell_type, source
    )
    .execute(pool)
    .await?;

    Ok(id)
}

pub async fn forget_spell(pool: &SqlitePool, player_id: &str, spell_id: &str) -> Result<bool> {
    let result = sqlx::query!(
        "DELETE FROM known_spells WHERE player_id = ? AND spell_id = ?",
        player_id, spell_id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn knows_spell(pool: &SqlitePool, player_id: &str, spell_id: &str) -> Result<bool> {
    let row = sqlx::query!(
        "SELECT id FROM known_spells WHERE player_id = ? AND spell_id = ?",
        player_id, spell_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.is_some())
}

// ─── Spell Slots ──────────────────────────────────────────────────────────────

pub async fn get_spell_slots(pool: &SqlitePool, player_id: &str) -> Result<Vec<Value>> {
    let rows = sqlx::query!(
        "SELECT slot_level, current_slots, max_slots
         FROM spell_slots
         WHERE player_id = ?
         ORDER BY slot_level",
        player_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.iter().map(|r| json!({
        "slot_level": r.slot_level,
        "current_slots": r.current_slots,
        "max_slots": r.max_slots,
    })).collect())
}

pub async fn get_spell_slots_summary(pool: &SqlitePool, player_id: &str) -> Result<Value> {
    let slots = get_spell_slots(pool, player_id).await?;
    Ok(json!({ "spell_slots": slots }))
}

/// Seed initial spell slots for an Eldritch Knight at a given fighter level.
/// Call this once when EK subclass is chosen (level 3) or when loading an existing EK.
pub async fn seed_ek_spell_slots(
    pool: &SqlitePool,
    campaign_id: &str,
    player_id: &str,
    fighter_level: i64,
) -> Result<()> {
    let (s1, s2, s3, s4) = ek_slot_table(fighter_level);

    // Upsert each slot level
    for (level, max) in [(1i64, s1), (2i64, s2), (3i64, s3), (4i64, s4)] {
        if max == 0 {
            // Remove if it exists and is now 0
            sqlx::query!(
                "DELETE FROM spell_slots WHERE player_id = ? AND slot_level = ?",
                player_id, level
            )
            .execute(pool)
            .await?;
            continue;
        }

        let existing = sqlx::query!(
            "SELECT id, current_slots FROM spell_slots WHERE player_id = ? AND slot_level = ?",
            player_id, level
        )
        .fetch_optional(pool)
        .await?;

        if let Some(row) = existing {
            // Update max; don't reduce current below 0 or raise above new max
            let new_current = row.current_slots.min(max);
            sqlx::query!(
                "UPDATE spell_slots SET max_slots = ?, current_slots = ? WHERE id = ?",
                max, new_current, row.id
            )
            .execute(pool)
            .await?;
        } else {
            let id = Uuid::new_v4().to_string();
            sqlx::query!(
                "INSERT INTO spell_slots (id, campaign_id, player_id, slot_level, current_slots, max_slots)
                 VALUES (?, ?, ?, ?, ?, ?)",
                id, campaign_id, player_id, level, max, max
            )
            .execute(pool)
            .await?;
        }
    }

    Ok(())
}

pub fn full_caster_slot_table(level: i64) -> [i64; 9] {
    match level {
        1  => [2,0,0,0,0,0,0,0,0],
        2  => [3,0,0,0,0,0,0,0,0],
        3  => [4,2,0,0,0,0,0,0,0],
        4  => [4,3,0,0,0,0,0,0,0],
        5  => [4,3,2,0,0,0,0,0,0],
        6  => [4,3,3,0,0,0,0,0,0],
        7  => [4,3,3,1,0,0,0,0,0],
        8  => [4,3,3,2,0,0,0,0,0],
        9  => [4,3,3,3,1,0,0,0,0],
        10 => [4,3,3,3,2,0,0,0,0],
        11 => [4,3,3,3,2,1,0,0,0],
        12 => [4,3,3,3,2,1,0,0,0],
        13 => [4,3,3,3,2,1,1,0,0],
        14 => [4,3,3,3,2,1,1,0,0],
        15 => [4,3,3,3,2,1,1,1,0],
        16 => [4,3,3,3,2,1,1,1,0],
        17 => [4,3,3,3,2,1,1,1,1],
        18 => [4,3,3,3,3,1,1,1,1],
        19 => [4,3,3,3,3,2,1,1,1],
        20 => [4,3,3,3,3,2,2,1,1],
        _  => [0,0,0,0,0,0,0,0,0],
    }
}

pub async fn seed_full_caster_spell_slots(
    pool: &SqlitePool,
    campaign_id: &str,
    player_id: &str,
    class_level: i64,
) -> Result<()> {
    let slots = full_caster_slot_table(class_level);
 
    for (i, &max) in slots.iter().enumerate() {
        let level = (i + 1) as i64;
 
        if max == 0 {
            // Remove any row that no longer has slots (shouldn't happen in normal play)
            sqlx::query!(
                "DELETE FROM spell_slots WHERE player_id = ? AND slot_level = ?",
                player_id, level
            )
            .execute(pool)
            .await?;
            continue;
        }
 
        let existing = sqlx::query!(
            "SELECT id, current_slots FROM spell_slots WHERE player_id = ? AND slot_level = ?",
            player_id, level
        )
        .fetch_optional(pool)
        .await?;
 
        if let Some(row) = existing {
            // Raise max without touching current (player may have expended some)
            let new_current = row.current_slots.min(max);
            sqlx::query!(
                "UPDATE spell_slots SET max_slots = ?, current_slots = ?, updated_at = datetime('now') WHERE id = ?",
                max, new_current, row.id
            )
            .execute(pool)
            .await?;
        } else {
            let id = Uuid::new_v4().to_string();
            sqlx::query!(
                "INSERT INTO spell_slots
                 (id, campaign_id, player_id, slot_level, current_slots, max_slots)
                 VALUES (?, ?, ?, ?, ?, ?)",
                id, campaign_id, player_id, level, max, max
            )
            .execute(pool)
            .await?;
        }
    }
 
    Ok(())
}

pub fn paladin_slot_table(level: i64) -> [i64; 5] {
    match level {
        1  => [2,0,0,0,0],
        2  => [2,0,0,0,0],
        3  => [3,0,0,0,0],
        4  => [3,0,0,0,0],
        5  => [4,2,0,0,0],
        6  => [4,2,0,0,0],
        7  => [4,3,0,0,0],
        8  => [4,3,0,0,0],
        9  => [4,3,2,0,0],
        10 => [4,3,2,0,0],
        11 => [4,3,3,0,0],
        12 => [4,3,3,0,0],
        13 => [4,3,3,1,0],
        14 => [4,3,3,1,0],
        15 => [4,3,3,2,0],
        16 => [4,3,3,2,0],
        17 => [4,3,3,3,1],
        18 => [4,3,3,3,1],
        19 => [4,3,3,3,2],
        20 => [4,3,3,3,2],
        _  => [0,0,0,0,0],
    }
}

pub async fn seed_half_caster_spell_slots(
    pool: &SqlitePool,
    campaign_id: &str,
    player_id: &str,
    class_level: i64,
) -> Result<()> {
    let slots = paladin_slot_table(class_level);
 
    for (i, &max) in slots.iter().enumerate() {
        let level = (i + 1) as i64;
 
        if max == 0 {
            sqlx::query!(
                "DELETE FROM spell_slots WHERE player_id = ? AND slot_level = ?",
                player_id, level
            )
            .execute(pool).await?;
            continue;
        }
 
        let existing = sqlx::query!(
            "SELECT id, current_slots FROM spell_slots WHERE player_id = ? AND slot_level = ?",
            player_id, level
        )
        .fetch_optional(pool).await?;
 
        if let Some(row) = existing {
            let new_current = row.current_slots.min(max);
            sqlx::query!(
                "UPDATE spell_slots SET max_slots = ?, current_slots = ?, updated_at = datetime('now') WHERE id = ?",
                max, new_current, row.id
            )
            .execute(pool).await?;
        } else {
            let id = Uuid::new_v4().to_string();
            sqlx::query!(
                "INSERT INTO spell_slots
                 (id, campaign_id, player_id, slot_level, current_slots, max_slots)
                 VALUES (?, ?, ?, ?, ?, ?)",
                id, campaign_id, player_id, level, max, max
            )
            .execute(pool).await?;
        }
    }
 
    Ok(())
}

/// EK spell slot table: returns (level1, level2, level3, level4) max slots
pub fn ek_slot_table(fighter_level: i64) -> (i64, i64, i64, i64) {
    match fighter_level {
        3  => (2, 0, 0, 0),
        4  => (3, 0, 0, 0),
        5 | 6  => (3, 0, 0, 0),
        7  => (4, 2, 0, 0),
        8  => (4, 2, 0, 0),
        9  => (4, 2, 0, 0),
        10 => (4, 3, 0, 0),
        11 | 12 => (4, 3, 0, 0),
        13 => (4, 3, 2, 0),
        14 | 15 => (4, 3, 2, 0),
        16 => (4, 3, 3, 0),
        17 | 18 => (4, 3, 3, 0),
        19 => (4, 3, 3, 1),
        20 => (4, 3, 3, 1),
        _  => (0, 0, 0, 0),
    }
}

/// How many spells an EK can have prepared (not counting cantrips)
pub fn ek_spells_prepared(fighter_level: i64) -> i64 {
    match fighter_level {
        3  => 3,
        4  => 4,
        5 | 6  => 4,
        7  => 5,
        8  => 5,
        9  => 5,
        10 => 7,
        11 | 12 => 7,
        13 => 9,
        14 | 15 => 9,
        16 => 11,
        17 | 18 => 11,
        19 => 12,
        20 => 13,
        _  => 0,
    }
}

/// Expend one spell slot of the given level. Returns remaining slots or error.
pub async fn expend_spell_slot(
    pool: &SqlitePool,
    player_id: &str,
    slot_level: i64,
) -> Result<i64> {
    let row = sqlx::query!(
        "SELECT id, current_slots FROM spell_slots WHERE player_id = ? AND slot_level = ?",
        player_id, slot_level
    )
    .fetch_optional(pool)
    .await?;

    match row {
        None => anyhow::bail!("No spell slots of level {} found", slot_level),
        Some(r) if r.current_slots <= 0 => {
            anyhow::bail!("No spell slots of level {} remaining", slot_level)
        }
        Some(r) => {
            let new_count = r.current_slots - 1;
            sqlx::query!(
                "UPDATE spell_slots SET current_slots = ? WHERE id = ?",
                new_count, r.id
            )
            .execute(pool)
            .await?;
            Ok(new_count)
        }
    }
}

/// Restore all spell slots (long rest).
pub async fn restore_all_spell_slots(pool: &SqlitePool, player_id: &str) -> Result<()> {
    sqlx::query!(
        "UPDATE spell_slots SET current_slots = max_slots WHERE player_id = ?",
        player_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Check if player has a slot available at the given level (or higher, for upcasting).
pub async fn has_spell_slot(
    pool: &SqlitePool,
    player_id: &str,
    min_level: i64,
) -> Result<Option<i64>> {
    // Find the lowest available slot at or above min_level
    let row = sqlx::query!(
        "SELECT slot_level FROM spell_slots
         WHERE player_id = ? AND slot_level >= ? AND current_slots > 0
         ORDER BY slot_level ASC
         LIMIT 1",
        player_id, min_level
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| r.slot_level))
}

// ─── Concentration ────────────────────────────────────────────────────────────

pub async fn get_concentration(pool: &SqlitePool, player_id: &str) -> Result<Option<Value>> {
    let row = sqlx::query!(
        "SELECT id, spell_id, spell_name, started_at, expires_at
         FROM concentration WHERE player_id = ?",
        player_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| json!({
        "id": r.id,
        "spell_id": r.spell_id,
        "spell_name": r.spell_name,
        "started_at": r.started_at,
        "expires_at": r.expires_at,
    })))
}

pub async fn set_concentration(
    pool: &SqlitePool,
    campaign_id: &str,
    player_id: &str,
    spell_id: &str,
    spell_name: &str,
    expires_at: Option<&str>,
) -> Result<()> {
    // Drop existing concentration first
    drop_concentration(pool, player_id).await?;

    let id = Uuid::new_v4().to_string();
    sqlx::query!(
        "INSERT INTO concentration (id, campaign_id, player_id, spell_id, spell_name, expires_at)
         VALUES (?, ?, ?, ?, ?, ?)",
        id, campaign_id, player_id, spell_id, spell_name, expires_at
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn drop_concentration(pool: &SqlitePool, player_id: &str) -> Result<Option<String>> {
    let existing = sqlx::query!(
        "SELECT spell_name FROM concentration WHERE player_id = ?",
        player_id
    )
    .fetch_optional(pool)
    .await?;

    let name = existing.map(|r| r.spell_name);

    sqlx::query!("DELETE FROM concentration WHERE player_id = ?", player_id)
        .execute(pool)
        .await?;

    Ok(name)
}

pub async fn is_concentrating_on(
    pool: &SqlitePool,
    player_id: &str,
    spell_id: &str,
) -> Result<bool> {
    let row = sqlx::query!(
        "SELECT id FROM concentration WHERE player_id = ? AND spell_id = ?",
        player_id, spell_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.is_some())
}

// ─── War Bond ─────────────────────────────────────────────────────────────────

pub async fn get_war_bonds(pool: &SqlitePool, player_id: &str) -> Result<Vec<Value>> {
    let rows = sqlx::query!(
        r#"SELECT wb.id, wb.item_id, wb.is_summoned, i.name as item_name
           FROM war_bonds wb
           JOIN items i ON wb.item_id = i.id
           WHERE wb.player_id = ?"#,
        player_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.iter().map(|r| json!({
        "id": r.id,
        "item_id": r.item_id,
        "item_name": r.item_name,
        "is_summoned": r.is_summoned,
    })).collect())
}

pub async fn create_war_bond(
    pool: &SqlitePool,
    campaign_id: &str,
    player_id: &str,
    item_id: &str,
) -> Result<String> {
    // EK can bond up to 2 weapons
    let count = sqlx::query!(
        "SELECT COUNT(*) as cnt FROM war_bonds WHERE player_id = ?",
        player_id
    )
    .fetch_one(pool)
    .await?
    .cnt;

    if count >= 2 {
        anyhow::bail!("Already bonded to 2 weapons. Break a bond before bonding a new one.");
    }

    // Check not already bonded
    let existing = sqlx::query!(
        "SELECT id FROM war_bonds WHERE player_id = ? AND item_id = ?",
        player_id, item_id
    )
    .fetch_optional(pool)
    .await?;

    if existing.is_some() {
        anyhow::bail!("Already bonded to this weapon.");
    }

    let id = Uuid::new_v4().to_string();
    sqlx::query!(
        "INSERT INTO war_bonds (id, campaign_id, player_id, item_id, is_summoned)
         VALUES (?, ?, ?, ?, 0)",
        id, campaign_id, player_id, item_id
    )
    .execute(pool)
    .await?;

    Ok(id)
}

pub async fn break_war_bond(pool: &SqlitePool, player_id: &str, item_id: &str) -> Result<bool> {
    let result = sqlx::query!(
        "DELETE FROM war_bonds WHERE player_id = ? AND item_id = ?",
        player_id, item_id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Summon a bonded weapon as a bonus action — teleports it to the EK's hand.
pub async fn summon_bonded_weapon(
    pool: &SqlitePool,
    player_id: &str,
    item_id: &str,
) -> Result<Value> {
    let bond = sqlx::query!(
        r#"SELECT wb.id, i.name as item_name
           FROM war_bonds wb
           JOIN items i ON wb.item_id = i.id
           WHERE wb.player_id = ? AND wb.item_id = ?"#,
        player_id, item_id
    )
    .fetch_optional(pool)
    .await?;

    match bond {
        None => anyhow::bail!("No war bond found for this weapon."),
        Some(b) => {
            sqlx::query!(
                "UPDATE war_bonds SET is_summoned = 1 WHERE player_id = ? AND item_id = ?",
                player_id, item_id
            )
            .execute(pool)
            .await?;

            Ok(json!({
                "message": format!("{} flies to your hand!", b.item_name),
                "item_id": item_id,
                "item_name": b.item_name,
            }))
        }
    }
}

// ─── Spell Resolution Helpers ─────────────────────────────────────────────────

/// Calculate cantrip damage dice count based on character level.
pub fn cantrip_dice_at_level(
    base_dice: i64,
    dice_5: Option<i64>,
    dice_11: Option<i64>,
    dice_17: Option<i64>,
    character_level: i64,
) -> i64 {
    if character_level >= 17 {
        dice_17.unwrap_or(base_dice)
    } else if character_level >= 11 {
        dice_11.unwrap_or(base_dice)
    } else if character_level >= 5 {
        dice_5.unwrap_or(base_dice)
    } else {
        base_dice
    }
}

/// Calculate extra damage dice when casting a leveled spell at a higher slot.
pub fn upcast_dice(base_dice: i64, slot_scale_dice: Option<i64>, base_level: i64, cast_level: i64) -> i64 {
    let extra_levels = (cast_level - base_level).max(0);
    base_dice + slot_scale_dice.unwrap_or(0) * extra_levels
}

/// Get all spells a player can currently cast (has slots for, or cantrips).
pub async fn get_castable_spells(pool: &SqlitePool, player_id: &str) -> Result<Value> {
    let known = get_known_spells(pool, player_id).await?;
    let slots = get_spell_slots(pool, player_id).await?;
    let concentration = get_concentration(pool, player_id).await?;

    // Determine max slot level available
    let max_available_level: i64 = slots.iter()
        .filter(|s| s["current_slots"].as_i64().unwrap_or(0) > 0)
        .map(|s| s["slot_level"].as_i64().unwrap_or(0))
        .max()
        .unwrap_or(0);

    // Filter to castable spells
    let castable: Vec<Value> = known.into_iter().filter(|spell| {
        let level = spell["level"].as_i64().unwrap_or(0);
        if level == 0 {
            // Cantrips always castable
            return true;
        }
        // Leveled spell: need a slot at that level or higher
        level <= max_available_level
    }).collect();

    Ok(json!({
        "castable_spells": castable,
        "spell_slots": slots,
        "concentrating_on": concentration,
        "max_available_slot": max_available_level,
    }))
}

/// Validate that a spell can be cast:
/// - Player knows the spell
/// - Has a slot available (if leveled)
/// - Not already concentrating on something (if concentration spell)
pub async fn validate_cast(
    pool: &SqlitePool,
    player_id: &str,
    spell_id: &str,
    slot_level: i64,
) -> Result<Value> {
    // Check knows spell
    if !knows_spell(pool, player_id, spell_id).await? {
        return Ok(json!({
            "valid": false,
            "reason": "You don't know that spell."
        }));
    }

    let spell = get_spell(pool, spell_id).await?
        .ok_or_else(|| anyhow::anyhow!("Spell not found"))?;

    let spell_level = spell["level"].as_i64().unwrap_or(0);

    // Cantrips don't need slots
    if spell_level > 0 {
        if slot_level < spell_level {
            return Ok(json!({
                "valid": false,
                "reason": format!("Need at least a level {} slot to cast {}.", spell_level, spell["name"])
            }));
        }

        let available = sqlx::query!(
            "SELECT current_slots FROM spell_slots WHERE player_id = ? AND slot_level = ?",
            player_id, slot_level
        )
        .fetch_optional(pool)
        .await?;

        match available {
            None | Some(_) if available.as_ref().map(|r| r.current_slots).unwrap_or(0) <= 0 => {
                return Ok(json!({
                    "valid": false,
                    "reason": format!("No level {} spell slots remaining.", slot_level)
                }));
            }
            _ => {}
        }
    }

    // Check concentration conflict
    if spell["concentration"].as_i64().unwrap_or(0) == 1 {
        if let Some(conc) = get_concentration(pool, player_id).await? {
            return Ok(json!({
                "valid": true,
                "concentration_warning": true,
                "will_drop": conc["spell_name"],
                "spell": spell,
            }));
        }
    }

    Ok(json!({
        "valid": true,
        "concentration_warning": false,
        "spell": spell,
    }))
}