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
                world::connect_locations(pool, campaign_id, connect_id, &loc.id, args["travel_notes"].as_str()).await?;
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

        "update_gold" => {
            let p = player::get_player_by_campaign(pool, campaign_id).await?
                .ok_or_else(|| anyhow::anyhow!("No player found"))?;
            let amount = parse_i64(&args["amount"]);
            let new_gold = (p.gold + amount).max(0);
            player::update_player_gold(pool, &p.id, new_gold).await?;
            Ok(json!({"message": "Gold updated", "previous": p.gold, "change": amount, "new_total": new_gold}))
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
        "apply_damage" => {
            let p = player::get_player_by_campaign(pool, campaign_id).await?
                .ok_or_else(|| anyhow::anyhow!("No player found"))?;
            let amount = parse_i64(&args["amount"]);
            let (damage_to_hp, new_temp) = if p.temp_hp > 0 {
                let absorbed = amount.min(p.temp_hp);
                (amount - absorbed, p.temp_hp - absorbed)
            } else {
                (amount, 0)
            };
            let new_hp = (p.current_hp - damage_to_hp).max(0);
            player::update_player_hp(pool, &p.id, new_hp).await?;
            Ok(json!({"damage_dealt": amount, "temp_hp_remaining": new_temp, "new_hp": new_hp, "max_hp": p.max_hp, "downed": new_hp == 0, "source": args["source"]}))
        }

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

        "apply_companion_damage" => {
            let companion_id = args["companion_id"].as_str().unwrap_or("");
            let amount = parse_i64(&args["amount"]);
            let (new_hp, is_dead) = companions::apply_companion_damage(pool, companion_id, amount).await?;
            Ok(json!({"new_hp": new_hp, "is_dead": is_dead, "companion_id": companion_id}))
        }

        "apply_companion_healing" => {
            let companion_id = args["companion_id"].as_str().unwrap_or("");
            let amount = parse_i64(&args["amount"]);
            let new_hp = companions::apply_companion_healing(pool, companion_id, amount).await?;
            Ok(json!({"new_hp": new_hp, "companion_id": companion_id}))
        }

        "move_companion" => {
            let companion_id = args["companion_id"].as_str().unwrap_or("");
            let loc_id = args["location_id"].as_str().unwrap_or("");
            companions::update_companion(pool, companion_id, &json!({"location_id": loc_id})).await?;
            Ok(json!({"message": "Companion moved"}))
        }

        "use_companion_ability" => {
            let ability_id = args["ability_id"].as_str().unwrap_or("");
            let remaining = world::use_ability(pool, ability_id, 1).await?;
            Ok(json!({"remaining_uses": remaining}))
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

        "level_up" => {
            let p = player::get_player_by_campaign(pool, campaign_id).await?
                .ok_or_else(|| anyhow::anyhow!("No player found"))?;
            if p.level >= 20 {
                return Ok(json!({"error": "Already at maximum level"}));
            }
            let result = player::level_up_player(pool, &p.id, &p).await?;

            // Seed new abilities for this level
            seed_level_up_abilities(pool, campaign_id, &p.id, &p.class, result.new_level, p.subclass.as_deref()).await;

            Ok(json!(result))
        }

        "set_subclass" => {
            let p = player::get_player_by_campaign(pool, campaign_id).await?
                .ok_or_else(|| anyhow::anyhow!("No player found"))?;
            let subclass = args["subclass"].as_str().unwrap_or("");

            // Validate Fighter subclasses
            if p.class == "Fighter" {
                let valid = ["Champion", "Battle Master", "Psi Warrior", "Eldritch Knight"];
                if !valid.contains(&subclass) {
                    return Ok(json!({"error": format!("Invalid Fighter subclass: {}", subclass)}));
                }
            }

            player::set_subclass(pool, &p.id, subclass).await?;
            fighter::seed_subclass(pool, campaign_id, &p.id, subclass, p.level).await?;

            Ok(json!({
                "message": format!("Subclass set to {}", subclass),
                "subclass": subclass,
            }))
        }

        "apply_asi" => {
            let p = player::get_player_by_campaign(pool, campaign_id).await?
                .ok_or_else(|| anyhow::anyhow!("No player found"))?;
            let stat1 = args["stat1"].as_str().unwrap_or("");
            let stat2 = args["stat2"].as_str();
            player::apply_asi(pool, &p.id, stat1, stat2).await?;
            items::recalculate_ac(pool, &p.id).await?;
            Ok(json!({"message": "ASI applied", "stat1": stat1, "stat2": stat2}))
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

            // Restore superiority dice on rest
            if let Some(ref sc) = p.subclass {
                match sc.as_str() {
                    "Battle Master" => {
                        // Recharges on short or long rest
                        fighter::restore_superiority_dice(pool, &p.id, "Battle Master").await?;
                    }
                    "Psi Warrior" => {
                        // Psi Warrior recharges one die on short rest, all on long rest
                        if rest_type == "long" {
                            fighter::restore_superiority_dice(pool, &p.id, "Psi Warrior").await?;
                        } else {
                            // Restore one die on short rest
                            let dice = fighter::get_superiority_dice(pool, &p.id, "Psi Warrior").await?;
                            if let Some(d) = dice {
                                if d.current_dice < d.max_dice {
                                    sqlx::query(
                                        "UPDATE superiority_dice SET current_dice = MIN(current_dice + 1, max_dice) WHERE player_id = ? AND pool_name = 'Psi Warrior'"
                                    )
                                    .bind(&p.id)
                                    .execute(pool)
                                    .await?;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            if rest_type == "long" {
                player::update_player_hp(pool, &p.id, p.max_hp).await?;
                player::update_death_saves(pool, &p.id, 0, 0, true, false).await?;
                // Reset indomitable uses on long rest
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

        // ── Death ─────────────────────────────────────────────────────────────
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

            Ok(json!({"successes": new_successes, "failures": new_failures, "stable": is_stable, "dead": is_dead}))
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
            world::add_world_fact(pool, campaign_id, Some("session_note"), "Session Note", note, None).await?;
            Ok(json!({"message": "Session note saved"}))
        }

        // ── Combat ────────────────────────────────────────────────────────────
        "start_combat" => {
            if let Ok(Some(_)) = crate::db::combat::get_active_encounter(pool, campaign_id).await {
                return Ok(json!({"error": "Combat already active. Use declare_attack to attack."}));
            }
            let p = player::get_player_by_campaign(pool, campaign_id).await?
                .ok_or_else(|| anyhow::anyhow!("No player found"))?;

            let raw_enemies = match args["enemies"].as_array() {
                Some(arr) => arr.clone(),
                None => serde_json::from_str::<Vec<Value>>(
                    args["enemies"].as_str().unwrap_or("[]")
                ).unwrap_or_default(),
            };

            let enemies: Vec<Value> = raw_enemies.iter().map(|e| {
                let damage_str = e["damage"].as_str()
                    .or(e["enemy_damage_die"].as_str())
                    .unwrap_or("d6");
                let damage_die = if let Some(pos) = damage_str.find('d') {
                    let die_part = &damage_str[pos..];
                    let end = die_part.find('+').or(die_part.find('-')).unwrap_or(die_part.len());
                    format!("d{}", &die_part[1..end].trim())
                } else { "d6".to_string() };
                let damage_bonus = if damage_str.contains('+') {
                    damage_str.split('+').nth(1).and_then(|s| s.trim().parse::<i64>().ok()).unwrap_or(0)
                } else { 0 };

                json!({
                    "enemy_name": e["name"].as_str().or(e["enemy_name"].as_str()).unwrap_or("Enemy"),
                    "enemy_description": e["description"].as_str().or(e["enemy_description"].as_str()),
                    "enemy_hp": e["hp"].as_i64().or(e["enemy_hp"].as_i64()).or_else(|| e["hp"].as_str().and_then(|s| s.parse().ok())).unwrap_or(10),
                    "enemy_ac": e["ac"].as_i64().or(e["enemy_ac"].as_i64()).or_else(|| e["ac"].as_str().and_then(|s| s.parse().ok())).unwrap_or(12),
                    "enemy_damage_die": damage_die,
                    "enemy_damage_bonus": e["damage_bonus"].as_i64().unwrap_or(damage_bonus),
                    "enemy_damage_type": e["damage_type"].as_str().or(e["enemy_damage_type"].as_str()).unwrap_or("slashing"),
                    "enemy_attack_bonus": e["attack_bonus"].as_i64().or(e["enemy_attack_bonus"].as_i64()).or_else(|| e["attack_bonus"].as_str().and_then(|s| s.parse().ok())).unwrap_or(0)
                })
            }).collect();

            if enemies.is_empty() {
                return Ok(json!({"error": "No enemies provided to start_combat"}));
            }

            crate::db::combat::start_combat(pool, campaign_id, &p, enemies).await
        }

        "declare_attack" => {
            let target_name = args["target_name"].as_str().unwrap_or("");
            crate::db::combat::declare_attack_target(pool, campaign_id, target_name).await
        }

        "add_companion_to_combat" => {
            let companion_id = args["companion_id"].as_str().unwrap_or("");
            crate::db::combat::add_companion_to_combat(pool, campaign_id, companion_id).await
        }

        "add_ally_to_combat" => {
            crate::db::combat::add_ally_to_combat(pool, campaign_id, args).await
        }

        // ── Fighter specific ──────────────────────────────────────────────────

        "use_second_wind" => {
            let p = player::get_player_by_campaign(pool, campaign_id).await?
                .ok_or_else(|| anyhow::anyhow!("No player found"))?;
            crate::db::combat::use_second_wind(pool, campaign_id, &p).await
        }

        "use_action_surge" => {
            let enc = match crate::db::combat::get_active_encounter(pool, campaign_id).await? {
                Some(e) => e,
                None => return Ok(json!({"error": "No active combat"})),
            };
            let used = fighter::use_action_surge(pool, &enc.id).await?;
            if used {
                // Deduct one use from the Action Surge ability
                let p = player::get_player_by_campaign(pool, campaign_id).await?
                    .ok_or_else(|| anyhow::anyhow!("No player found"))?;
                let _ = sqlx::query(
                    "UPDATE abilities SET current_uses = current_uses - 1 WHERE owner_id = ? AND name = 'Action Surge' AND current_uses > 0"
                )
                .bind(&p.id)
                .execute(pool)
                .await;

                Ok(json!({
                    "message": "Action Surge activated. You have one additional action this turn.",
                    "action_surge_used": true,
                }))
            } else {
                Ok(json!({"error": "Action Surge not available or already used this turn"}))
            }
        }

        "use_indomitable" => {
            let p = player::get_player_by_campaign(pool, campaign_id).await?
                .ok_or_else(|| anyhow::anyhow!("No player found"))?;
            let original_roll = parse_i64(&args["original_roll"]);
            crate::db::combat::use_indomitable(pool, &p, original_roll).await
        }

        "use_tactical_mind" => {
            let p = player::get_player_by_campaign(pool, campaign_id).await?
                .ok_or_else(|| anyhow::anyhow!("No player found"))?;
            crate::db::combat::use_tactical_mind(pool, &p).await
        }

        "commit_tactical_mind" => {
            let ability_id = args["ability_id"].as_str().unwrap_or("");
            crate::db::combat::commit_tactical_mind(pool, ability_id).await?;
            Ok(json!({"message": "Tactical Mind use committed"}))
        }

        "resolve_maneuver" => {
            let p = player::get_player_by_campaign(pool, campaign_id).await?
                .ok_or_else(|| anyhow::anyhow!("No player found"))?;
            if p.subclass.as_deref() != Some("Battle Master") {
                return Ok(json!({"error": "Battle Master maneuvers not available"}));
            }
            let maneuver_name = args["maneuver_name"].as_str().unwrap_or("");
            let target_id = args["target_id"].as_str();
            let superiority_roll = parse_i64(&args["superiority_roll"]);
            crate::db::combat::resolve_maneuver(
                pool, campaign_id, &p, maneuver_name, target_id, superiority_roll
            ).await
        }

        "use_psionic_strike" => {
            let p = player::get_player_by_campaign(pool, campaign_id).await?
                .ok_or_else(|| anyhow::anyhow!("No player found"))?;
            let psi_roll = parse_i64(&args["psi_roll"]);
            crate::db::combat::use_psionic_strike(pool, campaign_id, &p, psi_roll).await
        }

        "use_protective_field" => {
            let p = player::get_player_by_campaign(pool, campaign_id).await?
                .ok_or_else(|| anyhow::anyhow!("No player found"))?;
            let psi_roll = parse_i64(&args["psi_roll"]);
            crate::db::combat::use_protective_field(pool, &p, psi_roll).await
        }

        "change_weapon_mastery" => {
            let old_weapon = args["old_weapon"].as_str().unwrap_or("");
            let new_weapon = args["new_weapon"].as_str().unwrap_or("");
            let new_property = args["new_property"].as_str().unwrap_or("");
            let p = player::get_player_by_campaign(pool, campaign_id).await?
                .ok_or_else(|| anyhow::anyhow!("No player found"))?;
            fighter::change_weapon_mastery(pool, &p.id, old_weapon, new_weapon, new_property).await?;
            Ok(json!({
                "message": format!("Weapon mastery changed from {} to {} ({})", old_weapon, new_weapon, new_property)
            }))
        }

        "query_weapon_masteries" => {
            let p = player::get_player_by_campaign(pool, campaign_id).await?
                .ok_or_else(|| anyhow::anyhow!("No player found"))?;
            let masteries = fighter::get_weapon_masteries(pool, &p.id).await?;
            Ok(json!({"weapon_masteries": masteries}))
        }

        "replace_maneuver" => {
            let p = player::get_player_by_campaign(pool, campaign_id).await?
                .ok_or_else(|| anyhow::anyhow!("No player found"))?;
            let old_maneuver = args["old_maneuver"].as_str().unwrap_or("");
            let new_maneuver = args["new_maneuver"].as_str().unwrap_or("");
            fighter::replace_maneuver(pool, &p.id, old_maneuver, new_maneuver).await?;
            Ok(json!({
                "message": format!("Replaced {} with {}", old_maneuver, new_maneuver)
            }))
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

// ─── Level up ability seeding ─────────────────────────────────────────────────

async fn seed_level_up_abilities(
    pool: &SqlitePool,
    campaign_id: &str,
    player_id: &str,
    class: &str,
    new_level: i64,
    subclass: Option<&str>,
) {
    if class != "Fighter" {
        return;
    }

    let existing = world::get_abilities(pool, player_id, "player").await.unwrap_or_default();
    let has_ability = |name: &str| existing.iter().any(|a| a.name == name);

    match new_level {
        2 => {
            // Action Surge — 1 use, recharges on short rest
            if !has_ability("Action Surge") {
                let _ = world::create_ability(
                    pool, campaign_id, "player", player_id,
                    "Action Surge",
                    Some("On your turn, take one additional action (not the Magic action). Recharges on short or long rest."),
                    1, "short_rest"
                ).await;
            }
            if !has_ability("Tactical Mind") {
                let _ = world::create_ability(
                    pool, campaign_id, "player", player_id,
                    "Tactical Mind",
                    Some("When you fail an ability check, spend a Second Wind use to roll 1d10 and add to the check. Only expended if the check succeeds."),
                    1, "manual"
                ).await;
            }
        }
        5 => {
            if !has_ability("Tactical Shift") {
                let _ = world::create_ability(
                    pool, campaign_id, "player", player_id,
                    "Tactical Shift",
                    Some("When you use Second Wind as a Bonus Action, move up to half your Speed without provoking Opportunity Attacks."),
                    1, "manual"
                ).await;
            }
        }
        9 => {
            if !has_ability("Indomitable") {
                let _ = world::create_ability(
                    pool, campaign_id, "player", player_id,
                    "Indomitable",
                    Some("When you fail a saving throw, reroll it with a bonus equal to your Fighter level. You must use the new roll."),
                    1, "long_rest"
                ).await;
            }
            if !has_ability("Tactical Master") {
                let _ = world::create_ability(
                    pool, campaign_id, "player", player_id,
                    "Tactical Master",
                    Some("When you attack with a weapon whose mastery property you can use, you can replace that property with Push, Sap, or Slow for that attack."),
                    1, "manual"
                ).await;
            }
        }
        13 => {
            if !has_ability("Studied Attacks") {
                let _ = world::create_ability(
                    pool, campaign_id, "player", player_id,
                    "Studied Attacks",
                    Some("If you miss an attack roll against a creature, you have Advantage on your next attack roll against that creature before the end of your next turn."),
                    1, "manual"
                ).await;
            }
        }
        _ => {}
    }

    // Subclass-specific abilities
    match subclass {
        Some("Champion") => match new_level {
            7 => {
                if !has_ability("Additional Fighting Style") {
                    let _ = world::create_ability(
                        pool, campaign_id, "player", player_id,
                        "Additional Fighting Style",
                        Some("You gain another Fighting Style feat of your choice."),
                        1, "manual"
                    ).await;
                }
            }
            10 => {
                if !has_ability("Heroic Warrior") {
                    let _ = world::create_ability(
                        pool, campaign_id, "player", player_id,
                        "Heroic Warrior",
                        Some("During combat, give yourself Heroic Inspiration at the start of your turn if you don't already have it."),
                        1, "per_turn"
                    ).await;
                }
            }
            18 => {
                if !has_ability("Survivor: Heroic Rally") {
                    let _ = world::create_ability(
                        pool, campaign_id, "player", player_id,
                        "Survivor: Heroic Rally",
                        Some("At the start of each turn, regain HP equal to 5 + CON modifier if you are Bloodied and have at least 1 HP."),
                        1, "per_turn"
                    ).await;
                }
            }
            _ => {}
        },
        Some("Battle Master") => match new_level {
            7 => {
                if !has_ability("Know Your Enemy") {
                    let _ = world::create_ability(
                        pool, campaign_id, "player", player_id,
                        "Know Your Enemy",
                        Some("Bonus action: learn a creature's Immunities, Resistances, and Vulnerabilities within 30 feet. Recharges on long rest or by spending a Superiority Die."),
                        1, "long_rest"
                    ).await;
                }
            }
            15 => {
                if !has_ability("Relentless") {
                    let _ = world::create_ability(
                        pool, campaign_id, "player", player_id,
                        "Relentless",
                        Some("Once per turn when you use a maneuver, roll 1d8 and use that number instead of expending a Superiority Die."),
                        1, "per_turn"
                    ).await;
                }
            }
            _ => {}
        },
        Some("Psi Warrior") => match new_level {
            7 => {
                if !has_ability("Psi-Powered Leap") {
                    let _ = world::create_ability(
                        pool, campaign_id, "player", player_id,
                        "Psi-Powered Leap",
                        Some("Bonus action: gain Fly Speed equal to twice your Speed until end of turn. Recharges on short rest or by spending a Psionic Energy Die."),
                        1, "short_rest"
                    ).await;
                }
            }
            10 => {
                if !has_ability("Guarded Mind") {
                    let _ = world::create_ability(
                        pool, campaign_id, "player", player_id,
                        "Guarded Mind",
                        Some("Resistance to Psychic damage. At start of turn, spend a Psionic Energy Die to end Charmed or Frightened conditions on yourself."),
                        1, "manual"
                    ).await;
                }
            }
            15 => {
                if !has_ability("Bulwark of Force") {
                    let _ = world::create_ability(
                        pool, campaign_id, "player", player_id,
                        "Bulwark of Force",
                        Some("Bonus action: choose up to INT modifier creatures within 30 feet (including yourself). Each has Half Cover for 1 minute. Recharges on long rest or Psionic Energy Die."),
                        1, "long_rest"
                    ).await;
                }
            }
            18 => {
                if !has_ability("Telekinetic Master") {
                    let _ = world::create_ability(
                        pool, campaign_id, "player", player_id,
                        "Telekinetic Master",
                        Some("Always have Telekinesis prepared. Cast it without a spell slot. While concentrating on it, make one weapon attack as a Bonus Action each turn. Recharges on long rest or Psionic Energy Die."),
                        1, "long_rest"
                    ).await;
                }
            }
            _ => {}
        },
        _ => {}
    }
}