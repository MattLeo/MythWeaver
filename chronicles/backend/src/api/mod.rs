use axum::{
    extract::{Path, State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::db::{campaign, player, world, time};
use crate::llm::{LlmClient, ChatMessage, prompt};
use crate::models::*;
use crate::AppState;

// ─── Campaign ─────────────────────────────────────────────────────────────────

pub async fn create_campaign(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateCampaignRequest>,
) -> impl IntoResponse {
    let pool = &state.pool;

    // Create campaign
    let campaign_name = req.name.clone().unwrap_or_else(|| "MythWeaver Campaign".to_string());
    let camp = match campaign::create_campaign(pool, &campaign_name).await {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    };

    // Create player
    let p = match player::create_player(pool, &camp.id, &req).await {
        Ok(p) => p,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    };

    // Seed starting abilities based on class
    seed_class_abilities(pool, &camp.id, &p.id, &p.class).await;

    // Initialize campaign time
    if let Err(e) = time::init_campaign_time(pool, &camp.id).await {
        tracing::warn!("Failed to init campaign time: {}", e);
    }

    // Create session
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

    // Get session messages for summarization
    let messages = campaign::get_session_messages(pool, &session_id).await;

    if let Ok(msgs) = messages {
        // Build summary via LLM
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

    // Get player and time for system prompt
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

    let system = prompt::build_system_prompt(&p, camp_time.as_ref(), &summaries, &game_state);

    // Build message history from this session
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

    // Build user message content
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

    // Save user message
    let _ = campaign::save_message(pool, session_id, campaign_id, "user", &user_content, None).await;
    messages.push(ChatMessage::user(&user_content));

    // Check for random event trigger
    let event_context = check_random_event(pool, campaign_id, &game_state).await;
    if let Some(event) = event_context {
        // Inject event as a system note at the end of the user message
        let augmented = format!("{}\n\n[WORLD EVENT - narrate naturally]: {}", user_content, event);
        if let Some(last) = messages.last_mut() {
            last.content = Some(augmented);
        }
    }

    // Run the agentic loop
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

    // If a roll was requested, return that instead of narrative
    if let Some(roll_req) = result.roll_request {
        return (StatusCode::OK, Json(json!({
            "type": "roll_request",
            "roll": roll_req
        })));
    }

    // Save assistant response
    let _ = campaign::save_message(pool, session_id, campaign_id, "assistant", &result.narrative, None).await;

    // Return narrative response
    (StatusCode::OK, Json(json!({
        "type": "narrative",
        "content": result.narrative,
        "tools_used": result.tool_calls_made.iter().map(|t| &t.tool_name).collect::<Vec<_>>()
    })))
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

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

    // Always add Hit Dice
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