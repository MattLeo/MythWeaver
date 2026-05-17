use anyhow::Result;
use serde_json::{json, Value};
use sqlx::SqlitePool;
use uuid::Uuid;
 
/// Get all feats, optionally filtered by category.
pub async fn get_all_feats(pool: &SqlitePool, category: Option<&str>) -> Result<Vec<Value>> {
    let rows = sqlx::query!(
        "SELECT id, name, category, prerequisite_level, prerequisite_feature,
         prerequisite_ability, prerequisite_ability_score, description,
         ability_score_options, ability_score_increase, repeatable, has_choice,
         choice_description, grants_spells FROM feats ORDER BY category, name"
    )
    .fetch_all(pool).await?;

    Ok(rows.iter()
        .filter(|r| category.map_or(true, |cat| r.category == cat))
        .map(|r| json!({
            "id": r.id, "name": r.name, "category": r.category,
            "prerequisite_level": r.prerequisite_level,
            "prerequisite_feature": r.prerequisite_feature,
            "prerequisite_ability": r.prerequisite_ability,
            "prerequisite_ability_score": r.prerequisite_ability_score,
            "description": r.description,
            "ability_score_options": r.ability_score_options,
            "ability_score_increase": r.ability_score_increase,
            "repeatable": r.repeatable,
            "has_choice": r.has_choice,
            "choice_description": r.choice_description,
            "grants_spells": r.grants_spells,
        }))
        .collect())
}
 
/// Get feats available to a player given their level, class, and existing feats.
/// Filters out feats the player can't take (unmet prerequisites, already taken non-repeatable).
pub async fn get_available_feats(
    pool: &SqlitePool,
    player_id: &str,
    player_level: i64,
    subclass: Option<&str>,
    has_spellcasting: bool,
    has_fighting_style: bool,
    armor_training: &[&str],   // e.g. ["light", "medium", "heavy", "shield"]
    str: i64, dex: i64, _con: i64, int: i64, wis: i64, cha: i64,
    category_filter: Option<&str>,
) -> Result<Vec<Value>> {
    let all_feats = get_all_feats(pool, category_filter).await?;
    let player_feats = get_player_feats(pool, player_id).await?;
 
    let taken: std::collections::HashSet<String> = player_feats.iter()
        .filter_map(|f| f["feat_id"].as_str().map(|s| s.to_string()))
        .collect();
 
    let available: Vec<Value> = all_feats.into_iter().filter(|feat| {
        let feat_id = feat["id"].as_str().unwrap_or("");
        let req_level = feat["prerequisite_level"].as_i64().unwrap_or(0);
        let req_feature = feat["prerequisite_feature"].as_str();
        let req_ability = feat["prerequisite_ability"].as_str();
        let req_score = feat["prerequisite_ability_score"].as_i64().unwrap_or(0);
        let repeatable = feat["repeatable"].as_i64().unwrap_or(0) == 1;
 
        // Already taken non-repeatable feat
        if taken.contains(feat_id) && !repeatable { return false; }
 
        // Level prerequisite
        if player_level < req_level { return false; }
 
        // Feature prerequisite
        if let Some(feature) = req_feature {
            match feature {
                "spellcasting" | "pact_magic" => if !has_spellcasting { return false; },
                "fighting_style" => if !has_fighting_style { return false; },
                "heavy_armor_training" => if !armor_training.contains(&"heavy") { return false; },
                "medium_armor_training" => if !armor_training.contains(&"medium") { return false; },
                "light_armor_training" => if !armor_training.contains(&"light") { return false; },
                "shield_training" => if !armor_training.contains(&"shield") { return false; },
                _ => {}
            }
        }
 
        // Ability score prerequisite (slash-separated OR conditions)
        if let Some(ability_req) = req_ability {
            if req_score > 0 {
                let options: Vec<&str> = ability_req.split('/').collect();
                let meets = options.iter().any(|opt| {
                    let score = match *opt {
                        "str" => str, "dex" => dex, "int" => int,
                        "wis" => wis, "cha" => cha, _ => 0,
                    };
                    score >= req_score
                });
                if !meets { return false; }
            }
        }
 
        true
    }).collect();
 
    Ok(available)
}
 
/// Get all feats a player has taken.
pub async fn get_player_feats(
    pool: &SqlitePool,
    player_id: &str,
) -> Result<Vec<Value>> {
    let rows = sqlx::query!(
        "SELECT pf.*, f.description, f.category, f.ability_score_options,
                f.has_choice, f.choice_description, f.grants_spells
         FROM player_feats pf
         JOIN feats f ON pf.feat_id = f.id
         WHERE pf.player_id = ?
         ORDER BY pf.level_taken, pf.feat_name",
        player_id
    )
    .fetch_all(pool)
    .await?;
 
    let feats = rows.iter().map(|r| json!({
        "id": r.id,
        "feat_id": r.feat_id,
        "feat_name": r.feat_name,
        "source": r.source,
        "level_taken": r.level_taken,
        "choices": r.choices,
        "description": r.description,
        "category": r.category,
        "ability_score_options": r.ability_score_options,
        "has_choice": r.has_choice,
        "choice_description": r.choice_description,
        "grants_spells": r.grants_spells,
    })).collect();
 
    Ok(feats)
}
 
/// Record a feat being taken by a player.
/// The caller is responsible for applying mechanical effects (stat changes, spell learning, etc.)
pub async fn take_feat(
    pool: &SqlitePool,
    campaign_id: &str,
    player_id: &str,
    feat_id: &str,
    source: &str,           // 'background' | 'asi' | 'epic_boon'
    level_taken: i64,
    choices: Option<&str>,  // JSON string of any choices made
) -> Result<String> {
    // Verify feat exists
    let feat = sqlx::query!("SELECT name, repeatable FROM feats WHERE id = ?", feat_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Feat '{}' not found", feat_id))?;
 
    // Check if already taken (non-repeatable)
    if feat.repeatable == 0 {
        let existing = sqlx::query!(
            "SELECT id FROM player_feats WHERE player_id = ? AND feat_id = ?",
            player_id, feat_id
        )
        .fetch_optional(pool)
        .await?;
 
        if existing.is_some() {
            return Err(anyhow::anyhow!("Feat '{}' is not repeatable and already taken", feat.name));
        }
    }
 
    let id = Uuid::new_v4().to_string();
    sqlx::query!(
        "INSERT INTO player_feats
         (id, campaign_id, player_id, feat_id, feat_name, source, level_taken, choices)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        id, campaign_id, player_id, feat_id, feat.name, source, level_taken, choices
    )
    .execute(pool)
    .await?;
 
    Ok(id)
}
 
/// Remove a feat from a player (for correcting mistakes, not normal gameplay).
pub async fn remove_player_feat(
    pool: &SqlitePool,
    player_feat_id: &str,
) -> Result<bool> {
    let result = sqlx::query!(
        "DELETE FROM player_feats WHERE id = ?",
        player_feat_id
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}
 
/// Search feats by name (for UI feat picker).
pub async fn search_feats(pool: &SqlitePool, query: &str, category: Option<&str>) -> Result<Vec<Value>> {
    let pattern = format!("%{}%", query);
    let rows = sqlx::query!(
        "SELECT id, name, category, prerequisite_level, prerequisite_feature,
         prerequisite_ability, prerequisite_ability_score, description,
         ability_score_options, ability_score_increase, repeatable, has_choice,
         choice_description, grants_spells FROM feats
         WHERE name LIKE ? ORDER BY name LIMIT 20",
        pattern
    )
    .fetch_all(pool).await?;

let feats = rows.iter().map(|r| json!({
        "id": r.id,
        "name": r.name,
        "category": r.category,
        "prerequisite_level": r.prerequisite_level,
        "prerequisite_feature": r.prerequisite_feature,
        "prerequisite_ability": r.prerequisite_ability,
        "prerequisite_ability_score": r.prerequisite_ability_score,
        "description": r.description,
        "ability_score_options": r.ability_score_options,
        "ability_score_increase": r.ability_score_increase,
        "repeatable": r.repeatable,
        "has_choice": r.has_choice,
        "choice_description": r.choice_description,
        "grants_spells": r.grants_spells,
    })).collect();

    Ok(feats)
}
