use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ─── Campaign ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Campaign {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}

// ─── Session ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Session {
    pub id: String,
    pub campaign_id: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SessionSummary {
    pub id: String,
    pub campaign_id: String,
    pub session_id: String,
    pub summary: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Message {
    pub id: String,
    pub session_id: String,
    pub campaign_id: String,
    pub role: String,
    pub content: String,
    pub tool_calls: Option<String>,
    pub created_at: String,
}

// ─── Player ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Player {
    pub id: String,
    pub campaign_id: String,
    pub name: String,
    pub race: String,
    pub class: String,
    pub background: String,
    pub level: i64,
    pub experience: i64,
    pub current_hp: i64,
    pub max_hp: i64,
    pub temp_hp: i64,
    pub armor_class: i64,
    pub str: i64,
    pub dex: i64,
    pub con: i64,
    pub int: i64,
    pub wis: i64,
    pub cha: i64,
    pub proficiency_bonus: i64,
    pub gold: i64,
    pub current_location_id: Option<String>,
    pub backstory: Option<String>,
    pub death_save_successes: i64,
    pub death_save_failures: i64,
    pub is_stable: bool,
    pub is_dead: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl Player {
    pub fn modifier(score: i64) -> i64 {
        (score - 10) / 2
    }

    pub fn xp_threshold(level: i64) -> i64 {
        match level {
            1 => 300,
            2 => 900,
            3 => 2700,
            4 => 6500,
            5 => 14000,
            6 => 23000,
            7 => 34000,
            8 => 48000,
            9 => 64000,
            10 => 85000,
            11 => 100000,
            12 => 120000,
            13 => 140000,
            14 => 165000,
            15 => 195000,
            16 => 225000,
            17 => 265000,
            18 => 305000,
            19 => 355000,
            _ => i64::MAX,
        }
    }

    pub fn proficiency_for_level(level: i64) -> i64 {
        match level {
            1..=4 => 2,
            5..=8 => 3,
            9..=12 => 4,
            13..=16 => 5,
            _ => 6,
        }
    }

    pub fn is_asi_level(level: i64) -> bool {
        matches!(level, 4 | 8 | 12 | 16 | 19)
    }
}

// ─── Class Progression ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelUpResult {
    pub new_level: i64,
    pub hp_gained: i64,
    pub new_max_hp: i64,
    pub new_proficiency_bonus: i64,
    pub asi_available: bool,
    pub new_features: Vec<String>,
    pub spell_slots: Option<SpellSlots>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellSlots {
    pub level_1: Option<i64>,
    pub level_2: Option<i64>,
    pub level_3: Option<i64>,
    pub level_4: Option<i64>,
    pub level_5: Option<i64>,
    pub level_6: Option<i64>,
    pub level_7: Option<i64>,
    pub level_8: Option<i64>,
    pub level_9: Option<i64>,
}

pub fn hit_die_for_class(class: &str) -> i64 {
    match class {
        "Barbarian" => 12,
        "Fighter" | "Paladin" | "Ranger" => 10,
        "Cleric" | "Druid" | "Monk" | "Rogue" | "Bard" | "Warlock" => 8,
        "Sorcerer" | "Wizard" => 6,
        _ => 8,
    }
}

pub fn hp_gained_on_level(class: &str, con_modifier: i64) -> i64 {
    let hit_die = hit_die_for_class(class);
    (hit_die / 2 + 1) + con_modifier
}

// ─── Ability ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Ability {
    pub id: String,
    pub campaign_id: String,
    pub owner_type: String,
    pub owner_id: String,
    pub name: String,
    pub description: Option<String>,
    pub current_uses: i64,
    pub max_uses: i64,
    pub refresh_type: String,
    pub created_at: String,
}

// ─── Location ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Location {
    pub id: String,
    pub campaign_id: String,
    pub name: String,
    pub location_type: String,
    pub description: String,
    pub state: Option<String>,
    pub is_discovered: bool,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LocationConnection {
    pub id: String,
    pub campaign_id: String,
    pub from_location: String,
    pub to_location: String,
    pub travel_notes: Option<String>,
    pub is_hidden: bool,
}

// ─── NPC ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Npc {
    pub id: String,
    pub campaign_id: String,
    pub name: String,
    pub race: Option<String>,
    pub occupation: Option<String>,
    pub description: String,
    pub personality: Option<String>,
    pub disposition: String,
    pub current_location_id: Option<String>,
    pub is_alive: bool,
    pub is_hidden: bool,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

// ─── World Fact ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WorldFact {
    pub id: String,
    pub campaign_id: String,
    pub category: Option<String>,
    pub title: String,
    pub content: String,
    pub tags: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

// ─── Item ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Item {
    pub id: String,
    pub campaign_id: String,
    pub owner_type: Option<String>,
    pub owner_id: Option<String>,
    pub name: String,
    pub description: String,
    pub item_type: String,
    pub quantity: i64,
    pub is_equipped: bool,
    pub slot: Option<String>,
    pub damage_die: Option<String>,
    pub damage_type: Option<String>,
    pub weapon_range: Option<String>,
    pub base_ac: Option<i64>,
    pub armor_type: Option<String>,
    pub stealth_disadvantage: bool,
    pub rarity: String,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ItemEffect {
    pub id: String,
    pub item_id: String,
    pub effect_type: String,
    pub value: Option<i64>,
    pub target: Option<String>,
}

// ─── Companion ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Companion {
    pub id: String,
    pub campaign_id: String,
    pub name: String,
    pub companion_type: String,
    pub description: String,
    pub personality: Option<String>,
    pub disposition: String,
    pub current_hp: i64,
    pub max_hp: i64,
    pub armor_class: i64,
    pub is_alive: bool,
    pub is_active: bool,
    pub current_location_id: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

// ─── Time ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CampaignTime {
    pub id: String,
    pub campaign_id: String,
    pub time_of_day: String,
    pub current_day: i64,
    pub season: String,
    pub updated_at: String,
}

impl CampaignTime {
    pub fn advance_time_of_day(&self) -> String {
        match self.time_of_day.as_str() {
            "dawn" => "morning".to_string(),
            "morning" => "midday".to_string(),
            "midday" => "afternoon".to_string(),
            "afternoon" => "dusk".to_string(),
            "dusk" => "evening".to_string(),
            "evening" => "night".to_string(),
            "night" => "deep_night".to_string(),
            "deep_night" => "dawn".to_string(),
            _ => "morning".to_string(),
        }
    }
}

// ─── Events ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EventTable {
    pub id: String,
    pub campaign_id: String,
    pub name: String,
    pub location_type: Option<String>,
    pub trigger_type: String,
    pub trigger_chance: i64,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EventEntry {
    pub id: String,
    pub table_id: String,
    pub campaign_id: String,
    pub weight: i64,
    pub event_type: String,
    pub title: String,
    pub description: String,
    pub conditions: Option<String>,
    pub is_repeatable: bool,
    pub times_triggered: i64,
    pub created_at: String,
}

// ─── Game State ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum GameState {
    Exploration,
    Combat,
    Dialogue,
    Rest,
    Leveling,
    Shopping,
}

impl GameState {
    pub fn from_str(s: &str) -> Self {
        match s {
            "combat" => GameState::Combat,
            "dialogue" => GameState::Dialogue,
            "rest" => GameState::Rest,
            "leveling" => GameState::Leveling,
            "shopping" => GameState::Shopping,
            _ => GameState::Exploration,
        }
    }
}

// ─── API Types ────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateCampaignRequest {
    pub name: Option<String>,
    pub player_name: String,
    pub player_race: String,
    pub player_class: String,
    pub player_background: String,
    pub player_stats: PlayerStats,
    pub player_backstory: Option<String>,
    pub starting_gold: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct PlayerStats {
    pub str: i64,
    pub dex: i64,
    pub con: i64,
    pub int: i64,
    pub wis: i64,
    pub cha: i64,
}

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub campaign_id: String,
    pub session_id: String,
    pub content: String,
    pub game_state: Option<String>,
    pub roll_result: Option<RollResult>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RollResult {
    pub die: String,
    pub result: i64,
    pub skill: Option<String>,
    pub dc: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct PlayerStateResponse {
    pub player: Player,
    pub abilities: Vec<Ability>,
    pub equipped_items: Vec<Item>,
    pub inventory: Vec<Item>,
    pub active_companions: Vec<Companion>,
    pub current_location: Option<Location>,
    pub campaign_time: Option<CampaignTime>,
}