use anyhow::Result;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::models::Player;

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

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn roll_die(sides: i64) -> i64 {
    rand::thread_rng().gen_range(1..=sides)
}

fn parse_damage_die(die: &str) -> i64 {
    // Handle both "d8" and "1d8" formats
    let normalized = die.trim().to_lowercase();
    let die_part = if normalized.contains('d') {
        normalized.split('d').last().unwrap_or("6")
    } else {
        "6"
    };
    die_part.parse::<i64>().unwrap_or(6)
}

fn normalize_damage_die(die: &str) -> String {
    // Convert "1d8", "2d6" etc to just the die type "d8", "d6"
    // For simplicity take the largest die if multiple
    let lower = die.trim().to_lowercase();
    if let Some(pos) = lower.find('d') {
        format!("d{}", &lower[pos+1..])
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

fn get_ability_mod(player: &Player, weapon: Option<&crate::models::Item>) -> i64 {
    let str_mod = Player::modifier(player.str);
    let dex_mod = Player::modifier(player.dex);
    if let Some(w) = weapon {
        if w.weapon_range.as_deref() == Some("ranged") { dex_mod } else { str_mod.max(dex_mod) }
    } else {
        str_mod
    }
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
    sqlx::query("UPDATE combat_encounters SET is_active = 0 WHERE campaign_id = ?")
        .bind(campaign_id)
        .execute(pool)
        .await?;

    let encounter_id = Uuid::new_v4().to_string();

    sqlx::query("INSERT INTO combat_encounters (id, campaign_id) VALUES (?, ?)")
        .bind(&encounter_id)
        .bind(campaign_id)
        .execute(pool)
        .await?;

    let dex_mod = Player::modifier(player.dex);
    let player_initiative = roll_die(20) + dex_mod;

    let mut participants: Vec<TurnParticipant> = vec![
        TurnParticipant {
            participant_type: "player".to_string(),
            id: player.id.clone(),
            name: player.name.clone(),
            initiative: player_initiative,
        }
    ];

    for enemy in &enemies {
        let enemy_id = Uuid::new_v4().to_string();
        let attack_bonus = enemy["enemy_attack_bonus"].as_i64().unwrap_or(0);
        let enemy_initiative = roll_die(20) + attack_bonus.min(3);
        let hp = enemy["enemy_hp"].as_i64().unwrap_or(10);
        let name = enemy["enemy_name"].as_str().unwrap_or("Enemy").to_string();

        sqlx::query(
            "INSERT INTO combat_enemies (id, encounter_id, campaign_id, name, description,
             current_hp, max_hp, armor_class, attack_bonus, damage_die, damage_bonus,
             damage_type, initiative, turn_order)
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

    let first = participants.first().map(|p| p.participant_type.as_str()).unwrap_or("player");

    Ok(json!({
        "encounter_id": encounter_id,
        "turn_order": participants,
        "first_turn": first,
        "message": format!("Combat started. {} goes first.", participants.first().map(|p| p.name.as_str()).unwrap_or("Player"))
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
    }

    sqlx::query("UPDATE combat_encounters SET turn_index = ?, round_number = ? WHERE id = ?")
        .bind(next_index)
        .bind(next_round)
        .bind(&encounter.id)
        .execute(pool)
        .await?;

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
            // Do NOT return target_ac — model should not narrate mechanical values
            Ok(json!({
                "target_name": t.name,
                "attack_declared": true
            }))
        }
        None => Ok(json!({
            "error": format!("No living enemy named '{}'", target_name),
            "available_targets": enemies.iter().map(|e| &e.name).collect::<Vec<_>>()
        }))
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

    let weapon = sqlx::query_as::<_, crate::models::Item>(
        "SELECT * FROM items WHERE owner_type = 'player' AND owner_id = ?
         AND is_equipped = 1 AND item_type = 'weapon' LIMIT 1"
    )
    .bind(&player.id)
    .fetch_optional(pool)
    .await?;

    let ability_mod = get_ability_mod(player, weapon.as_ref());
    let total_attack = attack_roll + player.proficiency_bonus + ability_mod;
    let hit = total_attack >= target.armor_class;

    if !hit {
        sqlx::query("UPDATE combat_encounters SET pending_attack_target_id = NULL WHERE id = ?")
            .bind(&encounter.id)
            .execute(pool)
            .await?;
        advance_turn(pool, &encounter).await?;

        return Ok(json!({
            "hit": false,
            "target_name": target.name,
        }));
    }

    // Normalize damage die format (handle "1d8", "d8", etc.)
    let raw_die = weapon.as_ref()
        .and_then(|w| w.damage_die.as_deref())
        .unwrap_or("d6");
    let damage_die = normalize_damage_die(raw_die);

    Ok(json!({
        "hit": true,
        "target_name": target.name,
        "damage_die": damage_die,
        "needs_damage_roll": true
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

    let weapon = sqlx::query_as::<_, crate::models::Item>(
        "SELECT * FROM items WHERE owner_type = 'player' AND owner_id = ?
         AND is_equipped = 1 AND item_type = 'weapon' LIMIT 1"
    )
    .bind(&player.id)
    .fetch_optional(pool)
    .await?;

    let ability_mod = get_ability_mod(player, weapon.as_ref());
    let total_damage = (damage_roll + ability_mod).max(1);
    let new_hp = (target.current_hp - total_damage).max(0);
    let defeated = new_hp == 0;

    sqlx::query("UPDATE combat_enemies SET current_hp = ?, is_alive = ? WHERE id = ?")
        .bind(new_hp)
        .bind(!defeated)
        .bind(&target_id)
        .execute(pool)
        .await?;

    sqlx::query("UPDATE combat_encounters SET pending_attack_target_id = NULL WHERE id = ?")
        .bind(&encounter.id)
        .execute(pool)
        .await?;

    advance_turn(pool, &encounter).await?;

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
        "total_damage": total_damage,
        "damage_type": weapon.as_ref().and_then(|w| w.damage_type.as_deref()).unwrap_or("slashing"),
        "target_name": target.name,
        "enemy_condition": enemy_condition(new_hp, target.max_hp),
        "enemy_defeated": defeated,
        "all_enemies_defeated": alive_count == 0
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

    let attack_roll = roll_die(20) + enemy.attack_bonus;
    let hit = attack_roll >= player.armor_class;

    if !hit {
        advance_turn(pool, &encounter).await?;
        return Ok(json!({
            "hit": false,
            "attacker": enemy.name,
        }));
    }

    let damage_sides = parse_damage_die(&enemy.damage_die);
    let damage_roll = roll_die(damage_sides);
    let total_damage = (damage_roll + enemy.damage_bonus).max(1);

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

    advance_turn(pool, &encounter).await?;

    Ok(json!({
        "hit": true,
        "attacker": enemy.name,
        "total_damage": total_damage,
        "damage_type": enemy.damage_type,
        "player_new_hp": new_hp,
        "player_max_hp": player.max_hp,
        "player_downed": new_hp == 0
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
        "SELECT * FROM combat_enemies WHERE encounter_id = ? AND is_alive = 1 ORDER BY current_hp ASC LIMIT 1"
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
        "enemy_defeated": defeated
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

    let companion = sqlx::query_as::<_, crate::models::Companion>(
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
        "INSERT INTO combat_allies (id, encounter_id, campaign_id, ally_type, companion_id,
         name, description, current_hp, max_hp, armor_class, attack_bonus, damage_die,
         damage_bonus, damage_type, initiative, turn_order)
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
        "initiative": initiative
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
        "INSERT INTO combat_allies (id, encounter_id, campaign_id, ally_type,
         name, description, current_hp, max_hp, armor_class, attack_bonus, damage_die,
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
        "initiative": initiative
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

    if xp_award > 0 {
        let player = sqlx::query_as::<_, Player>(
            "SELECT * FROM players WHERE campaign_id = ? LIMIT 1"
        )
        .bind(campaign_id)
        .fetch_optional(pool)
        .await?;

        if let Some(p) = player {
            let new_xp = p.experience + xp_award;
            sqlx::query("UPDATE players SET experience = ? WHERE id = ?")
                .bind(new_xp)
                .bind(&p.id)
                .execute(pool)
                .await?;

            let threshold = Player::xp_threshold(p.level);
            let level_up = new_xp >= threshold && p.level < 20;

            return Ok(json!({
                "outcome": outcome,
                "xp_awarded": xp_award,
                "new_xp": new_xp,
                "level_up_available": level_up
            }));
        }
    }

    Ok(json!({"outcome": outcome, "xp_awarded": xp_award}))
}