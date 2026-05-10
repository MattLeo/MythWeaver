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
    pub species_subtype: Option<String>,
    pub sex: String,
    pub class: String,
    pub subclass: Option<String>,
    pub background: String,
    pub background_feat: Option<String>,
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
    pub platinum: i64,
    pub silver: i64,
    pub copper: i64,
    pub crit_range_min: i64,
    pub extra_attacks: i64,
    pub indomitable_uses: i64,
    pub indomitable_max: i64,
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
            1  => 300,
            2  => 900,
            3  => 2700,
            4  => 6500,
            5  => 14000,
            6  => 23000,
            7  => 34000,
            8  => 48000,
            9  => 64000,
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
            _  => i64::MAX,
        }
    }

    pub fn proficiency_for_level(level: i64) -> i64 {
        match level {
            1..=4  => 2,
            5..=8  => 3,
            9..=12 => 4,
            13..=16 => 5,
            _ => 6,
        }
    }

    pub fn is_asi_level(level: i64) -> bool {
        matches!(level, 4 | 8 | 12 | 16 | 19)
    }

    pub fn pronouns(&self) -> (&'static str, &'static str, &'static str) {
        // (subject, object, possessive)
        match self.sex.as_str() {
            "female" => ("she", "her", "her"),
            _        => ("he",  "him", "his"),
        }
    }

    pub fn maneuver_save_dc(&self) -> i64 {
        let best_mod = Player::modifier(self.str).max(Player::modifier(self.dex));
        8 + self.proficiency_bonus + best_mod
    }

    pub fn spellcasting_modifier(&self) -> i64 {
        match self.subclass.as_deref() {
            Some("Eldritch Knight") | Some("Psi Warrior") => Player::modifier(self.int),
            _ => 0,
        }
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
    pub subclass_choice_required: bool,
    pub new_features: Vec<String>,
    pub spell_slots: Option<SpellSlots>,
    // Fighter
    pub second_wind_uses: i64,
    pub weapon_mastery_count: i64,
    pub extra_attacks: i64,
    pub indomitable_max: i64,
    pub action_surge_uses: i64,
    // Barbarian
    pub rage_uses: i64,
    pub rage_damage: i64,
    // Bard
    pub bardic_die: i64,
    pub bardic_inspiration_uses: i64,
    pub bard_prepared_spells: i64,
    pub bard_cantrips: i64,
    // Cleric
    pub channel_divinity_uses: i64,
    pub cleric_cantrips: i64,
    pub cleric_prepared_spells: i64,
    // Druid
    pub wild_shape_uses: i64,
    pub druid_cantrips: i64,
    pub druid_prepared_spells: i64,
    // Monk
    pub focus_points: i64,
    pub martial_arts_die: i64,
    pub unarmored_movement: i64,
    // Paladin
    pub lay_on_hands_pool: i64,
    pub paladin_channel_divinity: i64,
    pub paladin_prepared_spells: i64,
    // Ranger
    pub favored_enemy_uses: i64,
    pub ranger_prepared_spells: i64,
    // Rogue
    pub sneak_attack_dice: i64,
    pub at_prepared_spells: i64,
    pub at_cantrips: i64,
    // Sorcerer
    pub sorcery_points: i64,
    pub sorcerer_cantrips: i64,
    pub sorcerer_prepared_spells: i64,
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

// ─── Fighter progression ──────────────────────────────────────────────────────

pub fn fighter_second_wind_uses(level: i64) -> i64 {
    match level {
        1..=3   => 2,
        4..=9   => 3,
        10..=19 => 4,
        _ => 4,
    }
}

pub fn fighter_weapon_mastery_count(level: i64) -> i64 {
    match level {
        1..=3   => 3,
        4..=15  => 4,
        16..=19 => 5,
        _ => 6,
    }
}

pub fn fighter_extra_attacks(level: i64) -> i64 {
    match level {
        1..=4   => 1,
        5..=10  => 2,
        11..=19 => 3,
        _ => 4,
    }
}

pub fn fighter_action_surge_uses(level: i64) -> i64 {
    match level {
        2..=16 => 1,
        17..=20 => 2,
        _ => 0,
    }
}

pub fn fighter_indomitable_max(level: i64) -> i64 {
    match level {
        9..=12  => 1,
        13..=16 => 2,
        17..=20 => 3,
        _ => 0,
    }
}

// ─── Battle Master ────────────────────────────────────────────────────────────

pub const ALL_MANEUVERS: &[&str] = &[
    "Ambush", "Bait and Switch", "Commander's Strike", "Commanding Presence",
    "Disarming Attack", "Distracting Strike", "Evasive Footwork", "Feinting Attack",
    "Goading Attack", "Lunging Attack", "Maneuvering Attack", "Menacing Attack",
    "Parry", "Precision Attack", "Pushing Attack", "Rally",
    "Riposte", "Sweeping Attack", "Tactical Assessment", "Trip Attack",
];

pub fn battle_master_maneuver_count(level: i64) -> i64 {
    match level {
        3..=6   => 3,
        7..=9   => 5,
        10..=14 => 7,
        15..=20 => 9,
        _ => 0,
    }
}

pub fn battle_master_superiority_dice(level: i64) -> (i64, i64) {
    match level {
        3..=6   => (4, 8),
        7..=9   => (5, 8),
        10..=14 => (5, 10),
        15..=17 => (6, 10),
        18..=20 => (6, 12),
        _ => (0, 8),
    }
}

pub fn psi_warrior_energy_dice(level: i64) -> (i64, i64) {
    match level {
        3..=4   => (4, 6),
        5..=8   => (6, 8),
        9..=10  => (8, 8),
        11..=12 => (8, 10),
        13..=16 => (10, 10),
        17..=20 => (12, 12),
        _ => (0, 6),
    }
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

// ─── Active Effect ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ActiveEffect {
    pub id: String,
    pub campaign_id: String,
    pub target_type: String,
    pub target_id: String,
    pub name: String,
    pub effect_type: String,
    pub value: Option<i64>,
    pub damage_type: Option<String>,
    pub duration_type: String,
    pub duration_value: Option<i64>,
    pub source: Option<String>,
    pub created_at: String,
}

// ─── Superiority Dice ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SuperiorityDice {
    pub id: String,
    pub campaign_id: String,
    pub player_id: String,
    pub pool_name: String,
    pub die_size: i64,
    pub current_dice: i64,
    pub max_dice: i64,
    pub refresh_type: String,
    pub created_at: String,
}

// ─── Known Maneuver ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct KnownManeuver {
    pub id: String,
    pub campaign_id: String,
    pub player_id: String,
    pub maneuver_name: String,
    pub created_at: String,
}

// ─── Weapon Mastery ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WeaponMastery {
    pub id: String,
    pub campaign_id: String,
    pub player_id: String,
    pub weapon_type: String,
    pub mastery_property: String,
    pub created_at: String,
}

// ─── Proficiency ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Proficiency {
    pub id: String,
    pub campaign_id: String,
    pub player_id: String,
    pub proficiency_type: String,
    pub name: String,
    pub expertise: bool,
    pub source: Option<String>,
    pub created_at: String,
}

// ─── Action Economy ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionEconomy {
    pub actions_remaining: i64,
    pub bonus_actions_remaining: i64,
    pub reactions_remaining: i64,
    pub action_surge_available: bool,
    pub action_surge_used: bool,
    pub attacks_made_this_action: i64,
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
    pub weapon_type: Option<String>,
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
    pub attack_bonus: i64,
    pub damage_die: String,
    pub damage_bonus: i64,
    pub damage_type: String,
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
            "dawn"       => "morning".to_string(),
            "morning"    => "midday".to_string(),
            "midday"     => "afternoon".to_string(),
            "afternoon"  => "dusk".to_string(),
            "dusk"       => "evening".to_string(),
            "evening"    => "night".to_string(),
            "night"      => "deep_night".to_string(),
            "deep_night" => "dawn".to_string(),
            _            => "morning".to_string(),
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
            "combat"      => GameState::Combat,
            "dialogue"    => GameState::Dialogue,
            "rest"        => GameState::Rest,
            "leveling"    => GameState::Leveling,
            "shopping"    => GameState::Shopping,
            _             => GameState::Exploration,
        }
    }
}

// ─── API Types ────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct BackgroundAsi {
    pub str: Option<i64>,
    pub dex: Option<i64>,
    pub con: Option<i64>,
    pub int: Option<i64>,
    pub wis: Option<i64>,
    pub cha: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCampaignRequest {
    pub name: Option<String>,
    pub player_name: String,
    pub player_race: String,
    pub player_species_subtype: Option<String>,
    pub player_sex: String,
    pub player_class: String,
    pub player_background: String,
    pub player_background_feat: Option<String>,
    pub player_background_skill_1: String,
    pub player_background_skill_2: String,
    pub player_background_tool: String,
    pub player_background_asi: BackgroundAsi,
    pub player_stats: PlayerStats,
    pub player_backstory: Option<String>,
    pub equipment_choice: String,
    pub divine_order: Option<String>,  // "Protector" | "Thaumaturge" — Cleric only
    pub thaumaturge_cantrip: Option<String>,
    pub primal_order: Option<String>, 
    pub magician_cantrip: Option<String>,
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

#[derive(Debug, serde::Deserialize)]
pub struct LevelUpRequest {
    pub subclass: Option<String>,
    pub asi_stat1: Option<String>,
    pub asi_stat2: Option<String>,
    pub new_maneuvers: Option<Vec<String>>,
    pub replaced_maneuver: Option<String>,
}
