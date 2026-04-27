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
    Ok(sqlx::query_as::<_, Player>("SELECT * FROM players WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

pub async fn get_player_by_campaign(pool: &SqlitePool, campaign_id: &str) -> Result<Option<Player>> {
    Ok(sqlx::query_as::<_, Player>("SELECT * FROM players WHERE campaign_id = ? LIMIT 1")
        .bind(campaign_id)
        .fetch_optional(pool)
        .await?)
}

pub async fn update_player_hp(pool: &SqlitePool, player_id: &str, new_hp: i64) -> Result<()> {
    sqlx::query("UPDATE players SET current_hp = ?, updated_at = datetime('now') WHERE id = ?")
        .bind(new_hp)
        .bind(player_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_player_location(pool: &SqlitePool, player_id: &str, location_id: &str) -> Result<()> {
    sqlx::query("UPDATE players SET current_location_id = ?, updated_at = datetime('now') WHERE id = ?")
        .bind(location_id)
        .bind(player_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_player_gold(pool: &SqlitePool, player_id: &str, new_gold: i64) -> Result<()> {
    // Legacy function - only adds gold
    sqlx::query("UPDATE players SET gold = ?, updated_at = datetime('now') WHERE id = ?")
        .bind(new_gold)
        .bind(player_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_currency(
    pool: &SqlitePool,
    player_id: &str,
    delta_pp: i64,
    delta_gp: i64,
    delta_sp: i64,
    delta_cp: i64,
) -> Result<(i64, i64, i64, i64)> {
    let player = get_player(pool, player_id).await?
        .ok_or_else(|| anyhow::anyhow!("Player not found"))?;

    let new_pp = player.platinum + delta_pp;
    let new_gp = player.gold + delta_gp;
    let new_sp = player.silver + delta_sp;
    let new_cp = player.copper + delta_cp;

    normalize_and_save_currency(pool, player_id, new_pp, new_gp, new_sp, new_cp).await
}

pub async fn normalize_and_save_currency(
    pool: &SqlitePool,
    player_id: &str,
    pp: i64,
    gp: i64,
    sp: i64,
    cp: i64,
) -> Result<(i64, i64, i64, i64)> {
    let mut total_cp = (pp * 1000) + (gp * 100) + (sp * 10) + cp;

    total_cp = total_cp.max(0);

    let final_pp = total_cp / 1000;
    total_cp %= 1000;
    let final_gp = total_cp / 100;
    total_cp %= 100;
    let final sp = total_cp /10;
    let final cp = tota_cp % 10;

    sqls::query(
        "UPDATE players SET platinum = ?, gold = ?, silver = ?, copper = ?, updated_at = datetime('now') WHERE id = ?"
    )
    .bind(final_pp)
    .bind(final_gp)
    .bind(final_sp)
    .bind(final_cp)
    .bind(player_id)
    .execute(pool)
    .await?;

    Ok((final_pp, final_gp, final_sp, final_cp))
}

pub async fn update_player_xp(pool: &SqlitePool, player_id: &str, new_xp: i64) -> Result<()> {
    sqlx::query("UPDATE players SET experience = ?, updated_at = datetime('now') WHERE id = ?")
        .bind(new_xp)
        .bind(player_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn set_subclass(pool: &SqlitePool, player_id: &str, subclass: &str) -> Result<()> {
    sqlx::query("UPDATE players SET subclass = ?, updated_at = datetime('now') WHERE id = ?")
        .bind(subclass)
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

    // Fighter-specific scaling
    let (extra_attacks, indomitable_max, action_surge_uses,
         second_wind_uses, weapon_mastery_count) = if player.class == "Fighter" {
        (
            fighter_extra_attacks(new_level),
            fighter_indomitable_max(new_level),
            fighter_action_surge_uses(new_level),
            fighter_second_wind_uses(new_level),
            fighter_weapon_mastery_count(new_level),
        )
    } else {
        (player.extra_attacks, player.indomitable_max, 0, 2, 0)
    };

    let asi_available = if player.class == "Fighter" {
        matches!(new_level, 4 | 6 | 8 | 12 | 14 | 16)
    } else {
        Player::is_asi_level(new_level)
    };

    let subclass_choice_required = new_level == 3 && player.subclass.is_none();

    sqlx::query(
        "UPDATE players SET
            level = ?,
            max_hp = ?,
            current_hp = ?,
            proficiency_bonus = ?,
            extra_attacks = ?,
            indomitable_max = ?,
            indomitable_uses = ?,
            updated_at = datetime('now')
         WHERE id = ?"
    )
    .bind(new_level)
    .bind(new_max_hp)
    .bind((player.current_hp + hp_gained).min(new_max_hp))
    .bind(new_prof)
    .bind(extra_attacks)
    .bind(indomitable_max)
    .bind(indomitable_max) // reset uses to max on level up
    .bind(player_id)
    .execute(pool)
    .await?;

    // Update Second Wind uses if Fighter
    if player.class == "Fighter" {
        sqlx::query(
            "UPDATE abilities SET max_uses = ?, current_uses = ?
             WHERE owner_id = ? AND name = 'Second Wind'"
        )
        .bind(second_wind_uses)
        .bind(second_wind_uses)
        .bind(player_id)
        .execute(pool)
        .await?;

        // Update Action Surge uses
        if action_surge_uses > 0 {
            sqlx::query(
                "UPDATE abilities SET max_uses = ?, current_uses = ?
                 WHERE owner_id = ? AND name = 'Action Surge'"
            )
            .bind(action_surge_uses)
            .bind(action_surge_uses)
            .bind(player_id)
            .execute(pool)
            .await?;
        }

        // Update superiority dice if Battle Master
        if player.subclass.as_deref() == Some("Battle Master") {
            let (dice_count, die_size) = battle_master_superiority_dice(new_level);
            sqlx::query(
                "UPDATE superiority_dice SET max_dice = ?, current_dice = ?, die_size = ?
                 WHERE player_id = ? AND pool_name = 'Battle Master'"
            )
            .bind(dice_count)
            .bind(dice_count)
            .bind(die_size)
            .bind(player_id)
            .execute(pool)
            .await?;
        }

        // Update Psi Warrior energy dice
        if player.subclass.as_deref() == Some("Psi Warrior") {
            let (dice_count, die_size) = psi_warrior_energy_dice(new_level);
            sqlx::query(
                "UPDATE superiority_dice SET max_dice = ?, current_dice = ?, die_size = ?
                 WHERE player_id = ? AND pool_name = 'Psi Warrior'"
            )
            .bind(dice_count)
            .bind(dice_count)
            .bind(die_size)
            .bind(player_id)
            .execute(pool)
            .await?;
        }

        // Champion: update crit range
        if player.subclass.as_deref() == Some("Champion") {
            let crit_range = match new_level {
                3..=14 => 19,
                15..=20 => 18,
                _ => 20,
            };
            sqlx::query(
                "UPDATE players SET crit_range_min = ? WHERE id = ?"
            )
            .bind(crit_range)
            .bind(player_id)
            .execute(pool)
            .await?;
        }
    }

    let new_features = fighter_features_at_level(&player.class, new_level, player.subclass.as_deref());

    Ok(LevelUpResult {
        new_level,
        hp_gained,
        new_max_hp,
        new_proficiency_bonus: new_prof,
        asi_available,
        subclass_choice_required,
        new_features,
        spell_slots: eldritch_knight_spell_slots(player.subclass.as_deref(), new_level),
        second_wind_uses,
        weapon_mastery_count,
        extra_attacks,
        indomitable_max,
        action_surge_uses,
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

pub async fn update_armor_class(pool: &SqlitePool, player_id: &str, new_ac: i64) -> Result<()> {
    sqlx::query("UPDATE players SET armor_class = ?, updated_at = datetime('now') WHERE id = ?")
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

    // If stat2 is the same as stat1, or not provided, apply +2 to stat1
    let is_same_stat = stat2.map(|s| s.eq_ignore_ascii_case(stat1)).unwrap_or(false);

    if let Some(s2) = stat2 {
        if !is_same_stat {
            let col2 = stat_to_column(s2)?;
            let query = format!(
                "UPDATE players SET {} = {} + 1, {} = {} + 1, updated_at = datetime('now') WHERE id = ?",
                col1, col1, col2, col2
            );
            sqlx::query(&query).bind(player_id).execute(pool).await?;
            return Ok(());
        }
    }

    // +2 to a single stat
    let query = format!(
        "UPDATE players SET {} = {} + 2, updated_at = datetime('now') WHERE id = ?",
        col1, col1
    );
    sqlx::query(&query).bind(player_id).execute(pool).await?;

    Ok(())
}

pub async fn use_indomitable(pool: &SqlitePool, player_id: &str, player: &Player) -> Result<bool> {
    if player.indomitable_uses <= 0 {
        return Ok(false);
    }
    sqlx::query(
        "UPDATE players SET indomitable_uses = indomitable_uses - 1, updated_at = datetime('now') WHERE id = ?"
    )
    .bind(player_id)
    .execute(pool)
    .await?;
    Ok(true)
}

fn stat_to_column(stat: &str) -> Result<&'static str> {
    match stat.to_lowercase().as_str() {
        "str" | "strength"     => Ok("str"),
        "dex" | "dexterity"   => Ok("dex"),
        "con" | "constitution" => Ok("con"),
        "int" | "intelligence" => Ok("int"),
        "wis" | "wisdom"       => Ok("wis"),
        "cha" | "charisma"     => Ok("cha"),
        _ => Err(anyhow::anyhow!("Unknown stat: {}", stat)),
    }
}

// ─── Feature tables ───────────────────────────────────────────────────────────

fn fighter_features_at_level(class: &str, level: i64, subclass: Option<&str>) -> Vec<String> {
    if class != "Fighter" {
        return class_features_at_level_generic(class, level);
    }

    let mut features = vec![];

    match level {
        1  => features.extend(["Fighting Style".to_string(), "Second Wind".to_string(), "Weapon Mastery".to_string()]),
        2  => features.extend(["Action Surge".to_string(), "Tactical Mind".to_string()]),
        3  => features.push("Fighter Subclass".to_string()),
        4  => features.push("Ability Score Improvement".to_string()),
        5  => features.extend(["Extra Attack".to_string(), "Tactical Shift".to_string()]),
        6  => features.push("Ability Score Improvement".to_string()),
        7  => features.push("Subclass Feature".to_string()),
        8  => features.push("Ability Score Improvement".to_string()),
        9  => features.extend(["Indomitable".to_string(), "Tactical Master".to_string()]),
        10 => features.push("Subclass Feature".to_string()),
        11 => features.push("Two Extra Attacks".to_string()),
        12 => features.push("Ability Score Improvement".to_string()),
        13 => features.extend(["Indomitable (two uses)".to_string(), "Studied Attacks".to_string()]),
        14 => features.push("Ability Score Improvement".to_string()),
        15 => features.push("Subclass Feature".to_string()),
        16 => features.push("Ability Score Improvement".to_string()),
        17 => features.extend(["Action Surge (two uses)".to_string(), "Indomitable (three uses)".to_string()]),
        18 => features.push("Subclass Feature".to_string()),
        19 => features.push("Epic Boon".to_string()),
        20 => features.push("Three Extra Attacks".to_string()),
        _  => {}
    }

    // Subclass features
    match subclass {
        Some("Champion") => match level {
            3  => features.extend(["Improved Critical".to_string(), "Remarkable Athlete".to_string()]),
            7  => features.push("Additional Fighting Style".to_string()),
            10 => features.push("Heroic Warrior".to_string()),
            15 => features.push("Superior Critical".to_string()),
            18 => features.push("Survivor".to_string()),
            _  => {}
        },
        Some("Battle Master") => match level {
            3  => features.extend(["Combat Superiority".to_string(), "Student of War".to_string()]),
            7  => features.push("Know Your Enemy".to_string()),
            10 => features.push("Improved Combat Superiority (d10)".to_string()),
            15 => features.extend(["Relentless".to_string(), "Improved Combat Superiority (d12)".to_string()]),
            18 => features.push("Ultimate Combat Superiority".to_string()),
            _  => {}
        },
        Some("Psi Warrior") => match level {
            3  => features.push("Psionic Power".to_string()),
            7  => features.push("Telekinetic Adept".to_string()),
            10 => features.push("Guarded Mind".to_string()),
            15 => features.push("Bulwark of Force".to_string()),
            18 => features.push("Telekinetic Master".to_string()),
            _  => {}
        },
        _ => {}
    }

    features
}

fn class_features_at_level_generic(class: &str, level: i64) -> Vec<String> {
    let mut features = vec![];
    match class {
        "Barbarian" => match level {
            2 => features.push("Reckless Attack, Danger Sense".to_string()),
            3 => features.push("Primal Path".to_string()),
            5 => features.push("Extra Attack, Fast Movement".to_string()),
            7 => features.push("Feral Instinct".to_string()),
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

fn eldritch_knight_spell_slots(subclass: Option<&str>, level: i64) -> Option<SpellSlots> {
    if subclass != Some("Eldritch Knight") {
        return None;
    }
    Some(match level {
        3..=4   => SpellSlots { level_1: Some(2), level_2: None, level_3: None, level_4: None, level_5: None, level_6: None, level_7: None, level_8: None, level_9: None },
        5..=6   => SpellSlots { level_1: Some(3), level_2: None, level_3: None, level_4: None, level_5: None, level_6: None, level_7: None, level_8: None, level_9: None },
        7..=9   => SpellSlots { level_1: Some(4), level_2: Some(2), level_3: None, level_4: None, level_5: None, level_6: None, level_7: None, level_8: None, level_9: None },
        10..=12 => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: None, level_4: None, level_5: None, level_6: None, level_7: None, level_8: None, level_9: None },
        13..=15 => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: Some(2), level_4: None, level_5: None, level_6: None, level_7: None, level_8: None, level_9: None },
        16..=18 => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: Some(3), level_4: None, level_5: None, level_6: None, level_7: None, level_8: None, level_9: None },
        19..=20 => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: Some(3), level_4: Some(1), level_5: None, level_6: None, level_7: None, level_8: None, level_9: None },
        _ => return None,
    })
}