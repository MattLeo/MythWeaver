use serde_json::{json, Value};
use crate::models::GameState;

/// Returns the tool definitions appropriate for the current game state
pub fn tools_for_state(state: &GameState) -> Vec<Value> {
    match state {
        GameState::Exploration => exploration_tools(),
        GameState::Combat => combat_tools(),
        GameState::Dialogue => dialogue_tools(),
        GameState::Rest => rest_tools(),
        GameState::Leveling => leveling_tools(),
        GameState::Shopping => shopping_tools(),
    }
}

fn exploration_tools() -> Vec<Value> {
    let mut tools = vec![];
    tools.extend(world_query_tools());
    tools.extend(world_write_tools());
    tools.extend(world_mutation_tools());
    tools.extend(item_tools());
    tools.extend(companion_query_tools());
    tools.extend(time_tools());
    tools.extend(event_tools());
    tools.extend(session_tools());
    tools
}

fn combat_tools() -> Vec<Value> {
    let mut tools = vec![];
    tools.extend(world_query_tools());
    tools.extend(mechanical_tools());
    tools.extend(companion_combat_tools());
    tools.extend(ability_tools());
    tools.extend(death_tools());
    tools.extend(session_tools());
    tools
}

fn dialogue_tools() -> Vec<Value> {
    let mut tools = vec![];
    tools.extend(world_query_tools());
    tools.extend(world_write_tools());
    tools.extend(world_mutation_tools());
    tools.extend(companion_query_tools());
    tools.extend(session_tools());
    tools
}

fn rest_tools() -> Vec<Value> {
    let mut tools = vec![];
    tools.extend(ability_tools());
    tools.extend(time_tools());
    tools.extend(session_tools());
    tools
}

fn leveling_tools() -> Vec<Value> {
    let mut tools = vec![];
    tools.extend(progression_tools());
    tools.extend(ability_tools());
    tools.extend(world_query_tools());
    tools
}

fn shopping_tools() -> Vec<Value> {
    let mut tools = vec![];
    tools.extend(world_query_tools());
    tools.extend(item_tools());
    tools.extend(world_mutation_tools());
    tools.extend(session_tools());
    tools
}

// ─── Tool Category Builders ───────────────────────────────────────────────────

fn world_query_tools() -> Vec<Value> {
    vec![
        tool("query_location",
            "Get a location by name or ID. Returns description, state, connections, and NPCs present.",
            json!({
                "type": "object",
                "properties": {
                    "identifier": { "type": "string", "description": "Location name or ID" }
                },
                "required": ["identifier"]
            })
        ),
        tool("query_npc",
            "Get an NPC by name or ID. Returns appearance, personality, disposition, location, alive status, and notes.",
            json!({
                "type": "object",
                "properties": {
                    "identifier": { "type": "string", "description": "NPC name or ID" }
                },
                "required": ["identifier"]
            })
        ),
        tool("query_world_facts",
            "Search canonized world lore by keyword. Returns matching facts, history, factions, and quests.",
            json!({
                "type": "object",
                "properties": {
                    "keyword": { "type": "string", "description": "Search term" }
                },
                "required": ["keyword"]
            })
        ),
        tool("query_nearby_npcs",
            "Get all NPCs currently at a given location.",
            json!({
                "type": "object",
                "properties": {
                    "location_id": { "type": "string", "description": "Location ID to query" }
                },
                "required": ["location_id"]
            })
        ),
        tool("query_connected_locations",
            "Get all locations connected to a given location for navigation and exploration.",
            json!({
                "type": "object",
                "properties": {
                    "location_id": { "type": "string", "description": "Location ID to query connections for" }
                },
                "required": ["location_id"]
            })
        ),
        tool("query_player_state",
            "Get the full current player state: HP, AC, XP, level, gold, location, inventory summary, active companions, and current time.",
            json!({
                "type": "object",
                "properties": {}
            })
        ),
        tool("query_time",
            "Get the current time of day, day number, and season.",
            json!({
                "type": "object",
                "properties": {}
            })
        ),
    ]
}

fn world_write_tools() -> Vec<Value> {
    vec![
        tool("create_location",
            "Create and persist a new location in the world.",
            json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string" },
                    "location_type": {
                        "type": "string",
                        "enum": ["city", "town", "village", "dungeon", "wilderness", "building", "tavern", "shop", "ruins", "cave", "road", "area"],
                        "description": "Type of location"
                    },
                    "description": { "type": "string", "description": "Vivid description of the location" },
                    "notes": { "type": "string", "description": "DM notes, secrets, or context" },
                    "connected_to": { "type": "string", "description": "Optional ID of a location to connect this to" },
                    "travel_notes": { "type": "string", "description": "Travel description between the connected locations" }
                },
                "required": ["name", "location_type", "description"]
            })
        ),
        tool("create_npc",
            "Create and persist a new NPC in the world.",
            json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string" },
                    "race": { "type": "string" },
                    "occupation": { "type": "string" },
                    "description": { "type": "string", "description": "Physical appearance and demeanor" },
                    "personality": { "type": "string", "description": "Personality traits, motivations, secrets" },
                    "disposition": {
                        "type": "string",
                        "enum": ["friendly", "neutral", "unfriendly", "hostile", "allied"]
                    },
                    "location_id": { "type": "string", "description": "ID of location where NPC currently is" }
                },
                "required": ["name", "description", "disposition"]
            })
        ),
        tool("add_world_fact",
            "Canonize a piece of lore permanently. Use for player-proposed facts, history, factions, quests, or rumors.",
            json!({
                "type": "object",
                "properties": {
                    "title": { "type": "string" },
                    "content": { "type": "string" },
                    "category": {
                        "type": "string",
                        "enum": ["faction", "history", "quest", "rumor", "geography", "religion", "magic", "character"]
                    },
                    "tags": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Searchable tags"
                    }
                },
                "required": ["title", "content"]
            })
        ),
    ]
}

fn world_mutation_tools() -> Vec<Value> {
    vec![
        tool("update_npc",
            "Update an existing NPC — change disposition, location, alive status, or add notes.",
            json!({
                "type": "object",
                "properties": {
                    "npc_id": { "type": "string" },
                    "disposition": { "type": "string", "enum": ["friendly", "neutral", "unfriendly", "hostile", "allied"] },
                    "location_id": { "type": "string" },
                    "is_alive": { "type": "boolean" },
                    "notes": { "type": "string" }
                },
                "required": ["npc_id"]
            })
        ),
        tool("update_location",
            "Update an existing location — change description, state, or add notes.",
            json!({
                "type": "object",
                "properties": {
                    "location_id": { "type": "string" },
                    "description": { "type": "string" },
                    "state": { "type": "string", "description": "e.g. 'burned down', 'occupied', 'abandoned'" },
                    "notes": { "type": "string" }
                },
                "required": ["location_id"]
            })
        ),
        tool("move_player",
            "Update the player's current location.",
            json!({
                "type": "object",
                "properties": {
                    "location_id": { "type": "string", "description": "ID of the location the player is moving to" }
                },
                "required": ["location_id"]
            })
        ),
        tool("update_gold",
            "Add or subtract gold from the player.",
            json!({
                "type": "object",
                "properties": {
                    "amount": { "type": "integer", "description": "Positive to add, negative to subtract" },
                    "reason": { "type": "string", "description": "Narrative reason for the change" }
                },
                "required": ["amount"]
            })
        ),
    ]
}

fn item_tools() -> Vec<Value> {
    vec![
        tool("create_item",
            "Create a new item in the world. Specify effects for magical items.",
            json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string" },
                    "description": { "type": "string" },
                    "item_type": { "type": "string", "enum": ["weapon", "armor", "shield", "consumable", "wondrous", "quest"] },
                    "owner_type": { "type": "string", "enum": ["player", "npc", "location"] },
                    "owner_id": { "type": "string" },
                    "quantity": { "type": "integer", "default": 1 },
                    "damage_die": { "type": "string", "description": "e.g. '1d8', '2d6' for weapons" },
                    "damage_type": { "type": "string", "description": "e.g. 'slashing', 'piercing', 'fire'" },
                    "base_ac": { "type": "integer", "description": "Base AC for armor" },
                    "armor_type": { "type": "string", "enum": ["light", "medium", "heavy"] },
                    "rarity": { "type": "string", "enum": ["common", "uncommon", "rare", "very_rare", "legendary"] },
                    "effects": {
                        "type": "array",
                        "description": "Mechanical effects of magical items",
                        "items": {
                            "type": "object",
                            "properties": {
                                "effect_type": { "type": "string", "enum": ["ac_bonus", "attack_bonus", "damage_bonus", "advantage_on", "resistance_to"] },
                                "value": { "type": "integer" },
                                "target": { "type": "string", "description": "e.g. 'stealth', 'fire', 'all_saves'" }
                            }
                        }
                    },
                    "notes": { "type": "string" }
                },
                "required": ["name", "description", "item_type"]
            })
        ),
        tool("give_item",
            "Transfer an item to the player's inventory.",
            json!({
                "type": "object",
                "properties": {
                    "item_id": { "type": "string" }
                },
                "required": ["item_id"]
            })
        ),
        tool("remove_item",
            "Remove an item from the player's inventory.",
            json!({
                "type": "object",
                "properties": {
                    "item_id": { "type": "string" },
                    "quantity": { "type": "integer", "default": 1 }
                },
                "required": ["item_id"]
            })
        ),
        tool("equip_item",
            "Move an item to an equipment slot. Recalculates AC automatically.",
            json!({
                "type": "object",
                "properties": {
                    "item_id": { "type": "string" },
                    "slot": {
                        "type": "string",
                        "enum": ["main_hand", "off_hand", "armor", "shield", "cloak", "ring_1", "ring_2", "boots", "helmet", "amulet"]
                    }
                },
                "required": ["item_id", "slot"]
            })
        ),
        tool("unequip_item",
            "Move an equipped item back to inventory. Recalculates AC.",
            json!({
                "type": "object",
                "properties": {
                    "item_id": { "type": "string" }
                },
                "required": ["item_id"]
            })
        ),
        tool("use_item",
            "Consume a consumable item and trigger its mechanical effect.",
            json!({
                "type": "object",
                "properties": {
                    "item_id": { "type": "string" }
                },
                "required": ["item_id"]
            })
        ),
        tool("query_items",
            "Get the player's full inventory and equipped items with all effects.",
            json!({
                "type": "object",
                "properties": {}
            })
        ),
    ]
}

fn mechanical_tools() -> Vec<Value> {
    vec![
        tool("request_roll",
            "Ask the player to roll dice. The frontend will animate the roll and automatically return the result.",
            json!({
                "type": "object",
                "properties": {
                    "die": { "type": "string", "enum": ["d4", "d6", "d8", "d10", "d12", "d20"] },
                    "skill": { "type": "string", "description": "e.g. 'Stealth', 'Perception', 'Constitution saving throw'" },
                    "dc": { "type": "integer", "description": "Difficulty class for the check" },
                    "reason": { "type": "string", "description": "Brief narrative reason for the roll" }
                },
                "required": ["die", "skill", "dc", "reason"]
            })
        ),
        tool("apply_damage",
            "Apply damage to the player. Handles temp HP first, then current HP.",
            json!({
                "type": "object",
                "properties": {
                    "amount": { "type": "integer" },
                    "damage_type": { "type": "string", "description": "e.g. 'slashing', 'fire', 'psychic'" },
                    "source": { "type": "string", "description": "What dealt the damage" }
                },
                "required": ["amount", "source"]
            })
        ),
        tool("apply_healing",
            "Heal the player. Cannot exceed max HP.",
            json!({
                "type": "object",
                "properties": {
                    "amount": { "type": "integer" },
                    "source": { "type": "string", "description": "Source of healing" }
                },
                "required": ["amount", "source"]
            })
        ),
    ]
}

fn companion_query_tools() -> Vec<Value> {
    vec![
        tool("query_companions",
            "Get all active companions with current stats, HP, and location.",
            json!({
                "type": "object",
                "properties": {}
            })
        ),
        tool("create_companion",
            "Create and persist a new companion.",
            json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string" },
                    "companion_type": { "type": "string", "enum": ["ally", "familiar", "animal", "hireling", "summon"] },
                    "description": { "type": "string" },
                    "personality": { "type": "string" },
                    "disposition": { "type": "string", "enum": ["friendly", "neutral", "unfriendly", "hostile", "allied"] },
                    "max_hp": { "type": "integer" },
                    "armor_class": { "type": "integer" },
                    "location_id": { "type": "string" },
                    "notes": { "type": "string" }
                },
                "required": ["name", "companion_type", "description"]
            })
        ),
        tool("update_companion",
            "Update a companion's disposition, location, active status, or notes.",
            json!({
                "type": "object",
                "properties": {
                    "companion_id": { "type": "string" },
                    "disposition": { "type": "string" },
                    "location_id": { "type": "string" },
                    "is_active": { "type": "boolean" },
                    "notes": { "type": "string" }
                },
                "required": ["companion_id"]
            })
        ),
        tool("move_companion",
            "Move a companion to a different location, independently of the player.",
            json!({
                "type": "object",
                "properties": {
                    "companion_id": { "type": "string" },
                    "location_id": { "type": "string" }
                },
                "required": ["companion_id", "location_id"]
            })
        ),
    ]
}

fn companion_combat_tools() -> Vec<Value> {
    let mut tools = companion_query_tools();
    tools.push(tool("apply_companion_damage",
        "Apply damage to a companion.",
        json!({
            "type": "object",
            "properties": {
                "companion_id": { "type": "string" },
                "amount": { "type": "integer" },
                "source": { "type": "string" }
            },
            "required": ["companion_id", "amount", "source"]
        })
    ));
    tools.push(tool("apply_companion_healing",
        "Heal a companion.",
        json!({
            "type": "object",
            "properties": {
                "companion_id": { "type": "string" },
                "amount": { "type": "integer" }
            },
            "required": ["companion_id", "amount"]
        })
    ));
    tools.push(tool("use_companion_ability",
        "Use a companion's ability, spending one use.",
        json!({
            "type": "object",
            "properties": {
                "companion_id": { "type": "string" },
                "ability_id": { "type": "string" }
            },
            "required": ["companion_id", "ability_id"]
        })
    ));
    tools
}

fn ability_tools() -> Vec<Value> {
    vec![
        tool("query_abilities",
            "Get all current ability pools for the player — name, current uses, max uses, refresh type.",
            json!({
                "type": "object",
                "properties": {}
            })
        ),
        tool("use_ability",
            "Spend one or more uses of a player ability.",
            json!({
                "type": "object",
                "properties": {
                    "ability_id": { "type": "string" },
                    "uses": { "type": "integer", "default": 1 }
                },
                "required": ["ability_id"]
            })
        ),
        tool("rest",
            "Take a rest. Short rest refreshes short_rest abilities; long rest refreshes all abilities, advances to next morning.",
            json!({
                "type": "object",
                "properties": {
                    "rest_type": { "type": "string", "enum": ["short", "long"] }
                },
                "required": ["rest_type"]
            })
        ),
    ]
}

fn progression_tools() -> Vec<Value> {
    vec![
        tool("award_experience",
            "Award XP to the player for combat, roleplay, or objectives. Returns new total and whether level threshold was crossed.",
            json!({
                "type": "object",
                "properties": {
                    "amount": { "type": "integer" },
                    "reason": { "type": "string", "description": "What earned the XP" }
                },
                "required": ["amount", "reason"]
            })
        ),
        tool("level_up",
            "Level up the player. Increases level, HP, proficiency bonus. Returns what changed and whether ASI is available.",
            json!({
                "type": "object",
                "properties": {}
            })
        ),
        tool("apply_asi",
            "Apply an Ability Score Improvement. Either raise two stats by 1 each, or one stat by 2.",
            json!({
                "type": "object",
                "properties": {
                    "stat1": { "type": "string", "enum": ["str", "dex", "con", "int", "wis", "cha"] },
                    "stat2": { "type": "string", "enum": ["str", "dex", "con", "int", "wis", "cha"], "description": "Optional second stat (different from stat1)" }
                },
                "required": ["stat1"]
            })
        ),
    ]
}

fn death_tools() -> Vec<Value> {
    vec![
        tool("roll_death_save",
            "Record a death saving throw result. 3 successes = stable, 3 failures = dead.",
            json!({
                "type": "object",
                "properties": {
                    "success": { "type": "boolean", "description": "true if the roll was 10 or higher" },
                    "natural_20": { "type": "boolean", "description": "true if the roll was a natural 20 (regain 1 HP)" }
                },
                "required": ["success"]
            })
        ),
        tool("stabilize_player",
            "Stabilize the player (healing spell, Spare the Dying, healer's kit). Resets death saves.",
            json!({
                "type": "object",
                "properties": {
                    "healing_amount": { "type": "integer", "default": 1, "description": "HP restored (default 1 for stabilization)" }
                }
            })
        ),
    ]
}

fn time_tools() -> Vec<Value> {
    vec![
        tool("advance_time",
            "Advance time for travel or downtime. Steps move through: dawn, morning, midday, afternoon, dusk, evening, night, deep_night.",
            json!({
                "type": "object",
                "properties": {
                    "steps": { "type": "integer", "description": "Number of time-of-day steps to advance (8 steps = 1 full day)" },
                    "reason": { "type": "string", "description": "Narrative reason, e.g. 'travel to Velmoor Crossing'" }
                },
                "required": ["steps", "reason"]
            })
        ),
    ]
}

fn event_tools() -> Vec<Value> {
    vec![
        tool("create_event_table",
            "Create a named event table for random encounters or world events.",
            json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string" },
                    "location_type": { "type": "string", "description": "Location type this applies to, or omit for global" },
                    "trigger_type": { "type": "string", "enum": ["travel", "rest", "time", "location_enter", "manual"] },
                    "trigger_chance": { "type": "integer", "description": "Percent chance per trigger (1-100)", "default": 30 }
                },
                "required": ["name", "trigger_type"]
            })
        ),
        tool("add_event_entry",
            "Add an event to an event table.",
            json!({
                "type": "object",
                "properties": {
                    "table_id": { "type": "string" },
                    "weight": { "type": "integer", "default": 10, "description": "Relative probability weight" },
                    "event_type": { "type": "string", "enum": ["encounter", "environmental", "world", "discovery", "personal"] },
                    "title": { "type": "string" },
                    "description": { "type": "string", "description": "Context injected into DM prompt when this event triggers" },
                    "is_repeatable": { "type": "boolean", "default": true }
                },
                "required": ["table_id", "event_type", "title", "description"]
            })
        ),
        tool("query_event_tables",
            "Get all event tables and their entries for this campaign.",
            json!({
                "type": "object",
                "properties": {}
            })
        ),
        tool("trigger_event",
            "Manually trigger a specific event entry.",
            json!({
                "type": "object",
                "properties": {
                    "event_entry_id": { "type": "string" }
                },
                "required": ["event_entry_id"]
            })
        ),
    ]
}

fn session_tools() -> Vec<Value> {
    vec![
        tool("get_session_summaries",
            "Retrieve compressed summaries of past sessions for context.",
            json!({
                "type": "object",
                "properties": {}
            })
        ),
        tool("add_session_note",
            "Bookmark an important story moment mid-session for the end-of-session summary.",
            json!({
                "type": "object",
                "properties": {
                    "note": { "type": "string", "description": "The story moment to bookmark" }
                },
                "required": ["note"]
            })
        ),
    ]
}

// ─── Helper ───────────────────────────────────────────────────────────────────

fn tool(name: &str, description: &str, parameters: Value) -> Value {
    json!({
        "type": "function",
        "function": {
            "name": name,
            "description": description,
            "parameters": parameters
        }
    })
}