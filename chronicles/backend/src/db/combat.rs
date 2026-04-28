use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::SqlitePool;
use uuid::Uuid;
use rand::Rng;

use crate::models::Player;

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatEncounter {
    pub id: String,
    pub campaign_id: String,
    pub status: String,
    pub round_number: i64,
    pub turn_index: i64,
    pub turn_order_json: Option<String>,
    pub player_rolled_initiative: bool,
    pub actions_remaining: i64,
    pub bonus_actions_remaining: i64,
    pub reactions_remaining: i64,
    pub attacks_remaining: i64,
    pub action_surge_available: bool,
    pub action_surge_used: bool,
    pub attacks_made_this_action: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatEnemy {
    pub id: String,
    pub encounter_id: String,
    pub campaign_id: String,
    pub name: String,
    pub description: Option<String>,
    pub participant_type: String,
    pub weapon_name: String,
    pub max_hp: i64,
    pub current_hp: i64,
    pub armor_class: i64,
    pub attack_bonus: i64,
    pub damage_die: String,
    pub damage_bonus: i64,
    pub damage_type: String,
    pub initiative_score: i64,
    pub is_alive: bool,
    pub is_bloodied: bool,
    pub is_prone: bool,
    pub is_frightened: bool,
    pub is_disarmed: bool,
    pub player_missed_last_attack: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnParticipant {
    pub id: String,
    pub name: String,
    pub participant_type: String, // "player", "enemy", "ally", "companion"
    pub initiative_score: i64,
    pub is_alive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatState {
    pub encounter: CombatEncounter,
    pub enemies: Vec<CombatEnemy>,
    pub turn_order: Vec<TurnParticipant>,
    pub current_actor: Option<TurnParticipant>,
    pub round_number: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoTurnResult {
    pub actor_name: String,
    pub actor_type: String,
    pub action: String,
    pub target: Option<String>,
    pub roll: Option<i64>,
    pub hit: Option<bool>,
    pub damage: Option<i64>,
    pub damage_type: Option<String>,
    pub text: String,
    pub combat_ended: bool,
    pub player_downed: bool,
}

// ─── Start combat ─────────────────────────────────────────────────────────────

pub async fn start_combat(
    pool: &SqlitePool,
    campaign_id: &str,
    player: &Player,
    enemies_data: Vec<Value>,
    allies_data: Vec<Value>,
) -> Result<Value> {
    let encounter_id = Uuid::new_v4().to_string();
    let mut rng = rand::thread_rng();

    let action_surge_available = player.class == "Fighter" && player.level >= 2;
    let attacks_remaining = crate::models::fighter_extra_attacks(player.level);

    sqlx::query(
        "INSERT INTO combat_encounters (
            id, campaign_id, status, round_number, turn_index,
            player_rolled_initiative, actions_remaining, bonus_actions_remaining,
            reactions_remaining, attacks_remaining,
            action_surge_available, action_surge_used, attacks_made_this_action
        ) VALUES (?, ?, 'active', 1, 0, 0, 1, 1, 1, ?, ?, 0, 0)"
    )
    .bind(&encounter_id)
    .bind(campaign_id)
    .bind(attacks_remaining)
    .bind(action_surge_available)
    .execute(pool)
    .await?;

    let mut participants: Vec<TurnParticipant> = vec![];

    // ── Seed enemies ──────────────────────────────────────────────────────────
    for enemy_data in &enemies_data {
        let enemy_id = Uuid::new_v4().to_string();
        let name = enemy_data["enemy_name"].as_str().unwrap_or("Enemy").to_string();
        let hp = enemy_data["enemy_hp"].as_i64().unwrap_or(10);
        let ac = enemy_data["enemy_ac"].as_i64().unwrap_or(12);
        let atk = enemy_data["enemy_attack_bonus"].as_i64().unwrap_or(0);
        let dmg_die = enemy_data["enemy_damage_die"].as_str().unwrap_or("d6").to_string();
        let dmg_bonus = enemy_data["enemy_damage_bonus"].as_i64().unwrap_or(0);
        let dmg_type = enemy_data["enemy_damage_type"].as_str().unwrap_or("slashing").to_string();
        let desc = enemy_data["enemy_description"].as_str().map(|s| s.to_string());
        let weapon_name = enemy_data["enemy_weapon_name"].as_str()
            .unwrap_or("weapon").to_string();

        let dex_mod = atk / 2;
        let init_roll: i64 = rng.gen_range(1..=20) + dex_mod;

        sqlx::query(
            "INSERT INTO combat_enemies (
                id, encounter_id, campaign_id, name, description,
                participant_type, weapon_name,
                max_hp, current_hp, armor_class, attack_bonus,
                damage_die, damage_bonus, damage_type, initiative_score
            ) VALUES (?, ?, ?, ?, ?, 'enemy', ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&enemy_id)
        .bind(&encounter_id)
        .bind(campaign_id)
        .bind(&name)
        .bind(&desc)
        .bind(&weapon_name)
        .bind(hp)
        .bind(hp)
        .bind(ac)
        .bind(atk)
        .bind(&dmg_die)
        .bind(dmg_bonus)
        .bind(&dmg_type)
        .bind(init_roll)
        .execute(pool)
        .await?;

        participants.push(TurnParticipant {
            id: enemy_id,
            name,
            participant_type: "enemy".to_string(),
            initiative_score: init_roll,
            is_alive: true,
        });
    }

    // ── Seed NPC allies ───────────────────────────────────────────────────────
    for ally_data in &allies_data {
        let ally_id = Uuid::new_v4().to_string();
        let name = ally_data["name"].as_str().unwrap_or("Ally").to_string();
        let hp = ally_data["hp"].as_i64().unwrap_or(10);
        let ac = ally_data["ac"].as_i64().unwrap_or(12);
        let atk = ally_data["attack_bonus"].as_i64().unwrap_or(0);
        let dmg_die = ally_data["damage_die"].as_str().unwrap_or("d6").to_string();
        let dmg_bonus = ally_data["damage_bonus"].as_i64().unwrap_or(0);
        let dmg_type = ally_data["damage_type"].as_str().unwrap_or("slashing").to_string();
        let desc = ally_data["description"].as_str().map(|s| s.to_string());
        let weapon_name = ally_data["weapon_name"].as_str()
            .unwrap_or("weapon").to_string();

        let init_roll: i64 = rng.gen_range(1..=20) + (atk / 2);

        sqlx::query(
            "INSERT INTO combat_enemies (
                id, encounter_id, campaign_id, name, description,
                participant_type, weapon_name,
                max_hp, current_hp, armor_class, attack_bonus,
                damage_die, damage_bonus, damage_type, initiative_score
            ) VALUES (?, ?, ?, ?, ?, 'ally', ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&ally_id)
        .bind(&encounter_id)
        .bind(campaign_id)
        .bind(&name)
        .bind(&desc)
        .bind(&weapon_name)
        .bind(hp)
        .bind(hp)
        .bind(ac)
        .bind(atk)
        .bind(&dmg_die)
        .bind(dmg_bonus)
        .bind(&dmg_type)
        .bind(init_roll)
        .execute(pool)
        .await?;

        participants.push(TurnParticipant {
            id: ally_id,
            name,
            participant_type: "ally".to_string(),
            initiative_score: init_roll,
            is_alive: true,
        });
    }

    // ── Seed registered companions ────────────────────────────────────────────
    let companions = crate::db::companions::get_active_companions(pool, campaign_id)
        .await
        .unwrap_or_default();

    for companion in &companions {
        let init_roll: i64 = rng.gen_range(1..=20);
        participants.push(TurnParticipant {
            id: companion.id.clone(),
            name: companion.name.clone(),
            participant_type: "companion".to_string(),
            initiative_score: init_roll,
            is_alive: companion.current_hp > 0,
        });
    }

    let dex_mod = Player::modifier(player.dex);
    let initiative_bonus = dex_mod;

    Ok(json!({
        "encounter_id": encounter_id,
        "needs_player_initiative": true,
        "initiative_bonus": initiative_bonus,
        "has_advantage": false,
        "enemy_count": enemies_data.len(),
        "ally_count": allies_data.len(),
        "companion_count": companions.len(),
        "participants_so_far": participants,
        "message": "Roll for initiative!"
    }))
}

// ─── Submit initiative ────────────────────────────────────────────────────────

pub async fn submit_player_initiative(
    pool: &SqlitePool,
    campaign_id: &str,
    player: &Player,
    roll_result: i64,
    advantage_rolls: Option<Vec<i64>>,
) -> Result<Value> {
    let enc = get_active_encounter(pool, campaign_id).await?
        .ok_or_else(|| anyhow::anyhow!("No active encounter"))?;

    let dex_mod = Player::modifier(player.dex);
    let initiative_bonus = dex_mod;
    let final_roll = roll_result + initiative_bonus;

    // Get all enemies
    let enemies = get_combat_enemies(pool, &enc.id).await?;

    // Build full turn order
    let mut participants: Vec<TurnParticipant> = vec![];

    // Add player
    participants.push(TurnParticipant {
        id: player.id.clone(),
        name: player.name.clone(),
        participant_type: "player".to_string(),
        initiative_score: final_roll,
        is_alive: true,
    });

    // Add enemies
    for enemy in &enemies {
        participants.push(TurnParticipant {
            id: enemy.id.clone(),
            name: enemy.name.clone(),
            participant_type: "enemy".to_string(),
            initiative_score: enemy.initiative_score,
            is_alive: enemy.is_alive,
        });
    }

    // Add companions
    let companions = crate::db::companions::get_active_companions(pool, campaign_id)
        .await
        .unwrap_or_default();

    for companion in &companions {
        let init: i64 = rand::thread_rng().gen_range(1..=20);
        participants.push(TurnParticipant {
            id: companion.id.clone(),
            name: companion.name.clone(),
            participant_type: "companion".to_string(),
            initiative_score: init,
            is_alive: companion.current_hp > 0,
        });
    }

    // Sort by initiative descending, ties broken by player first
    participants.sort_by(|a, b| {
        b.initiative_score.cmp(&a.initiative_score)
            .then_with(|| {
                if a.participant_type == "player" { std::cmp::Ordering::Less }
                else if b.participant_type == "player" { std::cmp::Ordering::Greater }
                else { std::cmp::Ordering::Equal }
            })
    });

    let order_json = serde_json::to_string(&participants)?;

    // Save to encounter
    sqlx::query(
        "UPDATE combat_encounters SET
            turn_order_json = ?,
            player_rolled_initiative = 1,
            turn_index = 0,
            updated_at = datetime('now')
         WHERE id = ?"
    )
    .bind(&order_json)
    .bind(&enc.id)
    .execute(pool)
    .await?;

    // Reset action economy for first turn if player goes first
    let first = participants.first();
    if first.map(|p| p.participant_type.as_str()) == Some("player") {
        reset_action_economy(pool, &enc.id, player).await?;
    }

    Ok(json!({
        "turn_order": participants,
        "player_initiative": final_roll,
        "raw_roll": roll_result,
        "initiative_bonus": initiative_bonus,
        "advantage_rolls": advantage_rolls,
        "combat_ready": true,
    }))
}

// ─── Get combat state ─────────────────────────────────────────────────────────

pub async fn get_combat_state(
    pool: &SqlitePool,
    campaign_id: &str,
) -> Result<Option<Value>> {
    let enc = match get_active_encounter(pool, campaign_id).await? {
        Some(e) => e,
        None => return Ok(None),
    };

    let enemies = get_combat_enemies(pool, &enc.id).await?;
    let turn_order: Vec<TurnParticipant> = enc.turn_order_json
        .as_deref()
        .and_then(|j| serde_json::from_str(j).ok())
        .unwrap_or_default();

    let current_actor = turn_order.get(enc.turn_index as usize).cloned();

    Ok(Some(json!({
        "encounter": enc,
        "enemies": enemies,
        "turn_order": turn_order,
        "current_actor": current_actor,
        "round_number": enc.round_number,
        "action_economy": {
            "actions_remaining": enc.actions_remaining,
            "bonus_actions_remaining": enc.bonus_actions_remaining,
            "reactions_remaining": enc.reactions_remaining,
            "attacks_remaining": enc.attacks_remaining,
            "action_surge_available": enc.action_surge_available,
            "action_surge_used": enc.action_surge_used,
            "attacks_made_this_action": enc.attacks_made_this_action,
        }
    })))
}

// ─── Action economy ───────────────────────────────────────────────────────────

pub async fn reset_action_economy(
    pool: &SqlitePool,
    encounter_id: &str,
    player: &Player,
) -> Result<()> {
    let attacks = if player.class == "Fighter" {
        crate::models::fighter_extra_attacks(player.level)
    } else { 1 };

    let action_surge_available = player.class == "Fighter"
        && player.level >= 2;

    sqlx::query(
        "UPDATE combat_encounters SET
            actions_remaining = 1,
            bonus_actions_remaining = 1,
            reactions_remaining = 1,
            attacks_remaining = ?,
            action_surge_used = 0,
            attacks_made_this_action = 0,
            updated_at = datetime('now')
         WHERE id = ?"
    )
    .bind(attacks)
    .bind(encounter_id)
    .execute(pool)
    .await?;

    // Restore action surge if not used this rest
    if action_surge_available {
        let surge_uses: i64 = sqlx::query_scalar(
            "SELECT current_uses FROM abilities WHERE owner_id = ? AND name = 'Action Surge'"
        )
        .bind(&player.id)
        .fetch_optional(pool)
        .await?
        .unwrap_or(0);

        sqlx::query(
            "UPDATE combat_encounters SET action_surge_available = ?
             WHERE id = ?"
        )
        .bind(surge_uses > 0)
        .bind(encounter_id)
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn use_action(pool: &SqlitePool, encounter_id: &str) -> Result<bool> {
    let remaining: i64 = sqlx::query_scalar(
        "SELECT actions_remaining FROM combat_encounters WHERE id = ?"
    )
    .bind(encounter_id)
    .fetch_one(pool)
    .await?;

    if remaining <= 0 { return Ok(false); }

    sqlx::query(
        "UPDATE combat_encounters SET
            actions_remaining = actions_remaining - 1,
            updated_at = datetime('now')
         WHERE id = ?"
    )
    .bind(encounter_id)
    .execute(pool)
    .await?;

    Ok(true)
}

pub async fn use_bonus_action(pool: &SqlitePool, encounter_id: &str) -> Result<bool> {
    let remaining: i64 = sqlx::query_scalar(
        "SELECT bonus_actions_remaining FROM combat_encounters WHERE id = ?"
    )
    .bind(encounter_id)
    .fetch_one(pool)
    .await?;

    if remaining <= 0 { return Ok(false); }

    sqlx::query(
        "UPDATE combat_encounters SET
            bonus_actions_remaining = bonus_actions_remaining - 1,
            updated_at = datetime('now')
         WHERE id = ?"
    )
    .bind(encounter_id)
    .execute(pool)
    .await?;

    Ok(true)
}

pub async fn use_action_surge(pool: &SqlitePool, encounter_id: &str) -> Result<bool> {
    let row: Option<(bool, bool)> = sqlx::query_as(
        "SELECT action_surge_available, action_surge_used FROM combat_encounters WHERE id = ?"
    )
    .bind(encounter_id)
    .fetch_optional(pool)
    .await?;

    let (available, used) = row.unwrap_or((false, false));
    if !available || used { return Ok(false); }

    sqlx::query(
        "UPDATE combat_encounters SET
            action_surge_available = 0,
            action_surge_used = 1,
            actions_remaining = actions_remaining + 1,
            updated_at = datetime('now')
         WHERE id = ?"
    )
    .bind(encounter_id)
    .execute(pool)
    .await?;

    Ok(true)
}

// ─── Attack resolution ────────────────────────────────────────────────────────

pub async fn resolve_player_attack(
    pool: &SqlitePool,
    campaign_id: &str,
    player: &Player,
    attack_roll: i64,
    target_id: &str,
) -> Result<Value> {
    let enc = get_active_encounter(pool, campaign_id).await?
        .ok_or_else(|| anyhow::anyhow!("No active encounter"))?;

    let enemy = get_enemy(pool, target_id).await?
        .ok_or_else(|| anyhow::anyhow!("Enemy not found"))?;

    if !enemy.is_alive {
        return Ok(json!({"error": "Target is already dead"}));
    }

    // Get equipped weapon
    let weapon = sqlx::query_as::<_, (String, String, String, Option<String>)>(
        "SELECT id, name, damage_die, weapon_type FROM items
         WHERE owner_id = ? AND is_equipped = 1 AND item_type = 'weapon'
         AND (slot = 'main_hand' OR slot = 'off_hand')
         ORDER BY slot ASC LIMIT 1"
    )
    .bind(&player.id)
    .fetch_optional(pool)
    .await?;

    let (weapon_id, weapon_name, damage_die, weapon_type) = weapon.unwrap_or((
        String::new(),
        "Unarmed Strike".to_string(),
        "d4".to_string(),
        None,
    ));

    // Calculate attack bonus
    let str_mod = Player::modifier(player.str);
    let dex_mod = Player::modifier(player.dex);
    let attack_mod = str_mod.max(dex_mod) + player.proficiency_bonus;
    let total_attack = attack_roll + attack_mod;
    let is_crit = attack_roll == 20;
    let is_miss = attack_roll == 1;
    let hits = is_crit || (!is_miss && total_attack >= enemy.armor_class);

    // Check weapon mastery
    let mastery = if !weapon_type.as_deref().unwrap_or("").is_empty() {
        sqlx::query_scalar::<_, String>(
            "SELECT mastery_property FROM weapon_mastery WHERE player_id = ? AND weapon_type = ?"
        )
        .bind(&player.id)
        .bind(weapon_type.as_deref().unwrap_or(""))
        .fetch_optional(pool)
        .await?
    } else { None };

    // Update studied attacks
    if !hits && !is_crit {
        sqlx::query(
            "UPDATE combat_enemies SET player_missed_last_attack = 1 WHERE id = ?"
        )
        .bind(&enemy.id)
        .execute(pool)
        .await?;
    } else {
        sqlx::query(
            "UPDATE combat_enemies SET player_missed_last_attack = 0 WHERE id = ?"
        )
        .bind(&enemy.id)
        .execute(pool)
        .await?;
    }

    // Update attack economy
    sqlx::query(
        "UPDATE combat_encounters SET
            attacks_made_this_action = attacks_made_this_action + 1,
            updated_at = datetime('now')
         WHERE id = ?"
    )
    .bind(&enc.id)
    .execute(pool)
    .await?;

    let attacks_made: i64 = sqlx::query_scalar(
        "SELECT attacks_made_this_action FROM combat_encounters WHERE id = ?"
    )
    .bind(&enc.id)
    .fetch_one(pool)
    .await?;

    let max_attacks = if player.class == "Fighter" {
        crate::models::fighter_extra_attacks(player.level)
    } else { 1 };

    Ok(json!({
        "hit": hits,
        "is_crit": is_crit,
        "is_miss": is_miss,
        "attack_roll": attack_roll,
        "attack_bonus": attack_mod,
        "total_attack": total_attack,
        "enemy_ac": enemy.armor_class,
        "weapon_name": weapon_name,
        "damage_die": damage_die,
        "needs_damage_roll": hits,
        "weapon_mastery": mastery,
        "attacks_made": attacks_made,
        "max_attacks": max_attacks,
        "can_attack_again": hits && attacks_made < max_attacks,
        "target_id": target_id,
        "target_name": enemy.name,
    }))
}

pub async fn resolve_player_attack_with_roll(
    pool: &SqlitePool,
    campaign_id: &str,
    player: &Player,
    roll: i64,
) -> Result<Value> {
    // Get current target from encounter state
    let target_id: Option<String> = sqlx::query_scalar(
        "SELECT current_target_id FROM combat_encounters WHERE campaign_id = ? AND status = 'active'"
    )
    .bind(campaign_id)
    .fetch_optional(pool)
    .await?
    .flatten();

    let target_id = target_id.ok_or_else(|| anyhow::anyhow!("No target selected"))?;
    resolve_player_attack(pool, campaign_id, player, roll, &target_id).await
}

pub async fn apply_player_damage(
    pool: &SqlitePool,
    campaign_id: &str,
    player: &Player,
    damage_roll: i64,
) -> Result<Value> {
    let enc = get_active_encounter(pool, campaign_id).await?
        .ok_or_else(|| anyhow::anyhow!("No active encounter"))?;

    let target_id: Option<String> = sqlx::query_scalar(
        "SELECT current_target_id FROM combat_encounters WHERE id = ?"
    )
    .bind(&enc.id)
    .fetch_optional(pool)
    .await?
    .flatten();

    let target_id = target_id.ok_or_else(|| anyhow::anyhow!("No target selected"))?;
    let enemy = get_enemy(pool, &target_id).await?
        .ok_or_else(|| anyhow::anyhow!("Enemy not found"))?;

    // Get STR mod for damage
    let str_mod = Player::modifier(player.str);
    let dex_mod = Player::modifier(player.dex);
    let dmg_mod = str_mod.max(dex_mod);
    let total_damage = (damage_roll + dmg_mod).max(1);

    let new_hp = (enemy.current_hp - total_damage).max(0);
    let is_dead = new_hp == 0;
    let is_bloodied = new_hp > 0 && new_hp <= enemy.max_hp / 2;

    sqlx::query(
        "UPDATE combat_enemies SET
            current_hp = ?,
            is_alive = ?,
            is_bloodied = ?,
            updated_at = datetime('now')
         WHERE id = ?"
    )
    .bind(new_hp)
    .bind(!is_dead)
    .bind(is_bloodied)
    .bind(&target_id)
    .execute(pool)
    .await?;

    // Check if all enemies dead
    let alive_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM combat_enemies WHERE encounter_id = ? AND is_alive = 1"
    )
    .bind(&enc.id)
    .fetch_one(pool)
    .await?;

    let attacks_made: i64 = sqlx::query_scalar(
        "SELECT attacks_made_this_action FROM combat_encounters WHERE id = ?"
    )
    .bind(&enc.id)
    .fetch_one(pool)
    .await?;

    let max_attacks = if player.class == "Fighter" {
        crate::models::fighter_extra_attacks(player.level)
    } else { 1 };

    Ok(json!({
        "damage_dealt": total_damage,
        "damage_roll": damage_roll,
        "damage_bonus": dmg_mod,
        "enemy_name": enemy.name,
        "enemy_new_hp": new_hp,
        "enemy_dead": is_dead,
        "enemy_bloodied": is_bloodied,
        "all_enemies_defeated": alive_count == 0,
        "can_attack_again": !is_dead && alive_count > 0 && attacks_made < max_attacks,
        "attacks_made": attacks_made,
        "max_attacks": max_attacks,
    }))
}

// ─── Set target ───────────────────────────────────────────────────────────────

pub async fn set_combat_target(
    pool: &SqlitePool,
    campaign_id: &str,
    target_id: &str,
) -> Result<()> {
    sqlx::query(
        "UPDATE combat_encounters SET
            current_target_id = ?,
            updated_at = datetime('now')
         WHERE campaign_id = ? AND status = 'active'"
    )
    .bind(target_id)
    .bind(campaign_id)
    .execute(pool)
    .await?;
    Ok(())
}

// ─── End turn / auto turns ────────────────────────────────────────────────────

pub async fn end_player_turn(
    pool: &SqlitePool,
    campaign_id: &str,
    player: &Player,
) -> Result<Vec<AutoTurnResult>> {
    let mut results = vec![];

    loop {
        // Advance turn index
        advance_turn(pool, campaign_id).await?;

        let enc = match get_active_encounter(pool, campaign_id).await? {
            Some(e) => e,
            None => break,
        };

        let turn_order: Vec<TurnParticipant> = enc.turn_order_json
            .as_deref()
            .and_then(|j| serde_json::from_str(j).ok())
            .unwrap_or_default();

        let current = match turn_order.get(enc.turn_index as usize) {
            Some(p) => p.clone(),
            None => break,
        };

        match current.participant_type.as_str() {
            "player" => {
                // Player's turn again — reset economy and stop
                reset_action_economy(pool, &enc.id, player).await?;
                break;
            }
            "enemy" => {
                if !current.is_alive { continue; }
                let result = resolve_enemy_turn(pool, campaign_id, player, &current).await?;
                let combat_ended = result.combat_ended;
                let player_downed = result.player_downed;
                results.push(result);
                if combat_ended || player_downed { break; }
            }
            "companion" | "ally" => {
                if !current.is_alive { continue; }
                let result = resolve_companion_turn(pool, campaign_id, &current).await?;
                let combat_ended = result.combat_ended;
                results.push(result);
                if combat_ended { break; }
            }
            _ => break,
        }
    }

    Ok(results)
}

async fn advance_turn(pool: &SqlitePool, campaign_id: &str) -> Result<()> {
    let enc = get_active_encounter(pool, campaign_id).await?
        .ok_or_else(|| anyhow::anyhow!("No active encounter"))?;

    let turn_order: Vec<TurnParticipant> = enc.turn_order_json
        .as_deref()
        .and_then(|j| serde_json::from_str(j).ok())
        .unwrap_or_default();

    let count = turn_order.len() as i64;
    if count == 0 { return Ok(()); }

    let new_index = (enc.turn_index + 1) % count;
    let new_round = if new_index == 0 { enc.round_number + 1 } else { enc.round_number };

    // Move current actor to end of order for next round feel
    // (we cycle via index mod instead for simplicity)

    sqlx::query(
        "UPDATE combat_encounters SET
            turn_index = ?,
            round_number = ?,
            updated_at = datetime('now')
         WHERE id = ?"
    )
    .bind(new_index)
    .bind(new_round)
    .bind(&enc.id)
    .execute(pool)
    .await?;

    Ok(())
}

async fn resolve_enemy_turn(
    pool: &SqlitePool,
    campaign_id: &str,
    player: &Player,
    enemy_participant: &TurnParticipant,
) -> Result<AutoTurnResult> {
    let mut rng = rand::thread_rng();

    let enemy = match get_enemy(pool, &enemy_participant.id).await? {
        Some(e) => e,
        None => return Ok(AutoTurnResult {
            actor_name: enemy_participant.name.clone(),
            actor_type: "enemy".to_string(),
            action: "skip".to_string(),
            target: None,
            roll: None,
            hit: None,
            damage: None,
            damage_type: None,
            text: format!("{} cannot act.", enemy_participant.name),
            combat_ended: false,
            player_downed: false,
        }),
    };

    if !enemy.is_alive {
        return Ok(AutoTurnResult {
            actor_name: enemy.name.clone(),
            actor_type: "enemy".to_string(),
            action: "skip".to_string(),
            target: None, roll: None, hit: None, damage: None, damage_type: None,
            text: format!("{} is defeated.", enemy.name),
            combat_ended: false, player_downed: false,
        });
    }

    // Roll attack
    let attack_roll: i64 = rng.gen_range(1..=20);
    let total_attack = attack_roll + enemy.attack_bonus;
    let is_crit = attack_roll == 20;
    let is_miss = attack_roll == 1;
    let hits = is_crit || (!is_miss && total_attack >= player.armor_class);

    if !hits {
        return Ok(AutoTurnResult {
            actor_name: enemy.name.clone(),
            actor_type: "enemy".to_string(),
            action: "attack".to_string(),
            target: Some(player.name.clone()),
            roll: Some(attack_roll),
            hit: Some(false),
            damage: None,
            damage_type: None,
            text: format!("{} attacks {} with their {} and misses (rolled {}).",
                enemy.name, player.name, enemy.weapon_name, total_attack),
            combat_ended: false,
            player_downed: false,
        });
    }

    // Roll damage
    let die_size = parse_die_size(&enemy.damage_die);
    let mut damage: i64 = rng.gen_range(1..=die_size) + enemy.damage_bonus;
    if is_crit { damage += rng.gen_range(1..=die_size); }
    damage = damage.max(1);

    // Apply damage to player
    let new_hp = (player.current_hp - damage).max(0);
    crate::db::player::update_player_hp(pool, &player.id, new_hp).await?;

    let player_downed = new_hp == 0;

    Ok(AutoTurnResult {
        actor_name: enemy.name.clone(),
        actor_type: "enemy".to_string(),
        action: "attack".to_string(),
        target: Some(player.name.clone()),
        roll: Some(attack_roll),
        hit: Some(true),
        damage: Some(damage),
        damage_type: Some(enemy.damage_type.clone()),
        text: if is_crit {
            format!("Critical hit! {} attacks {} with their {} for {} {} damage!",
                enemy.name, player.name, enemy.weapon_name, damage, enemy.damage_type)
        } else {
            format!("{} attacks {} with their {} and hits for {} {} damage.",
                enemy.name, player.name, enemy.weapon_name, damage, enemy.damage_type)
        },
        combat_ended: false,
        player_downed,
    })
}

async fn resolve_companion_turn(
    pool: &SqlitePool,
    campaign_id: &str,
    companion_participant: &TurnParticipant,
) -> Result<AutoTurnResult> {
    let mut rng = rand::thread_rng();

    let companion = sqlx::query_as::<_, crate::models::Companion>(
        "SELECT * FROM companions WHERE id = ? AND is_active = 1"
    )
    .bind(&companion_participant.id)
    .fetch_optional(pool)
    .await?;

    let companion = match companion {
        Some(c) => c,
        None => return Ok(AutoTurnResult {
            actor_name: companion_participant.name.clone(),
            actor_type: "companion".to_string(),
            action: "skip".to_string(),
            target: None, roll: None, hit: None, damage: None, damage_type: None,
            text: format!("{} holds.", companion_participant.name),
            combat_ended: false, player_downed: false,
        }),
    };

    // Pick a random living enemy
    let enc = get_active_encounter(pool, campaign_id).await?
        .ok_or_else(|| anyhow::anyhow!("No active encounter"))?;

    let enemies = get_combat_enemies(pool, &enc.id).await?;
    let living: Vec<&CombatEnemy> = enemies.iter().filter(|e| e.is_alive).collect();

    if living.is_empty() {
        return Ok(AutoTurnResult {
            actor_name: companion.name.clone(),
            actor_type: "companion".to_string(),
            action: "skip".to_string(),
            target: None, roll: None, hit: None, damage: None, damage_type: None,
            text: format!("{} looks around — no enemies remain.", companion.name),
            combat_ended: true, player_downed: false,
        });
    }

    let target = living[rng.gen_range(0..living.len())];
    let attack_roll: i64 = rng.gen_range(1..=20);
    let total = attack_roll + companion.attack_bonus;
    let hits = attack_roll == 20 || (attack_roll != 1 && total >= target.armor_class);

    if !hits {
        return Ok(AutoTurnResult {
            actor_name: companion.name.clone(),
            actor_type: "companion".to_string(),
            action: "attack".to_string(),
            target: Some(target.name.clone()),
            roll: Some(attack_roll),
            hit: Some(false),
            damage: None,
            damage_type: None,
            text: format!("{} attacks {} with their {} and misses.",
                companion.name, target.name, companion.damage_die),
            combat_ended: false, player_downed: false,
        });
    }

    let die_size = parse_die_size(&companion.damage_die);
    let damage = (rng.gen_range(1..=die_size) + companion.damage_bonus).max(1);
    let new_hp = (target.current_hp - damage).max(0);
    let is_dead = new_hp == 0;
    let is_bloodied = new_hp > 0 && new_hp <= target.max_hp / 2;

    sqlx::query(
        "UPDATE combat_enemies SET current_hp = ?, is_alive = ?, is_bloodied = ? WHERE id = ?"
    )
    .bind(new_hp)
    .bind(!is_dead)
    .bind(is_bloodied)
    .bind(&target.id)
    .execute(pool)
    .await?;

    let alive_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM combat_enemies WHERE encounter_id = ? AND is_alive = 1"
    )
    .bind(&enc.id)
    .fetch_one(pool)
    .await?;

    Ok(AutoTurnResult {
        actor_name: companion.name.clone(),
        actor_type: "companion".to_string(),
        action: "attack".to_string(),
        target: Some(target.name.clone()),
        roll: Some(attack_roll),
        hit: Some(true),
        damage: Some(damage),
        damage_type: Some(companion.damage_type.clone()),
        text: format!("{} attacks {} with their {} and hits for {} {} damage{}",
            companion.name, target.name, target.weapon_name, damage, companion.damage_type,
            if is_dead { " — the enemy falls!" } else { "." }
        ),
        combat_ended: alive_count == 0,
        player_downed: false,
    })
}

// ─── Flee ─────────────────────────────────────────────────────────────────────

pub async fn attempt_flee(
    pool: &SqlitePool,
    campaign_id: &str,
    player: &Player,
    roll: i64,
    skill: &str,
) -> Result<Value> {
    let enc = get_active_encounter(pool, campaign_id).await?
        .ok_or_else(|| anyhow::anyhow!("No active encounter"))?;

    let skill_mod = match skill {
        "Athletics"   => Player::modifier(player.str) + player.proficiency_bonus,
        "Acrobatics"  => Player::modifier(player.dex) + player.proficiency_bonus,
        _             => Player::modifier(player.dex),
    };

    let total = roll + skill_mod;
    let dc = 15i64;
    let success = total >= dc;

    if success {
        end_combat(pool, campaign_id, "fled", 0).await?;
        Ok(json!({
            "success": true,
            "roll": roll,
            "modifier": skill_mod,
            "total": total,
            "dc": dc,
            "text": format!("{} successfully flees combat! (rolled {} + {} = {} vs DC {})",
                player.name, roll, skill_mod, total, dc),
        }))
    } else {
        // Opportunity attack from nearest enemy
        let enemies = get_combat_enemies(pool, &enc.id).await?;
        let living: Vec<&CombatEnemy> = enemies.iter().filter(|e| e.is_alive).collect();
        let mut rng = rand::thread_rng();

        let opp_text = if let Some(enemy) = living.first() {
            let opp_roll: i64 = rng.gen_range(1..=20);
            let opp_total = opp_roll + enemy.attack_bonus;
            let opp_hits = opp_roll != 1 && (opp_roll == 20 || opp_total >= player.armor_class);
            if opp_hits {
                let die_size = parse_die_size(&enemy.damage_die);
                let dmg = (rng.gen_range(1..=die_size) + enemy.damage_bonus).max(1);
                let new_hp = (player.current_hp - dmg).max(0);
                crate::db::player::update_player_hp(pool, &player.id, new_hp).await?;
                format!("{} strikes as an opportunity attack for {} damage!", enemy.name, dmg)
            } else {
                format!("{} swings an opportunity attack but misses.", enemy.name)
            }
        } else {
            String::new()
        };

        Ok(json!({
            "success": false,
            "roll": roll,
            "modifier": skill_mod,
            "total": total,
            "dc": dc,
            "opportunity_attack": opp_text,
            "text": format!("{} fails to flee (rolled {} + {} = {} vs DC {}). {}",
                player.name, roll, skill_mod, total, dc, opp_text),
        }))
    }
}

// ─── End combat ───────────────────────────────────────────────────────────────

pub async fn end_combat(
    pool: &SqlitePool,
    campaign_id: &str,
    status: &str,
    _xp: i64,
) -> Result<()> {
    sqlx::query(
        "UPDATE combat_encounters SET status = ?, updated_at = datetime('now')
         WHERE campaign_id = ? AND status = 'active'"
    )
    .bind(status)
    .bind(campaign_id)
    .execute(pool)
    .await?;
    Ok(())
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

pub async fn get_active_encounter(
    pool: &SqlitePool,
    campaign_id: &str,
) -> Result<Option<CombatEncounter>> {
    Ok(sqlx::query_as::<_, CombatEncounter>(
        "SELECT * FROM combat_encounters WHERE campaign_id = ? AND status = 'active' LIMIT 1"
    )
    .bind(campaign_id)
    .fetch_optional(pool)
    .await?)
}

pub async fn get_combat_enemies(
    pool: &SqlitePool,
    encounter_id: &str,
) -> Result<Vec<CombatEnemy>> {
    Ok(sqlx::query_as::<_, CombatEnemy>(
        "SELECT * FROM combat_enemies WHERE encounter_id = ? ORDER BY initiative_score DESC"
    )
    .bind(encounter_id)
    .fetch_all(pool)
    .await?)
}

pub async fn get_enemy(pool: &SqlitePool, enemy_id: &str) -> Result<Option<CombatEnemy>> {
    Ok(sqlx::query_as::<_, CombatEnemy>(
        "SELECT * FROM combat_enemies WHERE id = ?"
    )
    .bind(enemy_id)
    .fetch_optional(pool)
    .await?)
}

pub async fn declare_attack_target(
    pool: &SqlitePool,
    campaign_id: &str,
    target_name: &str,
) -> Result<Value> {
    let enc = get_active_encounter(pool, campaign_id).await?
        .ok_or_else(|| anyhow::anyhow!("No active combat"))?;

    let enemy = sqlx::query_as::<_, CombatEnemy>(
        "SELECT * FROM combat_enemies WHERE encounter_id = ? AND is_alive = 1
         AND LOWER(name) LIKE LOWER(?) LIMIT 1"
    )
    .bind(&enc.id)
    .bind(format!("%{}%", target_name))
    .fetch_optional(pool)
    .await?;

    match enemy {
        Some(e) => {
            set_combat_target(pool, campaign_id, &e.id).await?;
            Ok(json!({"target_id": e.id, "target_name": e.name, "message": "Target acquired"}))
        }
        None => Ok(json!({"error": format!("No living enemy named '{}'", target_name)}))
    }
}

pub async fn resolve_enemy_attack(
    pool: &SqlitePool,
    campaign_id: &str,
    player: &Player,
    enemy_id: &str,
) -> Result<Value> {
    let enemy = get_enemy(pool, enemy_id).await?
        .ok_or_else(|| anyhow::anyhow!("Enemy not found"))?;

    let mut rng = rand::thread_rng();
    let attack_roll: i64 = rng.gen_range(1..=20);
    let total = attack_roll + enemy.attack_bonus;
    let hits = attack_roll == 20 || (attack_roll != 1 && total >= player.armor_class);

    if !hits {
        return Ok(json!({
            "hit": false,
            "roll": attack_roll,
            "enemy_name": enemy.name,
            "text": format!("{} attacks and misses.", enemy.name)
        }));
    }

    let die_size = parse_die_size(&enemy.damage_die);
    let damage: i64 = (rng.gen_range(1..=die_size) + enemy.damage_bonus).max(1);
    let new_hp = (player.current_hp - damage).max(0);
    crate::db::player::update_player_hp(pool, &player.id, new_hp).await?;

    Ok(json!({
        "hit": true,
        "roll": attack_roll,
        "damage": damage,
        "damage_type": enemy.damage_type,
        "enemy_name": enemy.name,
        "new_player_hp": new_hp,
        "player_downed": new_hp == 0,
        "text": format!("{} attacks and hits for {} {} damage.", enemy.name, damage, enemy.damage_type)
    }))
}

pub async fn resolve_ally_turn(
    pool: &SqlitePool,
    enc: &CombatEncounter,
    ally_id: &str,
) -> Result<Value> {
    let companion = sqlx::query_as::<_, crate::models::Companion>(
        "SELECT * FROM companions WHERE id = ?"
    )
    .bind(ally_id)
    .fetch_optional(pool)
    .await?;

    match companion {
        Some(c) => {
            let participant = TurnParticipant {
                id: c.id.clone(),
                name: c.name.clone(),
                participant_type: "companion".to_string(),
                initiative_score: 0,
                is_alive: c.current_hp > 0,
            };
            let result = resolve_companion_turn(pool, &enc.campaign_id, &participant).await?;
            Ok(json!({"ally_acted": true, "text": result.text}))
        }
        None => Ok(json!({"ally_acted": false}))
    }
}

pub async fn use_second_wind(
    pool: &SqlitePool,
    campaign_id: &str,
    player: &Player,
) -> Result<Value> {
    let mut rng = rand::thread_rng();
    let ability: Option<(String, i64)> = sqlx::query_as(
        "SELECT id, current_uses FROM abilities WHERE owner_id = ? AND name = 'Second Wind'"
    )
    .bind(&player.id)
    .fetch_optional(pool)
    .await?;

    let (ability_id, uses) = match ability {
        Some(a) => a,
        None => return Ok(json!({"error": "Second Wind not found"})),
    };

    if uses <= 0 {
        return Ok(json!({"error": "No Second Wind uses remaining"}));
    }

    sqlx::query("UPDATE abilities SET current_uses = current_uses - 1 WHERE id = ?")
        .bind(&ability_id)
        .execute(pool)
        .await?;

    let heal: i64 = rng.gen_range(1..=10) + player.level;
    let new_hp = (player.current_hp + heal).min(player.max_hp);
    crate::db::player::update_player_hp(pool, &player.id, new_hp).await?;

    Ok(json!({
        "healing": heal,
        "new_hp": new_hp,
        "uses_remaining": uses - 1,
        "message": format!("Second Wind restores {} HP.", heal)
    }))
}

pub async fn use_indomitable(
    pool: &SqlitePool,
    player: &Player,
    original_roll: i64,
) -> Result<Value> {
    if player.indomitable_uses <= 0 {
        return Ok(json!({"error": "No Indomitable uses remaining"}));
    }
    sqlx::query(
        "UPDATE players SET indomitable_uses = indomitable_uses - 1 WHERE id = ?"
    )
    .bind(&player.id)
    .execute(pool)
    .await?;

    Ok(json!({
        "original_roll": original_roll,
        "reroll_bonus": player.level,
        "message": format!("Indomitable used. Add {} to your reroll.", player.level),
        "uses_remaining": player.indomitable_uses - 1,
    }))
}

pub async fn use_tactical_mind(
    pool: &SqlitePool,
    player: &Player,
) -> Result<Value> {
    let ability: Option<(String, i64)> = sqlx::query_as(
        "SELECT id, current_uses FROM abilities WHERE owner_id = ? AND name = 'Second Wind'"
    )
    .bind(&player.id)
    .fetch_optional(pool)
    .await?;

    let (ability_id, uses) = match ability {
        Some(a) => a,
        None => return Ok(json!({"error": "Second Wind not found"})),
    };

    if uses <= 0 {
        return Ok(json!({"error": "No Second Wind uses remaining for Tactical Mind"}));
    }

    Ok(json!({
        "ability_id": ability_id,
        "message": "Roll 1d10. If the result plus your check total meets the DC, commit the use.",
        "uses_remaining": uses,
    }))
}

pub async fn commit_tactical_mind(pool: &SqlitePool, ability_id: &str) -> Result<()> {
    sqlx::query("UPDATE abilities SET current_uses = current_uses - 1 WHERE id = ?")
        .bind(ability_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn resolve_maneuver(
    pool: &SqlitePool,
    campaign_id: &str,
    player: &Player,
    maneuver_name: &str,
    target_id: Option<&str>,
    superiority_roll: i64,
) -> Result<Value> {
    // Spend a superiority die
    let spent = crate::db::fighter::spend_superiority_die(pool, &player.id, "Battle Master").await?;
    if !spent {
        return Ok(json!({"error": "No superiority dice remaining"}));
    }

    let save_dc = player.maneuver_save_dc();

    let effect = match maneuver_name {
        "Disarming Attack" => format!(
            "Target must succeed DC {} STR save or drop one item.", save_dc
        ),
        "Pushing Attack" => format!(
            "Target must succeed DC {} STR save or be pushed 15 feet.", save_dc
        ),
        "Trip Attack" => format!(
            "Target must succeed DC {} STR save or fall Prone.", save_dc
        ),
        "Menacing Attack" => format!(
            "Target must succeed DC {} WIS save or be Frightened until end of your next turn.", save_dc
        ),
        "Precision Attack" => format!(
            "Add {} to the attack roll.", superiority_roll
        ),
        "Parry" => format!(
            "Reduce incoming damage by {} + DEX modifier.", superiority_roll
        ),
        "Rally" => format!(
            "An ally gains {} temporary HP.", superiority_roll + Player::modifier(player.cha)
        ),
        _ => format!(
            "{} applied with superiority roll of {}.", maneuver_name, superiority_roll
        ),
    };

    if let Some(tid) = target_id {
        // Apply condition effects
        match maneuver_name {
            "Disarming Attack" => {
                sqlx::query(
                    "UPDATE combat_enemies SET is_disarmed = 1 WHERE id = ?"
                )
                .bind(tid)
                .execute(pool)
                .await?;
            }
            "Trip Attack" => {
                sqlx::query(
                    "UPDATE combat_enemies SET is_prone = 1 WHERE id = ?"
                )
                .bind(tid)
                .execute(pool)
                .await?;
            }
            "Menacing Attack" => {
                sqlx::query(
                    "UPDATE combat_enemies SET is_frightened = 1 WHERE id = ?"
                )
                .bind(tid)
                .execute(pool)
                .await?;
            }
            _ => {}
        }
    }

    Ok(json!({
        "maneuver": maneuver_name,
        "superiority_roll": superiority_roll,
        "save_dc": save_dc,
        "effect": effect,
        "extra_damage": superiority_roll,
    }))
}

pub async fn use_psionic_strike(
    pool: &SqlitePool,
    campaign_id: &str,
    player: &Player,
    psi_roll: i64,
) -> Result<Value> {
    let spent = crate::db::fighter::spend_superiority_die(pool, &player.id, "Psi Warrior").await?;
    if !spent {
        return Ok(json!({"error": "No Psionic Energy Dice remaining"}));
    }

    let int_mod = Player::modifier(player.int);
    let total_damage = (psi_roll + int_mod).max(1);

    Ok(json!({
        "psi_roll": psi_roll,
        "int_modifier": int_mod,
        "force_damage": total_damage,
        "message": format!("Psionic Strike deals {} force damage.", total_damage),
    }))
}

pub async fn use_protective_field(
    pool: &SqlitePool,
    player: &Player,
    psi_roll: i64,
) -> Result<Value> {
    let spent = crate::db::fighter::spend_superiority_die(pool, &player.id, "Psi Warrior").await?;
    if !spent {
        return Ok(json!({"error": "No Psionic Energy Dice remaining"}));
    }

    let int_mod = Player::modifier(player.int);
    let reduction = (psi_roll + int_mod).max(1);

    Ok(json!({
        "psi_roll": psi_roll,
        "damage_reduction": reduction,
        "message": format!("Protective Field reduces damage by {}.", reduction),
    }))
}

fn parse_die_size(die: &str) -> i64 {
    die.trim_start_matches('d')
        .parse::<i64>()
        .unwrap_or(6)
}