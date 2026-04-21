use axum::{
    extract::{Path, State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::db::{campaign, player, world, items, companions, time};
use crate::llm::{ChatMessage, prompt};
use crate::models::*;
use crate::AppState;

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn strip_state_tag(s: &str) -> (String, Option<String>) {
    let re = regex::Regex::new(r"\[STATE:(\w+)\]").unwrap();
    let state = re.captures(s).map(|c| c[1].to_string());
    let clean = re.replace_all(s, "").trim().to_string();
    (clean, state)
}

// ─── Campaign ─────────────────────────────────────────────────────────────────

pub async fn create_campaign(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateCampaignRequest>,
) -> impl IntoResponse {
    let pool = &state.pool;

    let campaign_name = req.name.clone().unwrap_or_else(|| "MythWeaver Campaign".to_string());
    let camp = match campaign::create_campaign(pool, &campaign_name).await {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    };

    let p = match player::create_player(pool, &camp.id, &req).await {
        Ok(p) => p,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    };

    seed_class_abilities(pool, &camp.id, &p.id, &p.class).await;
    seed_starting_equipment(pool, &camp.id, &p.id, &p.class).await;

    if let Err(e) = time::init_campaign_time(pool, &camp.id).await {
        tracing::warn!("Failed to init campaign time: {}", e);
    }

    let session = match campaign::create_session(pool, &camp.id).await {
        Ok(s) => s,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    };

    (StatusCode::OK, Json(json!({
        "campaign": camp,
        "player": p,
        "session": session
    })))
}

pub async fn get_campaign_state(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
) -> impl IntoResponse {
    let pool = &state.pool;

    let camp = campaign::get_campaign(pool, &campaign_id).await;
    let p = player::get_player_by_campaign(pool, &campaign_id).await;
    let session = campaign::get_active_session(pool, &campaign_id).await;
    let camp_time = time::get_campaign_time(pool, &campaign_id).await;

    match (camp, p, session) {
        (Ok(Some(c)), Ok(Some(p)), Ok(s)) => {
            (StatusCode::OK, Json(json!({
                "campaign": c,
                "player": p,
                "session": s,
                "time": camp_time.ok().flatten()
            })))
        }
        _ => (StatusCode::NOT_FOUND, Json(json!({"error": "Campaign not found"}))),
    }
}

pub async fn get_player_state(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
) -> impl IntoResponse {
    let pool = &state.pool;

    let p = match player::get_player_by_campaign(pool, &campaign_id).await {
        Ok(Some(p)) => p,
        _ => return (StatusCode::NOT_FOUND, Json(json!({"error": "Player not found"}))),
    };

    let abilities = world::get_abilities(pool, &p.id, "player").await.unwrap_or_default();
    let all_items = items::get_player_items(pool, &p.id).await.unwrap_or_default();
    let active_companions = companions::get_active_companions(pool, &campaign_id).await.unwrap_or_default();
    let camp_time = time::get_campaign_time(pool, &campaign_id).await.ok().flatten();

    (StatusCode::OK, Json(json!({
        "player": p,
        "abilities": abilities,
        "items": all_items,
        "companions": active_companions,
        "time": camp_time
    })))
}

// ─── Session ──────────────────────────────────────────────────────────────────

pub async fn start_session(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
) -> impl IntoResponse {
    match campaign::create_session(&state.pool, &campaign_id).await {
        Ok(s) => (StatusCode::OK, Json(json!({"session": s}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    }
}

pub async fn end_session(
    State(state): State<Arc<AppState>>,
    Path((_campaign_id, session_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let pool = &state.pool;

    let messages = campaign::get_session_messages(pool, &session_id).await;

    if let Ok(msgs) = messages {
        if !msgs.is_empty() {
            let narrative_msgs: Vec<&Message> = msgs.iter()
                .filter(|m| m.role != "tool")
                .collect();

            if !narrative_msgs.is_empty() {
                let conversation = narrative_msgs.iter()
                    .map(|m| format!("[{}]: {}", m.role.to_uppercase(), m.content))
                    .collect::<Vec<_>>()
                    .join("\n\n");

                let summary_prompt = format!(
                    "Summarize this D&D session in 3-5 sentences. Focus on key events, decisions, NPC encounters, locations visited, and plot developments. Be specific and narrative:\n\n{}",
                    &conversation[..conversation.len().min(8000)]
                );

                let summary_messages = vec![ChatMessage::user(&summary_prompt)];

                if let Ok(result) = state.llm.run_agentic_loop(
                    pool,
                    &msgs[0].campaign_id,
                    "You are a D&D session summarizer. Write concise, narrative summaries.",
                    summary_messages,
                    &GameState::Exploration,
                ).await {
                    let _ = campaign::save_session_summary(
                        pool,
                        &msgs[0].campaign_id,
                        &session_id,
                        &result.narrative,
                    ).await;
                }
            }
        }
    }

    match campaign::end_session(pool, &session_id).await {
        Ok(_) => (StatusCode::OK, Json(json!({"message": "Session ended and summarized"}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    }
}

// ─── Messaging ────────────────────────────────────────────────────────────────

pub async fn send_message(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SendMessageRequest>,
) -> impl IntoResponse {
    let pool = &state.pool;
    let campaign_id = &req.campaign_id;
    let session_id = &req.session_id;

    let p = match player::get_player_by_campaign(pool, campaign_id).await {
        Ok(Some(p)) => p,
        _ => return (StatusCode::NOT_FOUND, Json(json!({"error": "Player not found"}))),
    };

    let camp_time = time::get_campaign_time(pool, campaign_id).await.ok().flatten();
    let summaries = campaign::get_session_summaries(pool, campaign_id).await
        .unwrap_or_default()
        .into_iter()
        .map(|s| s.summary)
        .collect::<Vec<_>>();

    let game_state = req.game_state.as_deref()
        .map(GameState::from_str)
        .unwrap_or(GameState::Exploration);

    let system = prompt::build_system_prompt(&p, camp_time.as_ref(), &summaries);

    // ── Combat roll interception ──────────────────────────────────────────────
    if let Some(roll) = &req.roll_result {
        if let Ok(Some(_)) = crate::db::combat::get_active_encounter(pool, campaign_id).await {
            let skill = roll.skill.as_deref().unwrap_or("");

            if skill == "Attack" {
                let attack_result = crate::db::combat::resolve_player_attack_with_roll(
                    pool, campaign_id, &p, roll.result
                ).await.unwrap_or(json!({"error": "attack failed"}));

                if attack_result["needs_damage_roll"].as_bool().unwrap_or(false) {
                    let damage_die = attack_result["damage_die"].as_str().unwrap_or("d6");
                    return (StatusCode::OK, Json(json!({
                        "type": "roll_request",
                        "roll": {
                            "tool_call_id": "damage",
                            "die": damage_die,
                            "skill": "Damage",
                            "dc": 0,
                            "reason": format!("You hit! Roll {} damage.", damage_die)
                        }
                    })));
                }

                let raw = state.llm.narrate_combat_result(&system, &attack_result).await
                    .unwrap_or_else(|_| "Your attack misses.".to_string());
                let (narrative, _) = strip_state_tag(&raw);
                let _ = campaign::save_message(pool, session_id, campaign_id, "assistant", &narrative, None).await;
                let combat_turns = resolve_combat_turns(pool, campaign_id, &state.llm, &system).await;
                for t in &combat_turns {
                    let _ = campaign::save_message(pool, session_id, campaign_id, "assistant", t, None).await;
                }
                return (StatusCode::OK, Json(json!({
                    "type": "narrative",
                    "content": narrative,
                    "combat_turns": combat_turns
                })));
            }

            if skill == "Damage" {
                let damage_result = crate::db::combat::apply_player_damage(
                    pool, campaign_id, &p, roll.result
                ).await.unwrap_or(json!({"error": "damage failed"}));

                let raw = state.llm.narrate_combat_result(&system, &damage_result).await
                    .unwrap_or_else(|_| "The attack lands.".to_string());
                let (narrative, _) = strip_state_tag(&raw);
                let _ = campaign::save_message(pool, session_id, campaign_id, "assistant", &narrative, None).await;

                if damage_result["all_enemies_defeated"].as_bool().unwrap_or(false) {
                    let _ = crate::db::combat::end_combat(pool, campaign_id, "victory", 100).await;
                    return (StatusCode::OK, Json(json!({
                        "type": "narrative",
                        "content": narrative,
                        "combat_turns": [],
                        "combat_ended": true,
                        "new_state": "exploration"
                    })));
                }

                let combat_turns = resolve_combat_turns(pool, campaign_id, &state.llm, &system).await;
                for t in &combat_turns {
                    let _ = campaign::save_message(pool, session_id, campaign_id, "assistant", t, None).await;
                }
                return (StatusCode::OK, Json(json!({
                    "type": "narrative",
                    "content": narrative,
                    "combat_turns": combat_turns
                })));
            }
        }
    }

    // ── Normal agentic loop ───────────────────────────────────────────────────
    let history = campaign::get_session_messages(pool, session_id).await
        .unwrap_or_default();

    let mut messages: Vec<ChatMessage> = history.iter()
        .filter_map(|m| {
            match m.role.as_str() {
                "user" => Some(ChatMessage::user(&m.content)),
                "assistant" => Some(ChatMessage::assistant(&m.content)),
                _ => None,
            }
        })
        .collect();

    let user_content = if let Some(roll) = &req.roll_result {
        format!("{} [Roll result: {} on {} = {}{}]",
            req.content,
            roll.skill.as_deref().unwrap_or("check"),
            roll.die,
            roll.result,
            roll.dc.map(|dc| format!(" vs DC {}", dc)).unwrap_or_default()
        )
    } else {
        req.content.clone()
    };

    let _ = campaign::save_message(pool, session_id, campaign_id, "user", &user_content, None).await;
    messages.push(ChatMessage::user(&user_content));

    let event_context = check_random_event(pool, campaign_id, &game_state).await;
    if let Some(event) = event_context {
        let augmented = format!("{}\n\n[WORLD EVENT - narrate naturally]: {}", user_content, event);
        if let Some(last) = messages.last_mut() {
            last.content = Some(augmented);
        }
    }

    let result = match state.llm.run_agentic_loop(
        pool,
        campaign_id,
        &system,
        messages,
        &game_state,
    ).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Agent loop error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})));
        }
    };

    // If declare_attack was called, send attack roll request
    if result.tool_calls_made.iter().any(|t| t.tool_name == "declare_attack") {
        if let Ok(Some(_)) = crate::db::combat::get_active_encounter(pool, campaign_id).await {
            let (clean_narrative, _) = strip_state_tag(&result.narrative);
            if !clean_narrative.is_empty() {
                let _ = campaign::save_message(pool, session_id, campaign_id, "assistant", &clean_narrative, None).await;
            }
            return (StatusCode::OK, Json(json!({
                "type": "roll_request",
                "roll": {
                    "tool_call_id": "attack",
                    "die": "d20",
                    "skill": "Attack",
                    "dc": 0,
                    "reason": "Roll to attack!"
                },
                "opening_narrative": clean_narrative
            })));
        }
    }

    // If start_combat was called, resolve any initial non-player turns
    let mut combat_turns: Vec<String> = vec![];
    if result.tool_calls_made.iter().any(|t| t.tool_name == "start_combat") {
        combat_turns = resolve_combat_turns(pool, campaign_id, &state.llm, &system).await;
        for t in &combat_turns {
            let _ = campaign::save_message(pool, session_id, campaign_id, "assistant", t, None).await;
        }
    }

    if let Some(roll_req) = result.roll_request {
        return (StatusCode::OK, Json(json!({
            "type": "roll_request",
            "roll": roll_req
        })));
    }

    let (clean_narrative, new_state) = strip_state_tag(&result.narrative);
    let _ = campaign::save_message(pool, session_id, campaign_id, "assistant", &clean_narrative, None).await;

    (StatusCode::OK, Json(json!({
        "type": "narrative",
        "content": clean_narrative,
        "new_state": new_state,
        "combat_turns": combat_turns,
        "tools_used": result.tool_calls_made.iter().map(|t| &t.tool_name).collect::<Vec<_>>()
    })))
}

// ─── Combat turn resolver ─────────────────────────────────────────────────────

async fn resolve_combat_turns(
    pool: &sqlx::SqlitePool,
    campaign_id: &str,
    llm: &crate::llm::LlmClient,
    system: &str,
) -> Vec<String> {
    let mut narratives = vec![];

    loop {
        let enc = match crate::db::combat::get_active_encounter(pool, campaign_id).await.ok().flatten() {
            Some(e) => e,
            None => break,
        };

        let turn_order: Vec<crate::db::combat::TurnParticipant> = enc.turn_order_json
            .as_deref()
            .and_then(|j| serde_json::from_str(j).ok())
            .unwrap_or_default();

        let current = match turn_order.get(enc.turn_index as usize) {
            Some(p) => p.clone(),
            None => break,
        };

        match current.participant_type.as_str() {
            "player" => break,
            "enemy" => {
                let p = match player::get_player_by_campaign(pool, campaign_id).await.ok().flatten() {
                    Some(p) => p,
                    None => break,
                };
                let result = crate::db::combat::resolve_enemy_attack(
                    pool, campaign_id, &p, &current.id
                ).await.unwrap_or(json!({"error": "enemy attack failed"}));

                let raw = llm.narrate_combat_result(system, &result).await
                    .unwrap_or_else(|_| format!("{} attacks.", current.name));
                let (narration, _) = strip_state_tag(&raw);
                narratives.push(narration);

                if result["player_downed"].as_bool().unwrap_or(false) {
                    break;
                }
            }
            "ally" => {
                let enc2 = match crate::db::combat::get_active_encounter(pool, campaign_id).await.ok().flatten() {
                    Some(e) => e,
                    None => break,
                };
                let result = crate::db::combat::resolve_ally_turn(pool, &enc2, &current.id).await
                    .unwrap_or(json!({"ally_acted": false}));
                if result["ally_acted"].as_bool().unwrap_or(false) {
                    let raw = llm.narrate_combat_result(system, &result).await
                        .unwrap_or_else(|_| format!("{} acts.", current.name));
                    let (narration, _) = strip_state_tag(&raw);
                    narratives.push(narration);
                }
            }
            _ => break,
        }

        let enc = match crate::db::combat::get_active_encounter(pool, campaign_id).await.ok().flatten() {
            Some(e) => e,
            None => break,
        };
        let alive: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM combat_enemies WHERE encounter_id = ? AND is_alive = 1"
        )
        .bind(&enc.id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        if alive == 0 {
            let _ = crate::db::combat::end_combat(pool, campaign_id, "victory", 100).await;
            break;
        }
    }

    narratives
}

async fn check_random_event(
    pool: &sqlx::SqlitePool,
    campaign_id: &str,
    game_state: &GameState,
) -> Option<String> {
    let trigger = match game_state {
        GameState::Exploration => "travel",
        GameState::Rest => "rest",
        _ => return None,
    };

    match crate::db::time::roll_random_event(pool, campaign_id, trigger).await {
        Ok(Some(event)) => Some(event.description),
        _ => None,
    }
}

async fn seed_starting_equipment(
    pool: &sqlx::SqlitePool,
    campaign_id: &str,
    player_id: &str,
    class: &str,
) {
    // Each entry: (name, description, item_type, damage_die, damage_type, weapon_range, base_ac, armor_type, slot)
    type ItemDef<'a> = (&'a str, &'a str, &'a str, Option<&'a str>, Option<&'a str>, Option<&'a str>, Option<i64>, Option<&'a str>, &'a str);

    let equipment: Vec<ItemDef> = match class {
        "Fighter" | "Paladin" => vec![
            ("Longsword", "A versatile sword with a long blade.", "weapon", Some("d8"), Some("slashing"), Some("melee"), None, None, "main_hand"),
            ("Chain Mail", "Heavy armor made of interlocking metal rings.", "armor", None, None, None, Some(16), Some("heavy"), "armor"),
        ],
        "Barbarian" => vec![
            ("Greataxe", "A massive two-handed axe.", "weapon", Some("d12"), Some("slashing"), Some("melee"), None, None, "main_hand"),
        ],
        "Ranger" => vec![
            ("Longsword", "A versatile sword with a long blade.", "weapon", Some("d8"), Some("slashing"), Some("melee"), None, None, "main_hand"),
            ("Leather Armor", "Light armor made of cured leather.", "armor", None, None, None, Some(11), Some("light"), "armor"),
        ],
        "Rogue" => vec![
            ("Shortsword", "A light, quick blade ideal for close quarters.", "weapon", Some("d6"), Some("piercing"), Some("melee"), None, None, "main_hand"),
            ("Leather Armor", "Light armor made of cured leather.", "armor", None, None, None, Some(11), Some("light"), "armor"),
        ],
        "Cleric" => vec![
            ("Mace", "A heavy bludgeoning weapon with a flanged head.", "weapon", Some("d6"), Some("bludgeoning"), Some("melee"), None, None, "main_hand"),
            ("Scale Mail", "Armor made of overlapping metal scales.", "armor", None, None, None, Some(14), Some("medium"), "armor"),
        ],
        "Bard" => vec![
            ("Rapier", "A slender thrusting sword favored by duelists.", "weapon", Some("d8"), Some("piercing"), Some("melee"), None, None, "main_hand"),
            ("Leather Armor", "Light armor made of cured leather.", "armor", None, None, None, Some(11), Some("light"), "armor"),
        ],
        "Warlock" | "Druid" => vec![
            ("Quarterstaff", "A sturdy wooden staff used as a weapon.", "weapon", Some("d6"), Some("bludgeoning"), Some("melee"), None, None, "main_hand"),
            ("Leather Armor", "Light armor made of cured leather.", "armor", None, None, None, Some(11), Some("light"), "armor"),
        ],
        "Wizard" | "Sorcerer" => vec![
            ("Quarterstaff", "A sturdy wooden staff used as a weapon.", "weapon", Some("d6"), Some("bludgeoning"), Some("melee"), None, None, "main_hand"),
        ],
        "Monk" => vec![
            ("Quarterstaff", "A sturdy wooden staff used as a weapon.", "weapon", Some("d6"), Some("bludgeoning"), Some("melee"), None, None, "main_hand"),
        ],
        _ => vec![
            ("Dagger", "A simple short blade.", "weapon", Some("d4"), Some("piercing"), Some("melee"), None, None, "main_hand"),
        ],
    };

    for (name, desc, item_type, damage_die, damage_type, weapon_range, base_ac, armor_type, slot) in equipment {
        let item_data = json!({
            "name": name,
            "description": desc,
            "item_type": item_type,
            "damage_die": damage_die,
            "damage_type": damage_type,
            "weapon_range": weapon_range,
            "base_ac": base_ac,
            "armor_type": armor_type,
            "rarity": "common"
        });

        let item = match items::create_item(pool, campaign_id, &item_data).await {
            Ok(i) => i,
            Err(e) => {
                tracing::warn!("Failed to create starting item {}: {}", name, e);
                continue;
            }
        };

        if let Err(e) = items::give_item(pool, &item.id, "player", player_id).await {
            tracing::warn!("Failed to give starting item {}: {}", name, e);
            continue;
        }

        if let Err(e) = items::equip_item(pool, &item.id, slot, player_id).await {
            tracing::warn!("Failed to equip starting item {}: {}", name, e);
        }
    }

    // Recalculate AC after equipping all starting gear
    if let Err(e) = items::recalculate_ac(pool, player_id).await {
        tracing::warn!("Failed to recalculate AC after seeding equipment: {}", e);
    }
}

async fn seed_class_abilities(pool: &sqlx::SqlitePool, campaign_id: &str, player_id: &str, class: &str) {
    let abilities: Vec<(&str, Option<&str>, i64, &str)> = match class {
        "Barbarian" => vec![
            ("Rage", Some("Enter a rage as a bonus action. Advantage on STR checks/saves, bonus damage, resistance to physical damage. Lasts 1 minute."), 2, "long_rest"),
            ("Unarmored Defense", Some("AC = 10 + DEX mod + CON mod when not wearing armor."), 1, "manual"),
        ],
        "Fighter" => vec![
            ("Second Wind", Some("Regain 1d10 + Fighter level HP as a bonus action."), 1, "short_rest"),
        ],
        "Rogue" => vec![
            ("Sneak Attack", Some("Deal extra 1d6 damage when you have advantage or an ally is adjacent to target."), 1, "per_turn"),
            ("Cunning Action", Some("Use bonus action to Dash, Disengage, or Hide."), 1, "per_turn"),
        ],
        "Wizard" | "Sorcerer" => vec![
            ("Spell Slots (1st)", Some("1st level spell slots"), 2, "long_rest"),
        ],
        "Cleric" | "Druid" => vec![
            ("Spell Slots (1st)", Some("1st level spell slots"), 2, "long_rest"),
            ("Channel Divinity", Some("Channel divine energy for effects based on your divine domain."), 1, "short_rest"),
        ],
        "Paladin" => vec![
            ("Lay on Hands", Some("Healing pool of hit points equal to 5 × paladin level."), 5, "long_rest"),
            ("Divine Smite", Some("Expend spell slots to deal extra radiant damage on hit."), 1, "per_turn"),
        ],
        "Ranger" => vec![
            ("Favored Enemy", Some("Advantage on Survival checks to track, and INT checks to recall info about favored enemy type."), 1, "manual"),
        ],
        "Monk" => vec![
            ("Ki", Some("Ki points for monk abilities like Flurry of Blows, Patient Defense, Step of the Wind."), 1, "short_rest"),
            ("Unarmored Defense", Some("AC = 10 + DEX mod + WIS mod when not wearing armor."), 1, "manual"),
        ],
        "Bard" => vec![
            ("Bardic Inspiration", Some("Grant a creature a d6 inspiration die as a bonus action."), 3, "short_rest"),
            ("Spell Slots (1st)", Some("1st level spell slots"), 2, "long_rest"),
        ],
        "Warlock" => vec![
            ("Spell Slots", Some("Warlock spell slots — recover on short rest."), 1, "short_rest"),
            ("Eldritch Blast", Some("Ranged spell attack dealing 1d10 force damage."), 1, "per_turn"),
        ],
        _ => vec![],
    };

    for (name, desc, uses, refresh) in abilities {
        let _ = world::create_ability(
            pool,
            campaign_id,
            "player",
            player_id,
            name,
            desc,
            uses,
            refresh,
        ).await;
    }

    let hit_die = crate::models::hit_die_for_class(class);
    let _ = world::create_ability(
        pool,
        campaign_id,
        "player",
        player_id,
        "Hit Dice",
        Some(&format!("Spend during short rest to recover HP. Roll d{} + CON mod per die spent.", hit_die)),
        1,
        "long_rest",
    ).await;
}