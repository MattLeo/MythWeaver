use axum::{
    extract::{Path, State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::db::{
    campaign, 
    player, 
    world, 
    items, 
    companions, 
    time, 
    fighter, 
    spells as spells_db,
    feats as feats_db
};
use crate::llm::{ChatMessage, prompt};
use crate::models::*;
use crate::AppState;
use uuid::Uuid;

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

    if p.class == "Bard" {
        let cha_mod = Player::modifier(p.cha).max(1);
        let _ = sqlx::query(
            "UPDATE abilities SET max_uses = ?, current_uses = ?
             WHERE owner_id = ? AND name = 'Bardic Inspiration'"
        )
        .bind(cha_mod)
        .bind(cha_mod)
        .bind(&p.id)
        .execute(pool)
        .await;
    }

    if p.class == "Cleric" {
        // Channel Divinity is a level 2 feature — zero it out at creation
        let _ = sqlx::query(
            "UPDATE abilities SET max_uses = 0, current_uses = 0
             WHERE owner_id = ? AND name = 'Channel Divinity'"
        )
        .bind(&p.id)
        .execute(pool)
        .await;
 
        let wis_mod = Player::modifier(p.wis).max(1);
 
        match req.divine_order.as_deref() {
            Some("Protector") => {
                // Martial weapon proficiency
                let _ = sqlx::query(
                    "INSERT OR IGNORE INTO proficiencies
                     (id, campaign_id, player_id, proficiency_type, name, source)
                     VALUES (?, ?, ?, 'weapon', 'martial', 'class')"
                )
                .bind(Uuid::new_v4().to_string())
                .bind(&camp.id)
                .bind(&p.id)
                .execute(pool)
                .await;
 
                // Heavy armor training
                let _ = sqlx::query(
                    "INSERT OR IGNORE INTO proficiencies
                     (id, campaign_id, player_id, proficiency_type, name, source)
                     VALUES (?, ?, ?, 'armor', 'heavy', 'class')"
                )
                .bind(Uuid::new_v4().to_string())
                .bind(&camp.id)
                .bind(&p.id)
                .execute(pool)
                .await;
 
                // Update Divine Order ability with specific description
                let _ = sqlx::query(
                    "UPDATE abilities SET description = ?
                     WHERE owner_id = ? AND name = 'Divine Order'"
                )
                .bind("Protector: you have proficiency with Martial weapons and training with \
                       Heavy armor.")
                .bind(&p.id)
                .execute(pool)
                .await;
 
                // Recalculate AC now that heavy armor proficiency is available
                let _ = items::recalculate_ac(pool, &p.id).await;
            }
 
                        Some("Thaumaturge") => {
                let wis_mod = Player::modifier(p.wis).max(1);
 
                // Learn the chosen extra cantrip in the spells system
                if let Some(ref cantrip_name) = req.thaumaturge_cantrip {
                    match spells_db::get_spell_by_name(pool, cantrip_name).await {
                        Ok(Some(spell)) => {
                            if let Some(spell_id) = spell["id"].as_str() {
                                let _ = spells_db::learn_spell(
                                    pool, &camp.id, &p.id, spell_id, "cantrip", "thaumaturge"
                                ).await;
                            }
                        }
                        _ => {
                            tracing::warn!(
                                "Thaumaturge cantrip '{}' not found in spell database",
                                cantrip_name
                            );
                        }
                    }
                }
 
                // Update Divine Order ability description
                let cantrip_display = req.thaumaturge_cantrip.as_deref().unwrap_or("(not chosen)");
                let _ = sqlx::query(
                    "UPDATE abilities SET description = ? WHERE owner_id = ? AND name = 'Divine Order'"
                )
                .bind(&format!(
                    "Thaumaturge: you know one extra cantrip ({}) from the Cleric spell list. \
                     You add your WIS modifier (+{}) to Intelligence (Arcana or Religion) checks.",
                    cantrip_display, wis_mod
                ))
                .bind(&p.id)
                .execute(pool)
                .await;
            }
 
            _ => {
                // No choice recorded — leave the generic description in place.
                // This should not happen in normal play since the step is required.
            }
        }
    }

    if p.class == "Druid" {
        let wis_mod = Player::modifier(p.wis).max(1);
 
        // Speak with Animals — always prepared from Druidic (level 1)
        if let Ok(Some(spell)) = spells_db::get_spell_by_name(pool, "Speak with Animals").await {
            if let Some(spell_id) = spell["id"].as_str() {
                let _ = spells_db::learn_spell(
                    pool, &camp.id, &p.id, spell_id, "always_prepared", "druidic"
                ).await;
            }
        }
 
        // Seed level 1 spell slots (2 × Level 1)
        if let Err(e) = spells_db::seed_full_caster_spell_slots(pool, &camp.id, &p.id, 1).await {
            tracing::warn!("Failed to seed Druid spell slots: {}", e);
        }
 
        match req.primal_order.as_deref() {
            Some("Warden") => {
                // Martial weapon proficiency
                let _ = sqlx::query(
                    "INSERT OR IGNORE INTO proficiencies
                     (id, campaign_id, player_id, proficiency_type, name, source)
                     VALUES (?, ?, ?, 'weapon', 'martial', 'class')"
                )
                .bind(Uuid::new_v4().to_string()).bind(&camp.id).bind(&p.id)
                .execute(pool).await;
 
                // Medium armor training
                let _ = sqlx::query(
                    "INSERT OR IGNORE INTO proficiencies
                     (id, campaign_id, player_id, proficiency_type, name, source)
                     VALUES (?, ?, ?, 'armor', 'medium', 'class')"
                )
                .bind(Uuid::new_v4().to_string()).bind(&camp.id).bind(&p.id)
                .execute(pool).await;
 
                let _ = sqlx::query(
                    "UPDATE abilities SET description = ? WHERE owner_id = ? AND name = 'Primal Order'"
                )
                .bind("Warden: you have proficiency with Martial weapons and training with Medium armor.")
                .bind(&p.id)
                .execute(pool).await;
 
                let _ = items::recalculate_ac(pool, &p.id).await;
            }
 
            Some("Magician") => {
                // Learn the chosen extra cantrip
                if let Some(ref cantrip_name) = req.magician_cantrip {
                    match spells_db::get_spell_by_name(pool, cantrip_name).await {
                        Ok(Some(spell)) => {
                            if let Some(spell_id) = spell["id"].as_str() {
                                let _ = spells_db::learn_spell(
                                    pool, &camp.id, &p.id, spell_id, "cantrip", "magician"
                                ).await;
                            }
                        }
                        _ => tracing::warn!("Magician cantrip '{}' not found", cantrip_name),
                    }
                }
 
                let cantrip_display = req.magician_cantrip.as_deref().unwrap_or("(not chosen)");
                let _ = sqlx::query(
                    "UPDATE abilities SET description = ? WHERE owner_id = ? AND name = 'Primal Order'"
                )
                .bind(&format!(
                    "Magician: you know one extra cantrip ({}) from the Druid spell list. \
                     You add your WIS modifier (+{}) to Intelligence (Arcana or Nature) checks.",
                    cantrip_display, wis_mod
                ))
                .bind(&p.id)
                .execute(pool).await;
            }
 
            _ => {}
        }
    }

    // ── Paladin: seed initial spell slots + learn Divine Smite + Find Steed ──
    if p.class == "Paladin" {
        // Seed level 1 spell slots (2 × L1 slots)
        if let Err(e) = spells_db::seed_half_caster_spell_slots(pool, &camp.id, &p.id, 1).await {
            tracing::warn!("Failed to seed Paladin spell slots: {}", e);
        }
 
        // Divine Smite — always prepared from level 2 (seed at creation so LLM DM knows it)
        // Seed as 'prepared' now; it becomes a free-cast ability tracked as an ability at level 2
        if let Ok(Some(spell)) = spells_db::get_spell_by_name(pool, "Divine Smite").await {
            if let Some(spell_id) = spell["id"].as_str() {
                let _ = spells_db::learn_spell(
                    pool, &camp.id, &p.id, spell_id, "always_prepared", "paladin"
                ).await;
            }
        }
    }

    if p.class == "Ranger" {
        // Seed level 1 spell slots (2 × L1 slots)
        if let Err(e) = spells_db::seed_half_caster_spell_slots(pool, &camp.id, &p.id, 1).await {
            tracing::warn!("Failed to seed Ranger spell slots: {}", e);
        }
 
        // Hunter's Mark — always prepared from Favored Enemy
        if let Ok(Some(spell)) = spells_db::get_spell_by_name(pool, "Hunter's Mark").await {
            if let Some(spell_id) = spell["id"].as_str() {
                let _ = spells_db::learn_spell(
                    pool, &camp.id, &p.id, spell_id, "always_prepared", "favored_enemy"
                ).await;
            }
        }
    }

    if p.class == "Sorcerer" {
        if let Err(e) = spells_db::seed_full_caster_spell_slots(pool, &camp.id, &p.id, 1).await {
            tracing::warn!("Failed to seed Sorcerer spell slots: {}", e);
        }
    }
    
    if p.class == "Warlock" {
        if let Err(e) = spells_db::seed_warlock_spell_slots(pool, &camp.id, &p.id, 1).await {
            tracing::warn!("Failed to seed Warlock spell slots: {}", e);
        }
        // Learn Eldritch Blast as a cantrip
        if let Ok(Some(spell)) = spells_db::get_spell_by_name(pool, "Eldritch Blast").await {
            if let Some(spell_id) = spell["id"].as_str() {
                let _ = spells_db::learn_spell(
                    pool, &camp.id, &p.id, spell_id, "cantrip", "warlock"
                ).await;
            }
        }
    }

    if p.class == "Wizard" {
        if let Err(e) = spells_db::seed_full_caster_spell_slots(pool, &camp.id, &p.id, 1).await {
            tracing::warn!("Failed to seed Wizard spell slots: {}", e);
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

    // ── Background feat ───────────────────────────────────────────────────────
    if let Some(ref feat_id) = req.background_feat_id {
        let _ = feats_db::take_feat(
            pool, &camp.id, &p.id, feat_id,
            "background", 1,
            req.background_feat_choices.as_deref(),
        ).await;
        apply_feat_effects(
            pool, &camp.id, &p.id, feat_id,
            req.background_feat_choices.as_deref(),
            1,
        ).await;
    }

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

    let spell_slots = spells_db::get_spell_slots(pool, &p.id).await.unwrap_or_default();
    let known_spells = spells_db::get_known_spells(pool, &p.id).await.unwrap_or_default();
    let war_bonds = spells_db::get_war_bonds(pool, &p.id).await.unwrap_or_default();
    let concentration = spells_db::get_concentration(pool, &p.id).await.unwrap_or(None);
    let player_feats = feats_db::get_player_feats(pool, &p.id).await.unwrap_or_default();
 
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
        "spell_slots": spell_slots,
        "known_spells": known_spells,
        "war_bonds": war_bonds,
        "concentration": concentration,
        "feats": player_feats,   // ← add this
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

    // ── ASI or Feat ───────────────────────────────────────────────────────────

     if let Some(ref feat_id) = req.feat_id {
        if let Err(e) = feats_db::take_feat(
            pool, &campaign_id, &p.id, feat_id,
            "asi", result.new_level,
            req.feat_choices.as_deref(),
        ).await {
            return (StatusCode::BAD_REQUEST, Json(json!({"error": e.to_string()})));
        }
        apply_feat_effects(
            pool, &campaign_id, &p.id, feat_id,
            req.feat_choices.as_deref(),
            result.new_level,
        ).await;
        let _ = items::recalculate_ac(pool, &p.id).await;
    } else if let Some(ref stat1) = req.asi_stat1 {
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

    if matches!(p.class.as_str(), "Bard" | "Cleric" | "Druid" | "Sorcerer" | "Wizard") {
        if let Err(e) = spells_db::seed_full_caster_spell_slots(
            pool, &campaign_id, &p.id, result.new_level
        ).await {
            tracing::warn!("Failed to update full caster spell slots: {}", e);
        }
    }

    // Half-casters (Paladin, Ranger)
    if matches!(p.class.as_str(), "Paladin" | "Ranger") {
        if let Err(e) = spells_db::seed_half_caster_spell_slots(
            pool, &campaign_id, &p.id, result.new_level
        ).await {
            tracing::warn!("Failed to update half-caster spell slots: {}", e);
        }
    }

    if p.class == "Warlock" {
        if let Err(e) = spells_db::seed_warlock_spell_slots(
            pool, &campaign_id, &p.id, result.new_level
        ).await {
            tracing::warn!("Failed to update Warlock spell slots: {}", e);
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
            ("Rage",
             Some("Bonus Action: enter a Rage. While raging: Resistance to Bludgeoning/Piercing/Slashing; \
                   +2 damage on STR-based attacks and Unarmed Strikes; Advantage on STR checks and saves. \
                   Can't maintain Concentration or cast spells. Lasts until end of next turn — extend each \
                   round by attacking, forcing a save, or using a Bonus Action (max 10 min). \
                   Regain 1 use on Short Rest, all uses on Long Rest."),
             2, "long_rest"),
            ("Unarmored Defense",
             Some("While not wearing armor, AC = 10 + DEX modifier + CON modifier. You may still use a Shield."),
             1, "manual"),
            ("Weapon Mastery",
             Some("Use the Mastery property of 2 Simple or Martial Melee weapons you are proficient with. \
                   Change choices after each Long Rest. Count increases to 3 at level 4, and 4 at level 10."),
             1, "manual"),
            ("Reckless Attack",
             Some("On your first attack roll of the turn, you may attack recklessly: you have Advantage on all \
                   STR-based attack rolls this turn, but attack rolls against you also have Advantage until your next turn."),
             1, "per_turn"),
        ],
        "Fighter" => vec![
            ("Second Wind", Some("Bonus Action: regain 1d10 + Fighter level HP. Also usable for Tactical Mind (level 2+)."), 2, "short_rest"),
        ],
        "Rogue" => vec![
            ("Sneak Attack", Some("Deal extra 1d6 damage when you have advantage or an ally is adjacent to the target."), 1, "per_turn"),
            ("Cunning Action", Some("Bonus Action: Dash, Disengage, or Hide."), 1, "per_turn"),
        ],
        "Cleric" => vec![
            ("Channel Divinity",
             Some("Use Channel Divinity 2 times per rest (regain 1 on Short Rest, all on Long Rest). \
                   Effects: Divine Spark — Magic action: roll 1d8+WIS to heal or deal Necrotic/Radiant \
                   damage to a target within 30 ft (CON save for half on damage). Die count grows at \
                   levels 7 (2d8), 13 (3d8), 18 (4d8). \
                   Turn Undead — Magic action: Undead within 30 ft make a WIS save or gain Frightened \
                   and Incapacitated for 1 min. Plus your domain Channel Divinity feature at level 3."),
             2, "short_rest"),
            ("Divine Order",
             Some("Choose one at character creation — \
                   Protector: proficiency with Martial weapons and training with Heavy armor. \
                   Thaumaturge: one extra Cleric cantrip; add WIS modifier (min +1) to \
                   Intelligence (Arcana or Religion) checks."),
             1, "manual"),
        ],
        "Druid" => vec![
            ("Wild Shape",
             Some("Wild Shape is gained at level 2. At that point: Bonus Action — shapeshift into \
                   a known Beast form (CR 1/4 max, 4 forms). Regain 1 use on Short Rest, all on \
                   Long Rest. Ability will be updated when you reach level 2."),
             0, "long_rest"),  // 0 uses until level 2 unlocks it
            ("Druidic",
             Some("You know Druidic, the secret language of Druids. You can leave hidden messages \
                   legible only to those who know Druidic (DC 15 INT Investigation to spot the \
                   message's existence; can't be deciphered without magic). \
                   You also always have Speak with Animals prepared."),
             1, "manual"),
            ("Primal Order",
             Some("Choose one sacred role — Magician: one extra Druid cantrip + add WIS modifier \
                   (min +1) to Intelligence (Arcana or Nature) checks. \
                   Warden: proficiency with Martial weapons and training with Medium armor."),
             1, "manual"),
        ],
        "Paladin" => vec![
            ("Lay On Hands",
             Some("Bonus Action: touch a creature and restore HP from your pool (pool = 5 × Paladin \
                   level, restored on Long Rest). Expend 5 HP from the pool to remove the Poisoned \
                   condition instead. Level 14 (Restoring Touch): also expend 5 HP per condition \
                   removed: Blinded, Charmed, Deafened, Frightened, Paralyzed, or Stunned."),
             5, "long_rest"),  // 5 HP pool at level 1; updated each level-up
            ("Weapon Mastery",
             Some("Use the Mastery property of 2 weapons you are proficient with. \
                   Change choices after each Long Rest."),
             1, "manual"),
            ("Channel Divinity",
             Some("Channel Divinity unlocks at level 3 (2 uses, regain 1 on Short Rest). \
                   Divine Sense: Bonus Action — detect Celestials, Fiends, and Undead within 60 ft, \
                   plus consecrated/desecrated places, for 10 minutes. \
                   Additional effects from your Sacred Oath at level 3."),
             0, "short_rest"),  // 0 uses until level 3
        ],
        "Ranger" => vec![
            ("Favored Enemy",
             Some("Hunter's Mark is always prepared. You can cast it 2 times without expending a \
                   spell slot (recharges on Long Rest). Free cast count increases with level: \
                   3 at L5, 4 at L9, 5 at L13, 6 at L17. \
                   Hunter's Mark die becomes d10 at level 20 (Foe Slayer)."),
             2, "long_rest"),
            ("Weapon Mastery",
             Some("Use the Mastery property of 2 weapons you are proficient with. \
                   Change choices after each Long Rest."),
             1, "manual"),
        ],
        "Monk" => vec![
            ("Focus Points",
             Some("You have 0 Focus Points at level 1 (gained at level 2, equals Monk level). \
                   Restore on Short or Long Rest. Focus save DC = 8 + WIS mod + Prof Bonus. \
                   At level 2 — Flurry of Blows (1 FP): two Unarmed Strikes as Bonus Action. \
                   Patient Defense (1 FP): Disengage + Dodge as Bonus Action. \
                   Step of the Wind (1 FP): Disengage + Dash as Bonus Action, jump distance doubled."),
             0, "short_rest"),  // 0 uses until level 2; updated on first level-up
            ("Unarmored Defense",
             Some("While not wearing armor or wielding a Shield, your base AC equals \
                   10 + DEX modifier + WIS modifier."),
             1, "manual"),
            ("Martial Arts",
             Some("While unarmed or wielding only Monk weapons (Simple Melee or Light Martial Melee) \
                   and not wearing armor or a Shield: \
                   Bonus Unarmed Strike — make one Unarmed Strike as a Bonus Action. \
                   Martial Arts Die — roll 1d6 in place of Unarmed Strike or Monk weapon damage \
                   (d6 at L1-4, d8 at L5-10, d10 at L11-16, d12 at L17-20). \
                   Dexterous Attacks — use DEX for attack and damage rolls of Unarmed Strikes \
                   and Monk weapons; use DEX for Grapple/Shove DCs."),
             1, "manual"),
        ],
        "Bard" => vec![
            ("Bardic Inspiration",
             Some("Bonus Action: grant a Bardic Inspiration die (d6) to a creature within 60 ft \
                   that can see or hear you. They can add it to one failed D20 Test in the next hour. \
                   One die per creature at a time. Uses = CHA modifier (min 1). Refreshes on Long Rest."),
             1,   // actual max set from CHA mod after creation; seeded at 1 as placeholder
             "long_rest"),
            ("Jack of All Trades",
             Some("Add half your Proficiency Bonus (round down) to any ability check that uses a \
                   skill in which you lack proficiency and that doesn't already use your Proficiency Bonus."),
             1, "manual"),
        ],
        "Sorcerer" => vec![
            ("Sorcery Points",
             Some("Font of Magic: convert spell slots to Sorcery Points (1 SP per slot level) or \
                   create spell slots (L1=2SP, L2=3SP, L3=5SP, L4=6SP, L5=7SP). \
                   0 SP at level 1 — gain 2 SP at level 2 (equals Sorcerer level thereafter). \
                   Sorcerous Restoration (L5): once per Long Rest, regain up to half your Sorcerer \
                   level in SP when you finish a Short Rest."),
             0, "long_rest"),  // 0 uses until level 2
            ("Innate Sorcery",
             Some("Bonus Action: unleash your innate magic for 1 minute. While active: \
                   spell save DC increases by 1; Advantage on spell attack rolls. \
                   2 uses per Long Rest. Level 7 (Sorcery Incarnate): if you have no uses left, \
                   spend 2 Sorcery Points to activate it; also use up to two Metamagic options \
                   per spell while active."),
             2, "long_rest"),
            ("Metamagic",
             Some("You know 2 Metamagic options (gain 2 more at L10, 2 more at L17). \
                   Spend Sorcery Points to modify spells as you cast them. \
                   Options: Careful (1SP), Distant (1SP), Empowered (1SP), Extended (1SP), \
                   Heightened (2SP), Quickened (2SP), Seeking (1SP), Subtle (1SP), \
                   Transmuted (1SP), Twinned (1SP)."),
             1, "manual"),
        ],
        "Warlock" => vec![
            ("Pact Magic",
             Some("All Pact Magic slots are the same level. Regain ALL slots on Short or Long Rest. \
                   Slot count and level scale with Warlock level: \
                   L1: 1×L1, L2: 2×L1, L3-4: 2×L2, L5-6: 2×L3, L7-8: 2×L4, \
                   L9-10: 2×L5, L11-16: 3×L5, L17-20: 4×L5. \
                   Magical Cunning (L2): 1-minute rite to regain up to half your max slots. Once per Long Rest. \
                   Eldritch Master (L20): Magical Cunning regains ALL slots instead."),
             1, "short_rest"),
            ("Eldritch Invocations",
             Some("You know 1 invocation at level 1. Gain more as you level (see PHB table). \
                   Notable options: Agonizing Blast (+CHA to damage cantrip), Pact of the Blade \
                   (conjure melee weapon, use CHA for attacks), Pact of the Chain (enhanced familiar), \
                   Pact of the Tome (Book of Shadows with cantrips and rituals), Devil's Sight \
                   (see in magical darkness 120 ft), Thirsting Blade (Extra Attack with pact weapon), \
                   Mystic Arcanum spells (L6-9 once/LR). Replace one invocation per level-up."),
             1, "manual"),
            ("Eldritch Blast",
             Some("Ranged spell attack: one beam dealing 1d10 Force damage per hit. \
                   Gain additional beams at levels 5 (2 beams), 11 (3 beams), 17 (4 beams). \
                   Agonizing Blast invocation: add CHA modifier to each beam's damage."),
             1, "per_turn"),
        ],
        "Wizard" => vec![
            ("Arcane Recovery",
             Some("When you finish a Short Rest, recover expended spell slots with combined level \
                   ≤ half your Wizard level (round up). No slot of level 6 or higher. \
                   Once per Long Rest. \
                   Example: level 4 Wizard can recover up to 2 levels of slots."),
             1, "long_rest"),
            ("Ritual Adept",
             Some("You can cast any spell as a Ritual if it has the Ritual tag and is in your \
                   spellbook. You don't need to have the spell prepared, but must read from the book."),
             1, "manual"),
            ("Spellbook",
             Some("Your spellbook contains your known spells. Starts with 6 level 1 spells. \
                   Add 2 Wizard spells per level-up (of a level you can cast). \
                   You can also copy spells from scrolls or other spellbooks (2 hr + 50 GP per level). \
                   INT is your spellcasting ability. Use an Arcane Focus or your spellbook as \
                   a Spellcasting Focus."),
             1, "manual"),
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
    match class {
        "Fighter"   => seed_level_up_abilities_fighter(pool, campaign_id, player_id, new_level, subclass).await,
        "Barbarian" => seed_level_up_abilities_barbarian(pool, campaign_id, player_id, new_level, subclass).await,
        "Bard"      => seed_level_up_abilities_bard(pool, campaign_id, player_id, new_level, subclass).await,
        "Cleric"    => seed_level_up_abilities_cleric(pool, campaign_id, player_id, new_level, subclass).await,
        "Druid"     => seed_level_up_abilities_druid(pool, campaign_id, player_id, new_level, subclass).await,
        "Monk"      => seed_level_up_abilities_monk(pool, campaign_id, player_id, new_level, subclass).await,
        "Paladin"   => seed_level_up_abilities_paladin(pool, campaign_id, player_id, new_level, subclass).await,
        "Ranger"    => seed_level_up_abilities_ranger(pool, campaign_id, player_id, new_level, subclass).await,
        "Rogue"     => seed_level_up_abilities_rogue(pool, campaign_id, player_id, new_level, subclass).await,
        "Sorcerer"  => seed_level_up_abilities_sorcerer(pool, campaign_id, player_id, new_level, subclass).await,
        "Warlock"   => seed_level_up_abilities_warlock(pool, campaign_id, player_id, new_level, subclass).await,
        "Wizard"    => seed_level_up_abilities_wizard(pool, campaign_id, player_id, new_level, subclass).await,
        _           => {}
    }
}

async fn seed_level_up_abilities_fighter(
    pool: &sqlx::SqlitePool,
    campaign_id: &str,
    player_id: &str,
    new_level: i64,
    subclass: Option<&str>,
) {
    let existing = world::get_abilities(pool, player_id, "player").await.unwrap_or_default();
    let has = |name: &str| existing.iter().any(|a| a.name == name);

    // ── Base class features ───────────────────────────────────────────────────

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

    // ── Subclass features ─────────────────────────────────────────────────────

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

        Some("Eldritch Knight") => {
            // Seed/update spell slots for this fighter level
            if let Err(e) = spells_db::seed_ek_spell_slots(
                pool, campaign_id, player_id, new_level
            ).await {
                tracing::warn!("Failed to seed EK spell slots at level {}: {}", new_level, e);
            }
 
            match new_level {
                3 => {
                    if !has("War Bond") {
                        let _ = world::create_ability(pool, campaign_id, "player", player_id,
                            "War Bond",
                            Some("Bond with up to 2 weapons (ritual, 1 hour). Bonded weapons can't be disarmed. As a Bonus Action, summon a bonded weapon to your hand from any distance."),
                            1, "manual").await;
                    }
                }
                7 => {
                    if !has("War Magic") {
                        let _ = world::create_ability(pool, campaign_id, "player", player_id,
                            "War Magic",
                            Some("When you take the Attack action, replace one attack with a cantrip that has a casting time of an Action."),
                            1, "manual").await;
                    }
                }
                10 => {
                    if !has("Eldritch Strike") {
                        let _ = world::create_ability(pool, campaign_id, "player", player_id,
                            "Eldritch Strike",
                            Some("When you hit a creature with a weapon, that creature has Disadvantage on the next saving throw it makes against a spell you cast before the end of your next turn."),
                            1, "manual").await;
                    }
                }
                15 => {
                    if !has("Arcane Charge") {
                        let _ = world::create_ability(pool, campaign_id, "player", player_id,
                            "Arcane Charge",
                            Some("When you use Action Surge, you can teleport up to 30 feet to an unoccupied space you can see before or after the additional action."),
                            1, "manual").await;
                    }
                }
                18 => {
                    if !has("Improved War Magic") {
                        let _ = world::create_ability(pool, campaign_id, "player", player_id,
                            "Improved War Magic",
                            Some("When you take the Attack action, you can replace two of your attacks with casting a spell of level 1 or 2 that has a casting time of an Action."),
                            1, "manual").await;
                    }
                }
                _ => {}
            }
        }

        _ => {}
    }
}

async fn seed_level_up_abilities_barbarian(
    pool: &sqlx::SqlitePool,
    campaign_id: &str,
    player_id: &str,
    new_level: i64,
    subclass: Option<&str>,
) {
    let existing = world::get_abilities(pool, player_id, "player").await.unwrap_or_default();
    let has = |name: &str| existing.iter().any(|a| a.name == name);
 
    // ── Base class features ───────────────────────────────────────────────────
 
    match new_level {
        2 => {
            if !has("Danger Sense") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Danger Sense",
                    Some("You have Advantage on Dexterity saving throws unless you have the \
                          Incapacitated condition."),
                    1, "manual").await;
            }
            // Reckless Attack was seeded at character creation; no duplicate needed.
        }
        3 => {
            if !has("Primal Knowledge") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Primal Knowledge",
                    Some("You gain proficiency in one additional skill from the Barbarian list \
                          (Animal Handling, Athletics, Intimidation, Nature, Perception, Survival). \
                          While your Rage is active, you can make Acrobatics, Intimidation, \
                          Perception, Stealth, or Survival checks as Strength checks."),
                    1, "manual").await;
            }
        }
        5 => {
            if !has("Fast Movement") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Fast Movement",
                    Some("Your Speed increases by 10 feet while you aren't wearing Heavy armor."),
                    1, "manual").await;
            }
        }
        7 => {
            if !has("Feral Instinct") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Feral Instinct",
                    Some("You have Advantage on Initiative rolls."),
                    1, "manual").await;
            }
            if !has("Instinctive Pounce") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Instinctive Pounce",
                    Some("As part of the Bonus Action you take to enter your Rage, \
                          you can move up to half your Speed."),
                    1, "manual").await;
            }
        }
        9 => {
            if !has("Brutal Strike") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Brutal Strike",
                    Some("When using Reckless Attack, forgo Advantage on one STR-based attack roll \
                          (it must not have Disadvantage). If it hits: deal +1d10 damage of the \
                          weapon's type and apply one Brutal Strike effect. \
                          Forceful Blow: push target 15 ft straight away, then move up to half your \
                          Speed straight toward them (no Opportunity Attacks). \
                          Hamstring Blow: reduce target Speed by 15 ft until start of your next turn \
                          (only one Hamstring Blow at a time — most recent wins)."),
                    1, "manual").await;
            }
        }
        11 => {
            if !has("Relentless Rage") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Relentless Rage",
                    Some("If you drop to 0 HP while raging and don't die outright, make a DC 10 CON \
                          save. On success, your HP instead change to twice your Barbarian level. \
                          DC increases by 5 each subsequent use; resets to 10 on Short or Long Rest."),
                    1, "manual").await;
            }
        }
        13 => {
            // Add Staggering Blow and Sundering Blow; update the Brutal Strike description.
            if let Some(a) = existing.iter().find(|a| a.name == "Brutal Strike") {
                let _ = sqlx::query("UPDATE abilities SET description = ? WHERE id = ?")
                    .bind("Forgo Reckless Attack advantage on one STR attack → hit deals +1d10 damage \
                           and you apply ONE Brutal Strike effect: \
                           Forceful Blow (push 15 ft + move half Speed toward target), \
                           Hamstring Blow (reduce Speed by 15 ft until your next turn), \
                           NEW — Staggering Blow (target has Disadvantage on its next saving throw \
                           and can't make Opportunity Attacks until your next turn), \
                           NEW — Sundering Blow (next attack by another creature against the target \
                           before your next turn gains +5 to hit).")
                    .bind(&a.id)
                    .execute(pool)
                    .await;
            }
        }
        15 => {
            if !has("Persistent Rage") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Persistent Rage",
                    Some("When you roll Initiative, you can regain all expended Rage uses \
                          (once per Long Rest). \
                          Your Rage now lasts 10 minutes automatically — no action needed each \
                          round to extend it. Rage ends early only if you have the Unconscious \
                          condition or don Heavy armor."),
                    1, "long_rest").await;
            }
        }
        17 => {
            // Improved Brutal Strike upgrade: damage becomes 2d10, can apply TWO effects.
            if let Some(a) = existing.iter().find(|a| a.name == "Brutal Strike") {
                let _ = sqlx::query("UPDATE abilities SET description = ? WHERE id = ?")
                    .bind("Forgo Reckless Attack advantage on one STR attack → hit deals +2d10 damage \
                           and you apply TWO different Brutal Strike effects. \
                           Effects: Forceful Blow, Hamstring Blow, Staggering Blow, Sundering Blow.")
                    .bind(&a.id)
                    .execute(pool)
                    .await;
            }
        }
        18 => {
            if !has("Indomitable Might") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Indomitable Might",
                    Some("If your total for a Strength check or Strength saving throw is less than \
                          your Strength score, you can use your Strength score in place of the total."),
                    1, "manual").await;
            }
        }
        20 => {
            if !has("Primal Champion") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Primal Champion",
                    Some("Your Strength and Constitution scores each increase by 4, to a maximum of 25. \
                          (Applied automatically to your stats.)"),
                    1, "manual").await;
            }
        }
        _ => {}
    }
 
    // ── Subclass features ─────────────────────────────────────────────────────
 
    match subclass {
 
        Some("Path of the Berserker") => match new_level {
            3 => {
                if !has("Frenzy") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Frenzy",
                        Some("While raging, if you use Reckless Attack, you deal extra damage to the \
                              first target you hit that turn with a STR-based attack. Roll a number of \
                              d6s equal to your Rage Damage bonus (+2 = 2d6, +3 = 3d6, +4 = 4d6) and \
                              add them together. Damage type matches the weapon or Unarmed Strike."),
                        1, "manual").await;
                }
            }
            6 => {
                if !has("Mindless Rage") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Mindless Rage",
                        Some("You have Immunity to the Charmed and Frightened conditions while your \
                              Rage is active. If you are Charmed or Frightened when you enter your \
                              Rage, those conditions end on you immediately."),
                        1, "manual").await;
                }
            }
            10 => {
                if !has("Retaliation") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Retaliation",
                        Some("When you take damage from a creature within 5 feet of you, you can use \
                              your Reaction to make one melee weapon attack or Unarmed Strike against \
                              that creature."),
                        1, "per_turn").await;
                }
            }
            14 => {
                if !has("Intimidating Presence") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Intimidating Presence",
                        Some("Bonus Action: each creature of your choice in a 30-foot Emanation must \
                              make a WIS save (DC 8 + STR modifier + Proficiency Bonus) or have the \
                              Frightened condition for 1 minute. A Frightened creature repeats the \
                              save at the end of each of its turns, ending the effect on a success. \
                              Recharges on Long Rest, or expend a Rage use (no action required) to \
                              restore immediately."),
                        1, "long_rest").await;
                }
            }
            _ => {}
        },
 
        Some("Path of the Wild Heart") => match new_level {
            3 => {
                if !has("Animal Speaker") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Animal Speaker",
                        Some("You can cast Beast Sense and Speak with Animals, but only as Rituals. \
                              Wisdom is your spellcasting ability for them."),
                        1, "manual").await;
                }
                if !has("Rage of the Wilds") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Rage of the Wilds",
                        Some("When you activate your Rage, choose one option: \
                              Bear — Resistance to every damage type except Force, Necrotic, Psychic, \
                                and Radiant. \
                              Eagle — On activation, take Disengage and Dash as part of the Bonus \
                                Action; while raging, use a Bonus Action to take both again. \
                              Wolf — While raging, your allies have Advantage on attack rolls against \
                                any enemy of yours within 5 feet of you."),
                        1, "manual").await;
                }
            }
            6 => {
                if !has("Aspect of the Wilds") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Aspect of the Wilds",
                        Some("Gain one passive aspect (change after a Long Rest): \
                              Owl — Darkvision 60 ft, or +60 ft if you already have it. \
                              Panther — Climb Speed equal to your Speed. \
                              Salmon — Swim Speed equal to your Speed."),
                        1, "manual").await;
                }
            }
            10 => {
                if !has("Nature Speaker") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Nature Speaker",
                        Some("You can cast Commune with Nature, but only as a Ritual. \
                              Wisdom is your spellcasting ability for it."),
                        1, "manual").await;
                }
            }
            14 => {
                if !has("Power of the Wilds") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Power of the Wilds",
                        Some("When you activate your Rage, choose one additional option: \
                              Falcon — Fly Speed equal to your Speed (requires no armor). \
                              Lion — Enemies within 5 ft of you have Disadvantage on attack rolls \
                                against any target other than you or another Barbarian with this option. \
                              Ram — When you hit a creature with a melee attack, you can knock it Prone."),
                        1, "manual").await;
                }
            }
            _ => {}
        },
 
        Some("Path of the World Tree") => match new_level {
            3 => {
                if !has("Vitality of the Tree") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Vitality of the Tree",
                        Some("Vitality Surge: when you activate Rage, gain Temporary HP equal to your \
                              Barbarian level. \
                              Life-Giving Force: at the start of each of your turns while raging, \
                              choose a creature within 10 ft — it gains Temporary HP equal to a roll \
                              of d6s equal to your Rage Damage bonus. These temp HP vanish when \
                              your Rage ends."),
                        1, "manual").await;
                }
            }
            6 => {
                if !has("Branches of the Tree") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Branches of the Tree",
                        Some("Reaction: when a creature you can see starts its turn within 30 ft \
                              while your Rage is active, summon spectral branches. Target makes a \
                              STR save (DC 8 + STR mod + Prof Bonus) or is teleported to an \
                              unoccupied space within 5 ft of you (or the nearest unoccupied space \
                              you can see). After teleporting, you can reduce its Speed to 0 until \
                              the end of the current turn."),
                        1, "manual").await;
                }
            }
            10 => {
                if !has("Battering Roots") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Battering Roots",
                        Some("During your turn, your reach is 10 ft greater with any Melee weapon \
                              that has the Heavy or Versatile property. When you hit with such a \
                              weapon on your turn, you can activate the Push or Topple mastery \
                              property in addition to a different mastery property you're already \
                              using with that weapon."),
                        1, "manual").await;
                }
            }
            14 => {
                if !has("Travel along the Tree") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Travel along the Tree",
                        Some("When you activate your Rage, and as a Bonus Action while raging, \
                              you can teleport up to 60 ft to an unoccupied space you can see. \
                              Once per Rage, you can extend that range to 150 ft and bring up to \
                              6 willing creatures within 10 ft — each appears within 10 ft of \
                              your destination."),
                        1, "manual").await;
                }
            }
            _ => {}
        },
 
        Some("Path of the Zealot") => match new_level {
            3 => {
                if !has("Divine Fury") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Divine Fury",
                        Some("While raging, the first creature you hit on each of your turns with \
                              a weapon or Unarmed Strike takes extra damage equal to 1d6 + half your \
                              Barbarian level (round down). Choose Necrotic or Radiant each time \
                              you deal this damage."),
                        1, "manual").await;
                }
                if !has("Warrior of the Gods") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Warrior of the Gods",
                        Some("You have a pool of four d12s. Bonus Action: expend any number of \
                              these dice, roll them, and regain that many Hit Points. The pool \
                              fully restores on Long Rest. Pool grows to 5 dice at level 6, \
                              6 dice at level 12, and 7 dice at level 17."),
                        4, "long_rest").await;
                }
            }
            6 => {
                if !has("Fanatical Focus") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Fanatical Focus",
                        Some("Once per active Rage, when you fail a saving throw, you can reroll \
                              it with a bonus equal to your Rage Damage bonus. You must use the \
                              new roll."),
                        1, "manual").await;
                }
                // Warrior of the Gods grows to 5 dice at level 6
                if let Some(a) = existing.iter().find(|a| a.name == "Warrior of the Gods") {
                    let _ = sqlx::query(
                        "UPDATE abilities SET max_uses = 5, current_uses = 5 WHERE id = ?"
                    )
                    .bind(&a.id)
                    .execute(pool)
                    .await;
                }
            }
            10 => {
                if !has("Zealous Presence") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Zealous Presence",
                        Some("Bonus Action: up to 10 other creatures of your choice within 60 ft \
                              gain Advantage on attack rolls and saving throws until the start of \
                              your next turn. Recharges on Long Rest, or expend a Rage use (no \
                              action required) to restore immediately."),
                        1, "long_rest").await;
                }
            }
            12 => {
                // Warrior of the Gods grows to 6 dice at level 12
                if let Some(a) = existing.iter().find(|a| a.name == "Warrior of the Gods") {
                    let _ = sqlx::query(
                        "UPDATE abilities SET max_uses = 6, current_uses = 6 WHERE id = ?"
                    )
                    .bind(&a.id)
                    .execute(pool)
                    .await;
                }
            }
            14 => {
                if !has("Rage of the Gods") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Rage of the Gods",
                        Some("When you activate your Rage, you can assume a divine warrior form \
                              (lasts 1 minute or until 0 HP). Once per Long Rest. While in this form: \
                              Flight — Fly Speed equal to your Speed, can hover. \
                              Resistance — Resistance to Necrotic, Psychic, and Radiant damage. \
                              Revivification — Reaction: when a creature within 30 ft would drop \
                              to 0 HP, expend a Rage use to change its HP to your Barbarian level \
                              instead."),
                        1, "long_rest").await;
                }
            }
            17 => {
                // Warrior of the Gods grows to 7 dice at level 17
                if let Some(a) = existing.iter().find(|a| a.name == "Warrior of the Gods") {
                    let _ = sqlx::query(
                        "UPDATE abilities SET max_uses = 7, current_uses = 7 WHERE id = ?"
                    )
                    .bind(&a.id)
                    .execute(pool)
                    .await;
                }
            }
            _ => {}
        },
 
        _ => {}
    }
}

async fn seed_level_up_abilities_bard(
    pool: &sqlx::SqlitePool,
    campaign_id: &str,
    player_id: &str,
    new_level: i64,
    subclass: Option<&str>,
) {
    let existing = world::get_abilities(pool, player_id, "player").await.unwrap_or_default();
    let has = |name: &str| existing.iter().any(|a| a.name == name);
 
    // ── Base class features ───────────────────────────────────────────────────
 
    match new_level {
        2 => {
            if !has("Expertise") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Expertise",
                    Some("Choose 2 skill proficiencies you have (Performance and Persuasion recommended). \
                          You gain Expertise in those skills, doubling your Proficiency Bonus for checks \
                          with them. At level 9 you gain Expertise in 2 more skills."),
                    1, "manual").await;
            }
        }
        5 => {
            if !has("Font of Inspiration") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Font of Inspiration",
                    Some("You now regain all expended uses of Bardic Inspiration when you finish a \
                          Short or Long Rest. In addition, you can expend a spell slot (no action \
                          required) to regain one expended use of Bardic Inspiration."),
                    1, "manual").await;
            }
            // Already handled in level_up_player: Bardic Inspiration refresh_type → short_rest
        }
        7 => {
            if !has("Countercharm") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Countercharm",
                    Some("Reaction: when you or a creature within 30 ft fails a saving throw against \
                          an effect that applies the Charmed or Frightened condition, cause the save \
                          to be rerolled with Advantage."),
                    1, "per_turn").await;
            }
        }
        10 => {
            if !has("Magical Secrets") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Magical Secrets",
                    Some("Whenever the Prepared Spells number increases, you can choose any new \
                          prepared spells from the Bard, Cleric, Druid, or Wizard spell lists — \
                          they count as Bard spells. You can also replace prepared spells with \
                          spells from those lists."),
                    1, "manual").await;
            }
        }
        18 => {
            if !has("Superior Inspiration") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Superior Inspiration",
                    Some("When you roll Initiative, if you have fewer than 2 uses of Bardic \
                          Inspiration remaining, you regain uses until you have 2."),
                    1, "manual").await;
            }
        }
        20 => {
            if !has("Words of Creation") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Words of Creation",
                    Some("You always have Power Word Heal and Power Word Kill prepared. When you cast \
                          either spell, you can target a second creature within 10 ft of the first target."),
                    1, "manual").await;
            }
        }
        _ => {}
    }
 
    // ── Subclass features ─────────────────────────────────────────────────────
 
    match subclass {
 
        Some("College of Dance") => match new_level {
            3 => {
                if !has("Dazzling Footwork") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Dazzling Footwork",
                        Some("While not wearing armor or wielding a Shield: \
                              Unarmored Defense — AC = 10 + DEX mod + CHA mod. \
                              Dance Virtuoso — Advantage on CHA (Performance) checks involving dancing. \
                              Agile Strikes — when you expend a Bardic Inspiration die as part of an \
                                action/bonus/reaction, you can make one Unarmed Strike as part of it. \
                              Bardic Damage — use DEX for Unarmed Strike attack rolls; deal Bludgeoning \
                                damage equal to a Bardic Inspiration die roll + DEX mod (die not expended)."),
                        1, "manual").await;
                }
            }
            6 => {
                if !has("Inspiring Movement") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Inspiring Movement",
                        Some("Reaction: when an enemy you can see ends its turn within 5 ft of you, \
                              expend a Bardic Inspiration use to move up to half your Speed. Then one \
                              ally within 30 ft can also move up to half their Speed using their Reaction. \
                              None of this movement provokes Opportunity Attacks."),
                        1, "per_turn").await;
                }
                if !has("Tandem Footwork") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Tandem Footwork",
                        Some("When you roll Initiative (without being Incapacitated), expend a Bardic \
                              Inspiration use: roll the die and add that number to your Initiative and \
                              to the Initiative of each ally within 30 ft who can see or hear you."),
                        1, "per_turn").await;
                }
            }
            14 => {
                if !has("Leading Evasion") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Leading Evasion",
                        Some("When you are subjected to an effect requiring a DEX save to take half \
                              damage, you take no damage on a success and only half damage on a failure. \
                              You can share this benefit with creatures within 5 ft making the same save. \
                              Unavailable if Incapacitated."),
                        1, "manual").await;
                }
            }
            _ => {}
        },
 
        Some("College of Glamour") => match new_level {
            3 => {
                if !has("Beguiling Magic") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Beguiling Magic",
                        Some("You always have Charm Person and Mirror Image prepared. \
                              Immediately after casting an Enchantment or Illusion spell using a slot, \
                              you can force a creature within 60 ft to make a WIS save (DC = spell save DC). \
                              On fail: Charmed or Frightened (your choice) for 1 minute, repeating the \
                              save at end of each of its turns. Recharges on Long Rest, or expend a \
                              Bardic Inspiration use to restore immediately."),
                        1, "long_rest").await;
                }
                if !has("Mantle of Inspiration") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Mantle of Inspiration",
                        Some("Bonus Action: expend a Bardic Inspiration use and roll the die. Choose up \
                              to CHA modifier creatures (min 1) within 60 ft. Each gains Temporary HP \
                              equal to twice the roll, then can use their Reaction to move up to their \
                              Speed without provoking Opportunity Attacks."),
                        1, "manual").await;
                }
            }
            6 => {
                if !has("Mantle of Majesty") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Mantle of Majesty",
                        Some("You always have Command prepared. Bonus Action: cast Command without \
                              expending a slot and take on an unearthly appearance for 1 minute \
                              (or until Concentration ends). During this time, cast Command as a \
                              Bonus Action without a slot. Creatures Charmed by you automatically \
                              fail saves against your Command. Recharges on Long Rest, or expend a \
                              level 3+ spell slot to restore."),
                        1, "long_rest").await;
                }
            }
            14 => {
                if !has("Unbreakable Majesty") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Unbreakable Majesty",
                        Some("Bonus Action: assume a magically majestic presence for 1 minute \
                              (or until Incapacitated). While active, the first time any creature \
                              hits you with an attack roll on a turn, it must succeed on a CHA save \
                              (DC = your spell save DC) or the attack misses. \
                              Recharges on Short or Long Rest."),
                        1, "short_rest").await;
                }
            }
            _ => {}
        },
 
        Some("College of Lore") => match new_level {
            3 => {
                if !has("Bonus Proficiencies") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Bonus Proficiencies",
                        Some("You gain proficiency in 3 additional skills of your choice."),
                        1, "manual").await;
                }
                if !has("Cutting Words") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Cutting Words",
                        Some("Reaction: when a creature you can see within 60 ft makes a damage roll \
                              or succeeds on an ability check or attack roll, expend a Bardic Inspiration \
                              use, roll the die, and subtract the result from the creature's roll — \
                              potentially turning a success into a failure."),
                        1, "per_turn").await;
                }
            }
            6 => {
                if !has("Magical Discoveries") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Magical Discoveries",
                        Some("Choose 2 spells from the Cleric, Druid, or Wizard spell list (or any \
                              combination). The chosen spells must be cantrips or spells for which you \
                              have Bard spell slots. You always have them prepared. Whenever you gain a \
                              Bard level, you can replace one with another spell meeting these criteria."),
                        1, "manual").await;
                }
            }
            14 => {
                if !has("Peerless Skill") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Peerless Skill",
                        Some("When you fail an ability check or attack roll, expend a Bardic Inspiration \
                              use: roll the die and add the result to the d20, potentially turning a \
                              failure into a success. If it still fails, the Bardic Inspiration is \
                              not expended."),
                        1, "manual").await;
                }
            }
            _ => {}
        },
 
        Some("College of Valor") => match new_level {
            3 => {
                if !has("Combat Inspiration") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Combat Inspiration",
                        Some("A creature with your Bardic Inspiration die can use it for: \
                              Defense — Reaction when hit: roll the die and add it to AC against \
                                that attack, potentially causing a miss. \
                              Offense — immediately after hitting a target: roll the die and add it \
                                to the attack's damage."),
                        1, "manual").await;
                }
                if !has("Martial Training") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Martial Training",
                        Some("You gain proficiency with Martial weapons and training with Medium armor \
                              and Shields. You can also use a Simple or Martial weapon as a Spellcasting \
                              Focus for your Bard spells."),
                        1, "manual").await;
                }
            }
            6 => {
                if !has("Extra Attack") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Extra Attack",
                        Some("You can attack twice instead of once when you take the Attack action. \
                              In addition, you can replace one of those attacks with a cantrip that \
                              has a casting time of an action."),
                        1, "manual").await;
                }
            }
            14 => {
                if !has("Battle Magic") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Battle Magic",
                        Some("After you cast a spell that has a casting time of an action, you can \
                              make one attack with a weapon as a Bonus Action."),
                        1, "manual").await;
                }
            }
            _ => {}
        },
 
        _ => {}
    }
}

async fn seed_level_up_abilities_cleric(
    pool: &sqlx::SqlitePool,
    campaign_id: &str,
    player_id: &str,
    new_level: i64,
    subclass: Option<&str>,
) {
    let existing = world::get_abilities(pool, player_id, "player").await.unwrap_or_default();
    let has = |name: &str| existing.iter().any(|a| a.name == name);
 
    // Fetch player to get WIS modifier for ability use counts that scale with it
    let player = match player::get_player(pool, player_id).await {
        Ok(Some(p)) => p,
        _ => return,
    };
    let wis_mod = crate::models::Player::modifier(player.wis).max(1);
 
    // ── Base class features ───────────────────────────────────────────────────
 
    match new_level {
        2 => {
            // Channel Divinity unlocks at level 2 — update from 0 to 2 uses
            sqlx::query(
                "UPDATE abilities SET max_uses = 2, current_uses = 2
                 WHERE owner_id = ? AND name = 'Channel Divinity'"
            )
            .bind(player_id)
            .execute(pool)
            .await.ok();
        }
        5 => {
            if !has("Sear Undead") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Sear Undead",
                    Some("Whenever you use Turn Undead, roll a number of d8s equal to your WIS \
                          modifier (min 1d8) and add them together. Each Undead that fails its \
                          saving throw against that Turn Undead takes Radiant damage equal to the \
                          roll's total. This damage doesn't end the Turned condition."),
                    1, "manual").await;
            }
        }
        6 => {
            // Channel Divinity increases to 3 uses
            let cd_desc = "Use Channel Divinity 3 times per rest (regain 1 on Short Rest, all on \
                           Long Rest). Effects: Divine Spark (heal or deal Necrotic/Radiant damage \
                           equal to 1d8+WIS, scaling to 2d8 at L7 and 3d8 at L13) and Turn Undead \
                           (WIS save or Frightened+Incapacitated), plus your domain feature.";
            sqlx::query(
                "UPDATE abilities SET max_uses = 3, current_uses = 3, description = ?
                 WHERE owner_id = ? AND name = 'Channel Divinity'"
            )
            .bind(cd_desc)
            .bind(player_id)
            .execute(pool)
            .await.ok();
        }
        7 => {
            if !has("Blessed Strikes") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Blessed Strikes",
                    Some("Choose one option (inform the DM of your choice): \
                          Divine Strike — once per turn when you hit with a weapon attack, deal \
                            +1d8 Necrotic or Radiant damage (your choice each time). \
                          Potent Spellcasting — add your WIS modifier to damage dealt by Cleric \
                            cantrips. \
                          At level 14 (Improved): Divine Strike increases to +2d8. Potent \
                          Spellcasting also grants Temporary HP equal to 2×WIS modifier to yourself \
                          or a creature within 60 ft when you deal cantrip damage."),
                    1, "manual").await;
            }
            // Update Divine Spark description to reflect 2d8
            let cd_desc = "Use Channel Divinity 3 times per rest (regain 1 on Short Rest, all on \
                           Long Rest). Effects: Divine Spark (heal or deal 2d8+WIS Necrotic/Radiant, \
                           scales to 3d8 at L13 and 4d8 at L18), Turn Undead (WIS save or \
                           Frightened+Incapacitated), plus your domain feature.";
            sqlx::query(
                "UPDATE abilities SET description = ? WHERE owner_id = ? AND name = 'Channel Divinity'"
            )
            .bind(cd_desc)
            .bind(player_id)
            .execute(pool)
            .await.ok();
        }
        10 => {
            if !has("Divine Intervention") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Divine Intervention",
                    Some("Magic action: choose any Cleric spell of level 5 or lower that doesn't \
                          require a Reaction. Cast it without expending a spell slot or needing \
                          Material components. Recharges on Long Rest. \
                          At level 20 (Greater Divine Intervention): you may also choose the Wish \
                          spell — if you do, you can't use Divine Intervention again until you \
                          finish 2d4 Long Rests."),
                    1, "long_rest").await;
            }
        }
        13 => {
            // Update Divine Spark description to 3d8
            let cd_desc = "Use Channel Divinity 3 times per rest (regain 1 on Short Rest, all on \
                           Long Rest). Effects: Divine Spark (heal or deal 3d8+WIS Necrotic/Radiant, \
                           scales to 4d8 at L18), Turn Undead (WIS save or Frightened+Incapacitated), \
                           plus your domain feature.";
            sqlx::query(
                "UPDATE abilities SET description = ? WHERE owner_id = ? AND name = 'Channel Divinity'"
            )
            .bind(cd_desc)
            .bind(player_id)
            .execute(pool)
            .await.ok();
        }
        18 => {
            // Channel Divinity increases to 4 uses; Divine Spark becomes 4d8
            let cd_desc = "Use Channel Divinity 4 times per rest (regain 1 on Short Rest, all on \
                           Long Rest). Effects: Divine Spark (heal or deal 4d8+WIS Necrotic/Radiant), \
                           Turn Undead (WIS save or Frightened+Incapacitated), plus your domain feature.";
            sqlx::query(
                "UPDATE abilities SET max_uses = 4, current_uses = 4, description = ?
                 WHERE owner_id = ? AND name = 'Channel Divinity'"
            )
            .bind(cd_desc)
            .bind(player_id)
            .execute(pool)
            .await.ok();
        }
        _ => {}
    }
 
    // ── Subclass features ─────────────────────────────────────────────────────
 
    match subclass {
 
        Some("Life Domain") => match new_level {
            3 => {
                if !has("Disciple of Life") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Disciple of Life",
                        Some("When a spell you cast with a spell slot restores HP to a creature, \
                              that creature regains additional HP equal to 2 + the spell slot's level \
                              on the turn you cast the spell."),
                        1, "manual").await;
                }
                if !has("Preserve Life") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Preserve Life",
                        Some("Magic action: expend a Channel Divinity use to restore HP equal to \
                              5 × your Cleric level, divided among any Bloodied creatures within \
                              30 ft of your choice (including yourself). Cannot restore a creature \
                              beyond half its HP maximum."),
                        1, "manual").await;
                }
                if !has("Life Domain Spells") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Life Domain Spells",
                        Some("Always prepared (don't count against your limit): \
                              L3: Aid, Bless, Cure Wounds, Lesser Restoration. \
                              L5: Mass Healing Word, Revivify. \
                              L7: Aura of Life, Death Ward. \
                              L9: Greater Restoration, Mass Cure Wounds."),
                        1, "manual").await;
                }
            }
            6 => {
                if !has("Blessed Healer") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Blessed Healer",
                        Some("Immediately after you cast a spell with a spell slot that restores HP \
                              to one or more creatures other than yourself, you regain HP equal to \
                              2 + the spell slot's level."),
                        1, "manual").await;
                }
            }
            17 => {
                if !has("Supreme Healing") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Supreme Healing",
                        Some("When you would normally roll dice to restore HP with a spell or \
                              Channel Divinity, don't roll — use the highest possible number for \
                              each die instead. (e.g., 2d6 healing always becomes 12.)"),
                        1, "manual").await;
                }
            }
            _ => {}
        },
 
        Some("Light Domain") => match new_level {
            3 => {
                if !has("Warding Flare") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Warding Flare",
                        Some("Reaction: when a creature you can see within 30 ft makes an attack \
                              roll, impose Disadvantage on that roll as light flares. \
                              Uses = WIS modifier (min 1). Recharges on Long Rest. \
                              At level 6 (Improved Warding Flare): recharges on Short or Long Rest; \
                              also grants the attack's target Temporary HP equal to 2d6 + WIS mod."),
                        wis_mod, "long_rest").await;
                }
                if !has("Radiance of the Dawn") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Radiance of the Dawn",
                        Some("Magic action: expend a Channel Divinity use. Emit a flash of light \
                              in a 30-foot Emanation — dispels magical Darkness in the area. \
                              Each creature of your choice makes a CON save (DC = your spell save DC): \
                              fail: 2d10 + Cleric level Radiant damage; success: half."),
                        1, "manual").await;
                }
                if !has("Light Domain Spells") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Light Domain Spells",
                        Some("Always prepared: \
                              L3: Burning Hands, Faerie Fire, Scorching Ray, See Invisibility. \
                              L5: Daylight, Fireball. \
                              L7: Arcane Eye, Wall of Fire. \
                              L9: Flame Strike, Scrying."),
                        1, "manual").await;
                }
            }
            6 => {
                if !has("Improved Warding Flare") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Improved Warding Flare",
                        Some("Warding Flare now recharges on Short or Long Rest. When you use \
                              Warding Flare, the target of the triggering attack gains Temporary HP \
                              equal to 2d6 + your WIS modifier."),
                        1, "manual").await;
                }
                // Update Warding Flare refresh type to short_rest
                if let Some(a) = existing.iter().find(|a| a.name == "Warding Flare") {
                    sqlx::query(
                        "UPDATE abilities SET refresh_type = 'short_rest' WHERE id = ?"
                    )
                    .bind(&a.id)
                    .execute(pool)
                    .await.ok();
                }
            }
            17 => {
                if !has("Corona of Light") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Corona of Light",
                        Some("Magic action: emit an aura of sunlight for 1 minute (dismiss freely). \
                              Bright Light 60-foot radius, Dim Light for 30 ft beyond that. \
                              Enemies in the Bright Light have Disadvantage on saving throws \
                              against your Radiance of the Dawn and any spell dealing Fire or \
                              Radiant damage. Uses = WIS modifier (min 1). Recharges on Long Rest."),
                        wis_mod, "long_rest").await;
                }
            }
            _ => {}
        },
 
        Some("Trickery Domain") => match new_level {
            3 => {
                if !has("Blessing of the Trickster") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Blessing of the Trickster",
                        Some("Magic action: choose yourself or a willing creature within 30 ft. \
                              That creature has Advantage on DEX (Stealth) checks until you finish \
                              a Long Rest or use this feature again."),
                        1, "manual").await;
                }
                if !has("Invoke Duplicity") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Invoke Duplicity",
                        Some("Bonus Action: expend a Channel Divinity use to create a perfect visual \
                              illusion of yourself in an unoccupied space within 30 ft (lasts 1 min, \
                              ends if dismissed or Incapacitated). The illusion is intangible. \
                              While it persists — Cast Spells: from the illusion's space (your senses). \
                              Distract: Advantage on attacks against creatures within 5 ft of it. \
                              Move: Bonus Action to move it up to 30 ft (within 120 ft of you). \
                              At level 6 (Trickster's Transposition): when you create or move the \
                              illusion, you can teleport and swap places with it."),
                        1, "manual").await;
                }
                if !has("Trickery Domain Spells") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Trickery Domain Spells",
                        Some("Always prepared: \
                              L3: Charm Person, Disguise Self, Invisibility, Pass without Trace. \
                              L5: Hypnotic Pattern, Nondetection. \
                              L7: Confusion, Dimension Door. \
                              L9: Dominate Person, Modify Memory."),
                        1, "manual").await;
                }
            }
            6 => {
                if !has("Trickster's Transposition") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Trickster's Transposition",
                        Some("Whenever you take the Bonus Action to create or move your Invoke \
                              Duplicity illusion, you can teleport and swap places with it."),
                        1, "manual").await;
                }
            }
            17 => {
                if !has("Improved Duplicity") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Improved Duplicity",
                        Some("Shared Distraction: you and your allies have Advantage on attack \
                              rolls against any creature within 5 ft of your Invoke Duplicity \
                              illusion (previously only you had Advantage). \
                              Healing Illusion: when the illusion ends, you or a creature of your \
                              choice within 5 ft of it regains HP equal to your Cleric level."),
                        1, "manual").await;
                }
            }
            _ => {}
        },
 
        Some("War Domain") => match new_level {
            3 => {
                if !has("Guided Strike") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Guided Strike",
                        Some("When you or a creature within 30 ft misses with an attack roll, \
                              expend a Channel Divinity use to add +10 to that roll, potentially \
                              turning the miss into a hit. To use it for another creature's attack \
                              you must take a Reaction."),
                        1, "manual").await;
                }
                if !has("War Priest") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "War Priest",
                        Some("Bonus Action: make one attack with a weapon or Unarmed Strike. \
                              Uses = WIS modifier (min 1). Recharges on Short or Long Rest."),
                        wis_mod, "short_rest").await;
                }
                if !has("War Domain Spells") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "War Domain Spells",
                        Some("Always prepared: \
                              L3: Guiding Bolt, Magic Weapon, Shield of Faith, Spiritual Weapon. \
                              L5: Crusader's Mantle, Spirit Guardians. \
                              L7: Fire Shield, Freedom of Movement. \
                              L9: Hold Monster, Steel Wind Strike."),
                        1, "manual").await;
                }
            }
            6 => {
                if !has("War God's Blessing") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "War God's Blessing",
                        Some("Expend a Channel Divinity use to cast Shield of Faith or Spiritual \
                              Weapon without a spell slot. The spell lasts 1 minute without \
                              requiring Concentration. It ends early if you cast the same spell \
                              again, have the Incapacitated condition, or die."),
                        1, "manual").await;
                }
            }
            17 => {
                if !has("Avatar of Battle") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Avatar of Battle",
                        Some("You gain Resistance to Bludgeoning, Piercing, and Slashing damage."),
                        1, "manual").await;
                }
            }
            _ => {}
        },
 
        _ => {}
    }
}

async fn seed_level_up_abilities_druid(
    pool: &sqlx::SqlitePool,
    campaign_id: &str,
    player_id: &str,
    new_level: i64,
    subclass: Option<&str>,
) {
    let existing = world::get_abilities(pool, player_id, "player").await.unwrap_or_default();
    let has = |name: &str| existing.iter().any(|a| a.name == name);
 
    let player = match player::get_player(pool, player_id).await {
        Ok(Some(p)) => p,
        _ => return,
    };
    let wis_mod = crate::models::Player::modifier(player.wis).max(1);
 
    // ── Base class features ───────────────────────────────────────────────────
 
    match new_level {
        2 => {
            if !has("Wild Shape") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Wild Shape",
                    Some("Bonus Action: shapeshift into a known Beast form (CR 1/4 max, 4 forms). \
                          You retain your HP, hit dice, INT/WIS/CHA, class features, languages, feats, \
                          and skill/saving throw proficiencies. Gain Temporary HP equal to your Druid \
                          level when shifting. You can't cast spells while shifted, but existing \
                          Concentration isn't broken. Lasts half your Druid level in hours. \
                          Regain 1 use on Short Rest, all on Long Rest."),
                    2, "long_rest").await;
            }
            if !has("Wild Companion") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Wild Companion",
                    Some("Magic action: expend a spell slot or Wild Shape use to cast Find Familiar \
                          without Material components. The familiar is Fey and disappears when you \
                          finish a Long Rest."),
                    1, "manual").await;
            }
        }
        5 => {
            if !has("Wild Resurgence") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Wild Resurgence",
                    Some("Once per turn, if you have no Wild Shape uses remaining, you can expend \
                          a spell slot (no action required) to regain one Wild Shape use. \
                          Additionally, once per Long Rest, you can expend one Wild Shape use \
                          (no action required) to give yourself a level 1 spell slot."),
                    1, "manual").await;
            }
        }
        7 => {
            if !has("Elemental Fury") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Elemental Fury",
                    Some("Choose one option (inform the DM): \
                          Potent Spellcasting — add your WIS modifier to damage dealt by Druid cantrips. \
                            At level 15: range of affected cantrips (10 ft+) increases by 300 ft. \
                          Primal Strike — once per turn when you hit with a weapon or Beast form \
                            attack, deal +1d8 Cold, Fire, Lightning, or Thunder damage (choose each hit). \
                            At level 15: increases to +2d8."),
                    1, "manual").await;
            }
        }
        18 => {
            if !has("Beast Spells") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Beast Spells",
                    Some("While using Wild Shape, you can cast Druid spells in Beast form, \
                          except for spells with a Material component that has a cost or is consumed."),
                    1, "manual").await;
            }
        }
        20 => {
            if !has("Archdruid") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Archdruid",
                    Some("Evergreen Wild Shape: when you roll Initiative with no Wild Shape uses \
                          remaining, regain one use. \
                          Nature Magician (once per Long Rest): convert Wild Shape uses into a single \
                          spell slot — each use contributes 2 spell levels (e.g. 2 uses = level 4 slot). \
                          Longevity: primal magic causes you to age one year for every ten that pass."),
                    1, "long_rest").await;
            }
        }
        _ => {}
    }
 
    // ── Subclass features ─────────────────────────────────────────────────────
 
    match subclass {
 
        Some("Circle of the Land") => match new_level {
            3 => {
                if !has("Circle of the Land Spells") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Circle of the Land Spells",
                        Some("Whenever you finish a Long Rest, choose a land type. You have those \
                              circle spells prepared (they don't count against your limit). \
                              Arid: Blur, Burning Hands, Fire Bolt (L3); Fireball (L5); Blight (L7); Wall of Stone (L9). \
                              Polar: Fog Cloud, Hold Person, Ray of Frost (L3); Sleet Storm (L5); Ice Storm (L7); Cone of Cold (L9). \
                              Temperate: Misty Step, Shocking Grasp, Sleep (L3); Lightning Bolt (L5); Freedom of Movement (L7); Tree Stride (L9). \
                              Tropical: Acid Splash, Ray of Sickness, Web (L3); Stinking Cloud (L5); Polymorph (L7); Insect Plague (L9)."),
                        1, "manual").await;
                }
                if !has("Land's Aid") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Land's Aid",
                        Some("Magic action: expend a Wild Shape use and choose a point within 60 ft. \
                              Flowers and thorns appear in a 10-foot Sphere for a moment. \
                              Each creature of your choice makes a CON save (DC = spell save DC): \
                              fail: 2d6 Necrotic damage; success: half. One creature of your choice \
                              in the area regains 2d6 HP. Damage and healing increase to 3d6 at \
                              level 10, 4d6 at level 14."),
                        1, "manual").await;
                }
            }
            6 => {
                if !has("Natural Recovery") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Natural Recovery",
                        Some("Once per Long Rest: cast one prepared Circle Spell of level 1+ without \
                              expending a spell slot. \
                              On Short Rest: recover expended spell slots with combined level ≤ half \
                              your Druid level (round up), no slot of level 6+. \
                              (e.g. level 6 Druid: recover up to 3 levels of slots total.)"),
                        1, "long_rest").await;
                }
            }
            10 => {
                if !has("Nature's Ward") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Nature's Ward",
                        Some("You are immune to the Poisoned condition. You also have Resistance to a \
                              damage type based on your current land choice: \
                              Arid → Fire, Polar → Cold, Temperate → Lightning, Tropical → Poison."),
                        1, "manual").await;
                }
            }
            14 => {
                if !has("Nature's Sanctuary") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Nature's Sanctuary",
                        Some("Magic action: expend a Wild Shape use to summon spectral trees and vines \
                              in a 15-foot Cube on the ground within 120 ft (lasts 1 min or until \
                              Incapacitated/dead). You and allies in the area have Half Cover and \
                              gain the Nature's Ward Resistance for your current land. \
                              Bonus Action: move the Cube up to 60 ft (within 120 ft of you)."),
                        1, "manual").await;
                }
            }
            _ => {}
        },
 
        Some("Circle of the Moon") => match new_level {
            3 => {
                if !has("Circle Forms") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Circle Forms",
                        Some("When you use Wild Shape, you gain enhanced lunar benefits: \
                              Max CR equals your Druid level divided by 3 (round down) — \
                                e.g. CR 1 at level 3, CR 2 at level 6, CR 6 at level 18. \
                              AC equals 13 + WIS modifier if that total is higher than the Beast's AC. \
                              Temporary HP equals three times your Druid level (instead of once)."),
                        1, "manual").await;
                }
                if !has("Circle of the Moon Spells") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Circle of the Moon Spells",
                        Some("Always prepared: Cure Wounds, Moonbeam, Starry Wisp (L3); \
                              Conjure Animals (L5); Fount of Moonlight (L7); Mass Cure Wounds (L9). \
                              You can also cast these spells while in Wild Shape form."),
                        1, "manual").await;
                }
                // Learn the level 3 always-prepared spells
                for spell_name in &["Cure Wounds", "Moonbeam", "Starry Wisp"] {
                    if let Ok(Some(spell)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = spell["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "circle_of_the_moon").await;
                        }
                    }
                }
            }
            5 => {
                // Conjure Animals becomes available (level 5 circle spell)
                if let Ok(Some(spell)) = spells_db::get_spell_by_name(pool, "Conjure Animals").await {
                    if let Some(id) = spell["id"].as_str() {
                        let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "circle_of_the_moon").await;
                    }
                }
            }
            6 => {
                if !has("Improved Circle Forms") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Improved Circle Forms",
                        Some("While in a Wild Shape form: \
                              Lunar Radiance — each of your attacks can deal its normal damage type \
                                or Radiant damage (choose each time you hit). \
                              Increased Toughness — add your WIS modifier to Constitution saving throws."),
                        1, "manual").await;
                }
            }
            7 => {
                // Fount of Moonlight becomes available
                if let Ok(Some(spell)) = spells_db::get_spell_by_name(pool, "Fount of Moonlight").await {
                    if let Some(id) = spell["id"].as_str() {
                        let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "circle_of_the_moon").await;
                    }
                }
            }
            9 => {
                // Mass Cure Wounds becomes available
                if let Ok(Some(spell)) = spells_db::get_spell_by_name(pool, "Mass Cure Wounds").await {
                    if let Some(id) = spell["id"].as_str() {
                        let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "circle_of_the_moon").await;
                    }
                }
            }
            10 => {
                if !has("Moonlight Step") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Moonlight Step",
                        Some("Bonus Action: teleport up to 30 ft to an unoccupied space you can see, \
                              then have Advantage on the next attack roll you make before the end of \
                              your turn. Uses = WIS modifier (min 1). Recharges on Long Rest. \
                              You can also restore uses by expending a level 2+ spell slot per use \
                              (no action required)."),
                        wis_mod, "long_rest").await;
                }
            }
            14 => {
                if !has("Lunar Form") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Lunar Form",
                        Some("Improved Lunar Radiance: once per turn, deal an extra 2d10 Radiant \
                              damage to a target you hit with a Wild Shape form attack. \
                              Shared Moonlight: when you use Moonlight Step, you can also teleport \
                              one willing creature within 10 ft of you to an unoccupied space within \
                              10 ft of your destination."),
                        1, "manual").await;
                }
            }
            _ => {}
        },
 
        Some("Circle of the Sea") => match new_level {
            3 => {
                if !has("Circle of the Sea Spells") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Circle of the Sea Spells",
                        Some("Always prepared: Fog Cloud, Gust of Wind, Ray of Frost, Shatter, Thunderwave (L3); \
                              Lightning Bolt, Water Breathing (L5); Control Water, Ice Storm (L7); \
                              Conjure Elemental, Hold Monster (L9)."),
                        1, "manual").await;
                }
                if !has("Wrath of the Sea") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Wrath of the Sea",
                        Some("Bonus Action: expend a Wild Shape use to manifest a 5-foot Emanation \
                              of ocean spray around you for 10 minutes (dismiss freely; ends if \
                              Incapacitated or re-manifested). \
                              When manifesting and as a Bonus Action on subsequent turns, choose a \
                              creature you can see in the Emanation — it makes a CON save (DC = spell \
                              save DC) or take Cold damage and, if Large or smaller, be pushed up to \
                              15 ft away. Damage = roll d6s equal to your WIS modifier (min 1d6). \
                              Level 6: Emanation grows to 10 ft."),
                        1, "manual").await;
                }
                // Learn the level 3 always-prepared Sea spells
                for spell_name in &["Fog Cloud", "Gust of Wind", "Ray of Frost", "Shatter", "Thunderwave"] {
                    if let Ok(Some(spell)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = spell["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "circle_of_the_sea").await;
                        }
                    }
                }
            }
            5 => {
                for spell_name in &["Lightning Bolt", "Water Breathing"] {
                    if let Ok(Some(spell)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = spell["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "circle_of_the_sea").await;
                        }
                    }
                }
            }
            6 => {
                if !has("Aquatic Affinity") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Aquatic Affinity",
                        Some("The Emanation of your Wrath of the Sea increases to 10 feet. \
                              In addition, you gain a Swim Speed equal to your Speed."),
                        1, "manual").await;
                }
            }
            7 => {
                for spell_name in &["Control Water", "Ice Storm"] {
                    if let Ok(Some(spell)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = spell["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "circle_of_the_sea").await;
                        }
                    }
                }
            }
            9 => {
                for spell_name in &["Conjure Elemental", "Hold Monster"] {
                    if let Ok(Some(spell)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = spell["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "circle_of_the_sea").await;
                        }
                    }
                }
            }
            10 => {
                if !has("Stormborn") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Stormborn",
                        Some("While your Wrath of the Sea Emanation is active: \
                              Flight — you gain a Fly Speed equal to your Speed. \
                              Resistance — you have Resistance to Cold, Lightning, and Thunder damage."),
                        1, "manual").await;
                }
            }
            14 => {
                if !has("Oceanic Gift") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Oceanic Gift",
                        Some("When manifesting Wrath of the Sea, you can instead manifest it around \
                              one willing creature within 60 ft — that creature gains all benefits of \
                              the Emanation and uses your spell save DC and WIS modifier. \
                              You can also manifest the Emanation around both that creature and yourself \
                              by expending two Wild Shape uses instead of one."),
                        1, "manual").await;
                }
            }
            _ => {}
        },
 
        Some("Circle of the Stars") => match new_level {
            3 => {
                if !has("Star Map") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Star Map",
                        Some("You have a star chart (a Tiny object usable as a Spellcasting Focus). \
                              While holding it, you have Guidance and Guiding Bolt always prepared. \
                              Free Guiding Bolt: cast without a spell slot a number of times equal to \
                              your WIS modifier (min 1). Recharges on Long Rest. \
                              If lost, perform a 1-hour ceremony (during a Short or Long Rest) to \
                              create a replacement."),
                        wis_mod, "long_rest").await;
                }
                if !has("Starry Form") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Starry Form",
                        Some("Bonus Action: expend a Wild Shape use to take on a starry form for \
                              10 minutes (dismiss freely; ends if Incapacitated or used again). \
                              Your body becomes luminous (Bright Light 10 ft, Dim Light 10 ft beyond). \
                              Choose one constellation: \
                              Archer — On activation and as a Bonus Action each turn: ranged spell \
                                attack for 1d8+WIS Radiant damage against one creature within 60 ft. \
                              Chalice — When you cast a healing spell with a slot, you or another \
                                creature within 30 ft regains 1d8+WIS HP. \
                              Dragon — INT/WIS checks and CON saves for Concentration treat a d20 \
                                roll of 9 or lower as a 10. \
                              Level 10 (Twinkling Constellations): Archer and Chalice dice become 2d8; \
                              Dragon grants Fly Speed 20 ft + hover; you can change constellations \
                              at start of each turn."),
                        1, "manual").await;
                }
                // Learn Guidance and Guiding Bolt as always-prepared
                for spell_name in &["Guidance", "Guiding Bolt"] {
                    if let Ok(Some(spell)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = spell["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "circle_of_the_stars").await;
                        }
                    }
                }
            }
            6 => {
                if !has("Cosmic Omen") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Cosmic Omen",
                        Some("After each Long Rest, roll a die: even = Weal, odd = Woe. Until your \
                              next Long Rest, when a creature you can see within 30 ft is about to \
                              make a D20 Test, you can take a Reaction to roll 1d6 and: \
                              Weal (even): add the roll to the total. \
                              Woe (odd): subtract the roll from the total. \
                              Uses = WIS modifier (min 1). Recharges on Long Rest."),
                        wis_mod, "long_rest").await;
                }
            }
            10 => {
                // Update Starry Form description (Twinkling Constellations) — already included
                // in the level 3 Starry Form description for forward-looking players.
                // Separately note the upgrade:
                if !has("Twinkling Constellations") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Twinkling Constellations",
                        Some("Your Starry Form constellations improve: \
                              Archer and Chalice dice increase to 2d8. \
                              Dragon grants a Fly Speed of 20 feet and the ability to hover. \
                              At the start of each of your turns in Starry Form, you can change \
                              which constellation glimmers on your body."),
                        1, "manual").await;
                }
            }
            14 => {
                if !has("Full of Stars") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Full of Stars",
                        Some("While in your Starry Form, you become partially incorporeal, giving \
                              you Resistance to Bludgeoning, Piercing, and Slashing damage."),
                        1, "manual").await;
                }
            }
            _ => {}
        },
 
        _ => {}
    }
}

async fn seed_level_up_abilities_monk(
    pool: &sqlx::SqlitePool,
    campaign_id: &str,
    player_id: &str,
    new_level: i64,
    subclass: Option<&str>,
) {
    let existing = world::get_abilities(pool, player_id, "player").await.unwrap_or_default();
    let has = |name: &str| existing.iter().any(|a| a.name == name);
 
    let player = match player::get_player(pool, player_id).await {
        Ok(Some(p)) => p,
        _ => return,
    };
    let wis_mod = crate::models::Player::modifier(player.wis).max(1);
 
    // ── Base class features ───────────────────────────────────────────────────
 
    match new_level {
        2 => {
            // Focus Points unlock at level 2 — updated already by level_up_player.
            // Seed the three core Focus Point features as reference abilities.
            if !has("Uncanny Metabolism") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Uncanny Metabolism",
                    Some("When you roll Initiative, you can regain all expended Focus Points. \
                          When you do so, roll your Martial Arts die and regain that many HP \
                          plus your Monk level. Once per Long Rest."),
                    1, "long_rest").await;
            }
        }
        3 => {
            if !has("Deflect Attacks") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Deflect Attacks",
                    Some("Reaction: when an attack hits you dealing Bludgeoning, Piercing, or \
                          Slashing damage, reduce the total damage by 1d10 + DEX modifier + \
                          Monk level. If you reduce the damage to 0, expend 1 Focus Point to \
                          redirect the force: target a creature within 5 ft (melee attack) or \
                          60 ft (ranged attack) — DEX save or take damage equal to 2× Martial \
                          Arts die + DEX modifier of the same type. \
                          Level 13 (Deflect Energy): also works against any damage type."),
                    1, "per_turn").await;
            }
        }
        4 => {
            if !has("Slow Fall") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Slow Fall",
                    Some("Reaction: when you fall, reduce any falling damage you take by an \
                          amount equal to five times your Monk level."),
                    1, "per_turn").await;
            }
        }
        5 => {
            if !has("Stunning Strike") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Stunning Strike",
                    Some("Once per turn when you hit a creature with a Monk weapon or Unarmed \
                          Strike, expend 1 Focus Point to attempt a stunning strike. Target makes \
                          a CON save (DC = 8 + WIS mod + Prof Bonus). \
                          Fail: Stunned until start of your next turn. \
                          Success: Speed halved until start of your next turn, and the next attack \
                          against the target before then has Advantage."),
                    1, "per_turn").await;
            }
        }
        6 => {
            if !has("Empowered Strikes") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Empowered Strikes",
                    Some("Whenever you deal damage with your Unarmed Strike, you can choose to \
                          deal Force damage instead of its normal damage type."),
                    1, "manual").await;
            }
        }
        7 => {
            if !has("Evasion") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Evasion",
                    Some("When subjected to an effect that allows a DEX save to take half damage, \
                          you take no damage on a success and only half damage on a failure. \
                          Unavailable if Incapacitated."),
                    1, "manual").await;
            }
        }
        9 => {
            if !has("Acrobatic Movement") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Acrobatic Movement",
                    Some("While not wearing armor or wielding a Shield, you can move along \
                          vertical surfaces and across liquids on your turn without falling \
                          during the movement."),
                    1, "manual").await;
            }
        }
        10 => {
            if !has("Heightened Focus") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Heightened Focus",
                    Some("Your Focus Point abilities improve: \
                          Flurry of Blows (1 FP): now makes THREE Unarmed Strikes instead of two. \
                          Patient Defense (1 FP): also gain Temporary HP equal to 2× Martial Arts die roll. \
                          Step of the Wind (1 FP): also choose a willing Large-or-smaller creature \
                            within 5 ft — it moves with you until end of your turn without \
                            provoking Opportunity Attacks."),
                    1, "manual").await;
            }
            if !has("Self-Restoration") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Self-Restoration",
                    Some("At the end of each of your turns, you can remove one of the following \
                          conditions from yourself: Charmed, Frightened, or Poisoned. \
                          In addition, forgoing food and drink doesn't give you levels of Exhaustion."),
                    1, "manual").await;
            }
        }
        13 => {
            // Deflect Energy: update Deflect Attacks description
            if let Some(a) = existing.iter().find(|a| a.name == "Deflect Attacks") {
                let _ = sqlx::query("UPDATE abilities SET description = ? WHERE id = ?")
                    .bind("Deflect Attacks: Reaction to reduce Bludgeoning/Piercing/Slashing \
                           attack damage by 1d10 + DEX mod + Monk level. Reduce to 0 to redirect \
                           the damage (1 FP): DEX save or 2× Martial Arts die + DEX mod of same type. \
                           Deflect Energy (level 13): now works against any damage type, not just \
                           Bludgeoning/Piercing/Slashing.")
                    .bind(&a.id)
                    .execute(pool)
                    .await;
            }
        }
        14 => {
            if !has("Disciplined Survivor") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Disciplined Survivor",
                    Some("You gain proficiency in all saving throws. \
                          Additionally, whenever you make a saving throw and fail, you can expend \
                          1 Focus Point to reroll it, and you must use the new roll."),
                    1, "manual").await;
            }
        }
        15 => {
            if !has("Perfect Focus") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Perfect Focus",
                    Some("When you roll Initiative and don't use Uncanny Metabolism, you regain \
                          expended Focus Points until you have 4 if you currently have 3 or fewer."),
                    1, "manual").await;
            }
        }
        18 => {
            if !has("Superior Defense") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Superior Defense",
                    Some("At the start of your turn, you can expend 3 Focus Points to bolster \
                          yourself against harm for 1 minute (or until Incapacitated). During \
                          that time, you have Resistance to all damage except Force damage."),
                    1, "manual").await;
            }
        }
        20 => {
            if !has("Body and Mind") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Body and Mind",
                    Some("Your Dexterity and Wisdom scores each increase by 4, to a maximum of 25. \
                          (Applied automatically to your stats.)"),
                    1, "manual").await;
            }
        }
        _ => {}
    }
 
    // ── Subclass features ─────────────────────────────────────────────────────
 
    match subclass {
 
        Some("Warrior of Mercy") => match new_level {
            3 => {
                if !has("Hand of Harm") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Hand of Harm",
                        Some("Once per turn when you hit a creature with an Unarmed Strike and \
                              deal damage, expend 1 Focus Point to deal extra Necrotic damage \
                              equal to one roll of your Martial Arts die + WIS modifier. \
                              Level 6 (Physician's Touch): also give the target the Poisoned \
                              condition until end of your next turn."),
                        1, "per_turn").await;
                }
                if !has("Hand of Healing") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Hand of Healing",
                        Some("Magic action: expend 1 Focus Point to touch a creature and restore \
                              HP equal to one roll of your Martial Arts die + WIS modifier. \
                              When you use Flurry of Blows, you can replace one Unarmed Strike \
                              with a use of this feature at no Focus Point cost. \
                              Level 6 (Physician's Touch): also end one of Blinded, Deafened, \
                              Paralyzed, Poisoned, or Stunned on the creature you heal."),
                        1, "manual").await;
                }
                if !has("Implements of Mercy") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Implements of Mercy",
                        Some("You gain proficiency in the Insight and Medicine skills and \
                              proficiency with the Herbalism Kit."),
                        1, "manual").await;
                }
            }
            6 => {
                if !has("Physician's Touch") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Physician's Touch",
                        Some("Hand of Harm improved: when used, also give the target the Poisoned \
                              condition until end of your next turn. \
                              Hand of Healing improved: when used, also end one condition on the \
                              creature: Blinded, Deafened, Paralyzed, Poisoned, or Stunned. \
                              (These upgrades are included in the existing Hand of Harm and \
                              Hand of Healing ability descriptions.)"),
                        1, "manual").await;
                }
            }
            11 => {
                if !has("Flurry of Healing and Harm") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Flurry of Healing and Harm",
                        Some("When you use Flurry of Blows, you can replace each Unarmed Strike \
                              with a Hand of Healing use at no Focus Point cost. \
                              Additionally, when you make an Unarmed Strike with Flurry of Blows \
                              and deal damage, you can use Hand of Harm with that strike at no \
                              Focus Point cost (still only once per turn). \
                              These benefits can be used a total number of times equal to your WIS \
                              modifier (min 1). Recharges on Long Rest."),
                        wis_mod, "long_rest").await;
                }
            }
            17 => {
                if !has("Hand of Ultimate Mercy") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Hand of Ultimate Mercy",
                        Some("Magic action: touch the corpse of a creature that died within the \
                              past 24 hours and expend 5 Focus Points. The creature returns to \
                              life with HP equal to 4d10 + WIS modifier. The following conditions \
                              are removed on revival: Blinded, Deafened, Paralyzed, Poisoned, \
                              and Stunned. Once per Long Rest."),
                        1, "long_rest").await;
                }
            }
            _ => {}
        },
 
        Some("Warrior of Shadow") => match new_level {
            3 => {
                if !has("Shadow Arts") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Shadow Arts",
                        Some("You gain three benefits from the Shadowfell: \
                              Darkness (1 FP): cast Darkness without components. You can see within \
                                the spell's area. While it persists, move its area to within 60 ft \
                                of you at the start of each of your turns. \
                              Darkvision: 60-foot Darkvision (or +60 ft if you already have it). \
                              Shadowy Figments: you know the Minor Illusion spell. WIS is your \
                                spellcasting ability for it."),
                        1, "manual").await;
                }
                // Learn Minor Illusion as always-prepared
                if let Ok(Some(spell)) = spells_db::get_spell_by_name(pool, "Minor Illusion").await {
                    if let Some(id) = spell["id"].as_str() {
                        let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "cantrip", "warrior_of_shadow").await;
                    }
                }
            }
            6 => {
                if !has("Shadow Step") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Shadow Step",
                        Some("Bonus Action: while entirely within Dim Light or Darkness, teleport \
                              up to 60 ft to an unoccupied space you can see that is also in Dim \
                              Light or Darkness. You then have Advantage on the next melee attack \
                              you make before the end of the current turn. \
                              Level 11 (Improved Shadow Step): expend 1 FP to remove the \
                              Dim Light/Darkness requirement for one use; also make an Unarmed \
                              Strike immediately after teleporting."),
                        1, "per_turn").await;
                }
            }
            11 => {
                if !has("Improved Shadow Step") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Improved Shadow Step",
                        Some("When you use Shadow Step, you can expend 1 Focus Point to remove \
                              the requirement that you must start and end in Dim Light or Darkness. \
                              As part of this Bonus Action, you can make one Unarmed Strike \
                              immediately after teleporting."),
                        1, "manual").await;
                }
            }
            17 => {
                if !has("Cloak of Shadows") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Cloak of Shadows",
                        Some("Magic action: while entirely in Dim Light or Darkness, expend 3 \
                              Focus Points to shroud yourself with shadows for 1 minute (until \
                              Incapacitated or until you end your turn in Bright Light). \
                              While shrouded: Invisibility — you have the Invisible condition. \
                              Partially Incorporeal — move through occupied spaces as if Difficult \
                                Terrain (shunted to last unoccupied space if you end turn there). \
                              Shadow Flurry — use Flurry of Blows without expending Focus Points."),
                        1, "manual").await;
                }
            }
            _ => {}
        },
 
        Some("Warrior of the Elements") => match new_level {
            3 => {
                if !has("Elemental Attunement") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Elemental Attunement",
                        Some("At the start of your turn, expend 1 Focus Point to imbue yourself \
                              with elemental energy for 10 minutes (until Incapacitated). \
                              Reach: Unarmed Strike reach is 10 feet greater than normal. \
                              Elemental Strikes: Unarmed Strikes deal Acid, Cold, Fire, Lightning, \
                                or Thunder (your choice each hit) instead of normal damage. \
                                On a hit, force a STR save — fail: move the target up to 10 ft \
                                toward or away from you. \
                              Level 11 (Stride of Elements): also gain Fly Speed and Swim Speed \
                                equal to your Speed while active. \
                              Level 17 (Elemental Epitome): also gain damage Resistance (one type, \
                                changeable each turn), Destructive Stride on Step of Wind, and \
                                extra Martial Arts die damage on Unarmed Strikes."),
                        1, "manual").await;
                }
                if !has("Manipulate Elements") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Manipulate Elements",
                        Some("You know the Elementalism spell. WIS is your spellcasting ability."),
                        1, "manual").await;
                }
                // Learn Elementalism cantrip
                if let Ok(Some(spell)) = spells_db::get_spell_by_name(pool, "Elementalism").await {
                    if let Some(id) = spell["id"].as_str() {
                        let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "cantrip", "warrior_of_elements").await;
                    }
                }
            }
            6 => {
                if !has("Elemental Burst") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Elemental Burst",
                        Some("Magic action: expend 2 Focus Points to cause elemental energy to \
                              burst in a 20-foot Sphere centered on a point within 120 ft of you. \
                              Choose a damage type: Acid, Cold, Fire, Lightning, or Thunder. \
                              Each creature in the Sphere makes a DEX save (DC = Focus save DC): \
                              Fail: damage equal to three rolls of Martial Arts die. \
                              Success: half damage."),
                        1, "manual").await;
                }
            }
            11 => {
                if !has("Stride of the Elements") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Stride of the Elements",
                        Some("While your Elemental Attunement is active, you also gain a Fly Speed \
                              and a Swim Speed equal to your Speed."),
                        1, "manual").await;
                }
            }
            17 => {
                if !has("Elemental Epitome") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Elemental Epitome",
                        Some("While Elemental Attunement is active, you also gain: \
                              Damage Resistance: Resistance to one damage type (Acid, Cold, Fire, \
                                Lightning, or Thunder) — change your choice at the start of each turn. \
                              Destructive Stride: when you use Step of the Wind, Speed increases by \
                                20 ft until end of turn; creatures of your choice take Martial Arts \
                                die damage when you enter a space within 5 ft of them (once per turn). \
                              Empowered Strikes: once per turn, deal extra Martial Arts die damage \
                                of the same type when you hit with an Unarmed Strike."),
                        1, "manual").await;
                }
            }
            _ => {}
        },
 
        Some("Warrior of the Open Hand") => match new_level {
            3 => {
                if !has("Open Hand Technique") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Open Hand Technique",
                        Some("Whenever you hit a creature with an attack granted by Flurry of \
                              Blows, you can impose one of the following effects on that target: \
                              Addle — the target can't make Opportunity Attacks until start of \
                                its next turn. \
                              Push — STR save or be pushed up to 15 ft away from you. \
                              Topple — DEX save or have the Prone condition."),
                        1, "manual").await;
                }
            }
            6 => {
                if !has("Wholeness of Body") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Wholeness of Body",
                        Some("Bonus Action: roll your Martial Arts die and regain HP equal to the \
                              roll + WIS modifier (minimum 1 HP regained). \
                              Uses = WIS modifier (min 1). Recharges on Long Rest."),
                        wis_mod, "long_rest").await;
                }
            }
            11 => {
                if !has("Fleet Step") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Fleet Step",
                        Some("When you take a Bonus Action other than Step of the Wind, you can \
                              also use Step of the Wind immediately after that Bonus Action."),
                        1, "manual").await;
                }
            }
            17 => {
                if !has("Quivering Palm") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Quivering Palm",
                        Some("When you hit a creature with an Unarmed Strike, expend 4 Focus \
                              Points to start imperceptible lethal vibrations. They last for a \
                              number of days equal to your Monk level and are harmless unless \
                              you choose to end them. \
                              To end them: take an Action while on the same plane as the target \
                              (or forgo one attack on your turn). Target makes a CON save: \
                              Fail: 10d12 Force damage. Success: half. \
                              You can have only one creature under this effect at a time. \
                              You can end the vibrations harmlessly (no action required)."),
                        1, "manual").await;
                }
            }
            _ => {}
        },
 
        _ => {}
    }
}

async fn seed_level_up_abilities_paladin(
    pool: &sqlx::SqlitePool,
    campaign_id: &str,
    player_id: &str,
    new_level: i64,
    subclass: Option<&str>,
) {
    let existing = world::get_abilities(pool, player_id, "player").await.unwrap_or_default();
    let has = |name: &str| existing.iter().any(|a| a.name == name);
 
    let player = match player::get_player(pool, player_id).await {
        Ok(Some(p)) => p,
        _ => return,
    };
    let cha_mod = crate::models::Player::modifier(player.cha).max(1);
 
    // ── Base class features ───────────────────────────────────────────────────
 
    match new_level {
        2 => {
            if !has("Paladin's Smite") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Paladin's Smite",
                    Some("Divine Smite is always prepared. Once per Long Rest, you can cast it \
                          without expending a spell slot (Bonus Action after hitting with a weapon). \
                          Divine Smite deals 2d8 Radiant damage, +1d8 if the target is a Fiend or \
                          Undead, and +1d8 per additional spell slot level above 1."),
                    1, "long_rest").await;
            }
            // Also learn Divine Smite as always-prepared if not already done
            if let Ok(Some(spell)) = spells_db::get_spell_by_name(pool, "Divine Smite").await {
                if let Some(id) = spell["id"].as_str() {
                    let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "paladin").await;
                }
            }
        }
        5 => {
            if !has("Faithful Steed") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Faithful Steed",
                    Some("Find Steed is always prepared. Once per Long Rest, you can cast it \
                          without expending a spell slot."),
                    1, "long_rest").await;
            }
            // Learn Find Steed as always-prepared
            if let Ok(Some(spell)) = spells_db::get_spell_by_name(pool, "Find Steed").await {
                if let Some(id) = spell["id"].as_str() {
                    let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "paladin").await;
                }
            }
        }
        6 => {
            if !has("Aura of Protection") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Aura of Protection",
                    Some("You radiate a protective aura in a 10-foot Emanation (while not \
                          Incapacitated). You and allies in the aura gain a bonus to saving throws \
                          equal to your CHA modifier (min +1). Only one Paladin's Aura of Protection \
                          applies at a time. Level 18 (Aura Expansion): range increases to 30 feet."),
                    1, "manual").await;
            }
        }
        9 => {
            if !has("Abjure Foes") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Abjure Foes",
                    Some("Magic action: expend a Channel Divinity use. Target up to CHA modifier \
                          creatures (min 1) you can see within 60 ft. Each makes a WIS save (DC = \
                          spell save DC) or has the Frightened condition for 1 minute or until it \
                          takes damage. While Frightened this way, the creature can only do one of: \
                          move, take an action, or take a Bonus Action on its turn."),
                    1, "manual").await;
            }
        }
        10 => {
            if !has("Aura of Courage") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Aura of Courage",
                    Some("You and your allies have Immunity to the Frightened condition while in \
                          your Aura of Protection. If a Frightened ally enters the aura, the \
                          condition has no effect on them while there."),
                    1, "manual").await;
            }
        }
        11 => {
            if !has("Radiant Strikes") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Radiant Strikes",
                    Some("When you hit a target with a Melee weapon or Unarmed Strike, the target \
                          takes an extra 1d8 Radiant damage."),
                    1, "manual").await;
            }
        }
        14 => {
            if !has("Restoring Touch") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Restoring Touch",
                    Some("When you use Lay On Hands on a creature, you can also remove conditions: \
                          Blinded, Charmed, Deafened, Frightened, Paralyzed, or Stunned. \
                          Expend 5 HP from the Lay On Hands pool per condition removed (those \
                          points don't also restore Hit Points)."),
                    1, "manual").await;
            }
        }
        _ => {}
    }
 
    // ── Subclass features ─────────────────────────────────────────────────────
 
    match subclass {
 
        Some("Oath of Devotion") => match new_level {
            3 => {
                if !has("Sacred Weapon") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Sacred Weapon",
                        Some("When you take the Attack action, expend a Channel Divinity use to imbue \
                              one held Melee weapon with positive energy for 10 minutes (or until \
                              re-used or the weapon leaves your hand). While active: \
                              add CHA modifier to attack rolls with the weapon (min +1); \
                              each hit deals normal damage type or Radiant (your choice); \
                              weapon emits Bright Light 20 ft and Dim Light 20 ft beyond."),
                        1, "manual").await;
                }
                if !has("Oath of Devotion Spells") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Oath of Devotion Spells",
                        Some("Always prepared (don't count against your limit): \
                              L3: Protection from Evil and Good, Shield of Faith. \
                              L5: Aid, Zone of Truth. L9: Beacon of Hope, Dispel Magic. \
                              L13: Freedom of Movement, Guardian of Faith. \
                              L17: Commune, Flame Strike."),
                        1, "manual").await;
                }
                for spell_name in &["Protection from Evil and Good", "Shield of Faith"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "oath_of_devotion").await;
                        }
                    }
                }
            }
            5 => {
                for spell_name in &["Aid", "Zone of Truth"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "oath_of_devotion").await;
                        }
                    }
                }
            }
            7 => {
                if !has("Aura of Devotion") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Aura of Devotion",
                        Some("You and your allies have Immunity to the Charmed condition while in \
                              your Aura of Protection. If a Charmed ally enters the aura, the \
                              condition has no effect on them while there."),
                        1, "manual").await;
                }
                for spell_name in &["Beacon of Hope", "Dispel Magic"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "oath_of_devotion").await;
                        }
                    }
                }
            }
            9 => {
                for spell_name in &["Beacon of Hope", "Dispel Magic"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "oath_of_devotion").await;
                        }
                    }
                }
            }
            13 => {
                for spell_name in &["Freedom of Movement", "Guardian of Faith"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "oath_of_devotion").await;
                        }
                    }
                }
            }
            15 => {
                if !has("Smite of Protection") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Smite of Protection",
                        Some("Whenever you cast Divine Smite, you and your allies have Half Cover \
                              while in your Aura of Protection until the start of your next turn."),
                        1, "manual").await;
                }
                for spell_name in &["Commune", "Flame Strike"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "oath_of_devotion").await;
                        }
                    }
                }
            }
            17 => {
                for spell_name in &["Commune", "Flame Strike"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "oath_of_devotion").await;
                        }
                    }
                }
            }
            20 => {
                if !has("Holy Nimbus") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Holy Nimbus",
                        Some("Bonus Action: imbue your Aura of Protection with holy power for \
                              10 minutes (or end freely). Once per Long Rest, or expend a level 5 \
                              slot to restore. While active: \
                              Holy Ward — Advantage on saves forced by Fiends or Undead. \
                              Radiant Damage — enemies starting their turn in the aura take \
                                CHA modifier + Proficiency Bonus Radiant damage. \
                              Sunlight — the aura fills with Bright Light that is sunlight."),
                        1, "long_rest").await;
                }
            }
            _ => {}
        },
 
        Some("Oath of Glory") => match new_level {
            3 => {
                if !has("Inspiring Smite") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Inspiring Smite",
                        Some("Immediately after casting Divine Smite, expend a Channel Divinity use \
                              to distribute Temporary HP to creatures of your choice within 30 ft \
                              (can include you). Total = 2d8 + Paladin level, split however you like."),
                        1, "manual").await;
                }
                if !has("Peerless Athlete") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Peerless Athlete",
                        Some("Bonus Action: expend a Channel Divinity use. For 1 hour: \
                              Advantage on STR (Athletics) and DEX (Acrobatics) checks; \
                              Long and High Jump distances increase by 10 feet (costs movement normally)."),
                        1, "manual").await;
                }
                if !has("Oath of Glory Spells") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Oath of Glory Spells",
                        Some("Always prepared: L3: Guiding Bolt, Heroism. \
                              L5: Enhance Ability, Magic Weapon. L9: Haste, Protection from Energy. \
                              L13: Compulsion, Freedom of Movement. \
                              L17: Legend Lore, Yolande's Regal Presence."),
                        1, "manual").await;
                }
                for spell_name in &["Guiding Bolt", "Heroism"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "oath_of_glory").await;
                        }
                    }
                }
            }
            5 => {
                for spell_name in &["Enhance Ability", "Magic Weapon"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "oath_of_glory").await;
                        }
                    }
                }
            }
            7 => {
                if !has("Aura of Alacrity") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Aura of Alacrity",
                        Some("Your Speed increases by 10 feet. Whenever an ally enters your Aura \
                              of Protection for the first time on a turn or starts their turn there, \
                              that ally's Speed increases by 10 feet until end of their next turn."),
                        1, "manual").await;
                }
                for spell_name in &["Haste", "Protection from Energy"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "oath_of_glory").await;
                        }
                    }
                }
            }
            13 => {
                for spell_name in &["Compulsion", "Freedom of Movement"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "oath_of_glory").await;
                        }
                    }
                }
            }
            15 => {
                if !has("Glorious Defense") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Glorious Defense",
                        Some("Reaction: when you or a creature within 10 ft is hit by an attack, \
                              grant a bonus to the target's AC equal to your CHA modifier (min +1), \
                              potentially causing the attack to miss. If it misses, you can make one \
                              weapon attack against the attacker as part of this Reaction (if in range). \
                              Uses = CHA modifier (min 1). Recharges on Long Rest."),
                        cha_mod, "long_rest").await;
                }
                for spell_name in &["Legend Lore", "Yolande's Regal Presence"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "oath_of_glory").await;
                        }
                    }
                }
            }
            20 => {
                if !has("Living Legend") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Living Legend",
                        Some("Bonus Action: gain the following for 10 minutes (end freely). \
                              Once per Long Rest, or expend a level 5 slot to restore. \
                              Charismatic — Advantage on all CHA checks. \
                              Saving Throw Reroll — when you fail a save, take a Reaction to \
                                reroll it; you must use the new roll. \
                              Unerring Strike — once per turn when you miss with a weapon attack, \
                                cause it to hit instead."),
                        1, "long_rest").await;
                }
            }
            _ => {}
        },
 
        Some("Oath of the Ancients") => match new_level {
            3 => {
                if !has("Nature's Wrath") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Nature's Wrath",
                        Some("Magic action: expend a Channel Divinity use to conjure spectral vines \
                              around nearby creatures. Each creature of your choice within 15 ft that \
                              you can see makes a STR save (DC = spell save DC) or has the Restrained \
                              condition for 1 minute. A Restrained creature repeats the save at the \
                              end of each of its turns, ending the effect on success."),
                        1, "manual").await;
                }
                if !has("Oath of the Ancients Spells") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Oath of the Ancients Spells",
                        Some("Always prepared: L3: Ensnaring Strike, Speak with Animals. \
                              L5: Misty Step, Moonbeam. L9: Plant Growth, Protection from Energy. \
                              L13: Ice Storm, Stoneskin. L17: Commune with Nature, Tree Stride."),
                        1, "manual").await;
                }
                for spell_name in &["Ensnaring Strike", "Speak with Animals"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "oath_of_the_ancients").await;
                        }
                    }
                }
            }
            5 => {
                for spell_name in &["Misty Step", "Moonbeam"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "oath_of_the_ancients").await;
                        }
                    }
                }
            }
            7 => {
                if !has("Aura of Warding") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Aura of Warding",
                        Some("Ancient magic forms an eldritch ward around you. You and allies in \
                              your Aura of Protection have Resistance to Necrotic, Psychic, and \
                              Radiant damage."),
                        1, "manual").await;
                }
                for spell_name in &["Plant Growth", "Protection from Energy"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "oath_of_the_ancients").await;
                        }
                    }
                }
            }
            13 => {
                for spell_name in &["Ice Storm", "Stoneskin"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "oath_of_the_ancients").await;
                        }
                    }
                }
            }
            15 => {
                if !has("Undying Sentinel") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Undying Sentinel",
                        Some("When you are reduced to 0 HP and don't die outright, you can drop to \
                              1 HP instead and regain HP equal to three times your Paladin level. \
                              Once per Long Rest. \
                              Additionally, you can't be aged magically and cease visibly aging."),
                        1, "long_rest").await;
                }
                for spell_name in &["Commune with Nature", "Tree Stride"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "oath_of_the_ancients").await;
                        }
                    }
                }
            }
            20 => {
                if !has("Elder Champion") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Elder Champion",
                        Some("Bonus Action: imbue your Aura with primal power for 1 minute (end freely). \
                              Once per Long Rest, or expend a level 5 slot to restore. \
                              Diminish Defiance — enemies in the aura have Disadvantage on saves \
                                against your spells and Channel Divinity options. \
                              Regeneration — regain 10 HP at the start of each of your turns. \
                              Swift Spells — spells with a casting time of 1 action can be cast as \
                                a Bonus Action instead."),
                        1, "long_rest").await;
                }
            }
            _ => {}
        },
 
        Some("Oath of Vengeance") => match new_level {
            3 => {
                if !has("Vow of Enmity") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Vow of Enmity",
                        Some("When you take the Attack action, expend a Channel Divinity use to utter \
                              a vow of enmity against a creature you can see within 30 ft. You have \
                              Advantage on attack rolls against that creature for 1 minute or until \
                              you use this feature again. If the creature drops to 0 HP before the \
                              vow ends, you can transfer it to a different creature within 30 ft \
                              (no action required)."),
                        1, "manual").await;
                }
                if !has("Oath of Vengeance Spells") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Oath of Vengeance Spells",
                        Some("Always prepared: L3: Bane, Hunter's Mark. \
                              L5: Hold Person, Misty Step. L9: Haste, Protection from Energy. \
                              L13: Banishment, Dimension Door. L17: Hold Monster, Scrying."),
                        1, "manual").await;
                }
                for spell_name in &["Bane", "Hunter's Mark"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "oath_of_vengeance").await;
                        }
                    }
                }
            }
            5 => {
                for spell_name in &["Hold Person", "Misty Step"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "oath_of_vengeance").await;
                        }
                    }
                }
            }
            7 => {
                if !has("Relentless Avenger") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Relentless Avenger",
                        Some("When you hit a creature with an Opportunity Attack, you can reduce \
                              its Speed to 0 until the end of the current turn. You can then move \
                              up to half your Speed as part of the same Reaction. This movement \
                              doesn't provoke Opportunity Attacks."),
                        1, "per_turn").await;
                }
                for spell_name in &["Haste", "Protection from Energy"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "oath_of_vengeance").await;
                        }
                    }
                }
            }
            13 => {
                for spell_name in &["Banishment", "Dimension Door"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "oath_of_vengeance").await;
                        }
                    }
                }
            }
            15 => {
                if !has("Soul of Vengeance") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Soul of Vengeance",
                        Some("Immediately after a creature under the effect of your Vow of Enmity \
                              hits or misses with an attack roll, you can take a Reaction to make \
                              one melee attack against that creature if it's within range."),
                        1, "per_turn").await;
                }
                for spell_name in &["Hold Monster", "Scrying"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "oath_of_vengeance").await;
                        }
                    }
                }
            }
            20 => {
                if !has("Avenging Angel") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Avenging Angel",
                        Some("Bonus Action: gain the following for 10 minutes (end freely). \
                              Once per Long Rest, or expend a level 5 slot to restore. \
                              Flight — sprout spectral wings, gain Fly Speed 60 ft, and can hover. \
                              Frightful Aura — enemies starting their turn in your Aura of Protection \
                                make a WIS save (DC = spell save DC) or have the Frightened condition \
                                for 1 minute or until they take damage. Attack rolls against the \
                                Frightened creature have Advantage."),
                        1, "long_rest").await;
                }
            }
            _ => {}
        },
 
        _ => {}
    }
}

async fn seed_level_up_abilities_ranger(
    pool: &sqlx::SqlitePool,
    campaign_id: &str,
    player_id: &str,
    new_level: i64,
    subclass: Option<&str>,
) {
    let existing = world::get_abilities(pool, player_id, "player").await.unwrap_or_default();
    let has = |name: &str| existing.iter().any(|a| a.name == name);
 
    let player = match player::get_player(pool, player_id).await {
        Ok(Some(p)) => p,
        _ => return,
    };
    let wis_mod = crate::models::Player::modifier(player.wis).max(1);
 
    // ── Base class features ───────────────────────────────────────────────────
 
    match new_level {
        2 => {
            if !has("Deft Explorer") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Deft Explorer",
                    Some("Expertise: choose one skill proficiency you lack Expertise in — \
                          you gain Expertise in that skill. \
                          Languages: you know two additional languages of your choice."),
                    1, "manual").await;
            }
        }
        5 => {
            if !has("Extra Attack") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Extra Attack",
                    Some("You can attack twice instead of once whenever you take the Attack action."),
                    1, "manual").await;
            }
        }
        6 => {
            if !has("Roving") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Roving",
                    Some("Your Speed increases by 10 feet while you aren't wearing Heavy armor. \
                          You also gain a Climb Speed and a Swim Speed equal to your Speed."),
                    1, "manual").await;
            }
        }
        9 => {
            if !has("Expertise (Level 9)") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Expertise (Level 9)",
                    Some("Choose two skill proficiencies you lack Expertise in. \
                          You gain Expertise in both skills."),
                    1, "manual").await;
            }
        }
        10 => {
            if !has("Tireless") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Tireless",
                    Some("Temporary Hit Points: Magic action — give yourself 1d8 + WIS modifier \
                          Temporary HP. Uses = WIS modifier (min 1). Recharges on Long Rest. \
                          Decrease Exhaustion: whenever you finish a Short Rest, your Exhaustion \
                          level decreases by 1."),
                    wis_mod, "long_rest").await;
            }
        }
        13 => {
            if !has("Relentless Hunter") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Relentless Hunter",
                    Some("Taking damage can't break your Concentration on Hunter's Mark."),
                    1, "manual").await;
            }
        }
        14 => {
            if !has("Nature's Veil") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Nature's Veil",
                    Some("Bonus Action: give yourself the Invisible condition until the end of \
                          your next turn. Uses = WIS modifier (min 1). Recharges on Long Rest."),
                    wis_mod, "long_rest").await;
            }
        }
        17 => {
            if !has("Precise Hunter") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Precise Hunter",
                    Some("You have Advantage on attack rolls against the creature currently \
                          marked by your Hunter's Mark."),
                    1, "manual").await;
            }
        }
        18 => {
            if !has("Feral Senses") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Feral Senses",
                    Some("Your connection to nature grants you Blindsight with a range of 30 feet."),
                    1, "manual").await;
            }
        }
        20 => {
            if !has("Foe Slayer") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Foe Slayer",
                    Some("The damage die of your Hunter's Mark is now a d10 rather than a d6."),
                    1, "manual").await;
            }
        }
        _ => {}
    }
 
    // ── Subclass features ─────────────────────────────────────────────────────
 
    match subclass {
 
        Some("Beast Master") => match new_level {
            3 => {
                if !has("Primal Companion") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Primal Companion",
                        Some("You magically summon a primal beast (Beast of the Land, Sea, or Sky). \
                              Choose its stat block and appearance. It is Friendly, obeys your commands, \
                              and vanishes if you die. In combat it acts on your turn — it Dodges unless \
                              you take a Bonus Action to command it to use Beast's Strike or another action. \
                              You can sacrifice one of your attacks (Attack action) to command it. \
                              Restore: if it died within the last hour, touch it and expend a spell slot \
                              (Magic action) — it returns with full HP after 1 minute. \
                              On Long Rest, you can summon a different primal beast."),
                        1, "manual").await;
                }
            }
            7 => {
                if !has("Exceptional Training") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Exceptional Training",
                        Some("When you Bonus Action command your Primal Companion, it can also \
                              take the Dash, Disengage, Dodge, or Help action as its Bonus Action. \
                              Additionally, whenever it hits and deals damage, it can deal Force damage \
                              or its normal damage type (your choice each hit)."),
                        1, "manual").await;
                }
            }
            11 => {
                if !has("Bestial Fury") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Bestial Fury",
                        Some("When you command your Primal Companion to use Beast's Strike, it can \
                              use the action twice. Additionally, the first time each turn it hits a \
                              creature under your Hunter's Mark, it deals extra Force damage equal to \
                              Hunter's Mark's bonus damage."),
                        1, "manual").await;
                }
            }
            15 => {
                if !has("Share Spells") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Share Spells",
                        Some("When you cast a spell targeting yourself, you can also affect your \
                              Primal Companion beast with the spell if it is within 30 feet of you."),
                        1, "manual").await;
                }
            }
            _ => {}
        },
 
        Some("Fey Wanderer") => match new_level {
            3 => {
                if !has("Dreadful Strikes") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Dreadful Strikes",
                        Some("When you hit a creature with a weapon, you can deal an extra 1d4 \
                              Psychic damage (once per turn on the same creature). \
                              Increases to 1d6 at Ranger level 11."),
                        1, "per_turn").await;
                }
                if !has("Otherworldly Glamour") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Otherworldly Glamour",
                        Some("Whenever you make a Charisma check, you gain a bonus equal to your \
                              WIS modifier (minimum +1). You also gain proficiency in one of: \
                              Deception, Performance, or Persuasion (your choice)."),
                        1, "manual").await;
                }
                if !has("Fey Wanderer Spells") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Fey Wanderer Spells",
                        Some("Always prepared: Charm Person (L3), Misty Step (L5), \
                              Summon Fey (L9), Dimension Door (L13), Mislead (L17)."),
                        1, "manual").await;
                }
                // Learn Charm Person
                if let Ok(Some(spell)) = spells_db::get_spell_by_name(pool, "Charm Person").await {
                    if let Some(id) = spell["id"].as_str() {
                        let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "fey_wanderer").await;
                    }
                }
            }
            5 => {
                if let Ok(Some(spell)) = spells_db::get_spell_by_name(pool, "Misty Step").await {
                    if let Some(id) = spell["id"].as_str() {
                        let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "fey_wanderer").await;
                    }
                }
            }
            7 => {
                if !has("Beguiling Twist") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Beguiling Twist",
                        Some("You have Advantage on saving throws to avoid or end the Charmed or \
                              Frightened condition. When you or a creature you can see within 120 ft \
                              succeeds on a save to avoid or end Charmed or Frightened, you can take a \
                              Reaction to force a different creature within 120 ft to make a WIS save \
                              (DC = spell save DC) — fail: Charmed or Frightened (your choice) for 1 \
                              minute; repeats save at end of each of its turns."),
                        1, "per_turn").await;
                }
            }
            9 => {
                if let Ok(Some(spell)) = spells_db::get_spell_by_name(pool, "Summon Fey").await {
                    if let Some(id) = spell["id"].as_str() {
                        let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "fey_wanderer").await;
                    }
                }
            }
            11 => {
                if !has("Fey Reinforcements") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Fey Reinforcements",
                        Some("You can cast Summon Fey without a Material component. Once per Long \
                              Rest, you can cast it without a spell slot. When you start casting it, \
                              you can modify it so it doesn't require Concentration (duration becomes \
                              1 minute for that casting)."),
                        1, "long_rest").await;
                }
            }
            13 => {
                if let Ok(Some(spell)) = spells_db::get_spell_by_name(pool, "Dimension Door").await {
                    if let Some(id) = spell["id"].as_str() {
                        let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "fey_wanderer").await;
                    }
                }
            }
            15 => {
                if !has("Misty Wanderer") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Misty Wanderer",
                        Some("You can cast Misty Step without expending a spell slot. \
                              Uses = WIS modifier (min 1). Recharges on Long Rest. \
                              When you cast Misty Step, you can bring one willing creature \
                              within 5 ft — it teleports to an unoccupied space within 5 ft \
                              of your destination."),
                        wis_mod, "long_rest").await;
                }
            }
            17 => {
                if let Ok(Some(spell)) = spells_db::get_spell_by_name(pool, "Mislead").await {
                    if let Some(id) = spell["id"].as_str() {
                        let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "fey_wanderer").await;
                    }
                }
            }
            _ => {}
        },
 
        Some("Gloom Stalker") => match new_level {
            3 => {
                if !has("Dread Ambusher") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Dread Ambusher",
                        Some("Ambusher's Leap: at the start of your first combat turn, Speed +10 ft \
                              until end of that turn. \
                              Dreadful Strike: when you hit a creature with a weapon, deal extra 2d6 \
                              Psychic damage (once per turn). Uses = WIS modifier (min 1). \
                              Recharges on Long Rest. Becomes 2d8 at level 11 (Stalker's Flurry). \
                              Initiative Bonus: add your WIS modifier when you roll Initiative."),
                        wis_mod, "long_rest").await;
                }
                if !has("Umbral Sight") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Umbral Sight",
                        Some("You gain Darkvision 60 ft (or +60 ft if you already have Darkvision). \
                              While entirely in Darkness, you have the Invisible condition against any \
                              creature that relies on Darkvision to see you in that Darkness."),
                        1, "manual").await;
                }
                if !has("Gloom Stalker Spells") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Gloom Stalker Spells",
                        Some("Always prepared: Disguise Self (L3), Rope Trick (L5), \
                              Fear (L9), Greater Invisibility (L13), Seeming (L17)."),
                        1, "manual").await;
                }
                if let Ok(Some(spell)) = spells_db::get_spell_by_name(pool, "Disguise Self").await {
                    if let Some(id) = spell["id"].as_str() {
                        let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "gloom_stalker").await;
                    }
                }
            }
            5 => {
                for spell_name in &["Rope Trick"] {
                    if let Ok(Some(spell)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = spell["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "gloom_stalker").await;
                        }
                    }
                }
            }
            7 => {
                if !has("Iron Mind") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Iron Mind",
                        Some("You gain proficiency in Wisdom saving throws. If you already have \
                              this proficiency, you instead gain proficiency in Intelligence or \
                              Charisma saving throws (your choice)."),
                        1, "manual").await;
                }
            }
            9 => {
                if let Ok(Some(spell)) = spells_db::get_spell_by_name(pool, "Fear").await {
                    if let Some(id) = spell["id"].as_str() {
                        let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "gloom_stalker").await;
                    }
                }
            }
            11 => {
                if !has("Stalker's Flurry") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Stalker's Flurry",
                        Some("Your Dreadful Strike Psychic damage increases to 2d8. When you use \
                              Dreadful Strike, you can also cause one of: \
                              Sudden Strike — make another attack with the same weapon against a \
                                different creature within 5 ft of the original target and within range. \
                              Mass Fear — the target and each creature within 10 ft must make a WIS \
                                save (DC = spell save DC) or have the Frightened condition until the \
                                start of your next turn."),
                        1, "manual").await;
                }
            }
            13 => {
                if let Ok(Some(spell)) = spells_db::get_spell_by_name(pool, "Greater Invisibility").await {
                    if let Some(id) = spell["id"].as_str() {
                        let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "gloom_stalker").await;
                    }
                }
            }
            15 => {
                if !has("Shadowy Dodge") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Shadowy Dodge",
                        Some("Reaction: when a creature makes an attack roll against you, impose \
                              Disadvantage on that roll. Whether the attack hits or misses, you can \
                              then teleport up to 30 feet to an unoccupied space you can see."),
                        1, "per_turn").await;
                }
            }
            17 => {
                if let Ok(Some(spell)) = spells_db::get_spell_by_name(pool, "Seeming").await {
                    if let Some(id) = spell["id"].as_str() {
                        let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "gloom_stalker").await;
                    }
                }
            }
            _ => {}
        },
 
        Some("Hunter") => match new_level {
            3 => {
                if !has("Hunter's Lore") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Hunter's Lore",
                        Some("While a creature is marked by your Hunter's Mark, you know whether \
                              it has any Immunities, Resistances, or Vulnerabilities, and if so, \
                              what they are."),
                        1, "manual").await;
                }
                if !has("Hunter's Prey") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Hunter's Prey",
                        Some("Choose one option (switch on Short or Long Rest): \
                              Colossus Slayer — when you hit a creature with a weapon, if it is \
                                missing any HP, deal an extra 1d8 damage (once per turn). \
                              Horde Breaker — once per turn when you make a weapon attack, you can \
                                make another attack with the same weapon against a different creature \
                                within 5 ft of the original target and within weapon range."),
                        1, "manual").await;
                }
            }
            7 => {
                if !has("Defensive Tactics") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Defensive Tactics",
                        Some("Choose one option (switch on Short or Long Rest): \
                              Escape the Horde — Opportunity Attacks have Disadvantage against you. \
                              Multiattack Defense — when a creature hits you with an attack roll, \
                                that creature has Disadvantage on all other attack rolls against you \
                                this turn."),
                        1, "manual").await;
                }
            }
            11 => {
                if !has("Superior Hunter's Prey") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Superior Hunter's Prey",
                        Some("Once per turn when you deal damage to a creature marked by your \
                              Hunter's Mark, you can also deal that spell's extra damage to a \
                              different creature you can see within 30 feet of the first creature."),
                        1, "per_turn").await;
                }
            }
            15 => {
                if !has("Superior Hunter's Defense") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Superior Hunter's Defense",
                        Some("When you take damage, you can take a Reaction to give yourself \
                              Resistance to that damage type and any other damage of the same type \
                              until the end of the current turn."),
                        1, "per_turn").await;
                }
            }
            _ => {}
        },
 
        _ => {}
    }
}

async fn seed_level_up_abilities_rogue(
    pool: &sqlx::SqlitePool,
    campaign_id: &str,
    player_id: &str,
    new_level: i64,
    subclass: Option<&str>,
) {
    let existing = world::get_abilities(pool, player_id, "player").await.unwrap_or_default();
    let has = |name: &str| existing.iter().any(|a| a.name == name);
 
    let player = match player::get_player(pool, player_id).await {
        Ok(Some(p)) => p,
        _ => return,
    };
    let dex_mod = crate::models::Player::modifier(player.dex).max(1);
 
    // ── Base class features ───────────────────────────────────────────────────
 
    match new_level {
        2 => {
            // Cunning Action was seeded at creation; just a note here
            // that it's already present via seed_class_abilities
        }
        3 => {
            if !has("Steady Aim") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Steady Aim",
                    Some("Bonus Action: give yourself Advantage on your next attack roll this turn. \
                          You can use this only if you haven't moved this turn. After using it, \
                          your Speed is 0 until the end of the current turn."),
                    1, "per_turn").await;
            }
        }
        5 => {
            if !has("Cunning Strike") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Cunning Strike",
                    Some("When you deal Sneak Attack damage, you can sacrifice dice to add effects \
                          (save DC = 8 + DEX mod + Prof Bonus): \
                          Poison (1d6): CON save or Poisoned for 1 minute (repeats each turn). \
                            Requires a Poisoner's Kit on your person. \
                          Trip (1d6): DEX save or Prone (Large or smaller targets only). \
                          Withdraw (1d6): move up to half your Speed without provoking Opportunity Attacks. \
                          Level 11 (Improved): use up to two effects per Sneak Attack. \
                          Level 14 (Devious Strikes): Daze (2d6, CON save or limited actions next turn), \
                            Knock Out (6d6, CON save or Unconscious for 1 minute), \
                            Obscure (3d6, DEX save or Blinded until end of its next turn)."),
                    1, "per_turn").await;
            }
            if !has("Uncanny Dodge") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Uncanny Dodge",
                    Some("Reaction: when an attacker you can see hits you with an attack roll, \
                          halve the attack's damage against you (round down)."),
                    1, "per_turn").await;
            }
        }
        7 => {
            if !has("Evasion") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Evasion",
                    Some("When subjected to an effect that allows a DEX save for half damage, \
                          take no damage on a success and only half damage on a failure. \
                          Unavailable if Incapacitated."),
                    1, "manual").await;
            }
            if !has("Reliable Talent") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Reliable Talent",
                    Some("Whenever you make an ability check using a skill or tool you have \
                          proficiency in, treat a d20 roll of 9 or lower as a 10."),
                    1, "manual").await;
            }
        }
        11 => {
            // Improved Cunning Strike — description update on the existing ability
            if let Some(a) = existing.iter().find(|a| a.name == "Cunning Strike") {
                let _ = sqlx::query(
                    "UPDATE abilities SET description = ? WHERE id = ?"
                )
                .bind("Cunning Strike — Improved (level 11): you can now use up to TWO effects \
                       when you deal Sneak Attack damage, paying the die cost for each. \
                       Poison (1d6), Trip (1d6), Withdraw (1d6), \
                       Daze (2d6, level 14), Knock Out (6d6, level 14), Obscure (3d6, level 14). \
                       Save DC = 8 + DEX mod + Prof Bonus.")
                .bind(&a.id)
                .execute(pool)
                .await;
            }
        }
        15 => {
            if !has("Slippery Mind") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Slippery Mind",
                    Some("Your cunning mind is exceptionally difficult to control. You gain \
                          proficiency in Wisdom and Charisma saving throws."),
                    1, "manual").await;
            }
        }
        18 => {
            if !has("Elusive") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Elusive",
                    Some("No attack roll can have Advantage against you unless you have the \
                          Incapacitated condition."),
                    1, "manual").await;
            }
        }
        20 => {
            if !has("Stroke of Luck") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Stroke of Luck",
                    Some("If you fail a D20 Test, you can turn the roll into a 20. \
                          Once per Short or Long Rest."),
                    1, "short_rest").await;
            }
        }
        _ => {}
    }
 
    // ── Subclass features ─────────────────────────────────────────────────────
 
    match subclass {
 
        Some("Arcane Trickster") => {
            match new_level {
                3 => {
                    // Seed spell slots (2×L1 at L3)
                    let _ = spells_db::seed_arcane_trickster_spell_slots(
                        pool, campaign_id, player_id, new_level
                    ).await;
 
                    if !has("Mage Hand Legerdemain") {
                        let _ = world::create_ability(pool, campaign_id, "player", player_id,
                            "Mage Hand Legerdemain",
                            Some("When you cast Mage Hand, you can cast it as a Bonus Action and \
                                  make the spectral hand Invisible. You can control the hand as a \
                                  Bonus Action, and through it make DEX (Sleight of Hand) checks."),
                            1, "manual").await;
                    }
 
                    // Learn Mage Hand if not already known
                    if let Ok(Some(spell)) = spells_db::get_spell_by_name(pool, "Mage Hand").await {
                        if let Some(id) = spell["id"].as_str() {
                            let _ = spells_db::learn_spell(
                                pool, campaign_id, player_id, id, "cantrip", "arcane_trickster"
                            ).await;
                        }
                    }
                }
                9 => {
                    if !has("Magical Ambush") {
                        let _ = world::create_ability(pool, campaign_id, "player", player_id,
                            "Magical Ambush",
                            Some("If you have the Invisible condition when you cast a spell on a \
                                  creature, that creature has Disadvantage on any saving throw it \
                                  makes against the spell on the same turn."),
                            1, "manual").await;
                    }
                }
                13 => {
                    if !has("Versatile Trickster") {
                        let _ = world::create_ability(pool, campaign_id, "player", player_id,
                            "Versatile Trickster",
                            Some("When you use the Trip option of your Cunning Strike on a creature, \
                                  you can also use that option on another creature within 5 feet of \
                                  your spectral Mage Hand."),
                            1, "manual").await;
                    }
                }
                17 => {
                    if !has("Spell Thief") {
                        let _ = world::create_ability(pool, campaign_id, "player", player_id,
                            "Spell Thief",
                            Some("Reaction: immediately after a creature casts a spell that targets \
                                  you or includes you in its area of effect, force the creature to \
                                  make an INT save (DC = your spell save DC). On a failed save: \
                                  negate the spell's effect against you; steal its knowledge if it \
                                  is level 1+ and of a castable level — you have it prepared for 8 \
                                  hours; the creature can't cast it until the 8 hours have passed. \
                                  Once per Long Rest."),
                            1, "long_rest").await;
                    }
                }
                _ => {}
            }
        }
 
        Some("Assassin") => match new_level {
            3 => {
                if !has("Assassinate") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Assassinate",
                        Some("Initiative: you have Advantage on Initiative rolls. \
                              Surprising Strikes: during the first round of each combat, you have \
                              Advantage on attack rolls against any creature that hasn't taken a turn. \
                              If your Sneak Attack hits during that round, the target takes extra \
                              damage equal to your Rogue level of the weapon's type."),
                        1, "manual").await;
                }
                if !has("Assassin's Tools") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Assassin's Tools",
                        Some("You gain a Disguise Kit and a Poisoner's Kit, and you have \
                              proficiency with both."),
                        1, "manual").await;
                }
            }
            9 => {
                if !has("Infiltration Expertise") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Infiltration Expertise",
                        Some("Masterful Mimicry: you can unerringly mimic another person's speech, \
                              handwriting, or both after spending at least 1 hour studying them. \
                              Roving Aim: your Speed isn't reduced to 0 by using Steady Aim."),
                        1, "manual").await;
                }
            }
            13 => {
                if !has("Envenom Weapons") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Envenom Weapons",
                        Some("When you use the Poison option of Cunning Strike, the target also \
                              takes 2d6 Poison damage whenever it fails the saving throw. \
                              This damage ignores Resistance to Poison damage."),
                        1, "manual").await;
                }
            }
            17 => {
                if !has("Death Strike") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Death Strike",
                        Some("When you hit with your Sneak Attack on the first round of combat, \
                              the target must succeed on a CON save (DC = 8 + DEX mod + Prof Bonus) \
                              or the attack's damage is doubled against the target."),
                        1, "per_turn").await;
                }
            }
            _ => {}
        },
 
        Some("Soulknife") => match new_level {
            3 => {
                // Seed Psionic Energy Dice using the superiority_dice table
                // (same mechanical system as Battle Master and Psi Warrior)
                let _ = sqlx::query(
                    "INSERT OR REPLACE INTO superiority_dice
                     (id, campaign_id, player_id, pool_name, max_dice, current_dice, die_size)
                     VALUES (lower(hex(randomblob(16))), ?, ?, 'Soulknife', 4, 4, 6)"
                )
                .bind(campaign_id).bind(player_id)
                .execute(pool).await;
 
                if !has("Psionic Power") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Psionic Power",
                        Some("You have Psionic Energy Dice (4×d6 at L3, scaling up). \
                              Regain 1 on Short Rest, all on Long Rest. \
                              Psi-Bolstered Knack: if you fail a proficiency check, roll and add \
                                a Psionic Energy Die — the die is expended only if it succeeds. \
                              Psychic Whispers (Magic action): choose creatures up to your Prof Bonus \
                                that you can see — roll a die. For that many hours, you and those \
                                creatures can speak telepathically within 1 mile. First use per Long \
                                Rest is free."),
                        1, "manual").await;
                }
                if !has("Psychic Blades") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Psychic Blades",
                        Some("Whenever you take the Attack action or make an Opportunity Attack, \
                              you can manifest a Psychic Blade (Simple Melee, Finesse, Thrown 60/120 ft, \
                              Vex mastery). Damage: 1d6 Psychic + ability modifier. The blade vanishes \
                              after it hits or misses. After attacking with it on your turn, you can \
                              make a second psychic blade attack as a Bonus Action (1d4 Psychic + mod) \
                              if your other hand is free."),
                        1, "manual").await;
                }
            }
            5 => {
                // Update dice: 6×d8
                let _ = sqlx::query(
                    "UPDATE superiority_dice SET max_dice = 6, current_dice = 6, die_size = 8
                     WHERE player_id = ? AND pool_name = 'Soulknife'"
                ).bind(player_id).execute(pool).await;
            }
            9 => {
                // Update dice: 8×d8
                let _ = sqlx::query(
                    "UPDATE superiority_dice SET max_dice = 8, current_dice = 8, die_size = 8
                     WHERE player_id = ? AND pool_name = 'Soulknife'"
                ).bind(player_id).execute(pool).await;
 
                if !has("Soul Blades") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Soul Blades",
                        Some("New Psychic Blade powers: \
                              Homing Strikes: if you miss with a Psychic Blade attack, roll a \
                                Psionic Energy Die and add it to the attack roll — expended only \
                                if this causes a hit. \
                              Psychic Teleportation (Bonus Action): manifest a blade, expend and \
                                roll a Psionic Energy Die, throw the blade up to 10× the roll in \
                                feet — teleport to that space."),
                        1, "manual").await;
                }
            }
            11 => {
                // Update dice: 8×d10
                let _ = sqlx::query(
                    "UPDATE superiority_dice SET max_dice = 8, current_dice = 8, die_size = 10
                     WHERE player_id = ? AND pool_name = 'Soulknife'"
                ).bind(player_id).execute(pool).await;
            }
            13 => {
                // Update dice: 10×d10
                let _ = sqlx::query(
                    "UPDATE superiority_dice SET max_dice = 10, current_dice = 10, die_size = 10
                     WHERE player_id = ? AND pool_name = 'Soulknife'"
                ).bind(player_id).execute(pool).await;
 
                if !has("Psychic Veil") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Psychic Veil",
                        Some("Magic action: gain the Invisible condition for 1 hour or until \
                              you dismiss it (no action required). Ends early if you deal damage \
                              to a creature or force a creature to make a saving throw. \
                              Once per Long Rest, or expend a Psionic Energy Die to restore."),
                        1, "long_rest").await;
                }
            }
            17 => {
                // Update dice: 12×d12
                let _ = sqlx::query(
                    "UPDATE superiority_dice SET max_dice = 12, current_dice = 12, die_size = 12
                     WHERE player_id = ? AND pool_name = 'Soulknife'"
                ).bind(player_id).execute(pool).await;
 
                if !has("Rend Mind") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Rend Mind",
                        Some("When you deal Sneak Attack damage with your Psychic Blades, force \
                              the target to make a WIS save (DC = 8 + DEX mod + Prof Bonus). \
                              Fail: Stunned for 1 minute (repeats save at end of each of its turns). \
                              Once per Long Rest, or expend 3 Psionic Energy Dice to restore."),
                        1, "long_rest").await;
                }
            }
            _ => {}
        },
 
        Some("Thief") => match new_level {
            3 => {
                if !has("Fast Hands") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Fast Hands",
                        Some("Bonus Action — choose one: \
                              Sleight of Hand: make a DEX (Sleight of Hand) check to pick a lock \
                                or disarm a trap with Thieves' Tools, or to pick a pocket. \
                              Use an Object: take the Utilize action, or take the Magic action \
                                to use a magic item that requires that action."),
                        1, "per_turn").await;
                }
                if !has("Second-Story Work") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Second-Story Work",
                        Some("Climber: you gain a Climb Speed equal to your Speed. \
                              Jumper: you can use your Dexterity modifier instead of Strength \
                              to determine your jump distance."),
                        1, "manual").await;
                }
            }
            9 => {
                if !has("Supreme Sneak") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Supreme Sneak",
                        Some("New Cunning Strike option — Stealth Attack (Cost: 1d6): if you have \
                              the Hide action's Invisible condition, this attack doesn't end that \
                              condition on you if you end the turn behind Three-Quarters Cover or \
                              Total Cover."),
                        1, "manual").await;
                }
            }
            13 => {
                if !has("Use Magic Device") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Use Magic Device",
                        Some("Attunement: you can attune to up to four magic items at once. \
                              Charges: when you use a magic item property that expends charges, \
                                roll 1d6 — on a 6, use the property without expending charges. \
                              Scrolls: you can use any Spell Scroll (INT as spellcasting ability). \
                                Cantrips and level 1 spells: cast reliably. \
                                Higher level: INT (Arcana) check DC 10 + spell level to cast; \
                                on failure, the scroll disintegrates."),
                        1, "manual").await;
                }
            }
            17 => {
                if !has("Thief's Reflexes") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Thief's Reflexes",
                        Some("You can take two turns during the first round of any combat. \
                              First turn at your normal Initiative, second turn at your \
                              Initiative minus 10."),
                        1, "manual").await;
                }
            }
            _ => {}
        },
 
        _ => {}
    }
}

async fn seed_level_up_abilities_sorcerer(
    pool: &sqlx::SqlitePool,
    campaign_id: &str,
    player_id: &str,
    new_level: i64,
    subclass: Option<&str>,
) {
    let existing = world::get_abilities(pool, player_id, "player").await.unwrap_or_default();
    let has = |name: &str| existing.iter().any(|a| a.name == name);
 
    let player = match player::get_player(pool, player_id).await {
        Ok(Some(p)) => p,
        _ => return,
    };
    let cha_mod = crate::models::Player::modifier(player.cha).max(1);
 
    // ── Base class features ───────────────────────────────────────────────────
 
    match new_level {
        2 => {
            // Sorcery Points unlock — updated in level_up_player
            // Metamagic is already seeded at creation
        }
        5 => {
            if !has("Sorcerous Restoration") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Sorcerous Restoration",
                    Some("When you finish a Short Rest, you can regain expended Sorcery Points, \
                          up to a maximum equal to half your Sorcerer level (round down). \
                          Once per Long Rest."),
                    1, "long_rest").await;
            }
        }
        7 => {
            if !has("Sorcery Incarnate") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Sorcery Incarnate",
                    Some("If you have no uses of Innate Sorcery left, you can use it by spending \
                          2 Sorcery Points when you take the Bonus Action to activate it. \
                          Additionally, while Innate Sorcery is active, you can use up to TWO \
                          Metamagic options on each spell you cast."),
                    1, "manual").await;
            }
        }
        20 => {
            if !has("Arcane Apotheosis") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Arcane Apotheosis",
                    Some("While your Innate Sorcery feature is active, you can use one Metamagic \
                          option on each of your turns without spending Sorcery Points on it."),
                    1, "manual").await;
            }
        }
        _ => {}
    }
 
    // ── Subclass features ─────────────────────────────────────────────────────
 
    match subclass {
 
        Some("Aberrant Sorcery") => match new_level {
            3 => {
                if !has("Telepathic Speech") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Telepathic Speech",
                        Some("Bonus Action: choose one creature you can see within 30 ft. You and \
                              the target can communicate telepathically while within a number of \
                              miles equal to your CHA modifier (min 1). Lasts Sorcerer level minutes. \
                              Ends early if you form a connection with a different creature."),
                        1, "manual").await;
                }
                if !has("Psionic Spells") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Psionic Spells",
                        Some("Always prepared (don't count against your limit): \
                              L3: Arms of Hadar, Calm Emotions, Detect Thoughts, Dissonant Whispers, Mind Sliver. \
                              L5: Hunger of Hadar, Sending. \
                              L7: Evard's Black Tentacles, Summon Aberration. \
                              L9: Rary's Telepathic Bond, Telekinesis."),
                        1, "manual").await;
                }
                for spell_name in &["Arms of Hadar", "Calm Emotions", "Detect Thoughts", "Dissonant Whispers"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "aberrant_sorcery").await;
                        }
                    }
                }
                // Mind Sliver is a cantrip — seed it
                if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, "Mind Sliver").await {
                    if let Some(id) = s["id"].as_str() {
                        let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "aberrant_sorcery").await;
                    }
                }
            }
            5 => {
                for spell_name in &["Hunger of Hadar", "Sending"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "aberrant_sorcery").await;
                        }
                    }
                }
            }
            6 => {
                if !has("Psionic Sorcery") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Psionic Sorcery",
                        Some("When you cast any level 1+ spell from Psionic Spells, you can cast it \
                              by expending a spell slot as normal OR by spending a number of Sorcery \
                              Points equal to the spell's level. If cast with Sorcery Points, it \
                              requires no Verbal or Somatic components (and no Material components \
                              unless consumed or have a GP cost specified)."),
                        1, "manual").await;
                }
                if !has("Psychic Defenses") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Psychic Defenses",
                        Some("You have Resistance to Psychic damage and Advantage on saving throws \
                              to avoid or end the Charmed or Frightened condition."),
                        1, "manual").await;
                }
            }
            7 => {
                for spell_name in &["Evard's Black Tentacles", "Summon Aberration"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "aberrant_sorcery").await;
                        }
                    }
                }
            }
            9 => {
                for spell_name in &["Rary's Telepathic Bond", "Telekinesis"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "aberrant_sorcery").await;
                        }
                    }
                }
            }
            14 => {
                if !has("Revelation in Flesh") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Revelation in Flesh",
                        Some("Bonus Action: spend 1+ Sorcery Points to alter your body for 10 minutes. \
                              Each SP grants one benefit: \
                              Aquatic Adaptation — Swim Speed = 2× Speed, breathe underwater. \
                              Glistening Flight — Fly Speed = Speed, can hover. \
                              See the Invisible — see Invisible creatures within 60 ft not behind Total Cover. \
                              Wormlike Movement — move through spaces as narrow as 1 inch; \
                                spend 5 ft of movement to escape nonmagical restraints or Grappled."),
                        1, "manual").await;
                }
            }
            18 => {
                if !has("Warping Implosion") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Warping Implosion",
                        Some("Magic action: teleport to an unoccupied space you can see within 120 ft. \
                              Each creature within 30 ft of your previous space must make a STR save \
                              (DC = spell save DC). Fail: 3d10 Force damage and pulled toward your \
                              former space (as close as possible). Success: half damage only. \
                              Once per Long Rest, or spend 5 Sorcery Points to restore."),
                        1, "long_rest").await;
                }
            }
            _ => {}
        },
 
        Some("Clockwork Sorcery") => match new_level {
            3 => {
                if !has("Restore Balance") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Restore Balance",
                        Some("Reaction: when a creature you can see within 60 ft is about to roll \
                              a d20 with Advantage or Disadvantage, prevent the roll from being \
                              affected by either. Uses = CHA modifier (min 1). Recharges Long Rest."),
                        cha_mod, "long_rest").await;
                }
                if !has("Clockwork Spells") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Clockwork Spells",
                        Some("Always prepared: L3: Aid, Alarm, Lesser Restoration, Protection from Evil and Good. \
                              L5: Dispel Magic, Protection from Energy. \
                              L7: Freedom of Movement, Summon Construct. \
                              L9: Greater Restoration, Wall of Force."),
                        1, "manual").await;
                }
                for spell_name in &["Aid", "Alarm", "Lesser Restoration", "Protection from Evil and Good"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "clockwork_sorcery").await;
                        }
                    }
                }
            }
            5 => {
                for spell_name in &["Dispel Magic", "Protection from Energy"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "clockwork_sorcery").await;
                        }
                    }
                }
            }
            6 => {
                if !has("Bastion of Law") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Bastion of Law",
                        Some("Magic action: expend 1-5 Sorcery Points to create a ward on yourself \
                              or a creature you can see within 30 ft. The ward has a number of d8s \
                              equal to SP spent. When the warded creature takes damage, it can \
                              expend any number of those dice, roll them, and reduce the damage by \
                              the total. The ward lasts until Long Rest or you use this again."),
                        1, "manual").await;
                }
            }
            7 => {
                for spell_name in &["Freedom of Movement", "Summon Construct"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "clockwork_sorcery").await;
                        }
                    }
                }
            }
            9 => {
                for spell_name in &["Greater Restoration", "Wall of Force"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "clockwork_sorcery").await;
                        }
                    }
                }
            }
            14 => {
                if !has("Trance of Order") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Trance of Order",
                        Some("Bonus Action: enter a trance for 1 minute. While active: attack rolls \
                              against you can't benefit from Advantage; whenever you make a D20 Test, \
                              treat a roll of 9 or lower as a 10. Once per Long Rest, or spend 5 \
                              Sorcery Points to restore."),
                        1, "long_rest").await;
                }
            }
            18 => {
                if !has("Clockwork Cavalcade") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Clockwork Cavalcade",
                        Some("Magic action: summon order spirits in a 30-foot Cube originating from you. \
                              They create these effects before vanishing: \
                              Heal — restore up to 100 HP divided among creatures in the Cube. \
                              Repair — damaged objects in the Cube are instantly repaired. \
                              Dispel — every spell of level 6 or lower ends on targets of your choice. \
                              Once per Long Rest, or spend 7 Sorcery Points to restore."),
                        1, "long_rest").await;
                }
            }
            _ => {}
        },
 
        Some("Draconic Sorcery") => match new_level {
            3 => {
                // Draconic Resilience: +3 max HP at L3, +1 per subsequent level (handled in level_up_player)
                let _ = sqlx::query(
                    "UPDATE players SET max_hp = max_hp + 3, current_hp = current_hp + 3,
                     updated_at = datetime('now') WHERE id = ?"
                )
                .bind(player_id).execute(pool).await;
 
                if !has("Draconic Resilience") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Draconic Resilience",
                        Some("Your draconic magic manifests physically. Your max HP increases by 3 \
                              at level 3, plus 1 for each subsequent Sorcerer level. \
                              Unarmored Defense: while not wearing armor, your base AC equals \
                              10 + DEX modifier + CHA modifier."),
                        1, "manual").await;
                }
                if !has("Draconic Spells") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Draconic Spells",
                        Some("Always prepared: L3: Alter Self, Chromatic Orb, Command, Dragon's Breath. \
                              L5: Fear, Fly. L7: Arcane Eye, Charm Monster. \
                              L9: Legend Lore, Summon Dragon."),
                        1, "manual").await;
                }
                for spell_name in &["Alter Self", "Chromatic Orb", "Command", "Dragon's Breath"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "draconic_sorcery").await;
                        }
                    }
                }
            }
            5 => {
                for spell_name in &["Fear", "Fly"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "draconic_sorcery").await;
                        }
                    }
                }
            }
            6 => {
                if !has("Elemental Affinity") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Elemental Affinity",
                        Some("Choose a damage type associated with dragons: Acid, Cold, Fire, \
                              Lightning, or Poison. You have Resistance to that type. When you cast \
                              a spell dealing damage of that type, add your CHA modifier to one \
                              damage roll of that spell."),
                        1, "manual").await;
                }
            }
            7 => {
                for spell_name in &["Arcane Eye", "Charm Monster"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "draconic_sorcery").await;
                        }
                    }
                }
            }
            9 => {
                for spell_name in &["Legend Lore", "Summon Dragon"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "draconic_sorcery").await;
                        }
                    }
                }
            }
            14 => {
                if !has("Dragon Wings") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Dragon Wings",
                        Some("Bonus Action: cause draconic wings to appear on your back for 1 hour \
                              (or dismiss freely). While active: Fly Speed of 60 feet. \
                              Once per Long Rest, or spend 3 Sorcery Points to restore."),
                        1, "long_rest").await;
                }
            }
            18 => {
                if !has("Dragon Companion") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Dragon Companion",
                        Some("You can cast Summon Dragon without a Material component. Once per Long \
                              Rest, cast it without a spell slot. When you start casting, you can \
                              modify it to remove the Concentration requirement (duration becomes \
                              1 minute for that casting)."),
                        1, "long_rest").await;
                }
                if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, "Summon Dragon").await {
                    if let Some(id) = s["id"].as_str() {
                        let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "draconic_sorcery").await;
                    }
                }
            }
            _ => {}
        },
 
        Some("Wild Magic Sorcery") => match new_level {
            3 => {
                if !has("Wild Magic Surge") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Wild Magic Surge",
                        Some("Once per turn, immediately after casting a Sorcerer spell with a slot, \
                              you can roll 1d20. On a 20, roll on the Wild Magic Surge table for a \
                              random magical effect. If the effect is a spell, it isn't affected by \
                              your Metamagic. Level 14 (Controlled Chaos): roll twice, use either. \
                              Level 18 (Tamed Surge): choose any effect from the table instead of \
                              rolling (once per Long Rest)."),
                        1, "manual").await;
                }
                if !has("Tides of Chaos") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Tides of Chaos",
                        Some("Gain Advantage on one D20 Test before you roll. After doing so, you \
                              must cast a Sorcerer spell with a slot or finish a Long Rest before \
                              using this again. If you cast a Sorcerer spell with a slot before \
                              finishing a Long Rest, you automatically roll on the Wild Magic Surge \
                              table."),
                        1, "long_rest").await;
                }
            }
            6 => {
                if !has("Bend Luck") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Bend Luck",
                        Some("Reaction: immediately after another creature you can see rolls the d20 \
                              for a D20 Test, spend 1 Sorcery Point to roll 1d4 and apply the number \
                              rolled as a bonus or penalty (your choice) to that d20 roll."),
                        1, "per_turn").await;
                }
            }
            14 => {
                if !has("Controlled Chaos") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Controlled Chaos",
                        Some("Whenever you roll on the Wild Magic Surge table, you can roll twice \
                              and use either number."),
                        1, "manual").await;
                }
            }
            18 => {
                if !has("Tamed Surge") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Tamed Surge",
                        Some("Immediately after casting a Sorcerer spell with a slot, you can create \
                              an effect of your choice from the Wild Magic Surge table instead of \
                              rolling. You can choose any effect except the final row, and if it \
                              involves a roll, you must make it. Once per Long Rest."),
                        1, "long_rest").await;
                }
            }
            _ => {}
        },
 
        _ => {}
    }
}

async fn seed_level_up_abilities_warlock(
    pool: &sqlx::SqlitePool,
    campaign_id: &str,
    player_id: &str,
    new_level: i64,
    subclass: Option<&str>,
) {
    let existing = world::get_abilities(pool, player_id, "player").await.unwrap_or_default();
    let has = |name: &str| existing.iter().any(|a| a.name == name);
 
    let player = match player::get_player(pool, player_id).await {
        Ok(Some(p)) => p,
        _ => return,
    };
    let cha_mod = crate::models::Player::modifier(player.cha).max(1);
 
    // ── Base class features ───────────────────────────────────────────────────
 
    match new_level {
        2 => {
            if !has("Magical Cunning") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Magical Cunning",
                    Some("You can perform an esoteric rite for 1 minute. At the end, regain \
                          expended Pact Magic slots up to half your maximum (round up). \
                          Once per Long Rest. \
                          Level 20 (Eldritch Master): Magical Cunning regains ALL expended slots."),
                    1, "long_rest").await;
            }
        }
        9 => {
            if !has("Contact Patron") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Contact Patron",
                    Some("You always have Contact Other Plane prepared. With this feature, \
                          cast it without expending a spell slot to contact your patron, \
                          and automatically succeed on the saving throw. Once per Long Rest."),
                    1, "long_rest").await;
            }
            // Learn Contact Other Plane as always-prepared
            if let Ok(Some(spell)) = spells_db::get_spell_by_name(pool, "Contact Other Plane").await {
                if let Some(id) = spell["id"].as_str() {
                    let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "warlock").await;
                }
            }
        }
        11 => {
            if !has("Mystic Arcanum (Level 6)") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Mystic Arcanum (Level 6)",
                    Some("Your patron grants a magical secret. Choose one level 6 Warlock spell. \
                          Cast it once without expending a spell slot per Long Rest. \
                          At levels 13 (L7), 15 (L8), and 17 (L9), gain additional arcanum spells. \
                          Regain all uses on Long Rest. Can replace one arcanum per level-up."),
                    1, "long_rest").await;
            }
        }
        13 => {
            if !has("Mystic Arcanum (Level 7)") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Mystic Arcanum (Level 7)",
                    Some("Choose one level 7 Warlock spell as an arcanum. Cast it once without \
                          expending a spell slot per Long Rest."),
                    1, "long_rest").await;
            }
        }
        15 => {
            if !has("Mystic Arcanum (Level 8)") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Mystic Arcanum (Level 8)",
                    Some("Choose one level 8 Warlock spell as an arcanum. Cast it once without \
                          expending a spell slot per Long Rest."),
                    1, "long_rest").await;
            }
        }
        17 => {
            if !has("Mystic Arcanum (Level 9)") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Mystic Arcanum (Level 9)",
                    Some("Choose one level 9 Warlock spell as an arcanum. Cast it once without \
                          expending a spell slot per Long Rest."),
                    1, "long_rest").await;
            }
        }
        20 => {
            if !has("Eldritch Master") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Eldritch Master",
                    Some("When you use Magical Cunning, you regain ALL your expended Pact Magic \
                          spell slots instead of half."),
                    1, "manual").await;
            }
        }
        _ => {}
    }
 
    // ── Subclass features ─────────────────────────────────────────────────────
 
    match subclass {
 
        Some("Archfey Patron") => match new_level {
            3 => {
                if !has("Steps of the Fey") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Steps of the Fey",
                        Some("Cast Misty Step without expending a spell slot. \
                              Uses = CHA modifier (min 1). Recharges on Long Rest. \
                              Each time you cast it, choose one additional effect: \
                              Refreshing Step — you or a creature within 10 ft gain 1d10 Temp HP. \
                              Taunting Step — creatures within 5 ft of the space you left make WIS \
                                save or have Disadvantage on attacks against creatures other than you \
                                until your next turn. \
                              Level 6 additional options: \
                              Disappearing Step — Invisible until start of next turn or until you \
                                attack/damage/cast. \
                              Dreadful Step — creatures within 5 ft of space you left or appear \
                                make WIS save or take 2d10 Psychic."),
                        cha_mod, "long_rest").await;
                }
                if !has("Archfey Spells") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Archfey Spells",
                        Some("Always prepared: L3: Calm Emotions, Faerie Fire, Misty Step, \
                              Phantasmal Force, Sleep. L5: Blink, Plant Growth. \
                              L7: Dominate Beast, Greater Invisibility. \
                              L9: Dominate Person, Seeming."),
                        1, "manual").await;
                }
                for spell_name in &["Calm Emotions", "Faerie Fire", "Misty Step", "Phantasmal Force", "Sleep"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "archfey_patron").await;
                        }
                    }
                }
            }
            5 => {
                for spell_name in &["Blink", "Plant Growth"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "archfey_patron").await;
                        }
                    }
                }
            }
            6 => {
                if !has("Misty Escape") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Misty Escape",
                        Some("You can cast Misty Step as a Reaction when you take damage. \
                              Two new Steps of the Fey options are now available: \
                              Disappearing Step (Invisible until start of next turn or until \
                                you attack/damage/cast) and Dreadful Step (2d10 Psychic damage \
                                to creatures within 5 ft of departure or arrival point, WIS save)."),
                        1, "manual").await;
                }
            }
            7 => {
                for spell_name in &["Dominate Beast", "Greater Invisibility"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "archfey_patron").await;
                        }
                    }
                }
            }
            9 => {
                for spell_name in &["Dominate Person", "Seeming"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "archfey_patron").await;
                        }
                    }
                }
            }
            10 => {
                if !has("Beguiling Defenses") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Beguiling Defenses",
                        Some("You are immune to the Charmed condition. \
                              Reaction: when a creature you can see hits you with an attack roll, \
                              reduce damage taken by half (round down) and force the attacker to make \
                              a WIS save (DC = spell save DC). On a failed save, attacker takes \
                              Psychic damage equal to the damage you take. \
                              Once per Long Rest, or expend a Pact Magic slot to restore."),
                        1, "long_rest").await;
                }
            }
            14 => {
                if !has("Bewitching Magic") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Bewitching Magic",
                        Some("Immediately after you cast an Enchantment or Illusion spell using \
                              an action and a spell slot, you can cast Misty Step as part of the \
                              same action without expending a spell slot."),
                        1, "manual").await;
                }
            }
            _ => {}
        },
 
        Some("Celestial Patron") => match new_level {
            3 => {
                if !has("Healing Light") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Healing Light",
                        Some("Pool of d6s = 1 + Warlock level. Bonus Action: heal yourself or \
                              a creature you can see within 60 ft by expending dice (max dice = \
                              CHA modifier, min 1). Roll the dice and restore that many HP. \
                              Pool fully restores on Long Rest."),
                        new_level + 1, "long_rest").await;
                }
                if !has("Celestial Spells") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Celestial Spells",
                        Some("Always prepared: L3: Aid, Cure Wounds, Guiding Bolt, Lesser Restoration, \
                              Light, Sacred Flame. L5: Daylight, Revivify. \
                              L7: Guardian of Faith, Wall of Fire. \
                              L9: Greater Restoration, Summon Celestial."),
                        1, "manual").await;
                }
                for spell_name in &["Aid", "Cure Wounds", "Guiding Bolt", "Lesser Restoration"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "celestial_patron").await;
                        }
                    }
                }
                // Light and Sacred Flame are cantrips
                for spell_name in &["Light", "Sacred Flame"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "cantrip", "celestial_patron").await;
                        }
                    }
                }
            }
            4 => {
                // Update Healing Light pool (increases every level)
                if let Some(a) = existing.iter().find(|a| a.name == "Healing Light") {
                    let new_pool = new_level + 1;
                    let _ = sqlx::query("UPDATE abilities SET max_uses = ?, current_uses = ? WHERE id = ?")
                        .bind(new_pool).bind(new_pool).bind(&a.id)
                        .execute(pool).await;
                }
            }
            5 => {
                for spell_name in &["Daylight", "Revivify"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "celestial_patron").await;
                        }
                    }
                }
                // Update Healing Light pool
                if let Some(a) = existing.iter().find(|a| a.name == "Healing Light") {
                    let new_pool = new_level + 1;
                    let _ = sqlx::query("UPDATE abilities SET max_uses = ?, current_uses = ? WHERE id = ?")
                        .bind(new_pool).bind(new_pool).bind(&a.id).execute(pool).await;
                }
            }
            6 => {
                if !has("Radiant Soul") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Radiant Soul",
                        Some("You have Resistance to Radiant damage. Once per turn, when a spell \
                              you cast deals Radiant or Fire damage, add your CHA modifier to that \
                              spell's damage against one target."),
                        1, "manual").await;
                }
                // Update Healing Light pool
                if let Some(a) = existing.iter().find(|a| a.name == "Healing Light") {
                    let new_pool = new_level + 1;
                    let _ = sqlx::query("UPDATE abilities SET max_uses = ?, current_uses = ? WHERE id = ?")
                        .bind(new_pool).bind(new_pool).bind(&a.id).execute(pool).await;
                }
            }
            7 => {
                for spell_name in &["Guardian of Faith", "Wall of Fire"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "celestial_patron").await;
                        }
                    }
                }
                // Update Healing Light pool
                if let Some(a) = existing.iter().find(|a| a.name == "Healing Light") {
                    let new_pool = new_level + 1;
                    let _ = sqlx::query("UPDATE abilities SET max_uses = ?, current_uses = ? WHERE id = ?")
                        .bind(new_pool).bind(new_pool).bind(&a.id).execute(pool).await;
                }
            }
            8..=20 => {
                // Update Healing Light pool every level
                if let Some(a) = existing.iter().find(|a| a.name == "Healing Light") {
                    let new_pool = new_level + 1;
                    let _ = sqlx::query("UPDATE abilities SET max_uses = ?, current_uses = ? WHERE id = ?")
                        .bind(new_pool).bind(new_pool).bind(&a.id).execute(pool).await;
                }
                match new_level {
                    9 => {
                        for spell_name in &["Greater Restoration", "Summon Celestial"] {
                            if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                                if let Some(id) = s["id"].as_str() {
                                    let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "celestial_patron").await;
                                }
                            }
                        }
                    }
                    10 => {
                        if !has("Celestial Resilience") {
                            let _ = world::create_ability(pool, campaign_id, "player", player_id,
                                "Celestial Resilience",
                                Some("Whenever you use Magical Cunning or finish a Short or Long Rest, \
                                      gain Temporary HP = Warlock level + CHA modifier. \
                                      Additionally, choose up to 5 creatures you can see — each gains \
                                      Temporary HP = half your Warlock level + CHA modifier."),
                                1, "manual").await;
                        }
                    }
                    14 => {
                        if !has("Searing Vengeance") {
                            let _ = world::create_ability(pool, campaign_id, "player", player_id,
                                "Searing Vengeance",
                                Some("When you or an ally within 60 ft is about to make a Death Saving \
                                      Throw, unleash radiant energy: the creature regains HP equal to \
                                      half its max HP and can end the Prone condition. Each creature of \
                                      your choice within 30 ft of it takes 2d8 + CHA modifier Radiant \
                                      damage and has the Blinded condition until end of current turn. \
                                      Once per Long Rest."),
                                1, "long_rest").await;
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        },
 
        Some("Fiend Patron") => match new_level {
            3 => {
                if !has("Dark One's Blessing") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Dark One's Blessing",
                        Some("When you or someone within 10 ft of you reduces an enemy to 0 HP, \
                              you gain Temporary HP equal to your CHA modifier + Warlock level \
                              (minimum 1)."),
                        1, "manual").await;
                }
                if !has("Fiend Spells") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Fiend Spells",
                        Some("Always prepared: L3: Burning Hands, Command, Scorching Ray, Suggestion. \
                              L5: Fireball, Stinking Cloud. L7: Fire Shield, Wall of Fire. \
                              L9: Geas, Insect Plague."),
                        1, "manual").await;
                }
                for spell_name in &["Burning Hands", "Command", "Scorching Ray", "Suggestion"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "fiend_patron").await;
                        }
                    }
                }
            }
            5 => {
                for spell_name in &["Fireball", "Stinking Cloud"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "fiend_patron").await;
                        }
                    }
                }
            }
            6 => {
                if !has("Dark One's Own Luck") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Dark One's Own Luck",
                        Some("When you make an ability check or saving throw, add 1d10 to the roll \
                              (after seeing the roll, before effects occur). \
                              Uses = CHA modifier (min 1), max once per roll. Recharges on Long Rest."),
                        cha_mod, "long_rest").await;
                }
            }
            7 => {
                for spell_name in &["Fire Shield", "Wall of Fire"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "fiend_patron").await;
                        }
                    }
                }
            }
            9 => {
                for spell_name in &["Geas", "Insect Plague"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "fiend_patron").await;
                        }
                    }
                }
            }
            10 => {
                if !has("Fiendish Resilience") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Fiendish Resilience",
                        Some("Choose one damage type (other than Force) whenever you finish a \
                              Short or Long Rest. You have Resistance to that damage type until \
                              you choose a different one."),
                        1, "manual").await;
                }
            }
            14 => {
                if !has("Hurl Through Hell") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Hurl Through Hell",
                        Some("Once per turn when you hit a creature with an attack roll, try to \
                              transport it through the Lower Planes. Target makes a CHA save (DC = \
                              spell save DC) or disappears through a nightmare landscape until end \
                              of your next turn. Non-Fiends take 8d10 Psychic damage and have the \
                              Incapacitated condition until they return to their space. \
                              Once per Long Rest, or expend a Pact Magic slot to restore."),
                        1, "long_rest").await;
                }
            }
            _ => {}
        },
 
        Some("Great Old One Patron") => match new_level {
            3 => {
                if !has("Awakened Mind") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Awakened Mind",
                        Some("Bonus Action: choose one creature you can see within 30 ft. You and \
                              the target can communicate telepathically while within CHA modifier \
                              miles of each other (min 1 mile). Requires a shared language. \
                              Lasts Warlock level minutes. Ends early if you connect to a different \
                              creature."),
                        1, "manual").await;
                }
                if !has("Psychic Spells") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Psychic Spells",
                        Some("When you cast a Warlock spell that deals damage, you can change its \
                              damage type to Psychic. When you cast a Warlock spell that is an \
                              Enchantment or Illusion, you can do so without Verbal or Somatic \
                              components."),
                        1, "manual").await;
                }
                if !has("Great Old One Spells") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Great Old One Spells",
                        Some("Always prepared: L3: Detect Thoughts, Dissonant Whispers, \
                              Phantasmal Force, Tasha's Hideous Laughter. \
                              L5: Clairvoyance, Hunger of Hadar. \
                              L7: Confusion, Summon Aberration. \
                              L9: Modify Memory, Telekinesis."),
                        1, "manual").await;
                }
                for spell_name in &["Detect Thoughts", "Dissonant Whispers", "Phantasmal Force", "Tasha's Hideous Laughter"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "great_old_one").await;
                        }
                    }
                }
            }
            5 => {
                for spell_name in &["Clairvoyance", "Hunger of Hadar"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "great_old_one").await;
                        }
                    }
                }
            }
            6 => {
                if !has("Clairvoyant Combatant") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Clairvoyant Combatant",
                        Some("When you form a telepathic bond with Awakened Mind, force that creature \
                              to make a WIS save (DC = spell save DC). On a failed save, the creature \
                              has Disadvantage on attack rolls against you, and you have Advantage on \
                              attack rolls against it for the bond's duration. \
                              Once per Short or Long Rest, or expend a Pact Magic slot to restore."),
                        1, "short_rest").await;
                }
            }
            7 => {
                for spell_name in &["Confusion", "Summon Aberration"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "great_old_one").await;
                        }
                    }
                }
            }
            9 => {
                for spell_name in &["Modify Memory", "Telekinesis"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "great_old_one").await;
                        }
                    }
                }
            }
            10 => {
                if !has("Eldritch Hex") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Eldritch Hex",
                        Some("You always have the Hex spell prepared. When you cast Hex and choose \
                              an ability, the target also has Disadvantage on saving throws of the \
                              chosen ability for the duration of the spell."),
                        1, "manual").await;
                }
                if !has("Thought Shield") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Thought Shield",
                        Some("Your thoughts can't be read by telepathy or other means unless you \
                              allow it. You have Resistance to Psychic damage. Whenever a creature \
                              deals Psychic damage to you, that creature takes the same amount of \
                              damage that you take."),
                        1, "manual").await;
                }
                if let Ok(Some(spell)) = spells_db::get_spell_by_name(pool, "Hex").await {
                    if let Some(id) = spell["id"].as_str() {
                        let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "great_old_one").await;
                    }
                }
            }
            14 => {
                if !has("Create Thrall") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Create Thrall",
                        Some("When you cast Summon Aberration, you can modify it to not require \
                              Concentration (duration becomes 1 minute). The summoned Aberration \
                              gains Temporary HP = Warlock level + CHA modifier. The first time \
                              each turn the Aberration hits a creature under your Hex, it deals \
                              extra Psychic damage equal to Hex's bonus damage."),
                        1, "manual").await;
                }
            }
            _ => {}
        },
 
        _ => {}
    }
}

async fn seed_level_up_abilities_wizard(
    pool: &sqlx::SqlitePool,
    campaign_id: &str,
    player_id: &str,
    new_level: i64,
    subclass: Option<&str>,
) {
    let existing = world::get_abilities(pool, player_id, "player").await.unwrap_or_default();
    let has = |name: &str| existing.iter().any(|a| a.name == name);
 
    let player = match player::get_player(pool, player_id).await {
        Ok(Some(p)) => p,
        _ => return,
    };
    let int_mod = crate::models::Player::modifier(player.int).max(1);
 
    // ── Base class features ───────────────────────────────────────────────────
 
    match new_level {
        2 => {
            if !has("Scholar") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Scholar",
                    Some("You have Expertise in one of the following skills in which you have \
                          proficiency: Arcana, History, Investigation, Medicine, Nature, or Religion."),
                    1, "manual").await;
            }
        }
        5 => {
            if !has("Memorize Spell") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Memorize Spell",
                    Some("Whenever you finish a Short Rest, you can study your spellbook and \
                          replace one of the level 1+ Wizard spells you have prepared for your \
                          Spellcasting feature with another level 1+ spell from your spellbook."),
                    1, "manual").await;
            }
        }
        18 => {
            if !has("Spell Mastery") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Spell Mastery",
                    Some("Choose one level 1 spell and one level 2 spell from your spellbook that \
                          have a casting time of an action. You always have those spells prepared \
                          and can cast them at their lowest level without expending a spell slot. \
                          To cast either at a higher level, you must expend a slot. \
                          On Long Rest, you can replace one chosen spell with an eligible spell \
                          of the same level from your spellbook."),
                    1, "manual").await;
            }
        }
        20 => {
            if !has("Signature Spells") {
                let _ = world::create_ability(pool, campaign_id, "player", player_id,
                    "Signature Spells",
                    Some("Choose two level 3 spells from your spellbook as your signature spells. \
                          You always have them prepared. You can cast each once at level 3 without \
                          expending a spell slot per Short or Long Rest. To cast at a higher level, \
                          expend a slot."),
                    2, "short_rest").await;
            }
        }
        _ => {}
    }
 
    // ── Subclass features ─────────────────────────────────────────────────────
 
    match subclass {
 
        Some("Abjurer") => match new_level {
            3 => {
                if !has("Abjuration Savant") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Abjuration Savant",
                        Some("Choose two Abjuration spells (level 1-2) to add to your spellbook for \
                              free. Whenever you gain access to a new spell slot level in this class, \
                              add one Abjuration Wizard spell of that level to your spellbook for free."),
                        1, "manual").await;
                }
                if !has("Arcane Ward") {
                    let wiz_level = new_level;
                    let ward_max = 2 * wiz_level + int_mod;
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Arcane Ward",
                        Some(&format!(
                            "When you cast an Abjuration spell with a slot, simultaneously create \
                             a magical ward with HP max = 2×Wizard level + INT modifier ({} HP). \
                             Lasts until Long Rest. Damage hits the ward first (applying your \
                             Resistances/Vulnerabilities). If ward drops to 0, you take overflow. \
                             Recharge: casting an Abjuration spell with a slot restores 2× the \
                             slot level HP. Or Bonus Action: expend a slot to restore 2× slot level HP. \
                             Can't create a new ward until Long Rest.",
                            ward_max
                        )),
                        1, "long_rest").await;
                }
            }
            6 => {
                if !has("Projected Ward") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Projected Ward",
                        Some("Reaction: when a creature you can see within 30 ft takes damage, \
                              cause your Arcane Ward to absorb that damage instead. If this reduces \
                              the ward to 0 HP, the warded creature takes any remaining damage \
                              (applying its own Resistances/Vulnerabilities)."),
                        1, "per_turn").await;
                }
            }
            10 => {
                if !has("Spell Breaker") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Spell Breaker",
                        Some("You always have Counterspell and Dispel Magic prepared. \
                              You can cast Dispel Magic as a Bonus Action. \
                              You add your Proficiency Bonus to its ability check. \
                              When you cast either spell with a slot and the spell fails to stop \
                              a spell, the slot isn't expended."),
                        1, "manual").await;
                }
                for spell_name in &["Counterspell", "Dispel Magic"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "abjurer").await;
                        }
                    }
                }
            }
            14 => {
                if !has("Spell Resistance") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Spell Resistance",
                        Some("You have Advantage on saving throws against spells. \
                              You have Resistance to the damage of spells."),
                        1, "manual").await;
                }
            }
            _ => {}
        },
 
        Some("Diviner") => match new_level {
            3 => {
                if !has("Divination Savant") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Divination Savant",
                        Some("Choose two Divination spells (level 1-2) to add to your spellbook for \
                              free. Whenever you gain access to a new spell slot level, add one \
                              Divination Wizard spell of that level to your spellbook for free."),
                        1, "manual").await;
                }
                if !has("Portent") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Portent",
                        Some("When you finish a Long Rest, roll two d20s and record the results. \
                              Before a D20 Test, you can replace it with one of your portent rolls \
                              (before the roll, once per turn). Each portent roll can be used once. \
                              Unused rolls are lost on your next Long Rest. \
                              Level 14 (Greater Portent): roll THREE d20s instead of two."),
                        2, "long_rest").await;
                }
            }
            6 => {
                if !has("Expert Divination") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Expert Divination",
                        Some("When you cast a Divination spell using a level 2+ spell slot, regain \
                              one expended spell slot. The recovered slot must be lower level than \
                              the one you expended and no higher than level 5."),
                        1, "manual").await;
                }
            }
            10 => {
                if !has("The Third Eye") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "The Third Eye",
                        Some("Bonus Action: choose one benefit lasting until you start a Short or \
                              Long Rest. Can't use this feature again until you finish a Short or \
                              Long Rest. \
                              Darkvision — gain Darkvision 120 ft. \
                              Greater Comprehension — read any language. \
                              See Invisibility — cast See Invisibility without expending a slot."),
                        1, "short_rest").await;
                }
            }
            14 => {
                if !has("Greater Portent") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Greater Portent",
                        Some("Your Portent feature now rolls three d20s each Long Rest instead \
                              of two, giving you an additional foretelling roll."),
                        1, "manual").await;
                }
                // Update Portent to 3 uses
                if let Some(a) = existing.iter().find(|a| a.name == "Portent") {
                    let _ = sqlx::query("UPDATE abilities SET max_uses = 3, current_uses = 3 WHERE id = ?")
                        .bind(&a.id).execute(pool).await;
                }
            }
            _ => {}
        },
 
        Some("Evoker") => match new_level {
            3 => {
                if !has("Evocation Savant") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Evocation Savant",
                        Some("Choose two Evocation spells (level 1-2) to add to your spellbook for \
                              free. Whenever you gain access to a new spell slot level, add one \
                              Evocation Wizard spell of that level to your spellbook for free."),
                        1, "manual").await;
                }
                if !has("Potent Cantrip") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Potent Cantrip",
                        Some("When you cast a cantrip at a creature and miss the attack roll or \
                              the target succeeds on its saving throw, the target takes half the \
                              cantrip's damage (if any) but suffers no additional effects."),
                        1, "manual").await;
                }
            }
            6 => {
                if !has("Sculpt Spells") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Sculpt Spells",
                        Some("When you cast an Evocation spell that affects other creatures you can \
                              see, choose a number of them equal to 1 plus the spell's level. Chosen \
                              creatures automatically succeed on their saving throw against the spell \
                              and take no damage if they would normally take half on a successful save."),
                        1, "manual").await;
                }
            }
            10 => {
                if !has("Empowered Evocation") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Empowered Evocation",
                        Some("Whenever you cast a Wizard spell from the Evocation school, add your \
                              Intelligence modifier to one damage roll of that spell."),
                        1, "manual").await;
                }
            }
            14 => {
                if !has("Overchannel") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Overchannel",
                        Some("When you cast a Wizard spell with a slot of levels 1-5 that deals \
                              damage, you can deal maximum damage with that spell on the turn you cast it. \
                              First use per Long Rest: no adverse effect. \
                              Each subsequent use before Long Rest: take 2d12 Necrotic damage per \
                              spell slot level immediately after casting (ignores Resistance/Immunity). \
                              Each further use adds 1d12 Necrotic per slot level to the cost."),
                        1, "manual").await;
                }
            }
            _ => {}
        },
 
        Some("Illusionist") => match new_level {
            3 => {
                if !has("Illusion Savant") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Illusion Savant",
                        Some("Choose two Illusion spells (level 1-2) to add to your spellbook for \
                              free. Whenever you gain access to a new spell slot level, add one \
                              Illusion Wizard spell of that level to your spellbook for free."),
                        1, "manual").await;
                }
                if !has("Improved Illusions") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Improved Illusions",
                        Some("You can cast Illusion spells without providing Verbal components. \
                              If an Illusion spell you cast has a range of 10+ feet, the range \
                              increases by 60 feet. \
                              You know the Minor Illusion cantrip (if already known, learn a \
                              different Wizard cantrip; doesn't count against your cantrip total). \
                              You can create both a sound and an image with a single casting of \
                              Minor Illusion, and cast it as a Bonus Action."),
                        1, "manual").await;
                }
                // Learn Minor Illusion as a bonus cantrip
                if let Ok(Some(spell)) = spells_db::get_spell_by_name(pool, "Minor Illusion").await {
                    if let Some(id) = spell["id"].as_str() {
                        let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "cantrip", "illusionist").await;
                    }
                }
            }
            6 => {
                if !has("Phantasmal Creatures") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Phantasmal Creatures",
                        Some("You always have Summon Beast and Summon Fey prepared. When casting \
                              either, you can change its school to Illusion (summoned creature \
                              appears spectral). \
                              Free cast (no slot): cast the Illusion version once per Long Rest \
                              each, but the summoned creature has half its Hit Points. \
                              Casting with a slot works normally."),
                        2, "long_rest").await;
                }
                for spell_name in &["Summon Beast", "Summon Fey"] {
                    if let Ok(Some(s)) = spells_db::get_spell_by_name(pool, spell_name).await {
                        if let Some(id) = s["id"].as_str() {
                            let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "illusionist").await;
                        }
                    }
                }
            }
            10 => {
                if !has("Illusory Self") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Illusory Self",
                        Some("Reaction: when a creature hits you with an attack roll, interpose \
                              an illusory duplicate — the attack automatically misses and the \
                              illusion dissipates. Once per Short or Long Rest, or expend a \
                              level 2+ spell slot to restore."),
                        1, "short_rest").await;
                }
            }
            14 => {
                if !has("Illusory Reality") {
                    let _ = world::create_ability(pool, campaign_id, "player", player_id,
                        "Illusory Reality",
                        Some("When you cast an Illusion spell with a slot, choose one inanimate, \
                              nonmagical object that is part of the illusion and make it real as a \
                              Bonus Action on your turn. The object remains real for 1 minute. \
                              It can't deal damage or give any conditions during that time. \
                              Example: create an illusory bridge and make it real to cross a chasm."),
                        1, "manual").await;
                }
            }
            _ => {}
        },
 
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

// ─── Inventory handlers ───────────────────────────────────────────────────────

#[derive(Debug, serde::Deserialize)]
pub struct EquipItemRequest {
    pub item_id: String,
    pub slot: String,
}

pub async fn equip_item_handler(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
    Json(req): Json<EquipItemRequest>,
) -> impl IntoResponse {
    let pool = &state.pool;
    let p = match player::get_player_by_campaign(pool, &campaign_id).await {
        Ok(Some(p)) => p,
        _ => return (StatusCode::NOT_FOUND, Json(json!({"error": "Player not found"}))),
    };
    if let Err(e) = items::equip_item(pool, &req.item_id, &req.slot, &p.id).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})));
    }
    let new_ac = items::recalculate_ac(pool, &p.id).await.unwrap_or(p.armor_class);
    (StatusCode::OK, Json(json!({"message": "Item equipped", "new_ac": new_ac})))
}

#[derive(Debug, serde::Deserialize)]
pub struct UnequipItemRequest {
    pub item_id: String,
}

pub async fn unequip_item_handler(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
    Json(req): Json<UnequipItemRequest>,
) -> impl IntoResponse {
    let pool = &state.pool;
    let p = match player::get_player_by_campaign(pool, &campaign_id).await {
        Ok(Some(p)) => p,
        _ => return (StatusCode::NOT_FOUND, Json(json!({"error": "Player not found"}))),
    };
    if let Err(e) = items::unequip_item(pool, &req.item_id).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})));
    }
    let new_ac = items::recalculate_ac(pool, &p.id).await.unwrap_or(p.armor_class);
    (StatusCode::OK, Json(json!({"message": "Item unequipped", "new_ac": new_ac})))
}

#[derive(Debug, serde::Deserialize)]
pub struct DeleteItemRequest {
    pub item_id: String,
}

pub async fn delete_item_handler(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
    Json(req): Json<DeleteItemRequest>,
) -> impl IntoResponse {
    let pool = &state.pool;
    if let Err(e) = items::remove_item(pool, &req.item_id, 999).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})));
    }
    (StatusCode::OK, Json(json!({"message": "Item deleted"})))
}

// ─── Spell handlers ───────────────────────────────────────────────────────────
 
pub async fn get_spell_slots_handler(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
) -> impl IntoResponse {
    let pool = &state.pool;
    let p = match player::get_player_by_campaign(pool, &campaign_id).await {
        Ok(Some(p)) => p,
        _ => return (StatusCode::NOT_FOUND, Json(json!({"error": "Player not found"}))),
    };
    match spells_db::get_spell_slots(pool, &p.id).await {
        Ok(slots) => (StatusCode::OK, Json(json!({"spell_slots": slots}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    }
}
 
pub async fn get_known_spells_handler(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
) -> impl IntoResponse {
    let pool = &state.pool;
    let p = match player::get_player_by_campaign(pool, &campaign_id).await {
        Ok(Some(p)) => p,
        _ => return (StatusCode::NOT_FOUND, Json(json!({"error": "Player not found"}))),
    };
    match spells_db::get_known_spells(pool, &p.id).await {
        Ok(known) => (StatusCode::OK, Json(json!({"known_spells": known}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    }
}
 
pub async fn get_castable_spells_handler(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
) -> impl IntoResponse {
    let pool = &state.pool;
    let p = match player::get_player_by_campaign(pool, &campaign_id).await {
        Ok(Some(p)) => p,
        _ => return (StatusCode::NOT_FOUND, Json(json!({"error": "Player not found"}))),
    };
    match spells_db::get_castable_spells(pool, &p.id).await {
        Ok(result) => (StatusCode::OK, Json(result)),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    }
}
 
#[derive(Debug, serde::Deserialize)]
pub struct LearnSpellRequest {
    pub spell_id: String,
    pub spell_type: Option<String>, // defaults to "prepared"
}
 
pub async fn learn_spell_handler(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
    Json(req): Json<LearnSpellRequest>,
) -> impl IntoResponse {
    let pool = &state.pool;
    let p = match player::get_player_by_campaign(pool, &campaign_id).await {
        Ok(Some(p)) => p,
        _ => return (StatusCode::NOT_FOUND, Json(json!({"error": "Player not found"}))),
    };
 
    // Verify spell exists
    let spell = match spells_db::get_spell(pool, &req.spell_id).await {
        Ok(Some(s)) => s,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(json!({"error": "Spell not found"}))),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    };
 
    let spell_level = spell["level"].as_i64().unwrap_or(0);
    let spell_type = if spell_level == 0 {
        "cantrip".to_string()
    } else {
        req.spell_type.unwrap_or_else(|| "prepared".to_string())
    };
 
    // Enforce EK restrictions: only abjuration/evocation unless replacing
    // (This is advisory — the frontend should enforce; backend just records)
    let fighter_level = p.level;
    let max_prepared = spells_db::ek_spells_prepared(fighter_level);
 
    // Count current prepared (non-cantrip) spells
    let known = spells_db::get_known_spells(pool, &p.id).await.unwrap_or_default();
    let prepared_count = known.iter()
        .filter(|s| s["spell_type"].as_str() != Some("cantrip"))
        .count() as i64;
 
    if spell_type != "cantrip" && prepared_count >= max_prepared {
        return (StatusCode::BAD_REQUEST, Json(json!({
            "error": format!(
                "Already know {} spells (max {} at fighter level {}).",
                prepared_count, max_prepared, fighter_level
            )
        })));
    }
 
    match spells_db::learn_spell(pool, &campaign_id, &p.id, &req.spell_id, &spell_type, "eldritch_knight").await {
        Ok(status) if status == "already_known" => {
            (StatusCode::OK, Json(json!({"message": "Already know this spell", "spell": spell})))
        }
        Ok(_) => {
            let updated = spells_db::get_known_spells(pool, &p.id).await.unwrap_or_default();
            (StatusCode::OK, Json(json!({
                "message": format!("Learned {}", spell["name"]),
                "spell": spell,
                "known_spells": updated,
            })))
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    }
}
 
#[derive(Debug, serde::Deserialize)]
pub struct ForgetSpellRequest {
    pub spell_id: String,
}
 
pub async fn forget_spell_handler(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
    Json(req): Json<ForgetSpellRequest>,
) -> impl IntoResponse {
    let pool = &state.pool;
    let p = match player::get_player_by_campaign(pool, &campaign_id).await {
        Ok(Some(p)) => p,
        _ => return (StatusCode::NOT_FOUND, Json(json!({"error": "Player not found"}))),
    };
    match spells_db::forget_spell(pool, &p.id, &req.spell_id).await {
        Ok(true) => {
            let updated = spells_db::get_known_spells(pool, &p.id).await.unwrap_or_default();
            (StatusCode::OK, Json(json!({"message": "Spell forgotten", "known_spells": updated})))
        }
        Ok(false) => (StatusCode::NOT_FOUND, Json(json!({"error": "Spell not in known list"}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    }
}
 
#[derive(Debug, serde::Deserialize)]
pub struct CastSpellRequest {
    pub spell_id: String,
    pub slot_level: Option<i64>, // None for cantrips
    pub target_id: Option<String>,
    pub drop_concentration: Option<bool>,
}
 
pub async fn cast_spell_handler(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
    Json(req): Json<CastSpellRequest>,
) -> impl IntoResponse {
    let pool = &state.pool;
    let p = match player::get_player_by_campaign(pool, &campaign_id).await {
        Ok(Some(p)) => p,
        _ => return (StatusCode::NOT_FOUND, Json(json!({"error": "Player not found"}))),
    };
 
    let spell = match spells_db::get_spell(pool, &req.spell_id).await {
        Ok(Some(s)) => s,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(json!({"error": "Spell not found"}))),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    };
 
    let spell_level = spell["level"].as_i64().unwrap_or(0);
    let cast_at = req.slot_level.unwrap_or(spell_level);
 
    // Validate
    let validation = match spells_db::validate_cast(pool, &p.id, &req.spell_id, cast_at).await {
        Ok(v) => v,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    };
 
    if validation["valid"].as_bool() != Some(true) {
        return (StatusCode::BAD_REQUEST, Json(json!({
            "error": validation["reason"],
        })));
    }
 
    // Handle concentration drop
    if validation["concentration_warning"].as_bool() == Some(true) {
        if req.drop_concentration != Some(true) {
            // Ask frontend to confirm
            return (StatusCode::OK, Json(json!({
                "requires_confirmation": true,
                "message": format!(
                    "Casting this will drop concentration on {}. Confirm?",
                    validation["will_drop"]
                ),
                "will_drop": validation["will_drop"],
            })));
        }
        // Drop it
        let _ = spells_db::drop_concentration(pool, &p.id).await;
    }
 
    // Expend slot if leveled
    let slots_remaining = if spell_level > 0 {
        match spells_db::expend_spell_slot(pool, &p.id, cast_at).await {
            Ok(remaining) => Some(remaining),
            Err(e) => return (StatusCode::BAD_REQUEST, Json(json!({"error": e.to_string()}))),
        }
    } else {
        None
    };
 
    // Set concentration if needed
    if spell["concentration"].as_i64() == Some(1) {
        let _ = spells_db::set_concentration(
            pool, &campaign_id, &p.id,
            &req.spell_id,
            spell["name"].as_str().unwrap_or("Unknown"),
            None,
        ).await;
    }
 
    // Calculate damage for backend-resolved spells
    let damage_info = if spell["has_backend_resolver"].as_i64() == Some(1) {
        build_spell_damage_info(&spell, cast_at, p.level)
    } else {
        json!(null)
    };
 
    let updated_slots = spells_db::get_spell_slots(pool, &p.id).await.unwrap_or_default();
    let concentration = spells_db::get_concentration(pool, &p.id).await.unwrap_or(None);
 
    (StatusCode::OK, Json(json!({
        "message": format!("Cast {} at level {}!", spell["name"], cast_at),
        "spell": spell,
        "cast_at_level": cast_at,
        "slot_level_expended": if spell_level > 0 { Some(cast_at) } else { None },
        "slots_remaining_at_level": slots_remaining,
        "spell_slots": updated_slots,
        "concentration": concentration,
        "damage_info": damage_info,
        "target_id": req.target_id,
    })))
}
 
/// Build damage rolling info for the frontend to roll against.
fn build_spell_damage_info(spell: &Value, cast_at: i64, char_level: i64) -> Value {
    let spell_level = spell["level"].as_i64().unwrap_or(0);
    let base_dice = spell["damage_die_count"].as_i64().unwrap_or(0);
    let die = spell["damage_die"].as_str().unwrap_or("d6");
    let damage_type = spell["damage_type"].as_str().unwrap_or("unknown");
    let save_type = spell["save_type"].as_str();
    let attack_type = spell["attack_type"].as_str();
 
    let dice_count = if spell_level == 0 {
        // Cantrip scaling
        spells_db::cantrip_dice_at_level(
            base_dice,
            spell["cantrip_dice_5"].as_i64(),
            spell["cantrip_dice_11"].as_i64(),
            spell["cantrip_dice_17"].as_i64(),
            char_level,
        )
    } else {
        // Leveled spell — upcast scaling
        spells_db::upcast_dice(
            base_dice,
            spell["slot_scale_dice"].as_i64(),
            spell_level,
            cast_at,
        )
    };
 
    json!({
        "dice_count": dice_count,
        "die": die,
        "damage_type": damage_type,
        "save_type": save_type,
        "attack_type": attack_type,
        "rolls_needed": dice_count,
        "description": format!("Roll {}{}!", dice_count, die),
    })
}
 
// ─── Concentration handlers ────────────────────────────────────────────────────
 
pub async fn get_concentration_handler(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
) -> impl IntoResponse {
    let pool = &state.pool;
    let p = match player::get_player_by_campaign(pool, &campaign_id).await {
        Ok(Some(p)) => p,
        _ => return (StatusCode::NOT_FOUND, Json(json!({"error": "Player not found"}))),
    };
    match spells_db::get_concentration(pool, &p.id).await {
        Ok(c) => (StatusCode::OK, Json(json!({"concentration": c}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    }
}
 
pub async fn drop_concentration_handler(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
) -> impl IntoResponse {
    let pool = &state.pool;
    let p = match player::get_player_by_campaign(pool, &campaign_id).await {
        Ok(Some(p)) => p,
        _ => return (StatusCode::NOT_FOUND, Json(json!({"error": "Player not found"}))),
    };
    match spells_db::drop_concentration(pool, &p.id).await {
        Ok(Some(name)) => (StatusCode::OK, Json(json!({"message": format!("Dropped concentration on {}", name)}))),
        Ok(None) => (StatusCode::OK, Json(json!({"message": "Not concentrating on anything"}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    }
}
 
// ─── War Bond handlers ────────────────────────────────────────────────────────
 
pub async fn get_war_bonds_handler(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
) -> impl IntoResponse {
    let pool = &state.pool;
    let p = match player::get_player_by_campaign(pool, &campaign_id).await {
        Ok(Some(p)) => p,
        _ => return (StatusCode::NOT_FOUND, Json(json!({"error": "Player not found"}))),
    };
    match spells_db::get_war_bonds(pool, &p.id).await {
        Ok(bonds) => (StatusCode::OK, Json(json!({"war_bonds": bonds}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    }
}
 
#[derive(Debug, serde::Deserialize)]
pub struct WarBondRequest {
    pub item_id: String,
}
 
pub async fn create_war_bond_handler(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
    Json(req): Json<WarBondRequest>,
) -> impl IntoResponse {
    let pool = &state.pool;
    let p = match player::get_player_by_campaign(pool, &campaign_id).await {
        Ok(Some(p)) => p,
        _ => return (StatusCode::NOT_FOUND, Json(json!({"error": "Player not found"}))),
    };
    match spells_db::create_war_bond(pool, &campaign_id, &p.id, &req.item_id).await {
        Ok(_) => {
            let bonds = spells_db::get_war_bonds(pool, &p.id).await.unwrap_or_default();
            (StatusCode::OK, Json(json!({"message": "War Bond created", "war_bonds": bonds})))
        }
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({"error": e.to_string()}))),
    }
}
 
pub async fn break_war_bond_handler(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
    Json(req): Json<WarBondRequest>,
) -> impl IntoResponse {
    let pool = &state.pool;
    let p = match player::get_player_by_campaign(pool, &campaign_id).await {
        Ok(Some(p)) => p,
        _ => return (StatusCode::NOT_FOUND, Json(json!({"error": "Player not found"}))),
    };
    match spells_db::break_war_bond(pool, &p.id, &req.item_id).await {
        Ok(true) => {
            let bonds = spells_db::get_war_bonds(pool, &p.id).await.unwrap_or_default();
            (StatusCode::OK, Json(json!({"message": "War Bond broken", "war_bonds": bonds})))
        }
        Ok(false) => (StatusCode::NOT_FOUND, Json(json!({"error": "No bond found for that item"}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    }
}
 
pub async fn summon_bonded_weapon_handler(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
    Json(req): Json<WarBondRequest>,
) -> impl IntoResponse {
    let pool = &state.pool;
    let p = match player::get_player_by_campaign(pool, &campaign_id).await {
        Ok(Some(p)) => p,
        _ => return (StatusCode::NOT_FOUND, Json(json!({"error": "Player not found"}))),
    };
    match spells_db::summon_bonded_weapon(pool, &p.id, &req.item_id).await {
        Ok(result) => (StatusCode::OK, Json(result)),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({"error": e.to_string()}))),
    }
}
 
// ─── Spell search ─────────────────────────────────────────────────────────────
 
#[derive(Debug, serde::Deserialize)]
pub struct SpellSearchRequest {
    pub query: String,
    pub wizard_only: Option<bool>,
}
 
pub async fn search_spells_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SpellSearchRequest>,
) -> impl IntoResponse {
    let wizard_only = req.wizard_only.unwrap_or(false);
    match spells_db::search_spells(&state.pool, &req.query, wizard_only).await {
        Ok(results) => (StatusCode::OK, Json(json!({"spells": results}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    }
}
 
pub async fn seed_ek_slots_handler(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
) -> impl IntoResponse {
    let pool = &state.pool;
    let p = match player::get_player_by_campaign(pool, &campaign_id).await {
        Ok(Some(p)) => p,
        _ => return (StatusCode::NOT_FOUND, Json(json!({"error": "Player not found"}))),
    };
 
    if p.subclass.as_deref() != Some("Eldritch Knight") {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "Player is not an Eldritch Knight"})));
    }
 
    match spells_db::seed_ek_spell_slots(pool, &campaign_id, &p.id, p.level).await {
        Ok(_) => {
            let slots = spells_db::get_spell_slots(pool, &p.id).await.unwrap_or_default();
            (StatusCode::OK, Json(json!({"message": "Spell slots seeded", "spell_slots": slots})))
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    }
}

async fn feat_add_ability(
    pool: &sqlx::SqlitePool,
    campaign_id: &str,
    player_id: &str,
    name: &str,
    desc: &str,
    uses: i64,
    refresh: &str,
) {
    let existing = world::get_abilities(pool, player_id, "player").await.unwrap_or_default();
    if !existing.iter().any(|a| a.name == name) {
        let _ = world::create_ability(
            pool, campaign_id, "player", player_id, name, Some(desc), uses, refresh,
        ).await;
    }
}
 
async fn feat_add_prof(
    pool: &sqlx::SqlitePool,
    campaign_id: &str,
    player_id: &str,
    prof_type: &str,
    name: &str,
) {
    let id = Uuid::new_v4().to_string();
    let _ = sqlx::query(
        "INSERT OR IGNORE INTO proficiencies
         (id, campaign_id, player_id, proficiency_type, name, source)
         VALUES (?, ?, ?, ?, ?, 'feat')"
    )
    .bind(&id).bind(campaign_id).bind(player_id)
    .bind(prof_type).bind(name)
    .execute(pool).await;
}
 
async fn apply_feat_effects(
    pool: &sqlx::SqlitePool,
    campaign_id: &str,
    player_id: &str,
    feat_id: &str,
    choices_json: Option<&str>,
    level: i64,
) {
    let choices: serde_json::Value = choices_json
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or(serde_json::json!({}));
 
    let player = match player::get_player(pool, player_id).await {
        Ok(Some(p)) => p,
        _ => return,
    };
 
    match feat_id {
 
        // ── Origin feats ──────────────────────────────────────────────────────
 
        "feat_alert" => {
            feat_add_ability(pool, campaign_id, player_id, "Alert",
                "Add your Proficiency Bonus to Initiative rolls. After rolling Initiative, \
                 you can swap your Initiative with one willing ally (neither Incapacitated).",
                1, "manual").await;
        }
 
        "feat_crafter" => {
            feat_add_ability(pool, campaign_id, player_id, "Crafter",
                "Proficiency with 3 Artisan's Tools of your choice. 20% discount on nonmagical items. \
                 Fast Crafting: on Long Rest, craft one item from the Fast Crafting table (lasts until next Long Rest).",
                1, "long_rest").await;
        }
 
        "feat_healer" => {
            feat_add_ability(pool, campaign_id, player_id, "Healer",
                "Battle Medic: Utilize action — expend a Healer's Kit use to let a creature within 5 ft \
                 expend one Hit Die; roll it, creature regains HP = roll + Prof Bonus. \
                 Healing Rerolls: when you roll a 1 on any healing die, reroll (must use new roll).",
                1, "manual").await;
        }
 
        "feat_lucky" => {
            let prof = Player::proficiency_for_level(player.level);
            feat_add_ability(pool, campaign_id, player_id, "Lucky",
                "Luck Points (= Prof Bonus, regain on Long Rest). \
                 Spend 1 to give yourself Advantage on a D20 Test (before rolling). \
                 Spend 1 to impose Disadvantage on an attack roll against you.",
                prof, "long_rest").await;
        }
 
        "feat_magic_initiate" => {
            if let Some(c1) = choices["cantrip1"].as_str() {
                let _ = spells_db::learn_spell(pool, campaign_id, player_id, c1, "cantrip", "feat_magic_initiate").await;
            }
            if let Some(c2) = choices["cantrip2"].as_str() {
                let _ = spells_db::learn_spell(pool, campaign_id, player_id, c2, "cantrip", "feat_magic_initiate").await;
            }
            if let Some(s1) = choices["spell1"].as_str() {
                let _ = spells_db::learn_spell(pool, campaign_id, player_id, s1, "always_prepared", "feat_magic_initiate").await;
            }
        }
 
        "feat_musician" => {
            feat_add_ability(pool, campaign_id, player_id, "Encouraging Song",
                "When you finish a Short or Long Rest, play a Musical Instrument you have proficiency \
                 with to give Heroic Inspiration to a number of allies equal to your Proficiency Bonus.",
                1, "short_rest").await;
        }
 
        "feat_savage_attacker" => {
            feat_add_ability(pool, campaign_id, player_id, "Savage Attacker",
                "Once per turn when you hit a target with a weapon, roll the weapon's damage dice twice \
                 and use either roll against the target.",
                1, "per_turn").await;
        }
 
        "feat_tough" => {
            let hp_bonus = 2 * level;
            let _ = sqlx::query(
                "UPDATE players SET max_hp = max_hp + ?, current_hp = current_hp + ?,
                 updated_at = datetime('now') WHERE id = ?"
            )
            .bind(hp_bonus).bind(hp_bonus).bind(player_id)
            .execute(pool).await;
            feat_add_ability(pool, campaign_id, player_id, "Tough",
                "Your max HP increases by 2 each time you gain a character level.",
                1, "manual").await;
        }
 
        // ── General feats ─────────────────────────────────────────────────────
 
        "feat_asi" => {
            if let Some(stat1) = choices["stat1"].as_str() {
                let amount1 = choices["amount1"].as_i64().unwrap_or(1);
                for _ in 0..amount1 {
                    let _ = player::apply_asi(pool, player_id, stat1, None).await;
                }
            }
            if let Some(stat2) = choices["stat2"].as_str() {
                let _ = player::apply_asi(pool, player_id, stat2, None).await;
            }
        }
 
        "feat_actor" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            feat_add_ability(pool, campaign_id, player_id, "Actor",
                "Advantage on CHA (Deception or Performance) checks while disguised as a specific person. \
                 Mimicry: can mimic sounds/speech of other creatures.",
                1, "manual").await;
        }
 
        "feat_athlete" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            feat_add_ability(pool, campaign_id, player_id, "Athlete",
                "Climb Speed = your Speed. Hop Up: stand from Prone with only 5 ft of movement. \
                 Jumping: make running Long or High Jump after moving only 5 feet.",
                1, "manual").await;
        }
 
        "feat_charger" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            feat_add_ability(pool, campaign_id, player_id, "Charger",
                "Improved Dash: Speed +10 ft when Dashing. \
                 Charge Attack: after moving 10+ ft straight toward a target, choose +1d8 damage \
                 OR push it 10 ft (once per turn, Attack action only).",
                1, "per_turn").await;
        }
 
        "feat_chef" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            feat_add_ability(pool, campaign_id, player_id, "Chef",
                "Replenishing Meal (Short Rest): cook food for Prof Bonus + 4 creatures — those who eat \
                 and spend Hit Dice regain extra 1d8 HP. \
                 Bolstering Treats: Prof Bonus treats (8 hr) — Bonus Action to eat one for Temp HP = Prof Bonus.",
                1, "short_rest").await;
        }
 
        "feat_crossbow_expert" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            feat_add_ability(pool, campaign_id, player_id, "Crossbow Expert",
                "Ignore Loading property on crossbows; can load without a free hand. \
                 No Disadvantage on crossbow attacks within 5 ft of an enemy. \
                 Light crossbow dual wielding: add ability modifier to extra attack damage.",
                1, "manual").await;
        }
 
        "feat_crusher" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            feat_add_ability(pool, campaign_id, player_id, "Crusher",
                "Once per turn when you deal Bludgeoning damage, move target 5 ft to unoccupied space. \
                 Enhanced Critical: on Bludgeoning crit, attack rolls against that creature have Advantage \
                 until start of your next turn.",
                1, "manual").await;
        }
 
        "feat_defensive_duelist" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            feat_add_ability(pool, campaign_id, player_id, "Defensive Duelist",
                "Reaction: when holding a Finesse weapon and hit by a melee attack, add Prof Bonus to AC, \
                 potentially causing the attack to miss. Bonus lasts until start of your next turn.",
                1, "per_turn").await;
        }
 
        "feat_dual_wielder" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            feat_add_ability(pool, campaign_id, player_id, "Dual Wielder",
                "Enhanced Dual Wielding: when attacking with a Light weapon (Attack action), make an extra \
                 Bonus Action attack with a different non-Two-Handed Melee weapon. \
                 Quick Draw: draw/stow two non-Two-Handed weapons at once.",
                1, "manual").await;
        }
 
        "feat_durable" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            feat_add_ability(pool, campaign_id, player_id, "Durable",
                "Advantage on Death Saving Throws. \
                 Speedy Recovery: Bonus Action — expend one Hit Point Die and regain that many HP.",
                1, "manual").await;
        }
 
        "feat_elemental_adept" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            let damage_type = choices["damage_type"].as_str().unwrap_or("fire");
            let dt_cap = {
                let mut c = damage_type.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }
            };
            let name = format!("Elemental Adept ({})", dt_cap);
            let desc = format!(
                "Your spells ignore Resistance to {} damage. \
                 When you roll damage for a {} spell, treat any 1 on a damage die as a 2.",
                damage_type, damage_type
            );
            feat_add_ability(pool, campaign_id, player_id, &name, &desc, 1, "manual").await;
        }
 
        "feat_fey_touched" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            if let Ok(Some(spell)) = spells_db::get_spell_by_name(pool, "Misty Step").await {
                if let Some(id) = spell["id"].as_str() {
                    let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "feat_fey_touched").await;
                }
            }
            if let Some(spell_id) = choices["spell"].as_str() {
                let _ = spells_db::learn_spell(pool, campaign_id, player_id, spell_id, "always_prepared", "feat_fey_touched").await;
            }
            feat_add_ability(pool, campaign_id, player_id, "Fey Touched",
                "Always have Misty Step and one Divination or Enchantment spell prepared. \
                 Cast each once per Long Rest without a slot. Also castable with spell slots.",
                1, "long_rest").await;
        }
 
        "feat_grappler" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            feat_add_ability(pool, campaign_id, player_id, "Grappler",
                "Punch and Grab: when you hit with an Unarmed Strike (Attack action), use both Damage \
                 and Grapple options (once per turn). \
                 Advantage on attacks against Grappled creatures. \
                 No extra movement cost to move a Grappled creature your size or smaller.",
                1, "manual").await;
        }
 
        "feat_great_weapon_master" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            feat_add_ability(pool, campaign_id, player_id, "Great Weapon Master",
                "Heavy Weapon Mastery: when you hit with a Heavy weapon (Attack action), deal extra damage = Prof Bonus. \
                 Hew: immediately after a Critical Hit or reducing a creature to 0 HP with a Melee weapon, \
                 make one Bonus Action attack with the same weapon.",
                1, "per_turn").await;
        }
 
        "feat_heavily_armored" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            feat_add_prof(pool, campaign_id, player_id, "armor", "heavy").await;
        }
 
        "feat_heavy_armor_master" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            feat_add_ability(pool, campaign_id, player_id, "Heavy Armor Master",
                "When hit by an attack while wearing Heavy armor, reduce Bludgeoning, Piercing, and \
                 Slashing damage by an amount equal to your Proficiency Bonus.",
                1, "manual").await;
        }
 
        "feat_inspiring_leader" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            feat_add_ability(pool, campaign_id, player_id, "Inspiring Leader",
                "After a Short or Long Rest, give an inspiring performance. Choose up to 6 allies within \
                 30 ft — each gains Temp HP = character level + WIS or CHA modifier.",
                1, "short_rest").await;
        }
 
        "feat_keen_mind" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            feat_add_ability(pool, campaign_id, player_id, "Keen Mind",
                "Expertise or proficiency in Arcana, History, Investigation, Nature, or Religion (chosen). \
                 Quick Study: take the Study action as a Bonus Action.",
                1, "manual").await;
        }
 
        "feat_lightly_armored" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            feat_add_prof(pool, campaign_id, player_id, "armor", "light").await;
            feat_add_prof(pool, campaign_id, player_id, "armor", "shield").await;
        }
 
        "feat_mage_slayer" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            feat_add_ability(pool, campaign_id, player_id, "Mage Slayer",
                "Concentration Breaker: when you damage a concentrating creature, it has Disadvantage on \
                 its Concentration save. \
                 Guarded Mind (1/Short Rest): when you fail an INT, WIS, or CHA save, succeed instead.",
                1, "short_rest").await;
        }
 
        "feat_martial_weapon_training" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            feat_add_prof(pool, campaign_id, player_id, "weapon", "martial").await;
        }
 
        "feat_medium_armor_master" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            feat_add_ability(pool, campaign_id, player_id, "Medium Armor Master",
                "While wearing Medium armor, add 3 (instead of 2) to AC if DEX 16 or higher.",
                1, "manual").await;
        }
 
        "feat_moderately_armored" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            feat_add_prof(pool, campaign_id, player_id, "armor", "medium").await;
        }
 
        "feat_mounted_combatant" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            feat_add_ability(pool, campaign_id, player_id, "Mounted Combatant",
                "Mounted Strike: Advantage on attacks against unmounted creatures within 5 ft of mount \
                 (at least one size smaller). Leap Aside: mount takes no damage on DEX save for half. \
                 Veer: redirect attacks targeting your mount to hit you instead.",
                1, "manual").await;
        }
 
        "feat_observant" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            feat_add_ability(pool, campaign_id, player_id, "Observant",
                "Proficiency or Expertise in Insight, Investigation, or Perception (chosen). \
                 Quick Search: take the Search action as a Bonus Action.",
                1, "manual").await;
        }
 
        "feat_piercer" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            feat_add_ability(pool, campaign_id, player_id, "Piercer",
                "Puncture: once per turn when you deal Piercing damage, reroll one damage die. \
                 Enhanced Critical: on Piercing crit, roll one additional damage die.",
                1, "per_turn").await;
        }
 
        "feat_poisoner" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            feat_add_prof(pool, campaign_id, player_id, "tool", "poisoner's kit").await;
            feat_add_ability(pool, campaign_id, player_id, "Poisoner",
                "Potent Poison: spells and poison attacks ignore Resistance to Poison damage. \
                 Brew Poison (1 hr + 50 GP): create Prof Bonus doses. Bonus Action to apply. \
                 On hit: CON save or 2d8 Poison damage + Poisoned until end of your next turn.",
                1, "manual").await;
        }
 
        "feat_polearm_master" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            feat_add_ability(pool, campaign_id, player_id, "Polearm Master",
                "Pole Strike: after attacking with a Quarterstaff, Spear, or Heavy+Reach weapon (Attack \
                 action), Bonus Action to attack with the other end (1d4 Bludgeoning). \
                 Reactive Strike: Reaction to attack a creature that enters your reach.",
                1, "per_turn").await;
        }
 
        "feat_resilient" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
                feat_add_prof(pool, campaign_id, player_id, "saving_throw", s).await;
            }
        }
 
        "feat_ritual_caster" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            feat_add_ability(pool, campaign_id, player_id, "Ritual Caster",
                "Always prepared: ritual spells chosen (equal to Prof Bonus, level 1 with Ritual tag). \
                 Quick Ritual (1/Long Rest): cast a prepared ritual at normal speed without expending a slot.",
                1, "long_rest").await;
        }
 
        "feat_sentinel" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            feat_add_ability(pool, campaign_id, player_id, "Sentinel",
                "Guardian: Opportunity Attack when a creature within 5 ft takes Disengage or hits a \
                 target other than you. \
                 Halt: when you hit a creature with an Opportunity Attack, its Speed becomes 0.",
                1, "per_turn").await;
        }
 
        "feat_shadow_touched" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            if let Ok(Some(spell)) = spells_db::get_spell_by_name(pool, "Invisibility").await {
                if let Some(id) = spell["id"].as_str() {
                    let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "feat_shadow_touched").await;
                }
            }
            if let Some(spell_id) = choices["spell"].as_str() {
                let _ = spells_db::learn_spell(pool, campaign_id, player_id, spell_id, "always_prepared", "feat_shadow_touched").await;
            }
            feat_add_ability(pool, campaign_id, player_id, "Shadow Touched",
                "Always have Invisibility and one Illusion or Necromancy spell prepared. \
                 Cast each once per Long Rest without a slot. Also castable with spell slots.",
                1, "long_rest").await;
        }
 
        "feat_sharpshooter" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            feat_add_ability(pool, campaign_id, player_id, "Sharpshooter",
                "Bypass Cover: ranged weapon attacks ignore Half Cover and Three-Quarters Cover. \
                 Firing in Melee: no Disadvantage on ranged attacks within 5 ft of an enemy. \
                 Long Shots: no Disadvantage at long range.",
                1, "manual").await;
        }
 
        "feat_shield_master" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            feat_add_ability(pool, campaign_id, player_id, "Shield Master",
                "Shield Bash: after hitting with a Melee weapon (Attack action), bash with your Shield — \
                 STR save or push 5 ft / Prone (once per turn). \
                 Interpose Shield: Reaction on DEX save for half damage — take no damage on success.",
                1, "per_turn").await;
        }
 
        "feat_skill_expert" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
        }
 
        "feat_skulker" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            feat_add_ability(pool, campaign_id, player_id, "Skulker",
                "Blindsight 10 feet. \
                 Fog of War: Advantage on DEX (Stealth) checks as part of the Hide action in combat. \
                 Sniper: missing a hidden attack roll doesn't reveal your location.",
                1, "manual").await;
        }
 
        "feat_slasher" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            feat_add_ability(pool, campaign_id, player_id, "Slasher",
                "Hamstring: once per turn when you deal Slashing damage, reduce target Speed by 10 ft \
                 until start of your next turn. \
                 Enhanced Critical: on Slashing crit, target has Disadvantage on attacks until start of your next turn.",
                1, "per_turn").await;
        }
 
        "feat_speedy" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            let _ = sqlx::query(
                "UPDATE players SET speed = speed + 10, updated_at = datetime('now') WHERE id = ?"
            )
            .bind(player_id).execute(pool).await;
            feat_add_ability(pool, campaign_id, player_id, "Speedy",
                "Speed +10 ft (applied). \
                 Dash over Difficult Terrain: no extra movement cost when Dashing. \
                 Agile Movement: Opportunity Attacks have Disadvantage against you.",
                1, "manual").await;
        }
 
        "feat_spell_sniper" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            feat_add_ability(pool, campaign_id, player_id, "Spell Sniper",
                "Bypass Cover: spell attack rolls ignore Half Cover and Three-Quarters Cover. \
                 Casting in Melee: no Disadvantage on spell attacks within 5 ft of an enemy. \
                 Increased Range: spells with 10+ ft range requiring attack rolls gain +60 ft range.",
                1, "manual").await;
        }
 
        "feat_telekinetic" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            if let Ok(Some(spell)) = spells_db::get_spell_by_name(pool, "Mage Hand").await {
                if let Some(id) = spell["id"].as_str() {
                    let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "cantrip", "feat_telekinetic").await;
                }
            }
            feat_add_ability(pool, campaign_id, player_id, "Telekinetic",
                "Mage Hand: cast without Verbal/Somatic, can be Invisible, range +30 ft. \
                 Telekinetic Shove (Bonus Action): one creature within 30 ft makes STR save or moves \
                 5 ft toward or away from you.",
                1, "per_turn").await;
        }
 
        "feat_telepathic" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            if let Ok(Some(spell)) = spells_db::get_spell_by_name(pool, "Detect Thoughts").await {
                if let Some(id) = spell["id"].as_str() {
                    let _ = spells_db::learn_spell(pool, campaign_id, player_id, id, "always_prepared", "feat_telepathic").await;
                }
            }
            feat_add_ability(pool, campaign_id, player_id, "Telepathic",
                "Telepathic Utterance: speak telepathically to any creature you can see within 60 ft \
                 (one-way; requires shared language). \
                 Detect Thoughts: always prepared, cast once/LR without a slot.",
                1, "long_rest").await;
        }
 
        "feat_war_caster" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            feat_add_ability(pool, campaign_id, player_id, "War Caster",
                "Concentration: Advantage on CON saves to maintain Concentration. \
                 Reactive Spell: Reaction when a creature leaves your reach — cast a spell at it instead \
                 of an Opportunity Attack (action casting time, targets only that creature). \
                 Somatic Components: perform Somatic components even with weapons or Shield in hand.",
                1, "manual").await;
        }
 
        "feat_weapon_master" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            feat_add_ability(pool, campaign_id, player_id, "Weapon Master",
                "Use the Mastery property of one Simple or Martial weapon you have proficiency with. \
                 Change your weapon choice after each Long Rest.",
                1, "manual").await;
        }
 
        // ── Fighting Style feats ──────────────────────────────────────────────
 
        "feat_fs_archery" => {
            feat_add_ability(pool, campaign_id, player_id, "Archery",
                "+2 bonus to attack rolls with Ranged weapons.", 1, "manual").await;
        }
 
        "feat_fs_blind_fighting" => {
            feat_add_ability(pool, campaign_id, player_id, "Blind Fighting",
                "Blindsight with a range of 10 feet.", 1, "manual").await;
        }
 
        "feat_fs_defense" => {
            feat_add_ability(pool, campaign_id, player_id, "Defense",
                "+1 bonus to Armor Class while wearing Light, Medium, or Heavy armor.", 1, "manual").await;
            // AC is recalculated at the end of this function
        }
 
        "feat_fs_dueling" => {
            feat_add_ability(pool, campaign_id, player_id, "Dueling",
                "+2 bonus to damage rolls when holding a Melee weapon in one hand and no other weapons.",
                1, "manual").await;
        }
 
        "feat_fs_great_weapon_fighting" => {
            feat_add_ability(pool, campaign_id, player_id, "Great Weapon Fighting",
                "When rolling damage for a two-handed Melee attack, treat any 1 or 2 on a damage die as a 3.",
                1, "manual").await;
        }
 
        "feat_fs_interception" => {
            feat_add_ability(pool, campaign_id, player_id, "Interception",
                "Reaction: when a creature you can see hits another creature within 5 ft of you, \
                 reduce the damage by 1d10 + Prof Bonus (requires Shield or weapon).",
                1, "per_turn").await;
        }
 
        "feat_fs_protection" => {
            feat_add_ability(pool, campaign_id, player_id, "Protection",
                "Reaction: when a creature attacks a target other than you within 5 ft, impose \
                 Disadvantage on the attack (requires Shield). Lasts until start of your next turn.",
                1, "per_turn").await;
        }
 
        "feat_fs_thrown_weapon_fighting" => {
            feat_add_ability(pool, campaign_id, player_id, "Thrown Weapon Fighting",
                "+2 bonus to damage rolls with Thrown weapons.", 1, "manual").await;
        }
 
        "feat_fs_two_weapon_fighting" => {
            feat_add_ability(pool, campaign_id, player_id, "Two-Weapon Fighting",
                "When making the extra attack from a Light weapon, add your ability modifier to \
                 the damage of that attack.", 1, "manual").await;
        }
 
        "feat_fs_unarmed_fighting" => {
            feat_add_ability(pool, campaign_id, player_id, "Unarmed Fighting",
                "Unarmed Strikes deal 1d6 + STR Bludgeoning (1d8 if no weapons or Shield). \
                 Start of each turn: deal 1d4 Bludgeoning to one Grappled creature.",
                1, "manual").await;
        }
 
        // ── Epic Boon feats ───────────────────────────────────────────────────
 
        "feat_boon_fortitude" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            let _ = sqlx::query(
                "UPDATE players SET max_hp = max_hp + 40, current_hp = current_hp + 40,
                 updated_at = datetime('now') WHERE id = ?"
            )
            .bind(player_id).execute(pool).await;
            feat_add_ability(pool, campaign_id, player_id, "Boon of Fortitude",
                "HP maximum +40 (applied). When you regain HP, also regain additional HP = CON modifier (once per turn).",
                1, "manual").await;
        }
 
        "feat_boon_speed" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            let _ = sqlx::query(
                "UPDATE players SET speed = speed + 30, updated_at = datetime('now') WHERE id = ?"
            )
            .bind(player_id).execute(pool).await;
            feat_add_ability(pool, campaign_id, player_id, "Boon of Speed",
                "Speed +30 ft (applied). \
                 Escape Artist: Bonus Action — take Disengage, also ends Grappled condition. \
                 Quickness: Speed +30 ft.",
                1, "per_turn").await;
        }
 
        "feat_boon_spell_recall" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            feat_add_ability(pool, campaign_id, player_id, "Boon of Spell Recall",
                "Free Casting: whenever you cast a spell with a level 1-4 spell slot, roll 1d4. \
                 If the result matches the slot's level, the slot isn't expended.",
                1, "manual").await;
        }
 
        "feat_boon_night_spirit" => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            feat_add_ability(pool, campaign_id, player_id, "Boon of the Night Spirit",
                "Merge with Shadows: while in Dim Light or Darkness, Bonus Action to become Invisible \
                 (ends on Action/Bonus Action/Reaction). \
                 Shadowy Form: while in Dim Light or Darkness, Resistance to all damage except Psychic and Radiant.",
                1, "per_turn").await;
        }
 
        // All remaining Epic Boon feats: apply stat + seed ability reference
        feat_id if feat_id.starts_with("feat_boon_") => {
            if let Some(s) = choices["stat"].as_str() {
                let _ = player::apply_asi(pool, player_id, s, None).await;
            }
            if let Ok(feat) = sqlx::query!("SELECT name, description FROM feats WHERE id = ?", feat_id)
                .fetch_one(pool).await
            {
                feat_add_ability(pool, campaign_id, player_id, &feat.name, &feat.description, 1, "long_rest").await;
            }
        }
 
        _ => {
            tracing::warn!("apply_feat_effects: unhandled feat '{}'", feat_id);
        }
    }
 
    // Always recalculate AC — armor proficiency or Defense FS may have changed
    let _ = items::recalculate_ac(pool, player_id).await;
}

 
fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}


/// GET /api/feats?category=general  — list all feats, optionally by category
pub async fn list_feats_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let pool = &state.pool;
    let category = params.get("category").map(|s| s.as_str());
    match feats_db::get_all_feats(pool, category).await {
        Ok(feats) => (StatusCode::OK, Json(json!({"feats": feats}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    }
}
 
/// GET /api/campaigns/:id/feats  — get feats available to this player
pub async fn get_available_feats_handler(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let pool = &state.pool;
    let p = match player::get_player_by_campaign(pool, &campaign_id).await {
        Ok(Some(p)) => p,
        _ => return (StatusCode::NOT_FOUND, Json(json!({"error": "Player not found"}))),
    };
 
    let has_spellcasting = matches!(p.class.as_str(),
        "Bard" | "Cleric" | "Druid" | "Paladin" | "Ranger" | "Sorcerer" | "Warlock" | "Wizard"
    ) || p.subclass.as_deref() == Some("Eldritch Knight")
      || p.subclass.as_deref() == Some("Arcane Trickster");
 
    let has_fighting_style = matches!(p.class.as_str(), "Fighter" | "Paladin" | "Ranger")
        || (p.class == "Bard" && p.subclass.as_deref() == Some("College of Valor"));
 
    // Determine armor training from proficiencies
    let profs = fighter::get_proficiencies(pool, &p.id).await.unwrap_or_default();
    let armor_training: Vec<String> = profs.iter()
        .filter(|pr| pr.proficiency_type == "armor")
        .map(|pr| pr.name.clone())
        .collect();
    let armor_refs: Vec<&str> = armor_training.iter().map(|s| s.as_str()).collect();

    let category = params.get("category").map(|s| s.as_str());
 
    match feats_db::get_available_feats(
        pool, &p.id, p.level,
        p.subclass.as_deref(),
        has_spellcasting, has_fighting_style, &armor_refs,
        p.str, p.dex, p.con, p.int, p.wis, p.cha,
        category,
    ).await {
        Ok(feats) => (StatusCode::OK, Json(json!({"feats": feats}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    }
}

// GET /api/campaigns/:id/player-feats  — get feats the player has taken
pub async fn get_player_feats_handler(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
) -> impl IntoResponse {
    let pool = &state.pool;
    let p = match player::get_player_by_campaign(pool, &campaign_id).await {
        Ok(Some(p)) => p,
        _ => return (StatusCode::NOT_FOUND, Json(json!({"error": "Player not found"}))),
    };
    match feats_db::get_player_feats(pool, &p.id).await {
        Ok(feats) => (StatusCode::OK, Json(json!({"feats": feats}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct BonusDamageRequest { pub damage: i64 }

pub async fn apply_bonus_damage_handler(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
    Json(req): Json<BonusDamageRequest>,
) -> impl IntoResponse {
    match crate::db::combat::apply_bonus_damage(&state.pool, &campaign_id, req.damage).await {
        Ok(result) => (StatusCode::OK, Json(result)),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    }
}

pub async fn get_notes_handler(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
) -> impl IntoResponse {
    match campaign::get_player_notes(&state.pool, &campaign_id).await {
        Ok(notes) => (StatusCode::OK, Json(json!({ "notes": notes }))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))),
    }
}

pub async fn update_notes_handler(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let notes = body["notes"].as_str().unwrap_or("");
    match campaign::update_player_notes(&state.pool, &campaign_id, notes).await {
        Ok(_) => (StatusCode::OK, Json(json!({ "ok": true }))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))),
    }
}