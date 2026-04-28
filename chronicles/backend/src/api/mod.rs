use axum::{
    extract::{Path, State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::db::{campaign, player, world, items, companions, time, fighter};
use crate::llm::{ChatMessage, prompt};
use crate::models::*;
use crate::AppState;

const MAX_CONTEXT_MESSAGES: usize = 50;
const SUMMARIZE_THRESHOLD: usize = 20;

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

    let campaign_name = req.name.clone()
        .unwrap_or_else(|| "MythWeaver Campaign".to_string());

    let camp = match campaign::create_campaign(pool, &campaign_name).await {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()}))),
    };

    let p = match player::create_player(pool, &camp.id, &req).await {
        Ok(p) => p,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()}))),
    };

    // Seed class abilities
    seed_class_abilities(pool, &camp.id, &p.id, &p.class).await;

    // Seed equipment from player's choice
    seed_starting_equipment(pool, &camp.id, &p.id, &p.class, &req.equipment_choice).await;

    // Seed class proficiencies
    if p.class == "Fighter" {
        if let Err(e) = fighter::seed_fighter_proficiencies(pool, &camp.id, &p.id).await {
            tracing::warn!("Failed to seed fighter proficiencies: {}", e);
        }
        let default_masteries = [
            ("longsword", "sap"),
            ("greatsword", "graze"),
            ("handaxe", "vex"),
        ];
        for (weapon, mastery) in &default_masteries {
            let _ = fighter::add_weapon_mastery(pool, &camp.id, &p.id, weapon, mastery).await;
        }
    }

    // Seed background proficiencies
    if let Err(e) = player::seed_background_proficiencies(
        pool, &camp.id, &p.id,
        &req.player_background_skill_1,
        &req.player_background_skill_2,
        &req.player_background_tool,
    ).await {
        tracing::warn!("Failed to seed background proficiencies: {}", e);
    }

    // Seed species abilities
    seed_species_abilities(
        pool, &camp.id, &p.id,
        &p.race, p.species_subtype.as_deref(),
        &p
    ).await;

    if let Err(e) = time::init_campaign_time(pool, &camp.id).await {
        tracing::warn!("Failed to init campaign time: {}", e);
    }

    let session = match campaign::create_session(pool, &camp.id).await {
        Ok(s) => s,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()}))),
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
    let proficiencies = fighter::get_proficiencies(pool, &p.id).await.unwrap_or_default();
    let weapon_masteries = fighter::get_weapon_masteries(pool, &p.id).await.unwrap_or_default();
    let known_maneuvers = if p.subclass.as_deref() == Some("Battle Master") {
        fighter::get_known_maneuvers(pool, &p.id).await.unwrap_or_default()
    } else { vec![] };
    let superiority_dice = if let Some(ref sc) = p.subclass {
        fighter::get_superiority_dice(pool, &p.id, sc).await.unwrap_or(None)
    } else { None };

    (StatusCode::OK, Json(json!({
        "player": p,
        "abilities": abilities,
        "items": all_items,
        "companions": active_companions,
        "time": camp_time,
        "proficiencies": proficiencies,
        "weapon_masteries": weapon_masteries,
        "known_maneuvers": known_maneuvers,
        "superiority_dice": superiority_dice,
    })))
}

pub async fn list_campaigns(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let pool = &state.pool;
    match campaign::list_campaigns(pool).await {
        Ok(campaigns) => {
            let mut result = vec![];
            for c in campaigns {
                let p = player::get_player_by_campaign(pool, &c.id).await.ok().flatten();
                let session = campaign::get_active_session(pool, &c.id).await.ok().flatten();
                result.push(json!({
                    "campaign": c,
                    "player": p,
                    "has_active_session": session.is_some(),
                    "session": session
                }));
            }
            (StatusCode::OK, Json(json!({"campaigns": result})))
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()}))),
    }
}

// ─── Level up ─────────────────────────────────────────────────────────────────

#[derive(Debug, serde::Deserialize)]
pub struct LevelUpRequest {
    pub subclass: Option<String>,
    pub asi_stat1: Option<String>,
    pub asi_stat2: Option<String>,
    pub new_maneuvers: Option<Vec<String>>,
    pub replaced_maneuver: Option<String>,
}

pub async fn level_up(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
    Json(req): Json<LevelUpRequest>,
) -> impl IntoResponse {
    let pool = &state.pool;

    let p = match player::get_player_by_campaign(pool, &campaign_id).await {
        Ok(Some(p)) => p,
        _ => return (StatusCode::NOT_FOUND, Json(json!({"error": "Player not found"}))),
    };

    if p.level >= 20 {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "Already at maximum level"})));
    }

    let threshold = Player::xp_threshold(p.level);
    if p.experience < threshold {
        return (StatusCode::BAD_REQUEST, Json(json!({
            "error": "Not enough XP to level up",
            "current_xp": p.experience,
            "required_xp": threshold
        })));
    }

    let result = match player::level_up_player(pool, &p.id, &p).await {
        Ok(r) => r,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()}))),
    };

    if let Some(ref subclass) = req.subclass {
        if let Err(e) = player::set_subclass(pool, &p.id, subclass).await {
            return (StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})));
        }
        if let Err(e) = fighter::seed_subclass(
            pool, &campaign_id, &p.id, subclass, result.new_level
        ).await {
            tracing::warn!("Failed to seed subclass {}: {}", subclass, e);
        }
    }

    if let Some(ref stat1) = req.asi_stat1 {
        let stat2 = req.asi_stat2.as_deref();
        if let Err(e) = player::apply_asi(pool, &p.id, stat1, stat2).await {
            return (StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})));
        }
        let _ = items::recalculate_ac(pool, &p.id).await;
    }

    if let Some(ref maneuvers) = req.new_maneuvers {
        for maneuver in maneuvers {
            let _ = fighter::add_maneuver(pool, &campaign_id, &p.id, maneuver).await;
        }
    }

    if let (Some(ref old_m), Some(ref new_maneuvers)) =
        (&req.replaced_maneuver, &req.new_maneuvers)
    {
        if let Some(new_m) = new_maneuvers.last() {
            let _ = fighter::replace_maneuver(pool, &p.id, old_m, new_m).await;
        }
    }

    let subclass_now = req.subclass.as_deref().or(p.subclass.as_deref());
    seed_level_up_abilities_direct(
        pool, &campaign_id, &p.id, &p.class, result.new_level, subclass_now
    ).await;

    let updated = match player::get_player_by_campaign(pool, &campaign_id).await {
        Ok(Some(p)) => p,
        _ => return (StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to fetch updated player"}))),
    };

    let abilities = world::get_abilities(pool, &updated.id, "player").await.unwrap_or_default();
    let all_items = items::get_player_items(pool, &updated.id).await.unwrap_or_default();
    let active_companions = companions::get_active_companions(pool, &campaign_id).await.unwrap_or_default();
    let camp_time = time::get_campaign_time(pool, &campaign_id).await.ok().flatten();
    let weapon_masteries = fighter::get_weapon_masteries(pool, &updated.id).await.unwrap_or_default();
    let known_maneuvers = fighter::get_known_maneuvers(pool, &updated.id).await.unwrap_or_default();
    let superiority_dice = if let Some(ref sc) = updated.subclass {
        fighter::get_superiority_dice(pool, &updated.id, sc).await.unwrap_or(None)
    } else { None };

    (StatusCode::OK, Json(json!({
        "player": updated,
        "abilities": abilities,
        "items": all_items,
        "companions": active_companions,
        "time": camp_time,
        "weapon_masteries": weapon_masteries,
        "known_maneuvers": known_maneuvers,
        "superiority_dice": superiority_dice,
        "level_up_result": result,
    })))
}

// ─── Session ──────────────────────────────────────────────────────────────────

pub async fn start_session(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
) -> impl IntoResponse {
    match campaign::create_session(&state.pool, &campaign_id).await {
        Ok(s) => (StatusCode::OK, Json(json!({"session": s}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()}))),
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
                    pool, &msgs[0].campaign_id,
                    "You are a D&D session summarizer. Write concise, narrative summaries.",
                    summary_messages,
                    &GameState::Exploration,
                ).await {
                    let _ = campaign::save_session_summary(
                        pool, &msgs[0].campaign_id, &session_id, &result.narrative,
                    ).await;
                }
            }
        }
    }

    match campaign::end_session(pool, &session_id).await {
        Ok(_) => (StatusCode::OK, Json(json!({"message": "Session ended and summarized"}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()}))),
    }
}

pub async fn get_session_messages(
    State(state): State<Arc<AppState>>,
    Path((_campaign_id, session_id)): Path<(String, String)>,
) -> impl IntoResponse {
    match campaign::get_session_messages(&state.pool, &session_id).await {
        Ok(msgs) => (StatusCode::OK, Json(json!({"messages": msgs}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()}))),
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
                    let is_crit = attack_result["is_crit"].as_bool().unwrap_or(false);
                    return (StatusCode::OK, Json(json!({
                        "type": "roll_request",
                        "roll": {
                            "tool_call_id": "damage",
                            "die": damage_die,
                            "skill": "Damage",
                            "dc": 0,
                            "reason": if is_crit {
                                format!("Critical Hit! Roll {} damage twice and add together.", damage_die)
                            } else {
                                format!("You hit! Roll {} damage.", damage_die)
                            }
                        },
                        "is_crit": is_crit,
                        "weapon_mastery": attack_result["weapon_mastery"],
                    })));
                }

                let raw = state.llm.narrate_combat_result(&system, &attack_result).await
                    .unwrap_or_else(|_| "Your attack misses.".to_string());
                let (narrative, _) = strip_state_tag(&raw);
                let _ = campaign::save_message(pool, session_id, campaign_id, "assistant", &narrative, None).await;
                let (combat_turns, player_downed) = resolve_combat_turns(
                    pool, campaign_id, &state.llm, &system
                ).await;
                for t in &combat_turns {
                    let _ = campaign::save_message(pool, session_id, campaign_id, "assistant", t, None).await;
                }
                return (StatusCode::OK, Json(json!({
                    "type": "narrative",
                    "content": narrative,
                    "combat_turns": combat_turns,
                    "player_downed": player_downed,
                    "new_state": if player_downed { "combat" } else { "" }
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

                if damage_result["can_attack_again"].as_bool().unwrap_or(false) {
                    return (StatusCode::OK, Json(json!({
                        "type": "narrative",
                        "content": narrative,
                        "combat_turns": [],
                        "can_attack_again": true,
                        "attacks_made": damage_result["attacks_made"],
                        "max_attacks": damage_result["max_attacks"],
                    })));
                }

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

                let (combat_turns, player_downed) = resolve_combat_turns(
                    pool, campaign_id, &state.llm, &system
                ).await;
                for t in &combat_turns {
                    let _ = campaign::save_message(pool, session_id, campaign_id, "assistant", t, None).await;
                }
                return (StatusCode::OK, Json(json!({
                    "type": "narrative",
                    "content": narrative,
                    "combat_turns": combat_turns,
                    "player_downed": player_downed,
                    "new_state": if player_downed { "combat" } else { "" }
                })));
            }
        }
    }

    // ── Sliding window ────────────────────────────────────────────────────────
    let all_history = campaign::get_session_messages(pool, session_id).await
        .unwrap_or_default();

    if all_history.len() > MAX_CONTEXT_MESSAGES {
        let overflow_end = all_history.len() - MAX_CONTEXT_MESSAGES;
        if overflow_end % SUMMARIZE_THRESHOLD == 0 {
            let overflow: Vec<Message> = all_history[..overflow_end]
                .iter()
                .filter(|m| m.role == "user" || m.role == "assistant")
                .cloned()
                .collect();

            if !overflow.is_empty() {
                let conversation = overflow.iter()
                    .map(|m| format!("[{}]: {}", m.role.to_uppercase(), m.content))
                    .collect::<Vec<_>>()
                    .join("\n\n");

                let capped = conversation[..conversation.len().min(6000)].to_string();
                let pool_clone = pool.clone();
                let campaign_id_clone = campaign_id.clone();
                let session_id_clone = session_id.clone();
                let llm_clone = state.llm.clone();

                tokio::spawn(async move {
                    let summary_prompt = format!(
                        "Summarize these D&D exchanges in 3-4 sentences, preserving all key events, decisions, NPC interactions, and outcomes:\n\n{}",
                        capped
                    );
                    let msgs = vec![ChatMessage::user(&summary_prompt)];
                    if let Ok(result) = llm_clone.run_agentic_loop(
                        &pool_clone, &campaign_id_clone,
                        "You are a concise D&D session summarizer.",
                        msgs, &GameState::Exploration,
                    ).await {
                        let _ = campaign::save_session_summary(
                            &pool_clone, &campaign_id_clone,
                            &session_id_clone, &result.narrative,
                        ).await;
                        tracing::info!("Mid-session summary saved ({} messages summarized)", overflow.len());
                    }
                });
            }
        }
    }

    let history_slice = if all_history.len() > MAX_CONTEXT_MESSAGES {
        &all_history[all_history.len() - MAX_CONTEXT_MESSAGES..]
    } else {
        &all_history[..]
    };

    let mut messages: Vec<ChatMessage> = history_slice.iter()
        .filter_map(|m| match m.role.as_str() {
            "user"      => Some(ChatMessage::user(&m.content)),
            "assistant" => Some(ChatMessage::assistant(&m.content)),
            _ => None,
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
        pool, campaign_id, &system, messages, &game_state,
    ).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Agent loop error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})));
        }
    };

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

    let mut combat_turns: Vec<String> = vec![];
    let mut player_downed = false;
    if result.tool_calls_made.iter().any(|t| t.tool_name == "start_combat") {
        let (turns, downed) = resolve_combat_turns(
            pool, campaign_id, &state.llm, &system
        ).await;
        combat_turns = turns;
        player_downed = downed;
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
        "player_downed": player_downed,
        "tools_used": result.tool_calls_made.iter().map(|t| &t.tool_name).collect::<Vec<_>>()
    })))
}

// ─── Combat turn resolver ─────────────────────────────────────────────────────

async fn resolve_combat_turns(
    pool: &sqlx::SqlitePool,
    campaign_id: &str,
    llm: &crate::llm::LlmClient,
    system: &str,
) -> (Vec<String>, bool) {
    let mut narratives = vec![];
    let mut player_downed = false;

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
                    player_downed = true;
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

    (narratives, player_downed)
}

async fn check_random_event(
    pool: &sqlx::SqlitePool,
    campaign_id: &str,
    game_state: &GameState,
) -> Option<String> {
    let trigger = match game_state {
        GameState::Exploration => "travel",
        GameState::Rest        => "rest",
        _ => return None,
    };
    match crate::db::time::roll_random_event(pool, campaign_id, trigger).await {
        Ok(Some(event)) => Some(event.description),
        _ => None,
    }
}

// ─── Species abilities ────────────────────────────────────────────────────────

async fn seed_species_abilities(
    pool: &sqlx::SqlitePool,
    campaign_id: &str,
    player_id: &str,
    race: &str,
    subtype: Option<&str>,
    player: &Player,
) {
    let prof_bonus = player.proficiency_bonus;

    match race {
        "Human" => {
            let _ = world::create_ability(pool, campaign_id, "player", player_id,
                "Resourceful",
                Some("Gain Heroic Inspiration whenever you finish a Long Rest."),
                1, "long_rest").await;
        }

        "Aasimar" => {
            let _ = world::create_ability(pool, campaign_id, "player", player_id,
                "Healing Hands",
                Some("As a Magic action, touch a creature and roll a number of d4s equal to your Proficiency Bonus. The creature regains that many HP."),
                1, "long_rest").await;
            let _ = world::create_ability(pool, campaign_id, "player", player_id,
                "Celestial Revelation",
                Some("At level 3, transform as a Bonus Action: Heavenly Wings (Fly Speed), Inner Radiance (radiant aura), or Necrotic Shroud (frighten nearby creatures). Lasts 1 minute."),
                1, "long_rest").await;
        }

        "Dragonborn" => {
            let (damage_type, breath_desc) = match subtype {
                Some("Black") | Some("Copper") => ("Acid", "15-foot cone or 30-foot line of acid"),
                Some("Blue") | Some("Bronze")  => ("Lightning", "15-foot cone or 30-foot line of lightning"),
                Some("Brass") | Some("Gold") | Some("Red") => ("Fire", "15-foot cone or 30-foot line of fire"),
                Some("Green") => ("Poison", "15-foot cone or 30-foot line of poison"),
                Some("Silver") | Some("White") => ("Cold", "15-foot cone or 30-foot line of cold"),
                _ => ("Fire", "15-foot cone or 30-foot line of fire"),
            };
            let _ = world::create_ability(pool, campaign_id, "player", player_id,
                "Breath Weapon",
                Some(&format!("Replace one attack with a {} exhalation. Targets make a DEX save (DC 8 + CON mod + Prof). On fail: 1d10 {} damage, half on success. Scales at levels 5/11/17.", breath_desc, damage_type)),
                prof_bonus, "long_rest").await;
            let _ = world::create_ability(pool, campaign_id, "player", player_id,
                "Draconic Flight",
                Some("At level 5: Bonus Action to sprout spectral wings. Gain Fly Speed equal to your Speed for 10 minutes."),
                1, "long_rest").await;
        }

        "Dwarf" => {
            let _ = world::create_ability(pool, campaign_id, "player", player_id,
                "Stonecunning",
                Some("Bonus Action: gain Tremorsense 60 ft for 10 minutes while on or touching stone."),
                prof_bonus, "long_rest").await;
        }

        "Elf" => {
            let spells = match subtype {
                Some("Drow")     => "Level 3: Faerie Fire. Level 5: Darkness.",
                Some("High Elf") => "Level 3: Detect Magic. Level 5: Misty Step.",
                Some("Wood Elf") => "Level 3: Longstrider. Level 5: Pass without Trace.",
                _ => "Lineage spells granted at levels 3 and 5.",
            };
            let _ = world::create_ability(pool, campaign_id, "player", player_id,
                "Elven Lineage Spells",
                Some(&format!("Innate spellcasting from your lineage. {}  Each can be cast once without a slot per Long Rest.", spells)),
                1, "long_rest").await;
        }

        "Gnome" => {
            match subtype {
                Some("Forest Gnome") => {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Speak with Animals",
                        Some("Innate spell. Cast without a slot a number of times equal to your Proficiency Bonus per Long Rest."),
                        prof_bonus, "long_rest").await;
                }
                Some("Rock Gnome") | _ => {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Clockwork Device",
                        Some("Spend 10 minutes to create a Tiny clockwork device using Prestidigitation. Can have up to 3 at once. Each lasts 8 hours."),
                        3, "manual").await;
                }
            }
        }

        "Goliath" => {
            let (name, desc) = match subtype {
                Some("Cloud Giant") => ("Cloud's Jaunt",
                    "Bonus Action: magically teleport up to 30 feet to an unoccupied space you can see."),
                Some("Fire Giant") => ("Fire's Burn",
                    "When you hit with an attack, deal an extra 1d10 Fire damage."),
                Some("Frost Giant") => ("Frost's Chill",
                    "When you hit with an attack, deal 1d6 Cold damage and reduce target's Speed by 10 ft until your next turn."),
                Some("Hill Giant") => ("Hill's Tumble",
                    "When you hit a Large or smaller creature, you can give it the Prone condition."),
                Some("Stone Giant") | _ => ("Stone's Endurance",
                    "Reaction when you take damage: roll 1d12 + CON modifier and reduce the damage by that amount."),
                Some("Storm Giant") => ("Storm's Thunder",
                    "Reaction when a creature within 60 ft damages you: deal 1d8 Thunder damage to that creature."),
            };
            let _ = world::create_ability(pool, campaign_id, "player", player_id,
                name, Some(desc), prof_bonus, "long_rest").await;
            let _ = world::create_ability(pool, campaign_id, "player", player_id,
                "Large Form",
                Some("At level 5: Bonus Action to grow to Large size for 10 minutes. Advantage on STR checks, Speed +10 ft."),
                1, "long_rest").await;
        }

        "Halfling" => {
            let _ = world::create_ability(pool, campaign_id, "player", player_id,
                "Luck",
                Some("When you roll a 1 on a d20 test, you can reroll and must use the new roll."),
                1, "manual").await;
        }

        "Orc" => {
            let _ = world::create_ability(pool, campaign_id, "player", player_id,
                "Adrenaline Rush",
                Some("Take the Dash action as a Bonus Action and gain Temporary HP equal to your Proficiency Bonus."),
                prof_bonus, "short_rest").await;
            let _ = world::create_ability(pool, campaign_id, "player", player_id,
                "Relentless Endurance",
                Some("When reduced to 0 HP but not killed outright, drop to 1 HP instead."),
                1, "long_rest").await;
        }

        "Tiefling" => {
            let (resistance, spells) = match subtype {
                Some("Abyssal")  => ("Poison",   "Level 3: Ray of Sickness. Level 5: Hold Person."),
                Some("Chthonic") => ("Necrotic",  "Level 3: False Life. Level 5: Ray of Enfeeblement."),
                _                => ("Fire",      "Level 3: Hellish Rebuke. Level 5: Darkness."),
            };
            let _ = world::create_ability(pool, campaign_id, "player", player_id,
                "Fiendish Legacy Spells",
                Some(&format!("Innate spellcasting. Resistance to {} damage. {}  Each can be cast once without a slot per Long Rest.", resistance, spells)),
                1, "long_rest").await;
        }

        _ => {}
    }
}

// ─── Starting equipment ───────────────────────────────────────────────────────

async fn seed_starting_equipment(
    pool: &sqlx::SqlitePool,
    campaign_id: &str,
    player_id: &str,
    class: &str,
    choice: &str,
) {
    // Option B (and C for Fighter) is always gold only
    let gold_only = match (class, choice) {
        ("Barbarian", "B") => Some(75),
        ("Bard",      "B") => Some(90),
        ("Cleric",    "B") => Some(110),
        ("Druid",     "B") => Some(50),
        ("Fighter",   "C") => Some(155),
        ("Monk",      "B") => Some(50),
        ("Paladin",   "B") => Some(150),
        ("Ranger",    "B") => Some(150),
        ("Rogue",     "B") => Some(100),
        ("Sorcerer",  "B") => Some(50),
        ("Warlock",   "B") => Some(100),
        ("Wizard",    "B") => Some(55),
        _ => None,
    };

    if let Some(gp) = gold_only {
        let _ = player::normalize_and_save_currency(pool, player_id, 0, gp, 0, 0).await;
        return;
    }

    // Option A (and B for Fighter) — actual items
    // (name, description, item_type, damage_die, damage_type, weapon_range, weapon_type, base_ac, armor_type, slot, quantity, starting_gp)
    type ItemDef<'a> = (&'a str, &'a str, &'a str, Option<&'a str>, Option<&'a str>, Option<&'a str>, Option<&'a str>, Option<i64>, Option<&'a str>, Option<&'a str>, i64);

    let (items, starting_gp): (Vec<ItemDef>, i64) = match (class, choice) {

        ("Barbarian", _) => (vec![
            ("Greataxe",   "A massive two-handed axe.",            "weapon",    Some("d12"), Some("slashing"),     Some("melee"),  Some("greataxe"), None,     None,    Some("main_hand"), 1),
            ("Handaxe",    "A light axe suitable for throwing.",   "weapon",    Some("d6"),  Some("slashing"),     Some("melee"),  Some("handaxe"),  None,     None,    None,              4),
            ("Explorer's Pack", "Bedroll, mess kit, tinderbox, 10 torches, 10 days rations, waterskin, 50ft rope.", "wondrous", None, None, None, None, None, None, None, 1),
        ], 15),

        ("Bard", _) => (vec![
            ("Leather Armor", "Light armor made of cured leather.",          "armor",  None,        None,             None,           None,             Some(11), Some("light"), Some("armor"),     1),
            ("Dagger",        "A simple short blade.",                        "weapon", Some("d4"),  Some("piercing"), Some("melee"),  Some("dagger"),   None,     None,          Some("main_hand"), 2),
            ("Musical Instrument", "A musical instrument of your choice.",   "wondrous", None,      None,             None,           None,             None,     None,          None,              1),
            ("Entertainer's Pack", "Backpack, bedroll, 2 costumes, 5 candles, 5 days rations, waterskin, disguise kit.", "wondrous", None, None, None, None, None, None, None, 1),
        ], 19),

        ("Cleric", _) => (vec![
            ("Chain Shirt",  "Medium armor of interlocking rings.",          "armor",  None,        None,             None,           None,             Some(13), Some("medium"), Some("armor"),    1),
            ("Shield",       "A wooden or metal shield.",                    "armor",  None,        None,             None,           None,             Some(2),  Some("shield"), Some("shield"),   1),
            ("Mace",         "A bludgeoning weapon with a flanged head.",    "weapon", Some("d6"),  Some("bludgeoning"), Some("melee"), Some("mace"),  None,     None,           Some("main_hand"), 1),
            ("Holy Symbol",  "A symbol of your deity.",                      "wondrous", None,      None,             None,           None,             None,     None,           None,             1),
            ("Priest's Pack", "Backpack, blanket, 10 candles, tinderbox, alms box, 2 blocks incense, censer, vestments, 2 days rations, waterskin.", "wondrous", None, None, None, None, None, None, None, 1),
        ], 7),

        ("Druid", _) => (vec![
            ("Leather Armor",   "Light armor made of cured leather.",        "armor",  None,        None,             None,           None,             Some(11), Some("light"),  Some("armor"),    1),
            ("Shield",          "A wooden shield.",                          "armor",  None,        None,             None,           None,             Some(2),  Some("shield"), Some("shield"),   1),
            ("Sickle",          "A curved blade used in harvesting.",        "weapon", Some("d4"),  Some("slashing"), Some("melee"),  Some("sickle"),   None,     None,           Some("main_hand"), 1),
            ("Druidic Focus",   "A quarterstaff serving as a druidic focus.", "weapon", Some("d6"), Some("bludgeoning"), Some("melee"), Some("quarterstaff"), None, None,        Some("off_hand"), 1),
            ("Explorer's Pack", "Bedroll, mess kit, tinderbox, 10 torches, 10 days rations, waterskin, 50ft rope.", "wondrous", None, None, None, None, None, None, None, 1),
            ("Herbalism Kit",   "Tools for identifying and using herbs.",    "wondrous", None,      None,             None,           None,             None,     None,           None,             1),
        ], 9),

        ("Fighter", "A") => (vec![
            ("Chain Mail",   "Heavy armor of interlocking rings.",           "armor",  None,        None,             None,           None,             Some(16), Some("heavy"), Some("armor"),    1),
            ("Greatsword",   "A massive two-handed sword.",                  "weapon", Some("2d6"), Some("slashing"), Some("melee"),  Some("greatsword"), None,   None,          Some("main_hand"), 1),
            ("Flail",        "A spiked ball on a chain.",                    "weapon", Some("d8"),  Some("bludgeoning"), Some("melee"), Some("flail"),  None,     None,          None,             1),
            ("Javelin",      "A light thrown spear.",                        "weapon", Some("d6"),  Some("piercing"), Some("melee"),  Some("javelin"),  None,     None,          None,             8),
            ("Dungeoneer's Pack", "Backpack, crowbar, hammer, 10 pitons, 10 torches, tinderbox, 10 days rations, waterskin, 50ft rope.", "wondrous", None, None, None, None, None, None, None, 1),
        ], 4),

        ("Fighter", "B") => (vec![
            ("Studded Leather", "Light armor with metal studs.",             "armor",  None,        None,             None,           None,             Some(12), Some("light"), Some("armor"),    1),
            ("Scimitar",       "A curved slashing sword.",                   "weapon", Some("d6"),  Some("slashing"), Some("melee"),  Some("scimitar"), None,     None,          Some("main_hand"), 1),
            ("Shortsword",     "A light thrusting blade.",                   "weapon", Some("d6"),  Some("piercing"), Some("melee"),  Some("shortsword"), None,   None,          Some("off_hand"), 1),
            ("Longbow",        "A powerful ranged weapon.",                  "weapon", Some("d8"),  Some("piercing"), Some("ranged"), Some("longbow"),  None,     None,          None,             1),
            ("Arrow",          "Ammunition for a bow.",                      "wondrous", None,      None,             None,           None,             None,     None,          None,             20),
            ("Quiver",         "A container for arrows.",                    "wondrous", None,      None,             None,           None,             None,     None,          None,             1),
            ("Dungeoneer's Pack", "Backpack, crowbar, hammer, 10 pitons, 10 torches, tinderbox, 10 days rations, waterskin, 50ft rope.", "wondrous", None, None, None, None, None, None, None, 1),
        ], 11),

        ("Monk", _) => (vec![
            ("Spear",          "A long thrusting weapon.",                   "weapon", Some("d6"),  Some("piercing"), Some("melee"),  Some("spear"),    None,     None,          Some("main_hand"), 1),
            ("Dagger",         "A simple short blade.",                      "weapon", Some("d4"),  Some("piercing"), Some("melee"),  Some("dagger"),   None,     None,          None,             5),
            ("Artisan's Tools or Musical Instrument", "Tools matching your background tool proficiency.", "wondrous", None, None, None, None, None, None, None, 1),
            ("Explorer's Pack", "Bedroll, mess kit, tinderbox, 10 torches, 10 days rations, waterskin, 50ft rope.", "wondrous", None, None, None, None, None, None, None, 1),
        ], 11),

        ("Paladin", _) => (vec![
            ("Chain Mail",     "Heavy armor of interlocking rings.",         "armor",  None,        None,             None,           None,             Some(16), Some("heavy"),  Some("armor"),    1),
            ("Shield",         "A wooden or metal shield.",                  "armor",  None,        None,             None,           None,             Some(2),  Some("shield"), Some("shield"),   1),
            ("Longsword",      "A versatile sword.",                         "weapon", Some("d8"),  Some("slashing"), Some("melee"),  Some("longsword"), None,    None,           Some("main_hand"), 1),
            ("Javelin",        "A light thrown spear.",                      "weapon", Some("d6"),  Some("piercing"), Some("melee"),  Some("javelin"),  None,     None,           None,             6),
            ("Holy Symbol",    "A symbol of your deity.",                    "wondrous", None,      None,             None,           None,             None,     None,           None,             1),
            ("Priest's Pack",  "Backpack, blanket, 10 candles, tinderbox, alms box, 2 blocks incense, censer, vestments, 2 days rations, waterskin.", "wondrous", None, None, None, None, None, None, None, 1),
        ], 9),

        ("Ranger", _) => (vec![
            ("Studded Leather", "Light armor with metal studs.",             "armor",  None,        None,             None,           None,             Some(12), Some("light"),  Some("armor"),    1),
            ("Scimitar",        "A curved slashing sword.",                  "weapon", Some("d6"),  Some("slashing"), Some("melee"),  Some("scimitar"), None,     None,           Some("main_hand"), 1),
            ("Shortsword",      "A light thrusting blade.",                  "weapon", Some("d6"),  Some("piercing"), Some("melee"),  Some("shortsword"), None,   None,           Some("off_hand"), 1),
            ("Longbow",         "A powerful ranged weapon.",                 "weapon", Some("d8"),  Some("piercing"), Some("ranged"), Some("longbow"),  None,     None,           None,             1),
            ("Arrow",           "Ammunition for a bow.",                     "wondrous", None,      None,             None,           None,             None,     None,           None,             20),
            ("Quiver",          "A container for arrows.",                   "wondrous", None,      None,             None,           None,             None,     None,           None,             1),
            ("Druidic Focus",   "A sprig of mistletoe serving as a druidic focus.", "wondrous", None, None,          None,           None,             None,     None,           None,             1),
            ("Explorer's Pack", "Bedroll, mess kit, tinderbox, 10 torches, 10 days rations, waterskin, 50ft rope.", "wondrous", None, None, None, None, None, None, None, 1),
        ], 7),

        ("Rogue", _) => (vec![
            ("Leather Armor",   "Light armor made of cured leather.",        "armor",  None,        None,             None,           None,             Some(11), Some("light"),  Some("armor"),    1),
            ("Dagger",          "A simple short blade.",                     "weapon", Some("d4"),  Some("piercing"), Some("melee"),  Some("dagger"),   None,     None,           Some("main_hand"), 2),
            ("Shortsword",      "A light thrusting blade.",                  "weapon", Some("d6"),  Some("piercing"), Some("melee"),  Some("shortsword"), None,   None,           Some("off_hand"), 1),
            ("Shortbow",        "A compact ranged weapon.",                  "weapon", Some("d6"),  Some("piercing"), Some("ranged"), Some("shortbow"), None,     None,           None,             1),
            ("Arrow",           "Ammunition for a bow.",                     "wondrous", None,      None,             None,           None,             None,     None,           None,             20),
            ("Quiver",          "A container for arrows.",                   "wondrous", None,      None,             None,           None,             None,     None,           None,             1),
            ("Thieves' Tools",  "Tools for picking locks and disarming traps.", "wondrous", None,   None,             None,           None,             None,     None,           None,             1),
            ("Burglar's Pack",  "Backpack, 1000 ball bearings, 10ft string, bell, 5 candles, crowbar, hammer, 10 pitons, hooded lantern, 2 oil flasks, 5 days rations, tinderbox, waterskin.", "wondrous", None, None, None, None, None, None, None, 1),
        ], 8),

        ("Sorcerer", _) => (vec![
            ("Spear",           "A long thrusting weapon.",                  "weapon", Some("d6"),  Some("piercing"), Some("melee"),  Some("spear"),    None,     None,           Some("main_hand"), 1),
            ("Dagger",          "A simple short blade.",                     "weapon", Some("d4"),  Some("piercing"), Some("melee"),  Some("dagger"),   None,     None,           None,             2),
            ("Arcane Focus",    "A crystal serving as an arcane focus.",     "wondrous", None,      None,             None,           None,             None,     None,           None,             1),
            ("Dungeoneer's Pack", "Backpack, crowbar, hammer, 10 pitons, 10 torches, tinderbox, 10 days rations, waterskin, 50ft rope.", "wondrous", None, None, None, None, None, None, None, 1),
        ], 28),

        ("Warlock", _) => (vec![
            ("Leather Armor",   "Light armor made of cured leather.",        "armor",  None,        None,             None,           None,             Some(11), Some("light"),  Some("armor"),    1),
            ("Sickle",          "A curved blade.",                           "weapon", Some("d4"),  Some("slashing"), Some("melee"),  Some("sickle"),   None,     None,           Some("main_hand"), 1),
            ("Dagger",          "A simple short blade.",                     "weapon", Some("d4"),  Some("piercing"), Some("melee"),  Some("dagger"),   None,     None,           None,             2),
            ("Arcane Focus",    "An orb serving as an arcane focus.",        "wondrous", None,      None,             None,           None,             None,     None,           None,             1),
            ("Book of Occult Lore", "A book of occult knowledge.",           "wondrous", None,      None,             None,           None,             None,     None,           None,             1),
            ("Scholar's Pack",  "Backpack, book, ink, ink pen, 10 parchment sheets, a little bag of sand, small knife.", "wondrous", None, None, None, None, None, None, None, 1),
        ], 15),

        ("Wizard", _) => (vec![
            ("Dagger",          "A simple short blade.",                     "weapon", Some("d4"),  Some("piercing"), Some("melee"),  Some("dagger"),   None,     None,           Some("main_hand"), 2),
            ("Arcane Focus",    "A quarterstaff serving as an arcane focus.", "weapon", Some("d6"), Some("bludgeoning"), Some("melee"), Some("quarterstaff"), None, None,         None,             1),
            ("Robe",            "A comfortable robe.",                       "wondrous", None,      None,             None,           None,             None,     None,           None,             1),
            ("Spellbook",       "A book containing your wizard spells.",     "wondrous", None,      None,             None,           None,             None,     None,           None,             1),
            ("Scholar's Pack",  "Backpack, book, ink, ink pen, 10 parchment sheets, a little bag of sand, small knife.", "wondrous", None, None, None, None, None, None, None, 1),
        ], 5),

        _ => (vec![
            ("Dagger", "A simple short blade.", "weapon", Some("d4"), Some("piercing"), Some("melee"), Some("dagger"), None, None, Some("main_hand"), 1),
        ], 10),
    };

    // Seed the items
    for (name, desc, item_type, damage_die, damage_type, weapon_range, weapon_type, base_ac, armor_type, slot, qty) in &items {
        let item_data = json!({
            "name": name,
            "description": desc,
            "item_type": item_type,
            "damage_die": damage_die,
            "damage_type": damage_type,
            "weapon_range": weapon_range,
            "weapon_type": weapon_type,
            "base_ac": base_ac,
            "armor_type": armor_type,
            "rarity": "common",
            "quantity": qty,
        });

        let item = match items::create_item(pool, campaign_id, &item_data).await {
            Ok(i) => i,
            Err(e) => { tracing::warn!("Failed to create item {}: {}", name, e); continue; }
        };
        if let Err(e) = items::give_item(pool, &item.id, "player", player_id).await {
            tracing::warn!("Failed to give item {}: {}", name, e); continue;
        }
        if let Some(s) = slot {
            if let Err(e) = items::equip_item(pool, &item.id, s, player_id).await {
                tracing::warn!("Failed to equip item {}: {}", name, e);
            }
        }
    }

    // Seed starting gold
    if starting_gp > 0 {
        let _ = player::normalize_and_save_currency(pool, player_id, 0, starting_gp, 0, 0).await;
    }

    if let Err(e) = items::recalculate_ac(pool, player_id).await {
        tracing::warn!("Failed to recalculate AC: {}", e);
    }
}

// ─── Class abilities ──────────────────────────────────────────────────────────

async fn seed_class_abilities(
    pool: &sqlx::SqlitePool,
    campaign_id: &str,
    player_id: &str,
    class: &str,
) {
    let abilities: Vec<(&str, Option<&str>, i64, &str)> = match class {
        "Barbarian" => vec![
            ("Rage", Some("Bonus Action: enter a rage. Advantage on STR checks/saves, +2 damage on STR attacks, resistance to physical damage. Lasts 1 minute."), 2, "long_rest"),
            ("Unarmored Defense", Some("AC = 10 + DEX mod + CON mod when not wearing armor."), 1, "manual"),
        ],
        "Fighter" => vec![
            ("Second Wind", Some("Bonus Action: regain 1d10 + Fighter level HP. Also usable for Tactical Mind (level 2+)."), 2, "short_rest"),
        ],
        "Rogue" => vec![
            ("Sneak Attack", Some("Deal extra 1d6 damage when you have advantage or an ally is adjacent to the target."), 1, "per_turn"),
            ("Cunning Action", Some("Bonus Action: Dash, Disengage, or Hide."), 1, "per_turn"),
        ],
        "Wizard" | "Sorcerer" => vec![
            ("Spell Slots (1st)", Some("1st level spell slots."), 2, "long_rest"),
        ],
        "Cleric" | "Druid" => vec![
            ("Spell Slots (1st)", Some("1st level spell slots."), 2, "long_rest"),
            ("Channel Divinity", Some("Channel divine energy for effects based on your domain."), 1, "short_rest"),
        ],
        "Paladin" => vec![
            ("Lay on Hands", Some("Healing pool equal to 5 × paladin level. Use to restore HP or cure disease/poison."), 5, "long_rest"),
            ("Divine Smite", Some("Expend a spell slot on a hit to deal extra radiant damage."), 1, "per_turn"),
        ],
        "Ranger" => vec![
            ("Favored Enemy", Some("Advantage on Survival to track, INT checks to recall info about favored enemy type."), 1, "manual"),
        ],
        "Monk" => vec![
            ("Ki", Some("Ki points for Flurry of Blows, Patient Defense, Step of the Wind, and other monk features."), 1, "short_rest"),
            ("Unarmored Defense", Some("AC = 10 + DEX mod + WIS mod when not wearing armor or a shield."), 1, "manual"),
        ],
        "Bard" => vec![
            ("Bardic Inspiration", Some("Bonus Action: grant a creature a d6 inspiration die to add to one ability check, attack roll, or saving throw."), 3, "short_rest"),
            ("Spell Slots (1st)", Some("1st level spell slots."), 2, "long_rest"),
        ],
        "Warlock" => vec![
            ("Spell Slots", Some("Warlock spell slots — recover on short rest."), 1, "short_rest"),
            ("Eldritch Blast", Some("Ranged spell attack dealing 1d10 force damage."), 1, "per_turn"),
        ],
        _ => vec![],
    };

    for (name, desc, uses, refresh) in abilities {
        let _ = world::create_ability(
            pool, campaign_id, "player", player_id, name, desc, uses, refresh
        ).await;
    }

    let hit_die = hit_die_for_class(class);
    let _ = world::create_ability(
        pool, campaign_id, "player", player_id,
        "Hit Dice",
        Some(&format!("Spend during short rest to recover HP. Roll d{} + CON mod per die spent.", hit_die)),
        1, "long_rest",
    ).await;
}

// ─── Level up ability seeding ─────────────────────────────────────────────────

async fn seed_level_up_abilities_direct(
    pool: &sqlx::SqlitePool,
    campaign_id: &str,
    player_id: &str,
    class: &str,
    new_level: i64,
    subclass: Option<&str>,
) {
    if class != "Fighter" { return; }

    let existing = world::get_abilities(pool, player_id, "player").await.unwrap_or_default();
    let has = |name: &str| existing.iter().any(|a| a.name == name);

    match new_level {
        2 => {
            if !has("Action Surge") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Action Surge",
                    Some("Take one additional action on your turn (not Magic action). Recharges on short or long rest."),
                    1, "short_rest").await;
            }
            if !has("Tactical Mind") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Tactical Mind",
                    Some("When you fail an ability check, spend a Second Wind use to roll 1d10 and add to the check."),
                    1, "manual").await;
            }
        }
        5 => {
            if !has("Tactical Shift") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Tactical Shift",
                    Some("When you use Second Wind, move up to half your Speed without provoking Opportunity Attacks."),
                    1, "manual").await;
            }
        }
        9 => {
            if !has("Indomitable") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Indomitable",
                    Some("When you fail a saving throw, reroll it with a bonus equal to your Fighter level."),
                    1, "long_rest").await;
            }
            if !has("Tactical Master") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Tactical Master",
                    Some("When attacking with a mastered weapon, replace its mastery property with Push, Sap, or Slow."),
                    1, "manual").await;
            }
        }
        13 => {
            if !has("Studied Attacks") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Studied Attacks",
                    Some("If you miss an attack against a creature, you have Advantage on your next attack against it before end of your next turn."),
                    1, "manual").await;
            }
        }
        _ => {}
    }

    match subclass {
        Some("Champion") => match new_level {
            7  => { if !has("Additional Fighting Style") { let _ = world::create_ability(pool, campaign_id, "player", player_id, "Additional Fighting Style", Some("Gain another Fighting Style feat."), 1, "manual").await; } }
            10 => { if !has("Heroic Warrior") { let _ = world::create_ability(pool, campaign_id, "player", player_id, "Heroic Warrior", Some("Give yourself Heroic Inspiration at start of your turn if you don't have it."), 1, "per_turn").await; } }
            18 => { if !has("Survivor: Heroic Rally") { let _ = world::create_ability(pool, campaign_id, "player", player_id, "Survivor: Heroic Rally", Some("Regain 5 + CON modifier HP at start of each turn if Bloodied with at least 1 HP."), 1, "per_turn").await; } }
            _ => {}
        },
        Some("Battle Master") => match new_level {
            7  => { if !has("Know Your Enemy") { let _ = world::create_ability(pool, campaign_id, "player", player_id, "Know Your Enemy", Some("Learn a creature's Immunities, Resistances, and Vulnerabilities within 30 feet."), 1, "long_rest").await; } }
            15 => { if !has("Relentless") { let _ = world::create_ability(pool, campaign_id, "player", player_id, "Relentless", Some("Once per turn when you use a maneuver, roll 1d8 instead of expending a Superiority Die."), 1, "per_turn").await; } }
            _ => {}
        },
        Some("Psi Warrior") => match new_level {
            7  => { if !has("Psi-Powered Leap") { let _ = world::create_ability(pool, campaign_id, "player", player_id, "Psi-Powered Leap", Some("Gain Fly Speed equal to twice your Speed until end of turn."), 1, "short_rest").await; } }
            10 => { if !has("Guarded Mind") { let _ = world::create_ability(pool, campaign_id, "player", player_id, "Guarded Mind", Some("Resistance to Psychic damage. Spend a Psionic Energy Die to end Charmed or Frightened."), 1, "manual").await; } }
            15 => { if !has("Bulwark of Force") { let _ = world::create_ability(pool, campaign_id, "player", player_id, "Bulwark of Force", Some("Grant Half Cover to up to INT modifier creatures within 30 feet for 1 minute."), 1, "long_rest").await; } }
            18 => { if !has("Telekinetic Master") { let _ = world::create_ability(pool, campaign_id, "player", player_id, "Telekinetic Master", Some("Telekinesis always prepared, cast without spell slot. Make one weapon attack as Bonus Action each turn while concentrating."), 1, "long_rest").await; } }
            _ => {}
        },
        _ => {}
    }
}