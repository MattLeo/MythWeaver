use anyhow::Result;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::models::*;

// ─── Proficiencies ────────────────────────────────────────────────────────────

pub async fn seed_fighter_proficiencies(
    pool: &SqlitePool,
    campaign_id: &str,
    player_id: &str,
) -> Result<()> {
    let proficiencies: Vec<(&str, &str)> = vec![
        // Saving throws
        ("saving_throw", "strength"),
        ("saving_throw", "constitution"),
        // Skills — player chooses 2, we seed common defaults
        // These will be overridden by the level up UI when built
        ("skill", "athletics"),
        ("skill", "intimidation"),
        // Weapons
        ("weapon", "simple"),
        ("weapon", "martial"),
        // Armor
        ("armor", "light"),
        ("armor", "medium"),
        ("armor", "heavy"),
        ("armor", "shield"),
    ];

    for (prof_type, name) in proficiencies {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT OR IGNORE INTO proficiencies
             (id, campaign_id, player_id, proficiency_type, name, source)
             VALUES (?, ?, ?, ?, ?, 'class')"
        )
        .bind(&id)
        .bind(campaign_id)
        .bind(player_id)
        .bind(prof_type)
        .bind(name)
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn get_proficiencies(
    pool: &SqlitePool,
    player_id: &str,
) -> Result<Vec<Proficiency>> {
    Ok(sqlx::query_as::<_, Proficiency>(
        "SELECT * FROM proficiencies WHERE player_id = ? ORDER BY proficiency_type, name"
    )
    .bind(player_id)
    .fetch_all(pool)
    .await?)
}

pub async fn is_proficient(
    pool: &SqlitePool,
    player_id: &str,
    prof_type: &str,
    name: &str,
) -> Result<bool> {
    // Check exact match first, then category match (e.g. "martial" covers "longsword")
    let exact: Option<Proficiency> = sqlx::query_as::<_, Proficiency>(
        "SELECT * FROM proficiencies WHERE player_id = ? AND proficiency_type = ? AND name = ?"
    )
    .bind(player_id)
    .bind(prof_type)
    .bind(name)
    .fetch_optional(pool)
    .await?;

    if exact.is_some() {
        return Ok(true);
    }

    // Category check for weapons
    if prof_type == "weapon" {
        let is_martial = is_martial_weapon(name);
        let category = if is_martial { "martial" } else { "simple" };
        let cat: Option<Proficiency> = sqlx::query_as::<_, Proficiency>(
            "SELECT * FROM proficiencies WHERE player_id = ? AND proficiency_type = 'weapon' AND name = ?"
        )
        .bind(player_id)
        .bind(category)
        .fetch_optional(pool)
        .await?;
        return Ok(cat.is_some());
    }

    Ok(false)
}

fn is_martial_weapon(name: &str) -> bool {
    matches!(name.to_lowercase().as_str(),
        "longsword" | "shortsword" | "rapier" | "scimitar" | "greatsword" |
        "greataxe" | "handaxe" | "battleaxe" | "warhammer" | "maul" |
        "glaive" | "halberd" | "pike" | "lance" | "trident" |
        "war pick" | "flail" | "morningstar" | "whip" |
        "longbow" | "heavy crossbow" | "hand crossbow" | "net"
    )
}

// ─── Weapon Mastery ───────────────────────────────────────────────────────────

pub async fn get_weapon_masteries(
    pool: &SqlitePool,
    player_id: &str,
) -> Result<Vec<WeaponMastery>> {
    Ok(sqlx::query_as::<_, WeaponMastery>(
        "SELECT * FROM weapon_mastery WHERE player_id = ?"
    )
    .bind(player_id)
    .fetch_all(pool)
    .await?)
}

pub async fn add_weapon_mastery(
    pool: &SqlitePool,
    campaign_id: &str,
    player_id: &str,
    weapon_type: &str,
    mastery_property: &str,
) -> Result<()> {
    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT OR REPLACE INTO weapon_mastery
         (id, campaign_id, player_id, weapon_type, mastery_property)
         VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(campaign_id)
    .bind(player_id)
    .bind(weapon_type)
    .bind(mastery_property)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn change_weapon_mastery(
    pool: &SqlitePool,
    player_id: &str,
    old_weapon: &str,
    new_weapon: &str,
    new_property: &str,
) -> Result<()> {
    sqlx::query(
        "UPDATE weapon_mastery SET weapon_type = ?, mastery_property = ?
         WHERE player_id = ? AND weapon_type = ?"
    )
    .bind(new_weapon)
    .bind(new_property)
    .bind(player_id)
    .bind(old_weapon)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_weapon_mastery_property(
    pool: &SqlitePool,
    player_id: &str,
    weapon_type: &str,
) -> Result<Option<String>> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT mastery_property FROM weapon_mastery WHERE player_id = ? AND weapon_type = ?"
    )
    .bind(player_id)
    .bind(weapon_type)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(p,)| p))
}

// ─── Superiority Dice ─────────────────────────────────────────────────────────

pub async fn seed_superiority_dice(
    pool: &SqlitePool,
    campaign_id: &str,
    player_id: &str,
    pool_name: &str,
    die_size: i64,
    count: i64,
    refresh_type: &str,
) -> Result<()> {
    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO superiority_dice
         (id, campaign_id, player_id, pool_name, die_size, current_dice, max_dice, refresh_type)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(campaign_id)
    .bind(player_id)
    .bind(pool_name)
    .bind(die_size)
    .bind(count)
    .bind(count)
    .bind(refresh_type)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_superiority_dice(
    pool: &SqlitePool,
    player_id: &str,
    pool_name: &str,
) -> Result<Option<SuperiorityDice>> {
    Ok(sqlx::query_as::<_, SuperiorityDice>(
        "SELECT * FROM superiority_dice WHERE player_id = ? AND pool_name = ?"
    )
    .bind(player_id)
    .bind(pool_name)
    .fetch_optional(pool)
    .await?)
}

pub async fn spend_superiority_die(
    pool: &SqlitePool,
    player_id: &str,
    pool_name: &str,
) -> Result<Option<i64>> {
    let dice = match get_superiority_dice(pool, player_id, pool_name).await? {
        Some(d) => d,
        None => return Ok(None),
    };

    if dice.current_dice <= 0 {
        return Ok(None);
    }

    sqlx::query(
        "UPDATE superiority_dice SET current_dice = current_dice - 1
         WHERE player_id = ? AND pool_name = ?"
    )
    .bind(player_id)
    .bind(pool_name)
    .execute(pool)
    .await?;

    Ok(Some(dice.die_size))
}

pub async fn restore_superiority_dice(
    pool: &SqlitePool,
    player_id: &str,
    pool_name: &str,
) -> Result<()> {
    sqlx::query(
        "UPDATE superiority_dice SET current_dice = max_dice
         WHERE player_id = ? AND pool_name = ?"
    )
    .bind(player_id)
    .bind(pool_name)
    .execute(pool)
    .await?;
    Ok(())
}

// ─── Known Maneuvers ──────────────────────────────────────────────────────────

pub async fn get_known_maneuvers(
    pool: &SqlitePool,
    player_id: &str,
) -> Result<Vec<KnownManeuver>> {
    Ok(sqlx::query_as::<_, KnownManeuver>(
        "SELECT * FROM known_maneuvers WHERE player_id = ? ORDER BY maneuver_name"
    )
    .bind(player_id)
    .fetch_all(pool)
    .await?)
}

pub async fn add_maneuver(
    pool: &SqlitePool,
    campaign_id: &str,
    player_id: &str,
    maneuver_name: &str,
) -> Result<()> {
    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT OR IGNORE INTO known_maneuvers
         (id, campaign_id, player_id, maneuver_name)
         VALUES (?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(campaign_id)
    .bind(player_id)
    .bind(maneuver_name)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn replace_maneuver(
    pool: &SqlitePool,
    player_id: &str,
    old_maneuver: &str,
    new_maneuver: &str,
) -> Result<()> {
    sqlx::query(
        "UPDATE known_maneuvers SET maneuver_name = ?
         WHERE player_id = ? AND maneuver_name = ?"
    )
    .bind(new_maneuver)
    .bind(player_id)
    .bind(old_maneuver)
    .execute(pool)
    .await?;
    Ok(())
}

// ─── Active Effects ───────────────────────────────────────────────────────────

pub async fn add_active_effect(
    pool: &SqlitePool,
    campaign_id: &str,
    target_type: &str,
    target_id: &str,
    name: &str,
    effect_type: &str,
    value: Option<i64>,
    damage_type: Option<&str>,
    duration_type: &str,
    duration_value: Option<i64>,
    source: &str,
) -> Result<ActiveEffect> {
    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO active_effects
         (id, campaign_id, target_type, target_id, name, effect_type,
          value, damage_type, duration_type, duration_value, source)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(campaign_id)
    .bind(target_type)
    .bind(target_id)
    .bind(name)
    .bind(effect_type)
    .bind(value)
    .bind(damage_type)
    .bind(duration_type)
    .bind(duration_value)
    .bind(source)
    .execute(pool)
    .await?;

    Ok(sqlx::query_as::<_, ActiveEffect>(
        "SELECT * FROM active_effects WHERE id = ?"
    )
    .bind(&id)
    .fetch_one(pool)
    .await?)
}

pub async fn get_active_effects(
    pool: &SqlitePool,
    target_type: &str,
    target_id: &str,
) -> Result<Vec<ActiveEffect>> {
    Ok(sqlx::query_as::<_, ActiveEffect>(
        "SELECT * FROM active_effects WHERE target_type = ? AND target_id = ?"
    )
    .bind(target_type)
    .bind(target_id)
    .fetch_all(pool)
    .await?)
}

pub async fn remove_active_effect(
    pool: &SqlitePool,
    effect_id: &str,
) -> Result<()> {
    sqlx::query("DELETE FROM active_effects WHERE id = ?")
        .bind(effect_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn remove_effects_by_source(
    pool: &SqlitePool,
    target_id: &str,
    source: &str,
) -> Result<()> {
    sqlx::query(
        "DELETE FROM active_effects WHERE target_id = ? AND source = ?"
    )
    .bind(target_id)
    .bind(source)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn clear_turn_effects(
    pool: &SqlitePool,
    target_id: &str,
) -> Result<()> {
    // Remove effects that expire at end of turn or start of next turn
    sqlx::query(
        "DELETE FROM active_effects
         WHERE target_id = ?
         AND duration_type IN ('end_of_turn', 'start_of_next_turn', 'until_hit')"
    )
    .bind(target_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn tick_round_effects(
    pool: &SqlitePool,
    target_id: &str,
) -> Result<()> {
    // Decrement round-based effects and remove expired ones
    sqlx::query(
        "UPDATE active_effects SET duration_value = duration_value - 1
         WHERE target_id = ? AND duration_type = 'rounds' AND duration_value IS NOT NULL"
    )
    .bind(target_id)
    .execute(pool)
    .await?;

    sqlx::query(
        "DELETE FROM active_effects
         WHERE target_id = ? AND duration_type = 'rounds' AND duration_value <= 0"
    )
    .bind(target_id)
    .execute(pool)
    .await?;

    Ok(())
}

// ─── Action Economy ───────────────────────────────────────────────────────────

pub async fn get_action_economy(
    pool: &SqlitePool,
    encounter_id: &str,
) -> Result<ActionEconomy> {
    let row: (i64, i64, i64, i64, i64, i64) = sqlx::query_as(
        "SELECT actions_remaining, bonus_actions_remaining, reactions_remaining,
                action_surge_available, action_surge_used, attacks_made_this_action
         FROM combat_encounters WHERE id = ?"
    )
    .bind(encounter_id)
    .fetch_one(pool)
    .await?;

    Ok(ActionEconomy {
        actions_remaining: row.0,
        bonus_actions_remaining: row.1,
        reactions_remaining: row.2,
        action_surge_available: row.3 > 0,
        action_surge_used: row.4 > 0,
        attacks_made_this_action: row.5,
    })
}

pub async fn use_action(pool: &SqlitePool, encounter_id: &str) -> Result<bool> {
    let economy = get_action_economy(pool, encounter_id).await?;
    if economy.actions_remaining <= 0 {
        return Ok(false);
    }
    sqlx::query(
        "UPDATE combat_encounters SET actions_remaining = actions_remaining - 1 WHERE id = ?"
    )
    .bind(encounter_id)
    .execute(pool)
    .await?;
    Ok(true)
}

pub async fn use_bonus_action(pool: &SqlitePool, encounter_id: &str) -> Result<bool> {
    let economy = get_action_economy(pool, encounter_id).await?;
    if economy.bonus_actions_remaining <= 0 {
        return Ok(false);
    }
    sqlx::query(
        "UPDATE combat_encounters SET bonus_actions_remaining = bonus_actions_remaining - 1 WHERE id = ?"
    )
    .bind(encounter_id)
    .execute(pool)
    .await?;
    Ok(true)
}

pub async fn use_reaction(pool: &SqlitePool, encounter_id: &str) -> Result<bool> {
    let economy = get_action_economy(pool, encounter_id).await?;
    if economy.reactions_remaining <= 0 {
        return Ok(false);
    }
    sqlx::query(
        "UPDATE combat_encounters SET reactions_remaining = reactions_remaining - 1 WHERE id = ?"
    )
    .bind(encounter_id)
    .execute(pool)
    .await?;
    Ok(true)
}

pub async fn use_action_surge(pool: &SqlitePool, encounter_id: &str) -> Result<bool> {
    let economy = get_action_economy(pool, encounter_id).await?;
    if !economy.action_surge_available || economy.action_surge_used {
        return Ok(false);
    }
    sqlx::query(
        "UPDATE combat_encounters SET
             action_surge_available = action_surge_available - 1,
             action_surge_used = 1,
             actions_remaining = actions_remaining + 1
         WHERE id = ?"
    )
    .bind(encounter_id)
    .execute(pool)
    .await?;
    Ok(true)
}

pub async fn record_attack(pool: &SqlitePool, encounter_id: &str) -> Result<i64> {
    sqlx::query(
        "UPDATE combat_encounters SET attacks_made_this_action = attacks_made_this_action + 1 WHERE id = ?"
    )
    .bind(encounter_id)
    .execute(pool)
    .await?;

    let count: (i64,) = sqlx::query_as(
        "SELECT attacks_made_this_action FROM combat_encounters WHERE id = ?"
    )
    .bind(encounter_id)
    .fetch_one(pool)
    .await?;

    Ok(count.0)
}

pub async fn reset_turn_economy(pool: &SqlitePool, encounter_id: &str) -> Result<()> {
    sqlx::query(
        "UPDATE combat_encounters SET
             actions_remaining = 1,
             bonus_actions_remaining = 1,
             reactions_remaining = 1,
             attacks_made_this_action = 0
         WHERE id = ?"
    )
    .bind(encounter_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn init_action_surge(
    pool: &SqlitePool,
    encounter_id: &str,
    surge_uses: i64,
) -> Result<()> {
    sqlx::query(
        "UPDATE combat_encounters SET action_surge_available = ?, action_surge_used = 0 WHERE id = ?"
    )
    .bind(surge_uses)
    .bind(encounter_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn reset_surge_used(pool: &SqlitePool, encounter_id: &str) -> Result<()> {
    // Called at start of each round — surge can be used once per turn not per round
    sqlx::query(
        "UPDATE combat_encounters SET action_surge_used = 0 WHERE id = ?"
    )
    .bind(encounter_id)
    .execute(pool)
    .await?;
    Ok(())
}

// ─── Subclass seeding ─────────────────────────────────────────────────────────

pub async fn seed_subclass(
    pool: &SqlitePool,
    campaign_id: &str,
    player_id: &str,
    subclass: &str,
    level: i64,
) -> Result<()> {
    match subclass {
        "Battle Master" => {
            let (dice_count, die_size) = battle_master_superiority_dice(level);
            seed_superiority_dice(
                pool, campaign_id, player_id,
                "Battle Master", die_size, dice_count, "short_rest"
            ).await?;

            // Seed starting maneuvers (3 at level 3)
            let starting_maneuvers = ["Precision Attack", "Trip Attack", "Disarming Attack"];
            for m in &starting_maneuvers {
                add_maneuver(pool, campaign_id, player_id, m).await?;
            }
        }
        "Champion" => {
            // Improved Critical — update crit range on player
            sqlx::query(
                "UPDATE players SET crit_range_min = 19 WHERE id = ?"
            )
            .bind(player_id)
            .execute(pool)
            .await?;
        }
        "Psi Warrior" => {
            let (dice_count, die_size) = psi_warrior_energy_dice(level);
            seed_superiority_dice(
                pool, campaign_id, player_id,
                "Psi Warrior", die_size, dice_count, "long_rest"
            ).await?;
        }
        _ => {}
    }
    Ok(())
}