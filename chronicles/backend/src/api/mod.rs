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
//const SUMMARIZE_THRESHOLD: usize = 20;
const JOURNAL_UPDATE_THRESHOLD: usize = 20;

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

    seed_class_abilities(pool, &camp.id, &p.id, &p.class).await;
    seed_starting_equipment(pool, &camp.id, &p.id, &p.class, &req.equipment_choice).await;

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

    if let Err(e) = player::seed_background_proficiencies(
        pool, &camp.id, &p.id,
        &req.player_background_skill_1,
        &req.player_background_skill_2,
        &req.player_background_tool,
    ).await {
        tracing::warn!("Failed to seed background proficiencies: {}", e);
    }

    seed_species_abilities(pool, &camp.id, &p.id, &p.race, p.species_subtype.as_deref(), &p).await;

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

pub async fn delete_campaign(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
) -> impl IntoResponse {
    let pool = &state.pool;
    let tables = [
        "messages", "sessions", "session_summaries",
        "abilities", "items", "companions", "proficiencies",
        "weapon_mastery", "known_maneuvers", "superiority_dice",
        "active_effects", "combat_encounters", "combat_enemies",
        "locations", "location_connections", "npcs", "world_facts",
        "event_tables", "event_entries", "campaign_time", "players",
        "campaigns"
    ];
    for table in &tables {
        let _ = sqlx::query(&format!("DELETE FROM {} WHERE campaign_id = ?", table))
            .bind(&campaign_id)
            .execute(pool)
            .await;
    }
    (StatusCode::OK, Json(json!({"message": "Campaign deleted"})))
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
    /* Not currently being used in this version
    let summaries = campaign::get_session_summaries(pool, campaign_id).await
        .unwrap_or_default()
        .into_iter()
        .map(|s| s.summary)
        .collect::<Vec<_>>();
    */    

    let game_state = req.game_state.as_deref()
        .map(GameState::from_str)
        .unwrap_or(GameState::Exploration);

    let story_journal = campaign::get_story_journal(pool, campaign_id)
        .await.ok().flatten();

    let system = prompt::build_system_prompt(&p, camp_time.as_ref(), &[], story_journal.as_deref());


    
    // ── Sliding window ────────────────────────────────────────────────────────
    let all_history = campaign::get_session_messages(pool, session_id).await
        .unwrap_or_default();
    
    /* - Old Summary Task spawner -- Being replaced by World Journal below
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
    */

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

    let is_ephemeral = user_content.starts_with("[COMBAT RESOLVED");
    if !is_ephemeral {
        let _ = campaign::save_message(pool, session_id, campaign_id, "user", &user_content, None).await;
    }

    messages.push(ChatMessage::user(&user_content));

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

    // If start_combat was called, signal the frontend to begin the initiative flow
    let needs_initiative = result.tool_calls_made.iter()
        .any(|t| t.tool_name == "start_combat");

    let needs_shop = result.tool_calls_made.iter()
        .any(|t| t.tool_name == "open_shop");

    if let Some(roll_req) = result.roll_request {
        return (StatusCode::OK, Json(json!({
            "type": "roll_request",
            "roll": roll_req
        })));
    }

    let (clean_narrative, new_state) = strip_state_tag(&result.narrative);
    let _ = campaign::save_message(pool, session_id, campaign_id, "assistant", &clean_narrative, None).await;


    // ── Journal update ────────────────────────────────────────────────────────
    let total_messages = all_history.len() + 2;
    if total_messages > 0 && total_messages % JOURNAL_UPDATE_THRESHOLD == 0 {
        let pool_clone = pool.clone();
        let campaign_id_clone = campaign_id.clone();
        let llm_clone = state.llm.clone();

        tokio::spawn(async move {
            let current_journal = campaign::get_story_journal(&pool_clone, &campaign_id_clone)
                .await.ok().flatten();

            let recent = campaign::get_recent_messages(&pool_clone, &campaign_id_clone, 30)
                .await.unwrap_or_default();

            let conversation = recent.iter()
                .filter(|m| m.role == "user" || m.role == "assistant")
                .map(|m| format!("[{}]: {}", m.role.to_uppercase(), m.content))
                .collect::<Vec<_>>()
                .join("\n\n");

            let prompt = format!(
                "You are maintaining a World Story Journal for an ongoing D&D campaign.\n\n\
                CURRENT JOURNAL:\n{}\n\n\
                RECENT EVENTS:\n{}\n\n\
                Update the journal to reflect significant developments from the recent events. \
                Track: current situation and location, active quests and unresolved threads, \
                key NPC relationships and current status, important world state changes, \
                faction dynamics, secrets revealed, and anything critical the DM must remember. \
                Prune resolved or stale information. Keep it under 800 words. \
                Return only the updated journal text, no preamble, no commentary.",
                current_journal.as_deref().unwrap_or("(empty — this is a new campaign)"),
                conversation
            );

            let msgs = vec![ChatMessage::user(&prompt)];
            if let Ok(result) = llm_clone.run_agentic_loop(
                &pool_clone, &campaign_id_clone,
                "You are a concise narrative archivist for a D&D campaign. \
                Return only the updated journal text.",
                msgs,
                &GameState::Exploration,
            ).await {
                let _ = campaign::update_story_journal(
                    &pool_clone, &campaign_id_clone, &result.narrative
                ).await;
                tracing::info!("Story journal updated at {} messages", all_history.len());
            }
        });
    }

    (StatusCode::OK, Json(json!({
        "type": "narrative",
        "content": clean_narrative,
        "new_state": new_state,
        "needs_initiative": needs_initiative,
        "needs_shop": needs_shop,
        "tools_used": result.tool_calls_made.iter().map(|t| &t.tool_name).collect::<Vec<_>>()
    })))
}

// ─── Combat handlers ──────────────────────────────────────────────────────────

pub async fn get_combat_state_handler(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
) -> impl IntoResponse {
    match crate::db::combat::get_combat_state(&state.pool, &campaign_id).await {
        Ok(Some(combat)) => (StatusCode::OK, Json(combat)),
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({"error": "No active combat"}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct SubmitInitiativeRequest {
    pub roll: i64,
    pub advantage_rolls: Option<Vec<i64>>,
}

pub async fn submit_initiative(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
    Json(req): Json<SubmitInitiativeRequest>,
) -> impl IntoResponse {
    let pool = &state.pool;
    let p = match player::get_player_by_campaign(pool, &campaign_id).await {
        Ok(Some(p)) => p,
        _ => return (StatusCode::NOT_FOUND, Json(json!({"error": "Player not found"}))),
    };
    match crate::db::combat::submit_player_initiative(
        pool, &campaign_id, &p, req.roll, req.advantage_rolls
    ).await {
        Ok(result) => (StatusCode::OK, Json(result)),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct SetTargetRequest {
    pub target_id: String,
}

pub async fn set_combat_target(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
    Json(req): Json<SetTargetRequest>,
) -> impl IntoResponse {
    match crate::db::combat::set_combat_target(&state.pool, &campaign_id, &req.target_id).await {
        Ok(_) => (StatusCode::OK, Json(json!({"message": "Target set", "target_id": req.target_id}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct ResolveAttackRequest {
    pub roll: i64,
    pub advantage_rolls: Option<Vec<i64>>,
    pub target_id: String,
}

pub async fn resolve_attack(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
    Json(req): Json<ResolveAttackRequest>,
) -> impl IntoResponse {
    let pool = &state.pool;
    let p = match player::get_player_by_campaign(pool, &campaign_id).await {
        Ok(Some(p)) => p,
        _ => return (StatusCode::NOT_FOUND, Json(json!({"error": "Player not found"}))),
    };
    if let Err(e) = crate::db::combat::set_combat_target(pool, &campaign_id, &req.target_id).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})));
    }
    match crate::db::combat::resolve_player_attack(
        pool, &campaign_id, &p, req.roll, &req.target_id
    ).await {
        Ok(result) => (StatusCode::OK, Json(result)),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct ResolveDamageRequest {
    pub rolls: Vec<i64>,
    pub is_crit: bool,
}

pub async fn resolve_damage(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
    Json(req): Json<ResolveDamageRequest>,
) -> impl IntoResponse {
    let pool = &state.pool;
    let p = match player::get_player_by_campaign(pool, &campaign_id).await {
        Ok(Some(p)) => p,
        _ => return (StatusCode::NOT_FOUND, Json(json!({"error": "Player not found"}))),
    };
    let dice_total: i64 = req.rolls.iter().sum();
    let damage_roll = if req.is_crit { dice_total * 2 } else { dice_total };
    match crate::db::combat::apply_player_damage(pool, &campaign_id, &p, damage_roll).await {
        Ok(result) => (StatusCode::OK, Json(result)),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct UseCombatAbilityRequest {
    pub ability_type: String,
    pub target_id: Option<String>,
    pub roll: Option<i64>,
    pub maneuver_name: Option<String>,
}

pub async fn use_combat_ability(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
    Json(req): Json<UseCombatAbilityRequest>,
) -> impl IntoResponse {
    let pool = &state.pool;
    let p = match player::get_player_by_campaign(pool, &campaign_id).await {
        Ok(Some(p)) => p,
        _ => return (StatusCode::NOT_FOUND, Json(json!({"error": "Player not found"}))),
    };

    let result = match req.ability_type.as_str() {
        "second_wind" => {
            crate::db::combat::use_second_wind(pool, &campaign_id, &p).await
        }
        "action_surge" => {
            let enc = match crate::db::combat::get_active_encounter(pool, &campaign_id).await {
                Ok(Some(e)) => e,
                _ => return (StatusCode::NOT_FOUND, Json(json!({"error": "No active combat"}))),
            };
            let used = crate::db::fighter::use_action_surge(pool, &enc.id).await
                .unwrap_or(false);
            if used {
                let _ = sqlx::query(
                    "UPDATE abilities SET current_uses = current_uses - 1
                     WHERE owner_id = ? AND name = 'Action Surge' AND current_uses > 0"
                )
                .bind(&p.id)
                .execute(pool)
                .await;
                Ok(json!({"message": "Action Surge activated", "action_surge_used": true}))
            } else {
                Ok(json!({"error": "Action Surge not available"}))
            }
        }
        "indomitable" => {
            let original_roll = req.roll.unwrap_or(0);
            crate::db::combat::use_indomitable(pool, &p, original_roll).await
        }
        "maneuver" => {
            let maneuver_name = match req.maneuver_name.as_deref() {
                Some(m) => m,
                None => return (StatusCode::BAD_REQUEST,
                    Json(json!({"error": "maneuver_name required"}))),
            };
            let superiority_roll = req.roll.unwrap_or(0);
            crate::db::combat::resolve_maneuver(
                pool, &campaign_id, &p,
                maneuver_name,
                req.target_id.as_deref(),
                superiority_roll,
            ).await
        }
        "psionic_strike" => {
            crate::db::combat::use_psionic_strike(
                pool, &campaign_id, &p, req.roll.unwrap_or(0)
            ).await
        }
        "protective_field" => {
            crate::db::combat::use_protective_field(
                pool, &p, req.roll.unwrap_or(0)
            ).await
        }
        _ => Ok(json!({"error": format!("Unknown ability type: {}", req.ability_type)}))
    };

    match result {
        Ok(r) => (StatusCode::OK, Json(r)),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    }
}

pub async fn end_combat_turn(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
) -> impl IntoResponse {
    let pool = &state.pool;
    let p = match player::get_player_by_campaign(pool, &campaign_id).await {
        Ok(Some(p)) => p,
        _ => return (StatusCode::NOT_FOUND, Json(json!({"error": "Player not found"}))),
    };
    match crate::db::combat::end_player_turn(pool, &campaign_id, &p).await {
        Ok(results) => {
            let updated_player = player::get_player_by_campaign(pool, &campaign_id)
                .await.ok().flatten();
            let combat_state = crate::db::combat::get_combat_state(pool, &campaign_id)
                .await.ok().flatten();
            (StatusCode::OK, Json(json!({
                "turn_results": results,
                "player": updated_player,
                "combat_state": combat_state,
            })))
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    }
}

pub async fn process_initial_turns(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
) -> impl IntoResponse {
    let pool = &state.pool;
    let p = match player::get_player_by_campaign(pool, &campaign_id).await {
        Ok(Some(p)) => p,
        _ => return (StatusCode::NOT_FOUND, Json(json!({"error": "Player not found"}))),
    };
    match crate::db::combat::process_initial_turns(pool, &campaign_id, &p).await {
        Ok(results) => {
            let updated_player = player::get_player_by_campaign(pool, &campaign_id).await.ok().flatten();
            let combat_state = crate::db::combat::get_combat_state(pool, &campaign_id).await.ok().flatten();
            (StatusCode::OK, Json(json!({
                "turn_results": results,
                "player": updated_player,
                "combat_state": combat_state,
            })))
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct FleeRequest {
    pub roll: i64,
    pub skill: String,
}

pub async fn flee_combat(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
    Json(req): Json<FleeRequest>,
) -> impl IntoResponse {
    let pool = &state.pool;
    let p = match player::get_player_by_campaign(pool, &campaign_id).await {
        Ok(Some(p)) => p,
        _ => return (StatusCode::NOT_FOUND, Json(json!({"error": "Player not found"}))),
    };
    match crate::db::combat::attempt_flee(pool, &campaign_id, &p, req.roll, &req.skill).await {
        Ok(result) => (StatusCode::OK, Json(result)),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    }
}

pub async fn end_combat_handler(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
) -> impl IntoResponse {
    match crate::db::combat::end_combat(&state.pool, &campaign_id, "victory", 0).await {
        Ok(_) => (StatusCode::OK, Json(json!({"message": "Combat ended"}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
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
                Some("Drow")       => "Level 3: Faerie Fire. Level 5: Darkness.",
                Some("High Elf")   => "Level 3: Detect Magic. Level 5: Misty Step.",
                Some("Wood Elf")   => "Level 3: Longstrider. Level 5: Pass without Trace.",
                Some("Astral Elf") => "Radiant Soul once per Long Rest. Starlight Step teleportation.",
                _                  => "Lineage spells granted at levels 3 and 5.",
            };

            if subtype == Some("Astral Elf") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Starlight Step",
                    Some("Bonus Action: teleport up to 30 feet to an unoccupied space you can see."),
                    prof_bonus, "long_rest").await;
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Radiant Soul",
                    Some("Resistance to Radiant damage. Once per Long Rest add Proficiency Bonus as extra Radiant damage on a hit or spell."),
                    1, "long_rest").await;
            } else {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Elven Lineage Spells",
                    Some(&format!("Innate spellcasting from your lineage. {} Each can be cast once without a slot per Long Rest.", spells)),
                    1, "long_rest").await;
            }
        }

        "Gnome" => {
            match subtype {
                Some("Forest Gnome") => {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Speak with Animals",
                        Some("Innate spell. Cast without a slot a number of times equal to your Proficiency Bonus per Long Rest."),
                        prof_bonus, "long_rest").await;
                }
                _ => {
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
                Some("Storm Giant") => ("Storm's Thunder",
                    "Reaction when a creature within 60 ft damages you: deal 1d8 Thunder damage to that creature."),
                _ => ("Stone's Endurance",
                    "Reaction when you take damage: roll 1d12 + CON modifier and reduce the damage by that amount."),
            };
            let _ = world::create_ability(pool, campaign_id, "player", player_id,
                name, Some(desc), prof_bonus, "long_rest").await;
            let _ = world::create_ability(pool, campaign_id, "player", player_id,
                "Large Form",
                Some("At level 5: Bonus Action to grow to Large size for 10 minutes. Advantage on STR checks, Speed +10 ft."),
                1, "long_rest").await;
        }

        "Half-Elf" => {
            let heritage_desc = match subtype {
                Some("High Elf Heritage") => "You know the Prestidigitation cantrip.",
                Some("Wood Elf Heritage") => "Your Speed is 35 feet.",
                Some("Drow Heritage")     => "Your Darkvision range is 120 feet.",
                Some("Astral Elf Heritage") => "Resistance to Radiant damage. Starlight Step teleportation.",
                _                         => "Elven heritage trait.",
            };
            let _ = world::create_ability(pool, campaign_id, "player", player_id,
                "Fey Ancestry",
                Some("You have Advantage on saving throws to avoid or end the Charmed condition."),
                1, "manual").await;
            let _ = world::create_ability(pool, campaign_id, "player", player_id,
                "Elven Heritage",
                Some(heritage_desc),
                1, "manual").await;
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
                Some(&format!("Innate spellcasting. Resistance to {} damage. {} Each can be cast once without a slot per Long Rest.", resistance, spells)),
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

    type ItemDef<'a> = (&'a str, &'a str, &'a str, Option<&'a str>, Option<&'a str>, Option<&'a str>, Option<&'a str>, Option<i64>, Option<&'a str>, Option<&'a str>, i64);

    let (items, starting_gp): (Vec<ItemDef>, i64) = match (class, choice) {

        ("Barbarian", _) => (vec![
            ("Greataxe",        "A massive two-handed axe.",            "weapon",   Some("d12"), Some("slashing"),     Some("melee"),  Some("greataxe"),     None,     None,           Some("main_hand"), 1),
            ("Handaxe",         "A light axe suitable for throwing.",   "weapon",   Some("d6"),  Some("slashing"),     Some("melee"),  Some("handaxe"),      None,     None,           None,              4),
            ("Explorer's Pack", "Bedroll, mess kit, tinderbox, 10 torches, 10 days rations, waterskin, 50ft rope.", "wondrous", None, None, None, None, None, None, None, 1),
        ], 15),

        ("Bard", _) => (vec![
            ("Leather Armor",       "Light armor made of cured leather.",        "armor",    None,        None,             None,           None,             Some(11), Some("light"),  Some("armor"),     1),
            ("Dagger",              "A simple short blade.",                      "weapon",   Some("d4"),  Some("piercing"), Some("melee"),  Some("dagger"),   None,     None,           Some("main_hand"), 2),
            ("Musical Instrument",  "A musical instrument of your choice.",       "wondrous", None,        None,             None,           None,             None,     None,           None,              1),
            ("Entertainer's Pack",  "Backpack, bedroll, 2 costumes, 5 candles, 5 days rations, waterskin, disguise kit.", "wondrous", None, None, None, None, None, None, None, 1),
        ], 19),

        ("Cleric", _) => (vec![
            ("Chain Shirt",  "Medium armor of interlocking rings.",              "armor",    None,        None,               None,           None,           Some(13), Some("medium"), Some("armor"),     1),
            ("Shield",       "A wooden or metal shield.",                        "armor",    None,        None,               None,           None,           Some(2),  Some("shield"), Some("shield"),    1),
            ("Mace",         "A bludgeoning weapon with a flanged head.",        "weapon",   Some("d6"),  Some("bludgeoning"), Some("melee"), Some("mace"),   None,     None,           Some("main_hand"), 1),
            ("Holy Symbol",  "A symbol of your deity.",                          "wondrous", None,        None,               None,           None,           None,     None,           None,              1),
            ("Priest's Pack", "Backpack, blanket, 10 candles, tinderbox, alms box, 2 blocks incense, censer, vestments, 2 days rations, waterskin.", "wondrous", None, None, None, None, None, None, None, 1),
        ], 7),

        ("Druid", _) => (vec![
            ("Leather Armor",   "Light armor made of cured leather.",            "armor",    None,        None,               None,           None,                 Some(11), Some("light"),  Some("armor"),     1),
            ("Shield",          "A wooden shield.",                              "armor",    None,        None,               None,           None,                 Some(2),  Some("shield"), Some("shield"),    1),
            ("Sickle",          "A curved blade used in harvesting.",            "weapon",   Some("d4"),  Some("slashing"),   Some("melee"),  Some("sickle"),       None,     None,           Some("main_hand"), 1),
            ("Druidic Focus",   "A quarterstaff serving as a druidic focus.",    "weapon",   Some("d6"),  Some("bludgeoning"), Some("melee"), Some("quarterstaff"), None,     None,           Some("off_hand"),  1),
            ("Explorer's Pack", "Bedroll, mess kit, tinderbox, 10 torches, 10 days rations, waterskin, 50ft rope.", "wondrous", None, None, None, None, None, None, None, 1),
            ("Herbalism Kit",   "Tools for identifying and using herbs.",        "wondrous", None,        None,               None,           None,                 None,     None,           None,              1),
        ], 9),

        ("Fighter", "A") => (vec![
            ("Chain Mail",        "Heavy armor of interlocking rings.",          "armor",    None,        None,               None,           None,               Some(16), Some("heavy"), Some("armor"),     1),
            ("Greatsword",        "A massive two-handed sword.",                 "weapon",   Some("2d6"), Some("slashing"),   Some("melee"),  Some("greatsword"), None,     None,          Some("main_hand"), 1),
            ("Flail",             "A spiked ball on a chain.",                   "weapon",   Some("d8"),  Some("bludgeoning"), Some("melee"), Some("flail"),      None,     None,          None,              1),
            ("Javelin",           "A light thrown spear.",                       "weapon",   Some("d6"),  Some("piercing"),   Some("melee"),  Some("javelin"),    None,     None,          None,              8),
            ("Dungeoneer's Pack", "Backpack, crowbar, hammer, 10 pitons, 10 torches, tinderbox, 10 days rations, waterskin, 50ft rope.", "wondrous", None, None, None, None, None, None, None, 1),
        ], 4),

        ("Fighter", "B") => (vec![
            ("Studded Leather",   "Light armor with metal studs.",               "armor",    None,        None,             None,           None,               Some(12), Some("light"), Some("armor"),     1),
            ("Scimitar",          "A curved slashing sword.",                    "weapon",   Some("d6"),  Some("slashing"), Some("melee"),  Some("scimitar"),   None,     None,          Some("main_hand"), 1),
            ("Shortsword",        "A light thrusting blade.",                    "weapon",   Some("d6"),  Some("piercing"), Some("melee"),  Some("shortsword"), None,     None,          Some("off_hand"),  1),
            ("Longbow",           "A powerful ranged weapon.",                   "weapon",   Some("d8"),  Some("piercing"), Some("ranged"), Some("longbow"),    None,     None,          None,              1),
            ("Arrow",             "Ammunition for a bow.",                       "wondrous", None,        None,             None,           None,               None,     None,          None,              20),
            ("Quiver",            "A container for arrows.",                     "wondrous", None,        None,             None,           None,               None,     None,          None,              1),
            ("Dungeoneer's Pack", "Backpack, crowbar, hammer, 10 pitons, 10 torches, tinderbox, 10 days rations, waterskin, 50ft rope.", "wondrous", None, None, None, None, None, None, None, 1),
        ], 11),

        ("Monk", _) => (vec![
            ("Spear",  "A long thrusting weapon.",                               "weapon",   Some("d6"),  Some("piercing"), Some("melee"),  Some("spear"),    None, None, Some("main_hand"), 1),
            ("Dagger", "A simple short blade.",                                  "weapon",   Some("d4"),  Some("piercing"), Some("melee"),  Some("dagger"),   None, None, None,              5),
            ("Artisan's Tools or Musical Instrument", "Tools matching your background tool proficiency.", "wondrous", None, None, None, None, None, None, None, 1),
            ("Explorer's Pack", "Bedroll, mess kit, tinderbox, 10 torches, 10 days rations, waterskin, 50ft rope.", "wondrous", None, None, None, None, None, None, None, 1),
        ], 11),

        ("Paladin", _) => (vec![
            ("Chain Mail",  "Heavy armor of interlocking rings.",                "armor",    None,        None,             None,           None,               Some(16), Some("heavy"),  Some("armor"),     1),
            ("Shield",      "A wooden or metal shield.",                         "armor",    None,        None,             None,           None,               Some(2),  Some("shield"), Some("shield"),    1),
            ("Longsword",   "A versatile sword.",                                "weapon",   Some("d8"),  Some("slashing"), Some("melee"),  Some("longsword"),  None,     None,           Some("main_hand"), 1),
            ("Javelin",     "A light thrown spear.",                             "weapon",   Some("d6"),  Some("piercing"), Some("melee"),  Some("javelin"),    None,     None,           None,              6),
            ("Holy Symbol", "A symbol of your deity.",                           "wondrous", None,        None,             None,           None,               None,     None,           None,              1),
            ("Priest's Pack", "Backpack, blanket, 10 candles, tinderbox, alms box, 2 blocks incense, censer, vestments, 2 days rations, waterskin.", "wondrous", None, None, None, None, None, None, None, 1),
        ], 9),

        ("Ranger", _) => (vec![
            ("Studded Leather", "Light armor with metal studs.",                 "armor",    None,        None,             None,           None,               Some(12), Some("light"),  Some("armor"),     1),
            ("Scimitar",        "A curved slashing sword.",                      "weapon",   Some("d6"),  Some("slashing"), Some("melee"),  Some("scimitar"),   None,     None,           Some("main_hand"), 1),
            ("Shortsword",      "A light thrusting blade.",                      "weapon",   Some("d6"),  Some("piercing"), Some("melee"),  Some("shortsword"), None,     None,           Some("off_hand"),  1),
            ("Longbow",         "A powerful ranged weapon.",                     "weapon",   Some("d8"),  Some("piercing"), Some("ranged"), Some("longbow"),    None,     None,           None,              1),
            ("Arrow",           "Ammunition for a bow.",                         "wondrous", None,        None,             None,           None,               None,     None,           None,              20),
            ("Quiver",          "A container for arrows.",                       "wondrous", None,        None,             None,           None,               None,     None,           None,              1),
            ("Druidic Focus",   "A sprig of mistletoe serving as a druidic focus.", "wondrous", None,    None,             None,           None,               None,     None,           None,              1),
            ("Explorer's Pack", "Bedroll, mess kit, tinderbox, 10 torches, 10 days rations, waterskin, 50ft rope.", "wondrous", None, None, None, None, None, None, None, 1),
        ], 7),

        ("Rogue", _) => (vec![
            ("Leather Armor",  "Light armor made of cured leather.",             "armor",    None,        None,             None,           None,               Some(11), Some("light"),  Some("armor"),     1),
            ("Dagger",         "A simple short blade.",                          "weapon",   Some("d4"),  Some("piercing"), Some("melee"),  Some("dagger"),     None,     None,           Some("main_hand"), 2),
            ("Shortsword",     "A light thrusting blade.",                       "weapon",   Some("d6"),  Some("piercing"), Some("melee"),  Some("shortsword"), None,     None,           Some("off_hand"),  1),
            ("Shortbow",       "A compact ranged weapon.",                       "weapon",   Some("d6"),  Some("piercing"), Some("ranged"), Some("shortbow"),   None,     None,           None,              1),
            ("Arrow",          "Ammunition for a bow.",                          "wondrous", None,        None,             None,           None,               None,     None,           None,              20),
            ("Quiver",         "A container for arrows.",                        "wondrous", None,        None,             None,           None,               None,     None,           None,              1),
            ("Thieves' Tools", "Tools for picking locks and disarming traps.",   "wondrous", None,        None,             None,           None,               None,     None,           None,              1),
            ("Burglar's Pack", "Backpack, 1000 ball bearings, 10ft string, bell, 5 candles, crowbar, hammer, 10 pitons, hooded lantern, 2 oil flasks, 5 days rations, tinderbox, waterskin.", "wondrous", None, None, None, None, None, None, None, 1),
        ], 8),

        ("Sorcerer", _) => (vec![
            ("Spear",             "A long thrusting weapon.",                    "weapon",   Some("d6"), Some("piercing"), Some("melee"),  Some("spear"),  None, None, Some("main_hand"), 1),
            ("Dagger",            "A simple short blade.",                       "weapon",   Some("d4"), Some("piercing"), Some("melee"),  Some("dagger"), None, None, None,              2),
            ("Arcane Focus",      "A crystal serving as an arcane focus.",       "wondrous", None,       None,             None,           None,           None, None, None,              1),
            ("Dungeoneer's Pack", "Backpack, crowbar, hammer, 10 pitons, 10 torches, tinderbox, 10 days rations, waterskin, 50ft rope.", "wondrous", None, None, None, None, None, None, None, 1),
        ], 28),

        ("Warlock", _) => (vec![
            ("Leather Armor",       "Light armor made of cured leather.",        "armor",    None,       None,            None,          None,           Some(11), Some("light"), Some("armor"),     1),
            ("Sickle",              "A curved blade.",                           "weapon",   Some("d4"), Some("slashing"), Some("melee"), Some("sickle"), None,     None,          Some("main_hand"), 1),
            ("Dagger",              "A simple short blade.",                     "weapon",   Some("d4"), Some("piercing"), Some("melee"), Some("dagger"), None,     None,          None,              2),
            ("Arcane Focus",        "An orb serving as an arcane focus.",        "wondrous", None,       None,             None,          None,           None,     None,          None,              1),
            ("Book of Occult Lore", "A book of occult knowledge.",               "wondrous", None,       None,             None,          None,           None,     None,          None,              1),
            ("Scholar's Pack",      "Backpack, book, ink, ink pen, 10 parchment sheets, a little bag of sand, small knife.", "wondrous", None, None, None, None, None, None, None, 1),
        ], 15),

        ("Wizard", _) => (vec![
            ("Dagger",       "A simple short blade.",                            "weapon",   Some("d4"), Some("piercing"),   Some("melee"), Some("dagger"),       None, None, Some("main_hand"), 2),
            ("Arcane Focus", "A quarterstaff serving as an arcane focus.",       "weapon",   Some("d6"), Some("bludgeoning"), Some("melee"), Some("quarterstaff"), None, None, None,              1),
            ("Robe",         "A comfortable robe.",                              "wondrous", None,       None,                None,          None,                 None, None, None,              1),
            ("Spellbook",    "A book containing your wizard spells.",            "wondrous", None,       None,                None,          None,                 None, None, None,              1),
            ("Scholar's Pack", "Backpack, book, ink, ink pen, 10 parchment sheets, a little bag of sand, small knife.", "wondrous", None, None, None, None, None, None, None, 1),
        ], 5),

        _ => (vec![
            ("Dagger", "A simple short blade.", "weapon", Some("d4"), Some("piercing"), Some("melee"), Some("dagger"), None, None, Some("main_hand"), 1),
        ], 10),
    };

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
                    Some("Take one additional action on your turn. Recharges on short or long rest."),
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
        Some("Psi Warrior") => {
            let _ = fighter::update_psi_warrior_dice(pool, player_id, new_level).await;

            match new_level {
                3 => {
                    if !has("Psionic Strike") {
                        let _ = world::create_ability(pool, campaign_id, "player", player_id,
                            "Psionic Strike",
                            Some("Once per turn, after hitting a target within 30 ft with a weapon, expend one Psionic Energy Die. Deal Force damage = die roll + INT modifier."),
                            1, "manual").await;
                    }
                    if !has("Protective Field") {
                        let _ = world::create_ability(pool, campaign_id, "player", player_id,
                            "Protective Field",
                            Some("Reaction: when you or a creature within 30 ft takes damage, expend one Psionic Energy Die. Reduce damage by die roll + INT modifier (minimum 1)."),
                            1, "manual").await;
                    }
                    if !has("Telekinetic Movement") {
                        let _ = world::create_ability(pool, campaign_id, "player", player_id,
                            "Telekinetic Movement",
                            Some("Move one Large-or-smaller object or willing creature within 30 ft up to 30 ft. Free once per Short Rest, or expend a Psionic Energy Die to restore."),
                            1, "short_rest").await;
                    }
                }
                7 => {
                    if !has("Psi-Powered Leap") {
                        let _ = world::create_ability(pool, campaign_id, "player", player_id,
                            "Psi-Powered Leap",
                            Some("Bonus Action: gain Fly Speed equal to twice your Speed until end of turn. Free once per Short Rest, or expend a Psionic Energy Die to restore."),
                            1, "short_rest").await;
                    }
                    if !has("Telekinetic Thrust") {
                        let _ = world::create_ability(pool, campaign_id, "player", player_id,
                            "Telekinetic Thrust",
                            Some("After Psionic Strike deals damage, force target to make STR save (DC 8 + INT mod + Prof). On fail: target is knocked Prone OR pushed 10 ft horizontally."),
                            1, "manual").await;
                    }
                }
                10 => {
                    if !has("Guarded Mind") {
                        let _ = world::create_ability(pool, campaign_id, "player", player_id,
                            "Guarded Mind",
                            Some("Resistance to Psychic damage. At start of your turn, expend a Psionic Energy Die to end Charmed or Frightened conditions on yourself."),
                            1, "manual").await;
                    }
                }
                15 => {
                    if !has("Bulwark of Force") {
                        let _ = world::create_ability(pool, campaign_id, "player", player_id,
                            "Bulwark of Force",
                            Some("Bonus Action: grant yourself and up to INT modifier creatures within 30 ft a temporary AC bonus for 1 minute. Free once per Long Rest, or expend a Psionic Energy Die to restore."),
                            1, "long_rest").await;
                    }
                }
                18 => {
                    if !has("Telekinetic Master") {
                        let _ = world::create_ability(pool, campaign_id, "player", player_id,
                            "Telekinetic Master",
                            Some("Cast Telekinesis without a spell slot (INT is spellcasting ability). While concentrating, make one weapon attack as a Bonus Action each turn. Free once per Long Rest, or expend a Psionic Energy Die to restore."),
                            1, "long_rest").await;
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
}

// ─── Shop handlers ────────────────────────────────────────────────────────────

pub async fn get_shop_state(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
) -> impl IntoResponse {
    match crate::db::shop::get_active_shop(&state.pool, &campaign_id).await {
        Ok(Some(shop)) => (StatusCode::OK, Json(shop)),
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({"error": "No active shop"}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct BuyItemRequest {
    pub shop_item_id: String,
    pub quantity: Option<i64>,
}

pub async fn buy_item(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
    Json(req): Json<BuyItemRequest>,
) -> impl IntoResponse {
    let pool = &state.pool;
    let p = match player::get_player_by_campaign(pool, &campaign_id).await {
        Ok(Some(p)) => p,
        _ => return (StatusCode::NOT_FOUND, Json(json!({"error": "Player not found"}))),
    };
    let quantity = req.quantity.unwrap_or(1).max(1);
    match crate::db::shop::buy_item(pool, &campaign_id, &p, &req.shop_item_id, quantity).await {
        Ok(result) => (StatusCode::OK, Json(result)),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct SellItemRequest {
    pub player_item_id: String,
}

pub async fn sell_item(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
    Json(req): Json<SellItemRequest>,
) -> impl IntoResponse {
    let pool = &state.pool;
    let p = match player::get_player_by_campaign(pool, &campaign_id).await {
        Ok(Some(p)) => p,
        _ => return (StatusCode::NOT_FOUND, Json(json!({"error": "Player not found"}))),
    };
    match crate::db::shop::sell_item(pool, &campaign_id, &p, &req.player_item_id).await {
        Ok(result) => (StatusCode::OK, Json(result)),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    }
}

pub async fn close_shop(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
) -> impl IntoResponse {
    match crate::db::shop::close_shop(&state.pool, &campaign_id).await {
        Ok(result) => (StatusCode::OK, Json(result)),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    }
}