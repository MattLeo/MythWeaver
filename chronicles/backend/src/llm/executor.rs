use anyhow::Result;
use serde_json::{json, Value};
use sqlx::SqlitePool;

use crate::db::{campaign, player, world, items, companions, time, fighter};
use crate::models::Player;

fn parse_i64(v: &Value) -> i64 {
    v.as_i64()
        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        .unwrap_or(0)
}

pub async fn execute_tool(
    pool: &SqlitePool,
    campaign_id: &str,
    tool_name: &str,
    args: &Value,
) -> Result<Value> {
    match tool_name {

        // ── World Query ───────────────────────────────────────────────────────
        "query_location" => {
            let id = args["identifier"].as_str().unwrap_or("");
            let loc = world::get_location_by_name(pool, campaign_id, id).await?
                .or(world::get_location(pool, id).await?);
            match loc {
                Some(l) => {
                    let npcs = world::get_npcs_at_location(pool, &l.id).await?;
                    let connections = world::get_connected_locations(pool, &l.id).await?;
                    Ok(json!({
                        "location": l,
                        "npcs_present": npcs,
                        "connected_locations": connections.iter().map(|c| json!({
                            "id": c.id, "name": c.name
                        })).collect::<Vec<_>>()
                    }))
                }
                None => Ok(json!({"error": "Location not found", "identifier": id}))
            }
        }

        "query_npc" => {
            let id = args["identifier"].as_str().unwrap_or("");
            let npc = world::get_npc_by_name(pool, campaign_id, id).await?
                .or(world::get_npc(pool, id).await?);
            match npc {
                Some(n) => Ok(json!(n)),
                None => Ok(json!({"error": "NPC not found", "identifier": id}))
            }
        }

        "query_world_facts" => {
            let keyword = args["keyword"].as_str().unwrap_or("");
            let facts = world::search_world_facts(pool, campaign_id, keyword).await?;
            Ok(json!({"facts": facts, "count": facts.len()}))
        }

        "query_nearby_npcs" => {
            let loc_id = args["location_id"].as_str().unwrap_or("");
            let npcs = world::get_npcs_at_location(pool, loc_id).await?;
            Ok(json!({"npcs": npcs}))
        }

        "query_connected_locations" => {
            let loc_id = args["location_id"].as_str().unwrap_or("");
            let locations = world::get_connected_locations(pool, loc_id).await?;
            Ok(json!({"locations": locations}))
        }

        "query_player_state" => {
            let p = player::get_player_by_campaign(pool, campaign_id).await?
                .ok_or_else(|| anyhow::anyhow!("No player found"))?;
            let abilities = world::get_abilities(pool, &p.id, "player").await?;
            let all_items = items::get_player_items(pool, &p.id).await?;
            let equipped: Vec<_> = all_items.iter().filter(|i| i.is_equipped).cloned().collect();
            let inventory: Vec<_> = all_items.iter().filter(|i| !i.is_equipped).cloned().collect();
            let active_companions = companions::get_active_companions(pool, campaign_id).await?;
            let current_location = if let Some(loc_id) = &p.current_location_id {
                world::get_location(pool, loc_id).await?
            } else { None };
            let campaign_time = time::get_campaign_time(pool, campaign_id).await?;
            let weapon_masteries = fighter::get_weapon_masteries(pool, &p.id).await?;
            let known_maneuvers = fighter::get_known_maneuvers(pool, &p.id).await?;
            let superiority = if let Some(ref sc) = p.subclass {
                fighter::get_superiority_dice(pool, &p.id, sc).await?
            } else { None };
            Ok(json!({
                "player": p,
                "abilities": abilities,
                "equipped_items": equipped,
                "inventory": inventory,
                "active_companions": active_companions,
                "current_location": current_location,
                "time": campaign_time,
                "weapon_masteries": weapon_masteries,
                "known_maneuvers": known_maneuvers,
                "superiority_dice": superiority,
            }))
        }

        "query_time" => {
            let t = time::get_campaign_time(pool, campaign_id).await?;
            Ok(json!(t))
        }

        // ── World Write ───────────────────────────────────────────────────────
        "create_location" => {
            let loc = world::create_location(
                pool, campaign_id,
                args["name"].as_str().unwrap_or("Unknown Location"),
                args["location_type"].as_str().unwrap_or("area"),
                args["description"].as_str().unwrap_or(""),
                args["notes"].as_str(),
            ).await?;
            if let Some(connect_id) = args["connected_to"].as_str() {
                world::connect_locations(
                    pool, campaign_id, connect_id, &loc.id,
                    args["travel_notes"].as_str()
                ).await?;
            }
            Ok(json!({"location": loc, "message": "Location created"}))
        }

        "create_npc" => {
            let npc = world::create_npc(
                pool, campaign_id,
                args["name"].as_str().unwrap_or("Unknown"),
                args["race"].as_str(),
                args["occupation"].as_str(),
                args["description"].as_str().unwrap_or(""),
                args["personality"].as_str(),
                args["disposition"].as_str().unwrap_or("neutral"),
                args["location_id"].as_str(),
            ).await?;
            Ok(json!({"npc": npc, "message": "NPC created"}))
        }

        "add_world_fact" => {
            let tags = args["tags"].as_array()
                .map(|arr| serde_json::to_string(arr).unwrap_or_default());
            let fact = world::add_world_fact(
                pool, campaign_id,
                args["category"].as_str(),
                args["title"].as_str().unwrap_or(""),
                args["content"].as_str().unwrap_or(""),
                tags.as_deref(),
            ).await?;
            Ok(json!({"fact": fact, "message": "World fact canonized"}))
        }

        // ── World Mutation ────────────────────────────────────────────────────
        "update_npc" => {
            let npc_id = args["npc_id"].as_str().unwrap_or("");
            world::update_npc(pool, npc_id,
                args["disposition"].as_str(),
                args["location_id"].as_str(),
                args["is_alive"].as_bool(),
                args["notes"].as_str(),
            ).await?;
            Ok(json!({"message": "NPC updated", "npc_id": npc_id}))
        }

        "update_location" => {
            let loc_id = args["location_id"].as_str().unwrap_or("");
            world::update_location(pool, loc_id,
                args["description"].as_str(),
                args["state"].as_str(),
                args["notes"].as_str(),
            ).await?;
            Ok(json!({"message": "Location updated", "location_id": loc_id}))
        }

        "move_player" => {
            let p = player::get_player_by_campaign(pool, campaign_id).await?
                .ok_or_else(|| anyhow::anyhow!("No player found"))?;
            let loc_id = args["location_id"].as_str().unwrap_or("");
            let loc = world::get_location(pool, loc_id).await?;
            if loc.is_none() {
                return Ok(json!({
                    "error": "Location not found. Call create_location first.",
                    "location_id": loc_id
                }));
            }
            player::update_player_location(pool, &p.id, loc_id).await?;
            Ok(json!({"message": "Player moved", "new_location": loc}))
        }

        "update_currency" => {
            let p = player::get_player_by_campaign(pool, campaign_id).await?
                .ok_or_else(|| anyhow::anyhow!("No player found"))?;
            let delta_pp = parse_i64(&args["platinum"]);
            let delta_gp = parse_i64(&args["gold"]);
            let delta_sp = parse_i64(&args["silver"]);
            let delta_cp = parse_i64(&args["copper"]);
            let (new_pp, new_gp, new_sp, new_cp) = player::update_currency(
                pool, &p.id, delta_pp, delta_gp, delta_sp, delta_cp
            ).await?;
            Ok(json!({
                "message": "Currency updated",
                "reason": args["reason"],
                "new_balance": {
                    "platinum": new_pp,
                    "gold": new_gp,
                    "silver": new_sp,
                    "copper": new_cp
                }
            }))
        }

        "update_gold" => {
            let p = player::get_player_by_campaign(pool, campaign_id).await?
                .ok_or_else(|| anyhow::anyhow!("No player found"))?;
            let amount = parse_i64(&args["amount"]);
            let (new_pp, new_gp, new_sp, new_cp) = player::update_currency(
                pool, &p.id, 0, amount, 0, 0
            ).await?;
            Ok(json!({
                "message": "Gold updated",
                "new_balance": {
                    "platinum": new_pp,
                    "gold": new_gp,
                    "silver": new_sp,
                    "copper": new_cp
                }
            }))
        }

        // ── Items ─────────────────────────────────────────────────────────────
        "create_item" => {
            let item = items::create_item(pool, campaign_id, args).await?;
            Ok(json!({"item": item, "message": "Item created"}))
        }

        "give_item" => {
            let p = player::get_player_by_campaign(pool, campaign_id).await?
                .ok_or_else(|| anyhow::anyhow!("No player found"))?;
            let item_id = args["item_id"].as_str().unwrap_or("");
            items::give_item(pool, item_id, "player", &p.id).await?;
            Ok(json!({"message": "Item given to player", "item_id": item_id}))
        }

        "remove_item" => {
            let item_id = args["item_id"].as_str().unwrap_or("");
            let qty = parse_i64(&args["quantity"]).max(1);
            items::remove_item(pool, item_id, qty).await?;
            Ok(json!({"message": "Item removed"}))
        }

        "equip_item" => {
            let p = player::get_player_by_campaign(pool, campaign_id).await?
                .ok_or_else(|| anyhow::anyhow!("No player found"))?;
            let item_id = args["item_id"].as_str().unwrap_or("");
            let slot = args["slot"].as_str().unwrap_or("");
            items::equip_item(pool, item_id, slot, &p.id).await?;
            let new_ac = items::recalculate_ac(pool, &p.id).await?;
            Ok(json!({"message": "Item equipped", "new_ac": new_ac}))
        }

        "unequip_item" => {
            let p = player::get_player_by_campaign(pool, campaign_id).await?
                .ok_or_else(|| anyhow::anyhow!("No player found"))?;
            let item_id = args["item_id"].as_str().unwrap_or("");
            items::unequip_item(pool, item_id).await?;
            let new_ac = items::recalculate_ac(pool, &p.id).await?;
            Ok(json!({"message": "Item unequipped", "new_ac": new_ac}))
        }

        "use_item" => {
            let p = player::get_player_by_campaign(pool, campaign_id).await?
                .ok_or_else(|| anyhow::anyhow!("No player found"))?;
            let item_id = args["item_id"].as_str().unwrap_or("");
            let item = items::get_item(pool, item_id).await?
                .ok_or_else(|| anyhow::anyhow!("Item not found"))?;
            if item.item_type == "consumable" {
                let effects = items::get_item_effects(pool, item_id).await?;
                let mut result = json!({"item_used": item.name});
                for effect in &effects {
                    if effect.effect_type == "healing" {
                        let heal = effect.value.unwrap_or(4);
                        let new_hp = (p.current_hp + heal).min(p.max_hp);
                        player::update_player_hp(pool, &p.id, new_hp).await?;
                        result["healing_applied"] = json!(heal);
                        result["new_hp"] = json!(new_hp);
                    }
                }
                items::remove_item(pool, item_id, 1).await?;
                Ok(result)
            } else {
                Ok(json!({"message": "Item used", "item": item.name}))
            }
        }

        "query_items" => {
            let p = player::get_player_by_campaign(pool, campaign_id).await?
                .ok_or_else(|| anyhow::anyhow!("No player found"))?;
            let all_items = items::get_player_items(pool, &p.id).await?;
            let equipped: Vec<_> = all_items.iter().filter(|i| i.is_equipped).cloned().collect();
            let inventory: Vec<_> = all_items.iter().filter(|i| !i.is_equipped).cloned().collect();
            Ok(json!({"equipped": equipped, "inventory": inventory}))
        }

        // ── Mechanical ────────────────────────────────────────────────────────
        "apply_healing" => {
            let p = player::get_player_by_campaign(pool, campaign_id).await?
                .ok_or_else(|| anyhow::anyhow!("No player found"))?;
            let amount = parse_i64(&args["amount"]);
            let new_hp = (p.current_hp + amount).min(p.max_hp);
            player::update_player_hp(pool, &p.id, new_hp).await?;
            Ok(json!({"healing_applied": amount, "new_hp": new_hp, "max_hp": p.max_hp}))
        }

        // ── Companions ────────────────────────────────────────────────────────
        "create_companion" => {
            let companion = companions::create_companion(pool, campaign_id, args).await?;
            Ok(json!({"companion": companion, "message": "Companion created"}))
        }

        "query_companions" => {
            let all = companions::get_active_companions(pool, campaign_id).await?;
            Ok(json!({"companions": all}))
        }

        "update_companion" => {
            let companion_id = args["companion_id"].as_str().unwrap_or("");
            companions::update_companion(pool, companion_id, args).await?;
            Ok(json!({"message": "Companion updated"}))
        }

        "move_companion" => {
            let companion_id = args["companion_id"].as_str().unwrap_or("");
            let loc_id = args["location_id"].as_str().unwrap_or("");
            companions::update_companion(pool, companion_id, &json!({"location_id": loc_id})).await?;
            Ok(json!({"message": "Companion moved"}))
        }

        // ── Progression ───────────────────────────────────────────────────────
        "award_experience" => {
            let p = player::get_player_by_campaign(pool, campaign_id).await?
                .ok_or_else(|| anyhow::anyhow!("No player found"))?;
            let amount = parse_i64(&args["amount"]);
            let new_xp = p.experience + amount;
            player::update_player_xp(pool, &p.id, new_xp).await?;
            let threshold = Player::xp_threshold(p.level);
            let level_up_available = new_xp >= threshold && p.level < 20;
            Ok(json!({
                "xp_awarded": amount,
                "new_total": new_xp,
                "level_up_available": level_up_available,
                "xp_to_next_level": if level_up_available { 0 } else { threshold - new_xp }
            }))
        }

        // ── Abilities ─────────────────────────────────────────────────────────
        "query_abilities" => {
            let p = player::get_player_by_campaign(pool, campaign_id).await?
                .ok_or_else(|| anyhow::anyhow!("No player found"))?;
            let abilities = world::get_abilities(pool, &p.id, "player").await?;
            Ok(json!({"abilities": abilities}))
        }

        "use_ability" => {
            let ability_id = args["ability_id"].as_str().unwrap_or("");
            let uses = parse_i64(&args["uses"]).max(1);
            let remaining = world::use_ability(pool, ability_id, uses).await?;
            Ok(json!({"remaining_uses": remaining}))
        }

        "rest" => {
            let p = player::get_player_by_campaign(pool, campaign_id).await?
                .ok_or_else(|| anyhow::anyhow!("No player found"))?;
            let rest_type = args["rest_type"].as_str().unwrap_or("short");

            world::refresh_abilities(pool, &p.id, "player", rest_type).await?;

            let active = companions::get_active_companions(pool, campaign_id).await?;
            for companion in &active {
                world::refresh_abilities(pool, &companion.id, "companion", rest_type).await?;
            }

            if let Some(ref sc) = p.subclass {
                match sc.as_str() {
                    "Battle Master" => {
                        fighter::restore_superiority_dice(pool, &p.id, "Battle Master").await?;
                    }
                    "Psi Warrior" => {
                        if rest_type == "long" {
                            fighter::restore_superiority_dice(pool, &p.id, "Psi Warrior").await?;
                        } else {
                            sqlx::query(
                                "UPDATE superiority_dice SET current_dice = MIN(current_dice + 1, max_dice)
                                 WHERE player_id = ? AND pool_name = 'Psi Warrior'"
                            )
                            .bind(&p.id)
                            .execute(pool)
                            .await?;
                        }
                    }
                    _ => {}
                }
            }

            if rest_type == "long" {
                player::update_player_hp(pool, &p.id, p.max_hp).await?;
                player::update_death_saves(pool, &p.id, 0, 0, true, false).await?;
                if p.indomitable_max > 0 {
                    sqlx::query(
                        "UPDATE players SET indomitable_uses = indomitable_max WHERE id = ?"
                    )
                    .bind(&p.id)
                    .execute(pool)
                    .await?;
                }
                time::advance_time(pool, campaign_id, 8, "long rest").await?;
                Ok(json!({
                    "message": "Long rest complete",
                    "hp_restored": p.max_hp - p.current_hp,
                    "new_hp": p.max_hp,
                    "abilities_refreshed": "all"
                }))
            } else {
                time::advance_time(pool, campaign_id, 1, "short rest").await?;
                Ok(json!({
                    "message": "Short rest complete",
                    "abilities_refreshed": "short_rest only"
                }))
            }
        }

        // ── Death saves ───────────────────────────────────────────────────────
        "roll_death_save" => {
            let p = player::get_player_by_campaign(pool, campaign_id).await?
                .ok_or_else(|| anyhow::anyhow!("No player found"))?;
            let success = args["success"].as_bool().unwrap_or(false);
            let nat_20 = args["natural_20"].as_bool().unwrap_or(false);

            if nat_20 {
                player::update_player_hp(pool, &p.id, 1).await?;
                player::update_death_saves(pool, &p.id, 0, 0, true, false).await?;
                return Ok(json!({"result": "Natural 20! Player regains 1 HP and stabilizes"}));
            }

            let (new_successes, new_failures) = if success {
                (p.death_save_successes + 1, p.death_save_failures)
            } else {
                (p.death_save_successes, p.death_save_failures + 1)
            };

            let is_stable = new_successes >= 3;
            let is_dead = new_failures >= 3;
            player::update_death_saves(pool, &p.id, new_successes, new_failures, is_stable, is_dead).await?;

            Ok(json!({
                "successes": new_successes,
                "failures": new_failures,
                "stable": is_stable,
                "dead": is_dead
            }))
        }

        "stabilize_player" => {
            let p = player::get_player_by_campaign(pool, campaign_id).await?
                .ok_or_else(|| anyhow::anyhow!("No player found"))?;
            let heal = parse_i64(&args["healing_amount"]).max(1);
            let new_hp = (p.current_hp + heal).min(p.max_hp).max(1);
            player::update_player_hp(pool, &p.id, new_hp).await?;
            player::update_death_saves(pool, &p.id, 0, 0, true, false).await?;
            Ok(json!({"message": "Player stabilized", "new_hp": new_hp}))
        }

        // ── Time ──────────────────────────────────────────────────────────────
        "advance_time" => {
            let steps = parse_i64(&args["steps"]).max(1);
            let reason = args["reason"].as_str().unwrap_or("");
            let new_time = time::advance_time(pool, campaign_id, steps, reason).await?;
            Ok(json!(new_time))
        }

        // ── Events ────────────────────────────────────────────────────────────
        "create_event_table" => {
            let table = time::create_event_table(
                pool, campaign_id,
                args["name"].as_str().unwrap_or("Event Table"),
                args["location_type"].as_str(),
                args["trigger_type"].as_str().unwrap_or("manual"),
                parse_i64(&args["trigger_chance"]).max(1),
            ).await?;
            Ok(json!({"table": table}))
        }

        "add_event_entry" => {
            let table_id = args["table_id"].as_str().unwrap_or("");
            let entry = time::add_event_entry(pool, table_id, campaign_id, args).await?;
            Ok(json!({"entry": entry}))
        }

        "query_event_tables" => {
            let tables = time::get_event_tables(pool, campaign_id).await?;
            Ok(json!({"tables": tables}))
        }

        "trigger_event" => {
            let entry_id = args["event_entry_id"].as_str().unwrap_or("");
            let entry = time::get_event_entry(pool, entry_id).await?;
            Ok(json!({"event": entry}))
        }

        // ── Session ───────────────────────────────────────────────────────────
        "get_session_summaries" => {
            let summaries = campaign::get_session_summaries(pool, campaign_id).await?;
            Ok(json!({"summaries": summaries}))
        }

        "add_session_note" => {
            let note = args["note"].as_str().unwrap_or("");
            world::add_world_fact(
                pool, campaign_id, Some("session_note"), "Session Note", note, None
            ).await?;
            Ok(json!({"message": "Session note saved"}))
        }

        // ── Combat ────────────────────────────────────────────────────────────
        "start_combat" => {
            if let Ok(Some(_)) = crate::db::combat::get_active_encounter(pool, campaign_id).await {
                return Ok(json!({"error": "Combat already active."}));
            }
            let p = player::get_player_by_campaign(pool, campaign_id).await?
                .ok_or_else(|| anyhow::anyhow!("No player found"))?;

            let raw_enemies = match args["enemies"].as_array() {
                Some(arr) => arr.clone(),
                None => serde_json::from_str::<Vec<Value>>(
                    args["enemies"].as_str().unwrap_or("[]")
                ).unwrap_or_default(),
            };

            let raw_allies = match args["allies"].as_array() {
                Some(arr) => arr.clone(),
                None => vec![],
            };

            if raw_enemies.is_empty() {
                return Ok(json!({"error": "No enemies provided to start_combat"}));
            }

            crate::db::combat::start_combat(pool, campaign_id, &p, raw_enemies, raw_allies).await
        }

        // ── Fighter ───────────────────────────────────────────────────────────
        "change_weapon_mastery" => {
            let p = player::get_player_by_campaign(pool, campaign_id).await?
                .ok_or_else(|| anyhow::anyhow!("No player found"))?;
            fighter::change_weapon_mastery(
                pool, &p.id,
                args["old_weapon"].as_str().unwrap_or(""),
                args["new_weapon"].as_str().unwrap_or(""),
                args["new_property"].as_str().unwrap_or(""),
            ).await?;
            Ok(json!({"message": "Weapon mastery updated"}))
        }

        "query_weapon_masteries" => {
            let p = player::get_player_by_campaign(pool, campaign_id).await?
                .ok_or_else(|| anyhow::anyhow!("No player found"))?;
            let masteries = fighter::get_weapon_masteries(pool, &p.id).await?;
            Ok(json!({"weapon_masteries": masteries}))
        }

        "query_superiority_dice" => {
            let p = player::get_player_by_campaign(pool, campaign_id).await?
                .ok_or_else(|| anyhow::anyhow!("No player found"))?;
            let pool_name = p.subclass.as_deref().unwrap_or("Battle Master");
            let dice = fighter::get_superiority_dice(pool, &p.id, pool_name).await?;
            Ok(json!({"superiority_dice": dice}))
        }

        _ => {
            tracing::warn!("Unknown tool called: {}", tool_name);
            Ok(json!({"error": format!("Unknown tool: {}", tool_name)}))
        }
    }
}