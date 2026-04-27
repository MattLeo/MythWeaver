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
    tools.extend(event_tools());
    tools.extend(session_tools());
    tools.extend(progression_tools());
    tools.extend(fighter_exploration_tools());
    tools.push(request_roll_tool());
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
    tools.extend(base_combat_tools());
    tools.extend(fighter_combat_tools());
    tools
}

/* Commenting these out as they may no longer be necessary

fn dialogue_tools() -> Vec<Value> {
    let mut tools = vec![];
    tools.extend(world_query_tools());
    tools.extend(world_write_tools());
    tools.extend(world_mutation_tools());
    tools.extend(companion_query_tools());
    tools.extend(session_tools());
    tools.push(request_roll_tool());
    tools
}
*/

fn rest_tools() -> Vec<Value> {
    let mut tools = vec![];
    tools.extend(ability_tools());
    tools.extend(time_tools());
    tools.extend(session_tools());
    tools
}

fn leveling_tools() -> Vec<Value> {
    let mut tools = vec![];
    tools.extend(world_query_tools());
    tools.extend(session_tools());
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
        tool("query_player_state",
            "Get full current player state: HP, AC, XP, level, subclass, gold, location, inventory, abilities, weapon masteries, maneuvers, and superiority dice.",
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
            "Create and persist a new NPC.",
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
            "Update player currency. Pass the amount of each denomination to add or subtract. Positive adds, negative subtracts. The backend handles all conversion and rollup automatically — never do math yourself, just pass the denomination amounts as stated.",
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
                    "weapon_type": { "type": "string", "description": "Specific weapon name e.g. 'longsword', 'greataxe' — used for mastery lookup" },
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
            json!({
                "type": "object",
                "properties": {}
            })
        ),
    ]
}

// ─── Mechanical ───────────────────────────────────────────────────────────────

fn mechanical_tools() -> Vec<Value> {
    vec![
        request_roll_tool(),
        tool("apply_damage",
            "Apply damage to the player.",
            json!({
                "type": "object",
                "properties": {
                    "amount": { "type": "integer" },
                    "damage_type": { "type": "string" },
                    "source": { "type": "string" }
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
        "Use a companion's ability.",
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

// ─── Death ────────────────────────────────────────────────────────────────────

fn death_tools() -> Vec<Value> {
    vec![
        tool("roll_death_save",
            "Record a death saving throw result.",
            json!({
                "type": "object",
                "properties": {
                    "success": { "type": "boolean" },
                    "natural_20": { "type": "boolean" }
                },
                "required": ["success"]
            })
        ),
        tool("stabilize_player",
            "Stabilize the player with healing or medicine.",
            json!({
                "type": "object",
                "properties": {
                    "healing_amount": { "type": "integer", "default": 1 }
                }
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

// ─── Base Combat ──────────────────────────────────────────────────────────────

fn base_combat_tools() -> Vec<Value> {
    vec![
        tool("start_combat",
            "Initiate a combat encounter. Call this the instant any hostile encounter begins, before writing any narrative.",
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
                                "enemy_attack_bonus": { "type": "integer" }
                            },
                            "required": ["enemy_name","enemy_hp","enemy_ac","enemy_damage_die","enemy_damage_type"]
                        }
                    }
                },
                "required": ["enemies"]
            })
        ),
        tool("declare_attack",
            "Declare the player is attacking a specific target. The backend handles all rolls and resolution.",
            json!({
                "type": "object",
                "properties": {
                    "target_name": { "type": "string" }
                },
                "required": ["target_name"]
            })
        ),
        tool("add_companion_to_combat",
            "Add an existing companion to the encounter.",
            json!({
                "type": "object",
                "properties": {
                    "companion_id": { "type": "string" }
                },
                "required": ["companion_id"]
            })
        ),
        tool("add_ally_to_combat",
            "Add a temporary NPC ally to the encounter.",
            json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string" },
                    "description": { "type": "string" },
                    "hp": { "type": "integer" },
                    "ac": { "type": "integer" },
                    "attack_bonus": { "type": "integer" },
                    "damage_die": { "type": "string", "enum": ["d4","d6","d8","d10","d12"] },
                    "damage_bonus": { "type": "integer" },
                    "damage_type": { "type": "string" }
                },
                "required": ["name","hp","ac","damage_die","damage_type"]
            })
        ),
    ]
}

// ─── Fighter Combat Tools ─────────────────────────────────────────────────────

fn fighter_combat_tools() -> Vec<Value> {
    vec![
        tool("use_second_wind",
            "Fighter: Use Second Wind as a bonus action to regain 1d10 + Fighter level HP.",
            json!({ "type": "object", "properties": {} })
        ),
        tool("use_action_surge",
            "Fighter: Activate Action Surge to take one additional action this turn. Only usable once per turn.",
            json!({ "type": "object", "properties": {} })
        ),
        tool("use_indomitable",
            "Fighter (level 9+): When you fail a saving throw, reroll it with a bonus equal to your Fighter level.",
            json!({
                "type": "object",
                "properties": {
                    "original_roll": { "type": "integer" }
                },
                "required": ["original_roll"]
            })
        ),
        tool("use_tactical_mind",
            "Fighter (level 2+): When you fail an ability check, spend a Second Wind use to roll 1d10 and potentially add it to the check.",
            json!({ "type": "object", "properties": {} })
        ),
        tool("commit_tactical_mind",
            "Fighter: Commit the Tactical Mind use after confirming the check succeeded.",
            json!({
                "type": "object",
                "properties": {
                    "ability_id": { "type": "string" }
                },
                "required": ["ability_id"]
            })
        ),
        tool("resolve_maneuver",
            "Battle Master: Resolve a maneuver, spending a Superiority Die.",
            json!({
                "type": "object",
                "properties": {
                    "maneuver_name": {
                        "type": "string",
                        "enum": [
                            "Ambush", "Bait and Switch", "Commander's Strike",
                            "Commanding Presence", "Disarming Attack", "Distracting Strike",
                            "Evasive Footwork", "Feinting Attack", "Goading Attack",
                            "Lunging Attack", "Maneuvering Attack", "Menacing Attack",
                            "Parry", "Precision Attack", "Pushing Attack", "Rally",
                            "Riposte", "Sweeping Attack", "Tactical Assessment", "Trip Attack"
                        ]
                    },
                    "target_id": { "type": "string" },
                    "superiority_roll": { "type": "integer" }
                },
                "required": ["maneuver_name", "superiority_roll"]
            })
        ),
        tool("use_psionic_strike",
            "Psi Warrior: After hitting with a weapon attack, expend a Psionic Energy Die to deal bonus force damage.",
            json!({
                "type": "object",
                "properties": {
                    "psi_roll": { "type": "integer" }
                },
                "required": ["psi_roll"]
            })
        ),
        tool("use_protective_field",
            "Psi Warrior: As a reaction when you or a creature within 30 feet takes damage, expend a Psionic Energy Die to reduce the damage.",
            json!({
                "type": "object",
                "properties": {
                    "psi_roll": { "type": "integer" }
                },
                "required": ["psi_roll"]
            })
        ),
        tool("query_superiority_dice",
            "Check current superiority dice or psionic energy dice remaining.",
            json!({ "type": "object", "properties": {} })
        ),
    ]
}

// ─── Fighter Exploration Tools ────────────────────────────────────────────────

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