use serde_json::{json, Value};
use crate::models::GameState;

pub fn tools_for_state(state: &GameState) -> Vec<Value> {
    match state {
        GameState::Exploration => exploration_tools(),
        GameState::Combat      => combat_tools(),
        GameState::Dialogue    => exploration_tools(),
        GameState::Rest        => rest_tools(),
        GameState::Leveling    => leveling_tools(),
        GameState::Shopping    => shopping_tools(),
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
    tools.extend(session_tools());
    tools.extend(progression_tools());
    tools.extend(fighter_exploration_tools());
    tools.extend(mechanical_tools());
    tools.push(start_combat_tool());
    tools
}

fn combat_tools() -> Vec<Value> {
    // Combat is now fully UI-driven. The model only initiates combat
    // and awards XP after victory. Everything else is handled by the
    // combat UI making direct API calls to the backend.
    let mut tools = vec![];
    tools.extend(world_query_tools());
    tools.extend(session_tools());
    tools.push(start_combat_tool());
    tools.push(tool("award_experience",
        "Award XP to the player after combat ends.",
        json!({
            "type": "object",
            "properties": {
                "amount": { "type": "integer" },
                "reason": { "type": "string" }
            },
            "required": ["amount", "reason"]
        })
    ));
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
    vec![]
}

fn shopping_tools() -> Vec<Value> {
    let mut tools = vec![];
    tools.extend(world_query_tools());
    tools.extend(item_tools());
    tools.extend(world_mutation_tools());
    tools.extend(session_tools());
    tools
}

// ─── World Query ──────────────────────────────────────────────────────────────

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
            "Get an NPC by name or ID.",
            json!({
                "type": "object",
                "properties": {
                    "identifier": { "type": "string" }
                },
                "required": ["identifier"]
            })
        ),
        tool("query_world_facts",
            "Search canonized world lore by keyword.",
            json!({
                "type": "object",
                "properties": {
                    "keyword": { "type": "string" }
                },
                "required": ["keyword"]
            })
        ),
        /* -- Already being handled by query location 
        tool("query_nearby_npcs",
            "Get all NPCs at a given location.",
            json!({
                "type": "object",
                "properties": {
                    "location_id": { "type": "string" }
                },
                "required": ["location_id"]
            })
        ),
        */
        /* -- Should already be returned via the query_location
        tool("query_connected_locations",
            "Get all locations connected to a given location.",
            json!({
                "type": "object",
                "properties": {
                    "location_id": { "type": "string" }
                },
                "required": ["location_id"]
            })
        ),
        */
        tool("query_player_state",
            "Get full current player state: HP, AC, XP, level, subclass, currency, location, inventory, abilities, weapon masteries, maneuvers, and superiority dice.",
            json!({ "type": "object", "properties": {} })
        ),
        /*  -- TIme of day is being passed via the prompt, removing for now
        tool("query_time",
            "Get the current time of day, day number, and season.",
            json!({ "type": "object", "properties": {} })
        ),
        */
    ]
}

// ─── World Write ──────────────────────────────────────────────────────────────

fn world_write_tools() -> Vec<Value> {
    vec![
        tool("create_location",
            "Create and persist a new location in the world.",
            json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string" },
                    "location_type": { "type": "string", "enum": ["city","town","village","dungeon","wilderness","building","room","area"] },
                    "description": { "type": "string" },
                    "connected_to": { "type": "string", "description": "ID of location to connect to" },
                    "travel_notes": { "type": "string" },
                    "notes": { "type": "string" }
                },
                "required": ["name", "location_type", "description"]
            })
        ),
        tool("create_npc",
            "Create and persist a new NPC. ALWAYS call this before introducing any named NPC in the narrative.",
            json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string" },
                    "race": { "type": "string" },
                    "occupation": { "type": "string" },
                    "description": { "type": "string" },
                    "personality": { "type": "string" },
                    "disposition": { "type": "string", "enum": ["friendly","neutral","unfriendly","hostile","allied"] },
                    "location_id": { "type": "string" }
                },
                "required": ["name", "description"]
            })
        ),
        tool("add_world_fact",
            "Canonize a piece of world lore, history, faction info, or quest detail.",
            json!({
                "type": "object",
                "properties": {
                    "category": { "type": "string" },
                    "title": { "type": "string" },
                    "content": { "type": "string" },
                    "tags": { "type": "array", "items": { "type": "string" } }
                },
                "required": ["title", "content"]
            })
        ),
    ]
}

// ─── World Mutation ───────────────────────────────────────────────────────────

fn world_mutation_tools() -> Vec<Value> {
    vec![
        tool("update_npc",
            "Update an NPC's disposition, location, alive status, or notes.",
            json!({
                "type": "object",
                "properties": {
                    "npc_id": { "type": "string" },
                    "disposition": { "type": "string", "enum": ["friendly","neutral","unfriendly","hostile","allied"] },
                    "location_id": { "type": "string" },
                    "is_alive": { "type": "boolean" },
                    "notes": { "type": "string" }
                },
                "required": ["npc_id"]
            })
        ),
        tool("update_location",
            "Update a location's description, state, or notes.",
            json!({
                "type": "object",
                "properties": {
                    "location_id": { "type": "string" },
                    "description": { "type": "string" },
                    "state": { "type": "string" },
                    "notes": { "type": "string" }
                },
                "required": ["location_id"]
            })
        ),
        tool("move_player",
            "Move the player to a location. Must call create_location first if it does not exist.",
            json!({
                "type": "object",
                "properties": {
                    "location_id": { "type": "string" }
                },
                "required": ["location_id"]
            })
        ),
        tool("update_currency",
            "Update player currency. Pass the amount of each denomination to add (positive) or subtract (negative). Never do math yourself — just pass the denominations as stated. The backend handles all conversion and rollup automatically.",
            json!({
                "type": "object",
                "properties": {
                    "platinum": { "type": "integer", "description": "Platinum pieces to add or subtract" },
                    "gold":     { "type": "integer", "description": "Gold pieces to add or subtract" },
                    "silver":   { "type": "integer", "description": "Silver pieces to add or subtract" },
                    "copper":   { "type": "integer", "description": "Copper pieces to add or subtract" },
                    "reason":   { "type": "string",  "description": "Why currency is changing" }
                },
                "required": ["reason"]
            })
        ),
    ]
}

// ─── Items ────────────────────────────────────────────────────────────────────

fn item_tools() -> Vec<Value> {
    vec![
        tool("create_item",
            "Create a new item. Specify weapon_type for weapons so mastery can apply.",
            json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string" },
                    "description": { "type": "string" },
                    "item_type": { "type": "string", "enum": ["weapon","armor","shield","consumable","wondrous","quest"] },
                    "damage_die": { "type": "string" },
                    "damage_type": { "type": "string" },
                    "weapon_range": { "type": "string", "enum": ["melee","ranged"] },
                    "weapon_type": { "type": "string", "description": "Specific weapon name e.g. 'longsword', 'greataxe'" },
                    "base_ac": { "type": "integer" },
                    "armor_type": { "type": "string", "enum": ["light","medium","heavy"] },
                    "rarity": { "type": "string", "enum": ["common","uncommon","rare","very_rare","legendary"] },
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
            "Move an item to an equipment slot.",
            json!({
                "type": "object",
                "properties": {
                    "item_id": { "type": "string" },
                    "slot": { "type": "string", "enum": ["main_hand","off_hand","armor","shield","cloak","ring_1","ring_2","boots","helmet","amulet"] }
                },
                "required": ["item_id", "slot"]
            })
        ),
        tool("unequip_item",
            "Move an equipped item back to inventory.",
            json!({
                "type": "object",
                "properties": {
                    "item_id": { "type": "string" }
                },
                "required": ["item_id"]
            })
        ),
        tool("use_item",
            "Consume a consumable item and apply its mechanical effect.",
            json!({
                "type": "object",
                "properties": {
                    "item_id": { "type": "string" }
                },
                "required": ["item_id"]
            })
        ),
        tool("query_items",
            "Get the player's full inventory and equipped items.",
            json!({ "type": "object", "properties": {} })
        ),
    ]
}

// ─── Mechanical ───────────────────────────────────────────────────────────────
// apply_damage removed — all damage is resolved by the combat UI directly.
// apply_healing kept for narrative healing outside combat (potions, NPC healers, etc.)

fn mechanical_tools() -> Vec<Value> {
    vec![
        request_roll_tool(),
        tool("apply_healing",
            "Heal the player from a non-combat source — potion, NPC healing, environmental effect. Cannot exceed max HP.",
            json!({
                "type": "object",
                "properties": {
                    "amount": { "type": "integer" },
                    "source": { "type": "string" }
                },
                "required": ["amount"]
            })
        ),
    ]
}

// ─── Companions ───────────────────────────────────────────────────────────────

fn companion_query_tools() -> Vec<Value> {
    vec![
        tool("query_companions",
            "Get all active companions.",
            json!({ "type": "object", "properties": {} })
        ),
        tool("create_companion",
            "Create a new companion.",
            json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string" },
                    "companion_type": { "type": "string", "enum": ["ally","familiar","animal","hireling","summon"] },
                    "description": { "type": "string" },
                    "personality": { "type": "string" },
                    "disposition": { "type": "string", "enum": ["friendly","neutral","allied"] },
                    "current_hp": { "type": "integer" },
                    "max_hp": { "type": "integer" },
                    "armor_class": { "type": "integer" },
                    "attack_bonus": { "type": "integer" },
                    "damage_die": { "type": "string", "enum": ["d4","d6","d8","d10","d12"] },
                    "damage_bonus": { "type": "integer" },
                    "damage_type": { "type": "string" },
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
            "Move a companion to a different location.",
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

// ─── Progression ──────────────────────────────────────────────────────────────

fn progression_tools() -> Vec<Value> {
    vec![
        tool("award_experience",
            "Award XP to the player after meaningful combat or roleplay milestones.",
            json!({
                "type": "object",
                "properties": {
                    "amount": { "type": "integer" },
                    "reason": { "type": "string" }
                },
                "required": ["amount", "reason"]
            })
        ),
    ]
}

// ─── Abilities ────────────────────────────────────────────────────────────────

fn ability_tools() -> Vec<Value> {
    vec![
        tool("query_abilities",
            "Get all current ability pools for the player.",
            json!({ "type": "object", "properties": {} })
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
            "Take a short or long rest. Long rest restores all HP, resets death saves, and refreshes all abilities.",
            json!({
                "type": "object",
                "properties": {
                    "rest_type": { "type": "string", "enum": ["short","long"] }
                },
                "required": ["rest_type"]
            })
        ),
    ]
}

// ─── Time ─────────────────────────────────────────────────────────────────────

fn time_tools() -> Vec<Value> {
    vec![
        tool("advance_time",
            "Advance time. Use steps=1 for short rest, steps=8 for long rest, steps=2-4 for travel.",
            json!({
                "type": "object",
                "properties": {
                    "steps": { "type": "integer" },
                    "reason": { "type": "string" }
                },
                "required": ["steps", "reason"]
            })
        ),
    ]
}

// ─── Events ───────────────────────────────────────────────────────────────────

fn event_tools() -> Vec<Value> {
    vec![
        tool("create_event_table",
            "Create a named event table for random encounters or world events.",
            json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string" },
                    "location_type": { "type": "string" },
                    "trigger_type": { "type": "string", "enum": ["travel","rest","time","location_enter","manual"] },
                    "trigger_chance": { "type": "integer", "default": 30 }
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
                    "event_type": { "type": "string", "enum": ["encounter","environmental","world","discovery","personal"] },
                    "title": { "type": "string" },
                    "description": { "type": "string" },
                    "weight": { "type": "integer", "default": 10 },
                    "is_repeatable": { "type": "boolean", "default": true }
                },
                "required": ["table_id", "event_type", "title", "description"]
            })
        ),
        tool("query_event_tables",
            "List all event tables for this campaign.",
            json!({ "type": "object", "properties": {} })
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

// ─── Session ──────────────────────────────────────────────────────────────────

fn session_tools() -> Vec<Value> {
    vec![
        tool("get_session_summaries",
            "Get summaries of past sessions for continuity.",
            json!({ "type": "object", "properties": {} })
        ),
        tool("add_session_note",
            "Save an important note about this session.",
            json!({
                "type": "object",
                "properties": {
                    "note": { "type": "string" }
                },
                "required": ["note"]
            })
        ),
    ]
}

// ─── Combat ───────────────────────────────────────────────────────────────────

fn start_combat_tool() -> Value {
    tool("start_combat",
        "Initiate a combat encounter. Call this the instant any hostile encounter begins, before writing any narrative. Include ALL enemies and any NPC allies who would logically join the fight. Registered companions are added automatically.",
        json!({
            "type": "object",
            "properties": {
                "enemies": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "enemy_name": { "type": "string" },
                            "enemy_description": { "type": "string" },
                            "enemy_hp": { "type": "integer" },
                            "enemy_ac": { "type": "integer" },
                            "enemy_damage_die": { "type": "string", "enum": ["d4","d6","d8","d10","d12"] },
                            "enemy_damage_bonus": { "type": "integer" },
                            "enemy_damage_type": { "type": "string" },
                            "enemy_attack_bonus": { "type": "integer" },
                            "enemy_weapon_name": { "type": "string", "description": "Name of the weapon or natural attack e.g. 'rusty shortsword', 'claws', 'bone club'" }
                        },
                        "required": ["enemy_name","enemy_hp","enemy_ac","enemy_damage_die","enemy_damage_type"]
                    }
                },
                "allies": {
                    "type": "array",
                    "description": "NPC allies who join the fight — town guards, bystanders, hired help etc. Do not include registered companions here.",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string" },
                            "description": { "type": "string" },
                            "hp": { "type": "integer" },
                            "ac": { "type": "integer" },
                            "attack_bonus": { "type": "integer" },
                            "damage_die": { "type": "string", "enum": ["d4","d6","d8","d10","d12"] },
                            "damage_bonus": { "type": "integer" },
                            "damage_type": { "type": "string" },
                            "weapon_name": { "type": "string" }
                        },
                        "required": ["name","hp","ac","damage_die","damage_type"]
                    }
                }
            },
            "required": ["enemies"]
        })
    )
}

// ─── Fighter Exploration ──────────────────────────────────────────────────────

fn fighter_exploration_tools() -> Vec<Value> {
    vec![
        tool("change_weapon_mastery",
            "Fighter: Swap one weapon mastery choice after a long rest.",
            json!({
                "type": "object",
                "properties": {
                    "old_weapon": { "type": "string" },
                    "new_weapon": { "type": "string" },
                    "new_property": {
                        "type": "string",
                        "enum": ["cleave","graze","nick","push","sap","slow","topple","vex"]
                    }
                },
                "required": ["old_weapon", "new_weapon", "new_property"]
            })
        ),
        tool("query_weapon_masteries",
            "Get the player's current weapon mastery selections.",
            json!({ "type": "object", "properties": {} })
        ),
    ]
}

// ─── Shared ───────────────────────────────────────────────────────────────────

pub fn request_roll_tool() -> Value {
    tool("request_roll",
        "Request a dice roll from the player for skill checks, saving throws, and ability checks.",
        json!({
            "type": "object",
            "properties": {
                "die": { "type": "string", "enum": ["d4","d6","d8","d10","d12","d20","d100"] },
                "skill": { "type": "string", "description": "e.g. 'Perception', 'Stealth', 'Constitution saving throw'" },
                "dc": { "type": "integer", "description": "Difficulty class" },
                "reason": { "type": "string", "description": "Brief narrative reason for the roll" }
            },
            "required": ["die", "skill", "dc", "reason"]
        })
    )
}

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