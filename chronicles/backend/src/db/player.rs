use anyhow::Result;
use sqlx::SqlitePool;
use uuid::Uuid;
use crate::models::*;

pub async fn create_player(
    pool: &SqlitePool,
    campaign_id: &str,
    req: &CreateCampaignRequest,
) -> Result<Player> {
    let id = Uuid::new_v4().to_string();
    let hit_die = hit_die_for_class(&req.player_class);
    let con_mod = Player::modifier(req.player_stats.con);
    let max_hp = hit_die + con_mod;
    let current_hp = max_hp.max(1);
    let gold = req.starting_gold.unwrap_or(15);

    sqlx::query(
        "INSERT INTO players (
            id, campaign_id, name, race, class, background,
            current_hp, max_hp, str, dex, con, int, wis, cha,
            gold, backstory
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(campaign_id)
    .bind(&req.player_name)
    .bind(&req.player_race)
    .bind(&req.player_class)
    .bind(&req.player_background)
    .bind(current_hp)
    .bind(max_hp)
    .bind(req.player_stats.str)
    .bind(req.player_stats.dex)
    .bind(req.player_stats.con)
    .bind(req.player_stats.int)
    .bind(req.player_stats.wis)
    .bind(req.player_stats.cha)
    .bind(gold)
    .bind(&req.player_backstory)
    .execute(pool)
    .await?;

    get_player(pool, &id).await?.ok_or_else(|| anyhow::anyhow!("Player not found after creation"))
}

pub async fn get_player(pool: &SqlitePool, id: &str) -> Result<Option<Player>> {
    let player = sqlx::query_as::<_, Player>(
        "SELECT * FROM players WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(player)
}

pub async fn get_player_by_campaign(pool: &SqlitePool, campaign_id: &str) -> Result<Option<Player>> {
    let player = sqlx::query_as::<_, Player>(
        "SELECT * FROM players WHERE campaign_id = ? LIMIT 1"
    )
    .bind(campaign_id)
    .fetch_optional(pool)
    .await?;

    Ok(player)
}

pub async fn update_player_hp(
    pool: &SqlitePool,
    player_id: &str,
    new_hp: i64,
) -> Result<()> {
    sqlx::query(
        "UPDATE players SET current_hp = ?, updated_at = datetime('now') WHERE id = ?"
    )
    .bind(new_hp)
    .bind(player_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_player_location(
    pool: &SqlitePool,
    player_id: &str,
    location_id: &str,
) -> Result<()> {
    sqlx::query(
        "UPDATE players SET current_location_id = ?, updated_at = datetime('now') WHERE id = ?"
    )
    .bind(location_id)
    .bind(player_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_player_gold(
    pool: &SqlitePool,
    player_id: &str,
    new_gold: i64,
) -> Result<()> {
    sqlx::query(
        "UPDATE players SET gold = ?, updated_at = datetime('now') WHERE id = ?"
    )
    .bind(new_gold)
    .bind(player_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_player_xp(
    pool: &SqlitePool,
    player_id: &str,
    new_xp: i64,
) -> Result<()> {
    sqlx::query(
        "UPDATE players SET experience = ?, updated_at = datetime('now') WHERE id = ?"
    )
    .bind(new_xp)
    .bind(player_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn level_up_player(
    pool: &SqlitePool,
    player_id: &str,
    player: &Player,
) -> Result<LevelUpResult> {
    let new_level = player.level + 1;
    let con_mod = Player::modifier(player.con);
    let hp_gained = hp_gained_on_level(&player.class, con_mod);
    let new_max_hp = player.max_hp + hp_gained;
    let new_prof = Player::proficiency_for_level(new_level);
    let asi_available = Player::is_asi_level(new_level);

    sqlx::query(
        "UPDATE players SET
            level = ?,
            max_hp = ?,
            current_hp = ?,
            proficiency_bonus = ?,
            updated_at = datetime('now')
         WHERE id = ?"
    )
    .bind(new_level)
    .bind(new_max_hp)
    .bind((player.current_hp + hp_gained).min(new_max_hp))
    .bind(new_prof)
    .bind(player_id)
    .execute(pool)
    .await?;

    Ok(LevelUpResult {
        new_level,
        hp_gained,
        new_max_hp,
        new_proficiency_bonus: new_prof,
        asi_available,
        new_features: class_features_at_level(&player.class, new_level),
        spell_slots: spell_slots_at_level(&player.class, new_level),
    })
}

pub async fn update_death_saves(
    pool: &SqlitePool,
    player_id: &str,
    successes: i64,
    failures: i64,
    is_stable: bool,
    is_dead: bool,
) -> Result<()> {
    sqlx::query(
        "UPDATE players SET
            death_save_successes = ?,
            death_save_failures = ?,
            is_stable = ?,
            is_dead = ?,
            updated_at = datetime('now')
         WHERE id = ?"
    )
    .bind(successes)
    .bind(failures)
    .bind(is_stable)
    .bind(is_dead)
    .bind(player_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_armor_class(
    pool: &SqlitePool,
    player_id: &str,
    new_ac: i64,
) -> Result<()> {
    sqlx::query(
        "UPDATE players SET armor_class = ?, updated_at = datetime('now') WHERE id = ?"
    )
    .bind(new_ac)
    .bind(player_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn apply_asi(
    pool: &SqlitePool,
    player_id: &str,
    stat1: &str,
    stat2: Option<&str>,
) -> Result<()> {
    let col1 = stat_to_column(stat1)?;
    
    if let Some(s2) = stat2 {
        let col2 = stat_to_column(s2)?;
        let query = format!(
            "UPDATE players SET {} = {} + 1, {} = {} + 1, updated_at = datetime('now') WHERE id = ?",
            col1, col1, col2, col2
        );
        sqlx::query(&query).bind(player_id).execute(pool).await?;
    } else {
        let query = format!(
            "UPDATE players SET {} = {} + 2, updated_at = datetime('now') WHERE id = ?",
            col1, col1
        );
        sqlx::query(&query).bind(player_id).execute(pool).await?;
    }

    Ok(())
}

fn stat_to_column(stat: &str) -> Result<&'static str> {
    match stat.to_lowercase().as_str() {
        "str" | "strength" => Ok("str"),
        "dex" | "dexterity" => Ok("dex"),
        "con" | "constitution" => Ok("con"),
        "int" | "intelligence" => Ok("int"),
        "wis" | "wisdom" => Ok("wis"),
        "cha" | "charisma" => Ok("cha"),
        _ => Err(anyhow::anyhow!("Unknown stat: {}", stat)),
    }
}

// ─── Class Features ───────────────────────────────────────────────────────────

fn class_features_at_level(class: &str, level: i64) -> Vec<String> {
    let mut features = vec![];

    match class {
        "Barbarian" => match level {
            2 => features.push("Reckless Attack, Danger Sense".to_string()),
            3 => features.push("Primal Path".to_string()),
            5 => features.push("Extra Attack, Fast Movement".to_string()),
            7 => features.push("Feral Instinct".to_string()),
            _ => {}
        },
        "Fighter" => match level {
            2 => features.push("Action Surge".to_string()),
            3 => features.push("Martial Archetype".to_string()),
            5 => features.push("Extra Attack".to_string()),
            6 => features.push("Ability Score Improvement".to_string()),
            _ => {}
        },
        "Rogue" => match level {
            2 => features.push("Cunning Action".to_string()),
            3 => features.push("Roguish Archetype".to_string()),
            5 => features.push("Uncanny Dodge".to_string()),
            7 => features.push("Evasion".to_string()),
            _ => {}
        },
        "Wizard" => match level {
            2 => features.push("Arcane Tradition".to_string()),
            _ => {}
        },
        "Cleric" => match level {
            2 => features.push("Channel Divinity".to_string()),
            _ => {}
        },
        _ => {}
    }

    features
}

fn spell_slots_at_level(class: &str, level: i64) -> Option<SpellSlots> {
    let is_full_caster = matches!(class, "Wizard" | "Sorcerer" | "Cleric" | "Druid" | "Bard");
    let is_half_caster = matches!(class, "Paladin" | "Ranger");
    let is_warlock = class == "Warlock";

    if !is_full_caster && !is_half_caster && !is_warlock {
        return None;
    }

    // Full caster spell slots (simplified)
    if is_full_caster {
        return Some(match level {
            1 => SpellSlots { level_1: Some(2), level_2: None, level_3: None, level_4: None, level_5: None, level_6: None, level_7: None, level_8: None, level_9: None },
            2 => SpellSlots { level_1: Some(3), level_2: None, level_3: None, level_4: None, level_5: None, level_6: None, level_7: None, level_8: None, level_9: None },
            3 => SpellSlots { level_1: Some(4), level_2: Some(2), level_3: None, level_4: None, level_5: None, level_6: None, level_7: None, level_8: None, level_9: None },
            4 => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: None, level_4: None, level_5: None, level_6: None, level_7: None, level_8: None, level_9: None },
            5 => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: Some(2), level_4: None, level_5: None, level_6: None, level_7: None, level_8: None, level_9: None },
            _ => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: Some(3), level_4: Some(3), level_5: Some(2), level_6: None, level_7: None, level_8: None, level_9: None },
        });
    }

    None
}