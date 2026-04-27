use anyhow::Result;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::models::*;
use crate::db::fighter;

// ─── Models ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CombatEncounter {
    pub id: String,
    pub campaign_id: String,
    pub is_active: bool,
    pub round_number: i64,
    pub turn_index: i64,
    pub turn_order_json: Option<String>,
    pub pending_attack_target_id: Option<String>,
    pub actions_remaining: i64,
    pub bonus_actions_remaining: i64,
    pub reactions_remaining: i64,
    pub action_surge_available: i64,
    pub action_surge_used: i64,
    pub attacks_made_this_action: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CombatEnemy {
    pub id: String,
    pub encounter_id: String,
    pub campaign_id: String,
    pub name: String,
    pub description: Option<String>,
    pub current_hp: i64,
    pub max_hp: i64,
    pub armor_class: i64,
    pub attack_bonus: i64,
    pub damage_die: String,
    pub damage_bonus: i64,
    pub damage_type: String,
    pub initiative: i64,
    pub turn_order: i64,
    pub is_alive: bool,
    pub is_prone: bool,
    pub is_frightened: bool,
    pub is_disarmed: bool,
    pub player_missed_last_attack: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CombatAlly {
    pub id: String,
    pub encounter_id: String,
    pub campaign_id: String,
    pub ally_type: String,
    pub companion_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub current_hp: i64,
    pub max_hp: i64,
    pub armor_class: i64,
    pub attack_bonus: i64,
    pub damage_die: String,
    pub damage_bonus: i64,
    pub damage_type: String,
    pub initiative: i64,
    pub turn_order: i64,
    pub is_alive: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnParticipant {
    pub participant_type: String,
    pub id: String,
    pub name: String,
    pub initiative: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackResult {
    pub hit: bool,
    pub is_crit: bool,
    pub attack_roll: i64,
    pub total_attack: i64,
    pub target_ac: i64,
    pub target_name: String,
    pub damage_die: String,
    pub needs_damage_roll: bool,
    pub advantage: bool,
    pub disadvantage: bool,
    pub studied_attack_bonus: bool,
    pub weapon_mastery: Option<String>,
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn roll_die(sides: i64) -> i64 {
    rand::thread_rng().gen_range(1..=sides)
}

fn parse_damage_die(die: &str) -> i64 {
    let normalized = die.trim().to_lowercase();
    let die_part = if normalized.contains('d') {
        normalized.split('d').last().unwrap_or("6")
    } else {
        "6"
    };
    die_part.parse::<i64>().unwrap_or(6)
}

fn normalize_damage_die(die: &str) -> String {
    let lower = die.trim().to_lowercase();
    if let Some(pos) = lower.find('d') {
        format!("d{}", &lower[pos + 1..])
    } else {
        "d6".to_string()
    }
}

pub fn enemy_condition(hp: i64, max_hp: i64) -> &'static str {
    if max_hp == 0 { return "unknown"; }
    let pct = (hp * 100) / max_hp;
    match pct {
        76..=100 => "uninjured",
        51..=75  => "lightly wounded",
        26..=50  => "bloodied",
        1..=25   => "near death",
        _        => "defeated",
    }
}

fn get_ability_mod(player: &Player, weapon: Option<&Item>) -> i64 {
    let str_mod = Player::modifier(player.str);
    let dex_mod = Player::modifier(player.dex);
    if let Some(w) = weapon {
        if w.weapon_range.as_deref() == Some("ranged") {
            dex_mod
        } else {
            str_mod.max(dex_mod)
        }
    } else {
        str_mod
    }
}

// ─── Active effect helpers ────────────────────────────────────────────────────

async fn get_attack_bonus_from_effects(
    pool: &SqlitePool,
    player_id: &str,
) -> i64 {
    let effects = fighter::get_active_effects(pool, "player", player_id)
        .await
        .unwrap_or_default();
    effects.iter()
        .filter(|e| e.effect_type == "attack_bonus")
        .map(|e| e.value.unwrap_or(0))
        .sum()
}

async fn get_damage_bonus_from_effects(
    pool: &SqlitePool,
    player_id: &str,
) -> i64 {
    let effects = fighter::get_active_effects(pool, "player", player_id)
        .await
        .unwrap_or_default();
    effects.iter()
        .filter(|e| e.effect_type == "damage_bonus")
        .map(|e| e.value.unwrap_or(0))
        .sum()
}

async fn has_advantage_on_attack(
    pool: &SqlitePool,
    player_id: &str,
) -> bool {
    let effects = fighter::get_active_effects(pool, "player", player_id)
        .await
        .unwrap_or_default();
    effects.iter().any(|e| e.effect_type == "advantage_attack")
}

async fn has_disadvantage_on_attack(
    pool: &SqlitePool,
    player_id: &str,
) -> bool {
    let effects = fighter::get_active_effects(pool, "player", player_id)
        .await
        .unwrap_or_default();
    effects.iter().any(|e| e.effect_type == "disadvantage_attack")
}

async fn get_ac_bonus_from_effects(
    pool: &SqlitePool,
    player_id: &str,
) -> i64 {
    let effects = fighter::get_active_effects(pool, "player", player_id)
        .await
        .unwrap_or_default();
    effects.iter()
        .filter(|e| e.effect_type == "ac_bonus")
        .map(|e| e.value.unwrap_or(0))
        .sum()
}

async fn has_resistance(
    pool: &SqlitePool,
    player_id: &str,
    damage_type: &str,
) -> bool {
    let effects = fighter::get_active_effects(pool, "player", player_id)
        .await
        .unwrap_or_default();
    effects.iter().any(|e| {
        e.effect_type == "damage_resistance"
            && (e.damage_type.as_deref() == Some(damage_type)
                || e.damage_type.as_deref() == Some("all"))
    })
}

// ─── Encounter management ─────────────────────────────────────────────────────

pub async fn get_active_encounter(
    pool: &SqlitePool,
    campaign_id: &str,
) -> Result<Option<CombatEncounter>> {
    Ok(sqlx::query_as::<_, CombatEncounter>(
        "SELECT * FROM combat_encounters WHERE campaign_id = ? AND is_active = 1 LIMIT 1"
    )
    .bind(campaign_id)
    .fetch_optional(pool)
    .await?)
}

pub async fn start_combat(
    pool: &SqlitePool,
    campaign_id: &str,
    player: &Player,
    enemies: Vec<Value>,
) -> Result<Value> {
    // Deactivate any existing encounters
    sqlx::query("UPDATE combat_encounters SET is_active = 0 WHERE campaign_id = ?")
        .bind(campaign_id)
        .execute(pool)
        .await?;

    let encounter_id = Uuid::new_v4().to_string();

    // Determine if player has Action Surge available
    let surge_uses = if player.class == "Fighter" {
        fighter_action_surge_uses(player.level)
    } else {
        0
    };

    // Check if player has Second Wind use remaining for surge availability
    let surge_ability_available = if surge_uses > 0 {
        let count: i64 = sqlx::query_scalar(
            "SELECT current_uses FROM abilities WHERE owner_id = ? AND name = 'Action Surge'"
        )
        .bind(&player.id)
        .fetch_optional(pool)
        .await?
        .unwrap_or(0);
        count
    } else {
        0
    };

    sqlx::query(
        "INSERT INTO combat_encounters
         (id, campaign_id, action_surge_available)
         VALUES (?, ?, ?)"
    )
    .bind(&encounter_id)
    .bind(campaign_id)
    .bind(surge_ability_available)
    .execute(pool)
    .await?;

    let dex_mod = Player::modifier(player.dex);

    // Champion gets advantage on initiative
    let player_initiative = if player.subclass.as_deref() == Some("Champion") {
        let roll1 = roll_die(20) + dex_mod;
        let roll2 = roll_die(20) + dex_mod;
        roll1.max(roll2)
    } else {
        roll_die(20) + dex_mod
    };

    let mut participants: Vec<TurnParticipant> = vec![TurnParticipant {
        participant_type: "player".to_string(),
        id: player.id.clone(),
        name: player.name.clone(),
        initiative: player_initiative,
    }];

    for enemy in &enemies {
        let enemy_id = Uuid::new_v4().to_string();
        let attack_bonus = enemy["enemy_attack_bonus"].as_i64().unwrap_or(0);
        let enemy_initiative = roll_die(20) + attack_bonus.min(3);
        let hp = enemy["enemy_hp"].as_i64().unwrap_or(10);
        let name = enemy["enemy_name"].as_str().unwrap_or("Enemy").to_string();

        sqlx::query(
            "INSERT INTO combat_enemies
             (id, encounter_id, campaign_id, name, description,
              current_hp, max_hp, armor_class, attack_bonus, damage_die,
              damage_bonus, damage_type, initiative, turn_order)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0)"
        )
        .bind(&enemy_id)
        .bind(&encounter_id)
        .bind(campaign_id)
        .bind(&name)
        .bind(enemy["enemy_description"].as_str())
        .bind(hp)
        .bind(hp)
        .bind(enemy["enemy_ac"].as_i64().unwrap_or(12))
        .bind(attack_bonus)
        .bind(enemy["enemy_damage_die"].as_str().unwrap_or("d6"))
        .bind(enemy["enemy_damage_bonus"].as_i64().unwrap_or(0))
        .bind(enemy["enemy_damage_type"].as_str().unwrap_or("slashing"))
        .bind(enemy_initiative)
        .execute(pool)
        .await?;

        participants.push(TurnParticipant {
            participant_type: "enemy".to_string(),
            id: enemy_id,
            name,
            initiative: enemy_initiative,
        });
    }

    participants.sort_by(|a, b| b.initiative.cmp(&a.initiative));

    for (i, p) in participants.iter().enumerate() {
        if p.participant_type == "enemy" {
            sqlx::query("UPDATE combat_enemies SET turn_order = ? WHERE id = ?")
                .bind(i as i64)
                .bind(&p.id)
                .execute(pool)
                .await?;
        }
    }

    let turn_order_json = serde_json::to_string(&participants)?;
    sqlx::query("UPDATE combat_encounters SET turn_order_json = ? WHERE id = ?")
        .bind(&turn_order_json)
        .bind(&encounter_id)
        .execute(pool)
        .await?;

    let first = participants.first()
        .map(|p| p.participant_type.as_str())
        .unwrap_or("player");

    Ok(json!({
        "encounter_id": encounter_id,
        "turn_order": participants,
        "first_turn": first,
        "player_initiative": player_initiative,
        "message": format!("Combat started. {} goes first.",
            participants.first().map(|p| p.name.as_str()).unwrap_or("Player"))
    }))
}

pub async fn advance_turn(
    pool: &SqlitePool,
    encounter: &CombatEncounter,
) -> Result<TurnParticipant> {
    let turn_order: Vec<TurnParticipant> = encounter.turn_order_json
        .as_deref()
        .and_then(|j| serde_json::from_str(j).ok())
        .unwrap_or_default();

    let mut next_index = encounter.turn_index + 1;
    let mut next_round = encounter.round_number;

    if next_index >= turn_order.len() as i64 {
        next_index = 0;
        next_round += 1;
        // Reset action surge used flag at start of new round
        fighter::reset_surge_used(pool, &encounter.id).await?;
    }

    // Reset turn economy for the next participant
    fighter::reset_turn_economy(pool, &encounter.id).await?;

    sqlx::query(
        "UPDATE combat_encounters SET turn_index = ?, round_number = ? WHERE id = ?"
    )
    .bind(next_index)
    .bind(next_round)
    .bind(&encounter.id)
    .execute(pool)
    .await?;

    // Clear turn-based active effects for the current participant
    let current = &turn_order[encounter.turn_index as usize];
    fighter::clear_turn_effects(pool, &current.id).await?;

    Ok(turn_order[next_index as usize].clone())
}

// ─── Player attack ────────────────────────────────────────────────────────────

pub async fn declare_attack_target(
    pool: &SqlitePool,
    campaign_id: &str,
    target_name: &str,
) -> Result<Value> {
    let encounter = match get_active_encounter(pool, campaign_id).await? {
        Some(e) => e,
        None => return Ok(json!({"error": "No active combat"})),
    };

    let enemies = sqlx::query_as::<_, CombatEnemy>(
        "SELECT * FROM combat_enemies WHERE encounter_id = ? AND is_alive = 1"
    )
    .bind(&encounter.id)
    .fetch_all(pool)
    .await?;

    let target = enemies.iter().find(|e| {
        e.name.to_lowercase().contains(&target_name.to_lowercase())
    });

    match target {
        Some(t) => {
            sqlx::query(
                "UPDATE combat_encounters SET pending_attack_target_id = ? WHERE id = ?"
            )
            .bind(&t.id)
            .bind(&encounter.id)
            .execute(pool)
            .await?;
            Ok(json!({
                "target_name": t.name,
                "attack_declared": true
            }))
        }
        None => {
            // Fall back to first alive enemy
            match enemies.first() {
                Some(t) => {
                    sqlx::query(
                        "UPDATE combat_encounters SET pending_attack_target_id = ? WHERE id = ?"
                    )
                    .bind(&t.id)
                    .bind(&encounter.id)
                    .execute(pool)
                    .await?;
                    Ok(json!({
                        "target_name": t.name,
                        "attack_declared": true
                    }))
                }
                None => Ok(json!({"error": "No living enemies in combat"}))
            }
        }
    }
}

pub async fn resolve_player_attack_with_roll(
    pool: &SqlitePool,
    campaign_id: &str,
    player: &Player,
    attack_roll: i64,
) -> Result<Value> {
    let encounter = match get_active_encounter(pool, campaign_id).await? {
        Some(e) => e,
        None => return Ok(json!({"error": "No active combat"})),
    };

    let target_id = match &encounter.pending_attack_target_id {
        Some(id) => id.clone(),
        None => return Ok(json!({"error": "No attack target declared"})),
    };

    let target = sqlx::query_as::<_, CombatEnemy>(
        "SELECT * FROM combat_enemies WHERE id = ? AND is_alive = 1"
    )
    .bind(&target_id)
    .fetch_optional(pool)
    .await?;

    let target = match target {
        Some(t) => t,
        None => return Ok(json!({"error": "Target not found or already defeated"})),
    };

    let weapon = sqlx::query_as::<_, Item>(
        "SELECT * FROM items WHERE owner_type = 'player' AND owner_id = ?
         AND is_equipped = 1 AND item_type = 'weapon' LIMIT 1"
    )
    .bind(&player.id)
    .fetch_optional(pool)
    .await?;

    let ability_mod = get_ability_mod(player, weapon.as_ref());
    let effect_attack_bonus = get_attack_bonus_from_effects(pool, &player.id).await;

    // Studied Attacks — if player missed last attack against this target, advantage
    let studied_attack_bonus = player.subclass.as_deref() == Some("Fighter")
        && player.level >= 13
        && target.player_missed_last_attack;

    let has_adv = has_advantage_on_attack(pool, &player.id).await || studied_attack_bonus;
    let has_dis = has_disadvantage_on_attack(pool, &player.id).await;

    // Net advantage/disadvantage — they cancel out
    let effective_advantage = has_adv && !has_dis;
    let effective_disadvantage = has_dis && !has_adv;

    let total_attack = attack_roll
        + player.proficiency_bonus
        + ability_mod
        + effect_attack_bonus;

    // Check crit — use player's crit_range_min (Champion modifies this)
    let is_crit = attack_roll >= player.crit_range_min;
    let hit = is_crit || total_attack >= target.armor_class;

    if !hit {
        // Mark that player missed this target for Studied Attacks
        sqlx::query(
            "UPDATE combat_enemies SET player_missed_last_attack = 1 WHERE id = ?"
        )
        .bind(&target_id)
        .execute(pool)
        .await?;

        sqlx::query(
            "UPDATE combat_encounters SET pending_attack_target_id = NULL WHERE id = ?"
        )
        .bind(&encounter.id)
        .execute(pool)
        .await?;

        // Remove until_hit effects
        let effects = fighter::get_active_effects(pool, "player", &player.id).await?;
        for effect in effects.iter().filter(|e| e.duration_type == "until_hit") {
            fighter::remove_active_effect(pool, &effect.id).await?;
        }

        advance_turn(pool, &encounter).await?;

        return Ok(json!({
            "hit": false,
            "is_crit": false,
            "attack_roll": attack_roll,
            "total_attack": total_attack,
            "target_ac": target.armor_class,
            "target_name": target.name,
            "advantage": effective_advantage,
            "disadvantage": effective_disadvantage,
        }));
    }

    // Hit — clear missed flag and until_hit effects
    sqlx::query(
        "UPDATE combat_enemies SET player_missed_last_attack = 0 WHERE id = ?"
    )
    .bind(&target_id)
    .execute(pool)
    .await?;

    let effects = fighter::get_active_effects(pool, "player", &player.id).await?;
    for effect in effects.iter().filter(|e| e.duration_type == "until_hit") {
        fighter::remove_active_effect(pool, &effect.id).await?;
    }

    // Get damage die from weapon
    let raw_die = weapon.as_ref()
        .and_then(|w| w.damage_die.as_deref())
        .unwrap_or("d6");
    let damage_die = normalize_damage_die(raw_die);

    // Get weapon mastery property if applicable
    let weapon_mastery = if let Some(ref w) = weapon {
        if let Some(ref wt) = w.weapon_type {
            fighter::get_weapon_mastery_property(pool, &player.id, wt).await?
        } else {
            None
        }
    } else {
        None
    };

    // Record attack count
    fighter::record_attack(pool, &encounter.id).await?;

    Ok(json!({
        "hit": true,
        "is_crit": is_crit,
        "attack_roll": attack_roll,
        "total_attack": total_attack,
        "target_ac": target.armor_class,
        "target_name": target.name,
        "damage_die": damage_die,
        "needs_damage_roll": true,
        "advantage": effective_advantage,
        "disadvantage": effective_disadvantage,
        "weapon_mastery": weapon_mastery,
    }))
}

pub async fn apply_player_damage(
    pool: &SqlitePool,
    campaign_id: &str,
    player: &Player,
    damage_roll: i64,
) -> Result<Value> {
    let encounter = match get_active_encounter(pool, campaign_id).await? {
        Some(e) => e,
        None => return Ok(json!({"error": "No active combat"})),
    };

    let target_id = match &encounter.pending_attack_target_id {
        Some(id) => id.clone(),
        None => return Ok(json!({"error": "No pending attack"})),
    };

    let target = sqlx::query_as::<_, CombatEnemy>(
        "SELECT * FROM combat_enemies WHERE id = ?"
    )
    .bind(&target_id)
    .fetch_one(pool)
    .await?;

    let weapon = sqlx::query_as::<_, Item>(
        "SELECT * FROM items WHERE owner_type = 'player' AND owner_id = ?
         AND is_equipped = 1 AND item_type = 'weapon' LIMIT 1"
    )
    .bind(&player.id)
    .fetch_optional(pool)
    .await?;

    let ability_mod = get_ability_mod(player, weapon.as_ref());
    let effect_damage_bonus = get_damage_bonus_from_effects(pool, &player.id).await;

    // If crit, damage_roll already includes doubled dice from frontend
    // Just add modifiers on top
    let total_damage = (damage_roll + ability_mod + effect_damage_bonus).max(1);
    let new_hp = (target.current_hp - total_damage).max(0);
    let defeated = new_hp == 0;

    sqlx::query(
        "UPDATE combat_enemies SET current_hp = ?, is_alive = ? WHERE id = ?"
    )
    .bind(new_hp)
    .bind(!defeated)
    .bind(&target_id)
    .execute(pool)
    .await?;

    sqlx::query(
        "UPDATE combat_encounters SET pending_attack_target_id = NULL WHERE id = ?"
    )
    .bind(&encounter.id)
    .execute(pool)
    .await?;

    // Check if player has extra attacks remaining before advancing turn
    let attacks_made = encounter.attacks_made_this_action;
    let can_attack_again = attacks_made < player.extra_attacks
        && encounter.actions_remaining > 0;

    if !can_attack_again {
        advance_turn(pool, &encounter).await?;
    }

    let alive_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM combat_enemies WHERE encounter_id = ? AND is_alive = 1"
    )
    .bind(&encounter.id)
    .fetch_one(pool)
    .await?;

    Ok(json!({
        "hit": true,
        "damage_roll": damage_roll,
        "ability_mod": ability_mod,
        "effect_damage_bonus": effect_damage_bonus,
        "total_damage": total_damage,
        "damage_type": weapon.as_ref().and_then(|w| w.damage_type.as_deref()).unwrap_or("bludgeoning"),
        "target_name": target.name,
        "enemy_condition": enemy_condition(new_hp, target.max_hp),
        "enemy_defeated": defeated,
        "all_enemies_defeated": alive_count == 0,
        "can_attack_again": can_attack_again,
        "attacks_made": attacks_made + 1,
        "max_attacks": player.extra_attacks,
    }))
}

// ─── Second Wind ──────────────────────────────────────────────────────────────

pub async fn use_second_wind(
    pool: &SqlitePool,
    campaign_id: &str,
    player: &Player,
) -> Result<Value> {
    // Check uses remaining
    let ability: Option<(String, i64)> = sqlx::query_as(
        "SELECT id, current_uses FROM abilities WHERE owner_id = ? AND name = 'Second Wind'"
    )
    .bind(&player.id)
    .fetch_optional(pool)
    .await?;

    let (ability_id, current_uses) = match ability {
        Some(a) => a,
        None => return Ok(json!({"error": "Second Wind not found"})),
    };

    if current_uses <= 0 {
        return Ok(json!({"error": "No Second Wind uses remaining"}));
    }

    let heal_roll = roll_die(10) + player.level;
    let new_hp = (player.current_hp + heal_roll).min(player.max_hp);

    sqlx::query(
        "UPDATE players SET current_hp = ?, updated_at = datetime('now') WHERE id = ?"
    )
    .bind(new_hp)
    .bind(&player.id)
    .execute(pool)
    .await?;

    sqlx::query(
        "UPDATE abilities SET current_uses = current_uses - 1 WHERE id = ?"
    )
    .bind(&ability_id)
    .execute(pool)
    .await?;

    Ok(json!({
        "heal_roll": heal_roll,
        "new_hp": new_hp,
        "max_hp": player.max_hp,
        "uses_remaining": current_uses - 1,
    }))
}

// ─── Tactical Mind ────────────────────────────────────────────────────────────
// Spend a Second Wind use to add 1d10 to a failed ability check

pub async fn use_tactical_mind(
    pool: &SqlitePool,
    player: &Player,
) -> Result<Value> {
    if player.class != "Fighter" || player.level < 2 {
        return Ok(json!({"error": "Tactical Mind not available"}));
    }

    let ability: Option<(String, i64)> = sqlx::query_as(
        "SELECT id, current_uses FROM abilities WHERE owner_id = ? AND name = 'Second Wind'"
    )
    .bind(&player.id)
    .fetch_optional(pool)
    .await?;

    let (ability_id, current_uses) = match ability {
        Some(a) => a,
        None => return Ok(json!({"error": "Second Wind not found"})),
    };

    if current_uses <= 0 {
        return Ok(json!({"error": "No Second Wind uses remaining for Tactical Mind"}));
    }

    let bonus_roll = roll_die(10);

    // Only spend the use if it turns the check into a success
    // The caller must confirm success before committing — return the roll
    // and let the frontend/backend decide whether to consume the use
    Ok(json!({
        "bonus_roll": bonus_roll,
        "ability_id": ability_id,
        "current_uses": current_uses,
        "message": format!("Roll {} added to ability check. Spend use only if check succeeds.", bonus_roll)
    }))
}

pub async fn commit_tactical_mind(
    pool: &SqlitePool,
    ability_id: &str,
) -> Result<()> {
    sqlx::query(
        "UPDATE abilities SET current_uses = current_uses - 1 WHERE id = ?"
    )
    .bind(ability_id)
    .execute(pool)
    .await?;
    Ok(())
}

// ─── Indomitable ─────────────────────────────────────────────────────────────

pub async fn use_indomitable(
    pool: &SqlitePool,
    player: &Player,
    original_roll: i64,
) -> Result<Value> {
    if player.indomitable_uses <= 0 {
        return Ok(json!({"error": "No Indomitable uses remaining"}));
    }

    let reroll = roll_die(20) + player.level;

    sqlx::query(
        "UPDATE players SET indomitable_uses = indomitable_uses - 1,
         updated_at = datetime('now') WHERE id = ?"
    )
    .bind(&player.id)
    .execute(pool)
    .await?;

    Ok(json!({
        "original_roll": original_roll,
        "new_roll": reroll,
        "uses_remaining": player.indomitable_uses - 1,
        "message": format!("Indomitable: rerolled {} as {} + {} (Fighter level) = {}",
            original_roll, reroll - player.level, player.level, reroll)
    }))
}

// ─── Maneuver resolution ──────────────────────────────────────────────────────

pub async fn resolve_maneuver(
    pool: &SqlitePool,
    campaign_id: &str,
    player: &Player,
    maneuver_name: &str,
    target_id: Option<&str>,
    superiority_roll: i64,
) -> Result<Value> {
    // Verify player knows this maneuver
    let known = fighter::get_known_maneuvers(pool, &player.id).await?;
    if !known.iter().any(|m| m.maneuver_name == maneuver_name) {
        return Ok(json!({"error": format!("Maneuver '{}' not known", maneuver_name)}));
    }

    // Spend a superiority die
    let die_size = match fighter::spend_superiority_die(pool, &player.id, "Battle Master").await? {
        Some(d) => d,
        None => return Ok(json!({"error": "No superiority dice remaining"})),
    };

    let save_dc = player.maneuver_save_dc();

    match maneuver_name {
        "Precision Attack" => {
            // Add to a missed attack roll — no target needed
            Ok(json!({
                "maneuver": "Precision Attack",
                "superiority_roll": superiority_roll,
                "effect": format!("Add {} to your attack roll", superiority_roll),
                "save_dc": null,
            }))
        }

        "Trip Attack" => {
            let target_id = target_id.ok_or_else(|| anyhow::anyhow!("Target required"))?;
            // Add damage and force STR save vs prone
            sqlx::query(
                "UPDATE combat_enemies SET is_prone = 1 WHERE id = ?"
            )
            .bind(target_id)
            .execute(pool)
            .await?;

            Ok(json!({
                "maneuver": "Trip Attack",
                "superiority_roll": superiority_roll,
                "bonus_damage": superiority_roll,
                "effect": "Target must make STR saving throw or become prone",
                "save_dc": save_dc,
                "condition_applied": "prone",
            }))
        }

        "Disarming Attack" => {
            let target_id = target_id.ok_or_else(|| anyhow::anyhow!("Target required"))?;
            sqlx::query(
                "UPDATE combat_enemies SET is_disarmed = 1 WHERE id = ?"
            )
            .bind(target_id)
            .execute(pool)
            .await?;

            Ok(json!({
                "maneuver": "Disarming Attack",
                "superiority_roll": superiority_roll,
                "bonus_damage": superiority_roll,
                "effect": "Target must make STR saving throw or drop one held object",
                "save_dc": save_dc,
                "condition_applied": "disarmed",
            }))
        }

        "Menacing Attack" => {
            let target_id = target_id.ok_or_else(|| anyhow::anyhow!("Target required"))?;
            sqlx::query(
                "UPDATE combat_enemies SET is_frightened = 1 WHERE id = ?"
            )
            .bind(target_id)
            .execute(pool)
            .await?;

            Ok(json!({
                "maneuver": "Menacing Attack",
                "superiority_roll": superiority_roll,
                "bonus_damage": superiority_roll,
                "effect": "Target must make WIS saving throw or become frightened until end of your next turn",
                "save_dc": save_dc,
                "condition_applied": "frightened",
            }))
        }

        "Goading Attack" => {
            Ok(json!({
                "maneuver": "Goading Attack",
                "superiority_roll": superiority_roll,
                "bonus_damage": superiority_roll,
                "effect": "Target must make WIS saving throw or have disadvantage on attacks against targets other than you",
                "save_dc": save_dc,
            }))
        }

        "Pushing Attack" => {
            Ok(json!({
                "maneuver": "Pushing Attack",
                "superiority_roll": superiority_roll,
                "bonus_damage": superiority_roll,
                "effect": "Target must make STR saving throw or be pushed up to 15 feet away",
                "save_dc": save_dc,
            }))
        }

        "Sweeping Attack" => {
            Ok(json!({
                "maneuver": "Sweeping Attack",
                "superiority_roll": superiority_roll,
                "effect": format!("Choose another creature within 5 feet of target. If original attack would hit it, deals {} {} damage",
                    superiority_roll,
                    player.subclass.as_deref().unwrap_or("slashing")),
            }))
        }

        "Feinting Attack" => {
            // Bonus action — grants advantage on next attack this turn
            fighter::add_active_effect(
                pool, campaign_id, "player", &player.id,
                "Feinting Attack", "advantage_attack",
                None, None, "until_hit", None, "Feinting Attack"
            ).await?;

            Ok(json!({
                "maneuver": "Feinting Attack",
                "superiority_roll": superiority_roll,
                "effect": "You have advantage on your next attack this turn. If it hits, add superiority die to damage.",
                "bonus_damage_on_hit": superiority_roll,
            }))
        }

        "Parry" => {
            // Reaction — reduce damage taken
            let best_mod = Player::modifier(player.str).max(Player::modifier(player.dex));
            let reduction = superiority_roll + best_mod;
            Ok(json!({
                "maneuver": "Parry",
                "superiority_roll": superiority_roll,
                "damage_reduction": reduction,
                "effect": format!("Reduce incoming melee damage by {}", reduction),
            }))
        }

        "Riposte" => {
            // Reaction after being missed — make a melee attack
            Ok(json!({
                "maneuver": "Riposte",
                "superiority_roll": superiority_roll,
                "effect": "Make a melee attack against the creature that missed you. Add superiority die to damage on hit.",
                "bonus_damage": superiority_roll,
                "requires_attack_roll": true,
            }))
        }

        "Rally" => {
            // Bonus action — give ally temp HP
            let temp_hp = superiority_roll + (player.level / 2);
            Ok(json!({
                "maneuver": "Rally",
                "superiority_roll": superiority_roll,
                "temp_hp_granted": temp_hp,
                "effect": format!("Choose an ally within 30 feet. They gain {} temporary HP.", temp_hp),
            }))
        }

        "Distracting Strike" => {
            Ok(json!({
                "maneuver": "Distracting Strike",
                "superiority_roll": superiority_roll,
                "bonus_damage": superiority_roll,
                "effect": "Next attack roll against this target by someone other than you has advantage before start of your next turn",
            }))
        }

        "Commander's Strike" => {
            Ok(json!({
                "maneuver": "Commander's Strike",
                "superiority_roll": superiority_roll,
                "effect": "Choose a willing ally who can see or hear you. They can use their reaction to make one attack, adding the superiority die to damage on hit.",
            }))
        }

        "Maneuvering Attack" => {
            Ok(json!({
                "maneuver": "Maneuvering Attack",
                "superiority_roll": superiority_roll,
                "bonus_damage": superiority_roll,
                "effect": "Choose a willing ally. They can use their reaction to move up to half their speed without provoking opportunity attacks from your target.",
            }))
        }

        "Evasive Footwork" => {
            // Bonus action — add to AC until start of next turn
            fighter::add_active_effect(
                pool, campaign_id, "player", &player.id,
                "Evasive Footwork", "ac_bonus",
                Some(superiority_roll), None,
                "start_of_next_turn", None, "Evasive Footwork"
            ).await?;

            Ok(json!({
                "maneuver": "Evasive Footwork",
                "superiority_roll": superiority_roll,
                "ac_bonus": superiority_roll,
                "effect": format!("Your AC increases by {} until the start of your next turn.", superiority_roll),
            }))
        }

        "Lunging Attack" => {
            Ok(json!({
                "maneuver": "Lunging Attack",
                "superiority_roll": superiority_roll,
                "bonus_damage": superiority_roll,
                "effect": "You can add the superiority die to damage if you moved 5+ feet in a straight line before hitting.",
            }))
        }

        "Bait and Switch" => {
            fighter::add_active_effect(
                pool, campaign_id, "player", &player.id,
                "Bait and Switch", "ac_bonus",
                Some(superiority_roll), None,
                "start_of_next_turn", None, "Bait and Switch"
            ).await?;

            Ok(json!({
                "maneuver": "Bait and Switch",
                "superiority_roll": superiority_roll,
                "effect": format!("Switch places with a willing creature. You or it gains +{} AC until your next turn.", superiority_roll),
            }))
        }

        "Ambush" => {
            Ok(json!({
                "maneuver": "Ambush",
                "superiority_roll": superiority_roll,
                "effect": format!("Add {} to a Stealth check or Initiative roll.", superiority_roll),
            }))
        }

        "Commanding Presence" => {
            Ok(json!({
                "maneuver": "Commanding Presence",
                "superiority_roll": superiority_roll,
                "effect": format!("Add {} to a Charisma (Intimidation, Performance, or Persuasion) check.", superiority_roll),
            }))
        }

        "Tactical Assessment" => {
            Ok(json!({
                "maneuver": "Tactical Assessment",
                "superiority_roll": superiority_roll,
                "effect": format!("Add {} to an Intelligence (History or Investigation) or Wisdom (Insight) check.", superiority_roll),
            }))
        }

        _ => Ok(json!({"error": format!("Unknown maneuver: {}", maneuver_name)}))
    }
}

// ─── Psi Warrior ─────────────────────────────────────────────────────────────

pub async fn use_psionic_strike(
    pool: &SqlitePool,
    campaign_id: &str,
    player: &Player,
    psi_roll: i64,
) -> Result<Value> {
    if player.subclass.as_deref() != Some("Psi Warrior") {
        return Ok(json!({"error": "Psionic Strike not available"}));
    }

    let die_size = match fighter::spend_superiority_die(pool, &player.id, "Psi Warrior").await? {
        Some(d) => d,
        None => return Ok(json!({"error": "No Psionic Energy Dice remaining"})),
    };

    let int_mod = Player::modifier(player.int);
    let total_force_damage = psi_roll + int_mod;

    Ok(json!({
        "ability": "Psionic Strike",
        "psi_roll": psi_roll,
        "int_modifier": int_mod,
        "total_force_damage": total_force_damage,
        "damage_type": "force",
        "die_size": die_size,
    }))
}

pub async fn use_protective_field(
    pool: &SqlitePool,
    player: &Player,
    psi_roll: i64,
) -> Result<Value> {
    if player.subclass.as_deref() != Some("Psi Warrior") {
        return Ok(json!({"error": "Protective Field not available"}));
    }

    let die_size = match fighter::spend_superiority_die(pool, &player.id, "Psi Warrior").await? {
        Some(d) => d,
        None => return Ok(json!({"error": "No Psionic Energy Dice remaining"})),
    };

    let int_mod = Player::modifier(player.int);
    let damage_reduction = (psi_roll + int_mod).max(1);

    Ok(json!({
        "ability": "Protective Field",
        "psi_roll": psi_roll,
        "int_modifier": int_mod,
        "damage_reduction": damage_reduction,
        "die_size": die_size,
    }))
}

// ─── Enemy attack ─────────────────────────────────────────────────────────────

pub async fn resolve_enemy_attack(
    pool: &SqlitePool,
    campaign_id: &str,
    player: &Player,
    enemy_id: &str,
) -> Result<Value> {
    let encounter = match get_active_encounter(pool, campaign_id).await? {
        Some(e) => e,
        None => return Ok(json!({"error": "No active combat"})),
    };

    let enemy = sqlx::query_as::<_, CombatEnemy>(
        "SELECT * FROM combat_enemies WHERE id = ? AND encounter_id = ?"
    )
    .bind(enemy_id)
    .bind(&encounter.id)
    .fetch_optional(pool)
    .await?;

    let enemy = match enemy {
        Some(e) if e.is_alive => e,
        _ => {
            advance_turn(pool, &encounter).await?;
            return Ok(json!({"skipped": true, "reason": "enemy not found or already defeated"}));
        }
    };

    // Frightened enemies have disadvantage on attacks
    let base_roll = if enemy.is_frightened {
        let (roll1, roll2) = (roll_die(20), roll_die(20));
        roll1.min(roll2)
    } else {
        roll_die(20)
    };
    let attack_roll = base_roll + enemy.attack_bonus;

    // Player AC includes active effect bonuses
    let ac_bonus = get_ac_bonus_from_effects(pool, &player.id).await;
    let effective_ac = player.armor_class + ac_bonus;

    let hit = attack_roll >= effective_ac;

    if !hit {
        advance_turn(pool, &encounter).await?;
        return Ok(json!({
            "hit": false,
            "attacker": enemy.name,
            "attack_roll": attack_roll,
            "player_ac": effective_ac,
        }));
    }

    let damage_sides = parse_damage_die(&enemy.damage_die);
    let damage_roll = roll_die(damage_sides);
    let total_damage_raw = (damage_roll + enemy.damage_bonus).max(1);

    // Check resistance (e.g. from Rage for Barbarian — future use)
    let total_damage = if has_resistance(pool, &player.id, &enemy.damage_type).await {
        total_damage_raw / 2
    } else {
        total_damage_raw
    };

    let (damage_to_hp, new_temp) = if player.temp_hp > 0 {
        let absorbed = total_damage.min(player.temp_hp);
        (total_damage - absorbed, player.temp_hp - absorbed)
    } else {
        (total_damage, 0)
    };

    let new_hp = (player.current_hp - damage_to_hp).max(0);

    sqlx::query(
        "UPDATE players SET current_hp = ?, temp_hp = ?, updated_at = datetime('now') WHERE id = ?"
    )
    .bind(new_hp)
    .bind(new_temp)
    .bind(&player.id)
    .execute(pool)
    .await?;

    // Champion Survivor — at start of turn if bloodied, regain HP
    // This is handled in advance_turn for the player's turn

    advance_turn(pool, &encounter).await?;

    Ok(json!({
        "hit": true,
        "attacker": enemy.name,
        "attack_roll": attack_roll,
        "player_ac": effective_ac,
        "damage_roll": damage_roll,
        "damage_bonus": enemy.damage_bonus,
        "total_damage": total_damage,
        "damage_type": enemy.damage_type,
        "was_resisted": total_damage < total_damage_raw,
        "player_new_hp": new_hp,
        "player_max_hp": player.max_hp,
        "player_downed": new_hp == 0,
    }))
}

// ─── Ally turn ────────────────────────────────────────────────────────────────

pub async fn resolve_ally_turn(
    pool: &SqlitePool,
    encounter: &CombatEncounter,
    ally_id: &str,
) -> Result<Value> {
    let ally = sqlx::query_as::<_, CombatAlly>(
        "SELECT * FROM combat_allies WHERE id = ? AND encounter_id = ?"
    )
    .bind(ally_id)
    .bind(&encounter.id)
    .fetch_optional(pool)
    .await?;

    let ally = match ally {
        Some(a) if a.is_alive => a,
        _ => {
            advance_turn(pool, encounter).await?;
            return Ok(json!({"ally_acted": false}));
        }
    };

    let target = sqlx::query_as::<_, CombatEnemy>(
        "SELECT * FROM combat_enemies WHERE encounter_id = ? AND is_alive = 1
         ORDER BY current_hp ASC LIMIT 1"
    )
    .bind(&encounter.id)
    .fetch_optional(pool)
    .await?;

    let target = match target {
        Some(t) => t,
        None => {
            advance_turn(pool, encounter).await?;
            return Ok(json!({"ally_acted": false, "reason": "no targets"}));
        }
    };

    let attack_roll = roll_die(20) + ally.attack_bonus;
    let hit = attack_roll >= target.armor_class;

    if !hit {
        advance_turn(pool, encounter).await?;
        return Ok(json!({
            "ally_acted": true,
            "ally_name": ally.name,
            "target": target.name,
            "hit": false,
        }));
    }

    let damage_sides = parse_damage_die(&ally.damage_die);
    let damage_roll = roll_die(damage_sides);
    let total_damage = (damage_roll + ally.damage_bonus).max(1);
    let new_hp = (target.current_hp - total_damage).max(0);
    let defeated = new_hp == 0;

    sqlx::query("UPDATE combat_enemies SET current_hp = ?, is_alive = ? WHERE id = ?")
        .bind(new_hp)
        .bind(!defeated)
        .bind(&target.id)
        .execute(pool)
        .await?;

    advance_turn(pool, encounter).await?;

    Ok(json!({
        "ally_acted": true,
        "ally_name": ally.name,
        "target": target.name,
        "hit": true,
        "total_damage": total_damage,
        "damage_type": ally.damage_type,
        "enemy_condition": enemy_condition(new_hp, target.max_hp),
        "enemy_defeated": defeated,
    }))
}

// ─── Companion to combat ──────────────────────────────────────────────────────

pub async fn add_companion_to_combat(
    pool: &SqlitePool,
    campaign_id: &str,
    companion_id: &str,
) -> Result<Value> {
    let encounter = match get_active_encounter(pool, campaign_id).await? {
        Some(e) => e,
        None => return Ok(json!({"error": "No active combat"})),
    };

    let companion = sqlx::query_as::<_, Companion>(
        "SELECT * FROM companions WHERE id = ? AND campaign_id = ?"
    )
    .bind(companion_id)
    .bind(campaign_id)
    .fetch_optional(pool)
    .await?;

    let companion = match companion {
        Some(c) => c,
        None => return Ok(json!({"error": "Companion not found"})),
    };

    let initiative = roll_die(20) + 1;
    let ally_id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO combat_allies
         (id, encounter_id, campaign_id, ally_type, companion_id,
          name, description, current_hp, max_hp, armor_class, attack_bonus,
          damage_die, damage_bonus, damage_type, initiative, turn_order)
         VALUES (?, ?, ?, 'companion', ?, ?, NULL, ?, ?, ?, ?, ?, ?, ?, ?, 0)"
    )
    .bind(&ally_id)
    .bind(&encounter.id)
    .bind(campaign_id)
    .bind(companion_id)
    .bind(&companion.name)
    .bind(companion.current_hp)
    .bind(companion.max_hp)
    .bind(companion.armor_class)
    .bind(companion.attack_bonus)
    .bind(&companion.damage_die)
    .bind(companion.damage_bonus)
    .bind(&companion.damage_type)
    .bind(initiative)
    .execute(pool)
    .await?;

    let mut turn_order: Vec<TurnParticipant> = encounter.turn_order_json
        .as_deref()
        .and_then(|j| serde_json::from_str(j).ok())
        .unwrap_or_default();

    turn_order.push(TurnParticipant {
        participant_type: "ally".to_string(),
        id: ally_id.clone(),
        name: companion.name.clone(),
        initiative,
    });
    turn_order.sort_by(|a, b| b.initiative.cmp(&a.initiative));

    let new_json = serde_json::to_string(&turn_order)?;
    sqlx::query("UPDATE combat_encounters SET turn_order_json = ? WHERE id = ?")
        .bind(&new_json)
        .bind(&encounter.id)
        .execute(pool)
        .await?;

    Ok(json!({
        "message": format!("{} joins the combat", companion.name),
        "ally_id": ally_id,
        "initiative": initiative,
    }))
}

pub async fn add_ally_to_combat(
    pool: &SqlitePool,
    campaign_id: &str,
    data: &Value,
) -> Result<Value> {
    let encounter = match get_active_encounter(pool, campaign_id).await? {
        Some(e) => e,
        None => return Ok(json!({"error": "No active combat"})),
    };

    let initiative = roll_die(20) + data["attack_bonus"].as_i64().unwrap_or(0).min(3);
    let ally_id = Uuid::new_v4().to_string();
    let name = data["name"].as_str().unwrap_or("Ally").to_string();
    let hp = data["hp"].as_i64().unwrap_or(10);

    sqlx::query(
        "INSERT INTO combat_allies
         (id, encounter_id, campaign_id, ally_type, name, description,
          current_hp, max_hp, armor_class, attack_bonus, damage_die,
          damage_bonus, damage_type, initiative, turn_order)
         VALUES (?, ?, ?, 'npc', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0)"
    )
    .bind(&ally_id)
    .bind(&encounter.id)
    .bind(campaign_id)
    .bind(&name)
    .bind(data["description"].as_str())
    .bind(hp)
    .bind(hp)
    .bind(data["ac"].as_i64().unwrap_or(12))
    .bind(data["attack_bonus"].as_i64().unwrap_or(2))
    .bind(data["damage_die"].as_str().unwrap_or("d6"))
    .bind(data["damage_bonus"].as_i64().unwrap_or(0))
    .bind(data["damage_type"].as_str().unwrap_or("slashing"))
    .bind(initiative)
    .execute(pool)
    .await?;

    let mut turn_order: Vec<TurnParticipant> = encounter.turn_order_json
        .as_deref()
        .and_then(|j| serde_json::from_str(j).ok())
        .unwrap_or_default();

    turn_order.push(TurnParticipant {
        participant_type: "ally".to_string(),
        id: ally_id.clone(),
        name: name.clone(),
        initiative,
    });
    turn_order.sort_by(|a, b| b.initiative.cmp(&a.initiative));

    let new_json = serde_json::to_string(&turn_order)?;
    sqlx::query("UPDATE combat_encounters SET turn_order_json = ? WHERE id = ?")
        .bind(&new_json)
        .bind(&encounter.id)
        .execute(pool)
        .await?;

    Ok(json!({
        "message": format!("{} joins the combat as an ally", name),
        "ally_id": ally_id,
        "initiative": initiative,
    }))
}

// ─── End combat ───────────────────────────────────────────────────────────────

pub async fn end_combat(
    pool: &SqlitePool,
    campaign_id: &str,
    outcome: &str,
    xp_award: i64,
) -> Result<Value> {
    sqlx::query(
        "UPDATE combat_encounters SET is_active = 0 WHERE campaign_id = ? AND is_active = 1"
    )
    .bind(campaign_id)
    .execute(pool)
    .await?;

    // Clear all active effects on the player
    let player = sqlx::query_as::<_, Player>(
        "SELECT * FROM players WHERE campaign_id = ? LIMIT 1"
    )
    .bind(campaign_id)
    .fetch_optional(pool)
    .await?;

    if let Some(ref p) = player {
        sqlx::query(
            "DELETE FROM active_effects WHERE target_id = ? AND duration_type != 'permanent'"
        )
        .bind(&p.id)
        .execute(pool)
        .await?;
    }

    if xp_award > 0 {
        if let Some(p) = player {
            let new_xp = p.experience + xp_award;
            sqlx::query("UPDATE players SET experience = ? WHERE id = ?")
                .bind(new_xp)
                .bind(&p.id)
                .execute(pool)
                .await?;

            let threshold = Player::xp_threshold(p.level);
            let level_up_available = new_xp >= threshold && p.level < 20;

            return Ok(json!({
                "outcome": outcome,
                "xp_awarded": xp_award,
                "new_xp": new_xp,
                "level_up_available": level_up_available,
            }));
        }
    }

    Ok(json!({"outcome": outcome, "xp_awarded": xp_award}))
}