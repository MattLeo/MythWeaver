-- MythWeaver Database Schema
-- Migration 001: Initial schema

PRAGMA journal_mode=WAL;
PRAGMA foreign_keys=ON;

-- ─── Campaign ────────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS campaigns (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL DEFAULT 'MythWeaver Campaign',
    story_journal   TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ─── Sessions ────────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS sessions (
    id          TEXT PRIMARY KEY,
    campaign_id TEXT NOT NULL REFERENCES campaigns(id),
    started_at  TEXT NOT NULL DEFAULT (datetime('now')),
    ended_at    TEXT,
    is_active   INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS session_summaries (
    id          TEXT PRIMARY KEY,
    campaign_id TEXT NOT NULL REFERENCES campaigns(id),
    session_id  TEXT NOT NULL REFERENCES sessions(id),
    summary     TEXT NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS messages (
    id          TEXT PRIMARY KEY,
    session_id  TEXT NOT NULL REFERENCES sessions(id),
    campaign_id TEXT NOT NULL REFERENCES campaigns(id),
    role        TEXT NOT NULL CHECK(role IN ('user', 'assistant', 'tool')),
    content     TEXT NOT NULL,
    tool_calls  TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ─── Player ──────────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS players (
        id                   TEXT PRIMARY KEY,
    campaign_id          TEXT NOT NULL REFERENCES campaigns(id),
    name                 TEXT NOT NULL,
    race                 TEXT NOT NULL,
    species_subtype      TEXT,
    sex                  TEXT NOT NULL DEFAULT 'male'
                         CHECK(sex IN ('male', 'female')),
    class                TEXT NOT NULL,
    subclass             TEXT,
    background           TEXT NOT NULL,
    background_feat      TEXT,
    level                INTEGER NOT NULL DEFAULT 1,
    experience           INTEGER NOT NULL DEFAULT 0,
    current_hp           INTEGER NOT NULL DEFAULT 10,
    max_hp               INTEGER NOT NULL DEFAULT 10,
    temp_hp              INTEGER NOT NULL DEFAULT 0,
    armor_class          INTEGER NOT NULL DEFAULT 10,
    str                  INTEGER NOT NULL DEFAULT 10,
    dex                  INTEGER NOT NULL DEFAULT 10,
    con                  INTEGER NOT NULL DEFAULT 10,
    int                  INTEGER NOT NULL DEFAULT 10,
    wis                  INTEGER NOT NULL DEFAULT 10,
    cha                  INTEGER NOT NULL DEFAULT 10,
    proficiency_bonus    INTEGER NOT NULL DEFAULT 2,
    gold                 INTEGER NOT NULL DEFAULT 0,
    platinum             INTEGER NOT NULL DEFAULT 0,
    silver               INTEGER NOT NULL DEFAULT 0,
    copper               INTEGER NOT NULL DEFAULT 0,
    -- Combat modifiers
    crit_range_min       INTEGER NOT NULL DEFAULT 20,   -- Champion: 19, then 18
    extra_attacks        INTEGER NOT NULL DEFAULT 1,    -- Fighter 5: 2, 11: 3, 20: 4
    -- Indomitable
    indomitable_uses     INTEGER NOT NULL DEFAULT 0,
    indomitable_max      INTEGER NOT NULL DEFAULT 0,
    -- Current location
    current_location_id  TEXT REFERENCES locations(id),
    backstory            TEXT,
    -- Death saves
    death_save_successes INTEGER NOT NULL DEFAULT 0,
    death_save_failures  INTEGER NOT NULL DEFAULT 0,
    is_stable            INTEGER NOT NULL DEFAULT 1,
    is_dead              INTEGER NOT NULL DEFAULT 0,
    created_at           TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at           TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ─── Proficiencies ───────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS proficiencies (
    id               TEXT PRIMARY KEY,
    campaign_id      TEXT NOT NULL REFERENCES campaigns(id),
    player_id        TEXT NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    proficiency_type TEXT NOT NULL
                     CHECK(proficiency_type IN (
                         'weapon', 'armor', 'tool',
                         'skill', 'saving_throw', 'language'
                     )),
    name             TEXT NOT NULL,   -- "longsword", "stealth", "constitution"
    expertise        INTEGER NOT NULL DEFAULT 0,
    source           TEXT,            -- "class", "background", "feat", "racial"
    created_at       TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ─── Abilities ───────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS abilities (
    id              TEXT PRIMARY KEY,
    campaign_id     TEXT NOT NULL REFERENCES campaigns(id),
    owner_type      TEXT NOT NULL CHECK(owner_type IN ('player', 'companion')),
    owner_id        TEXT NOT NULL,
    name            TEXT NOT NULL,
    description     TEXT,
    current_uses    INTEGER NOT NULL DEFAULT 1,
    max_uses        INTEGER NOT NULL DEFAULT 1,
    refresh_type    TEXT NOT NULL CHECK(refresh_type IN ('short_rest', 'long_rest', 'per_turn', 'manual')),
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ─── Active Effects ──────────────────────────────────────────────────────────
-- Temporary modifiers applied during combat or for a duration

CREATE TABLE IF NOT EXISTS active_effects (
    id              TEXT PRIMARY KEY,
    campaign_id     TEXT NOT NULL REFERENCES campaigns(id),
    target_type     TEXT NOT NULL CHECK(target_type IN ('player', 'companion', 'enemy')),
    target_id       TEXT NOT NULL,
    name            TEXT NOT NULL,   -- "Rage", "Bless", "Bait and Switch AC"
    effect_type     TEXT NOT NULL
                    CHECK(effect_type IN (
                        'damage_bonus',      -- flat bonus to damage rolls
                        'attack_bonus',      -- flat bonus to attack rolls
                        'ac_bonus',          -- flat bonus to AC
                        'damage_resistance', -- halve damage of specified type
                        'advantage_attack',  -- advantage on attack rolls
                        'advantage_save',    -- advantage on saving throws
                        'disadvantage_attack',
                        'disadvantage_save',
                        'temp_hp',           -- temporary hit points
                        'speed_bonus',
                        'crit_range',        -- changes crit threshold
                        'custom'             -- narrative/model-handled
                    )),
    value           INTEGER,         -- numeric value where applicable
    damage_type     TEXT,            -- for resistance: "fire", "all", etc.
    duration_type   TEXT NOT NULL
                    CHECK(duration_type IN (
                        'end_of_turn',
                        'start_of_next_turn',
                        'until_hit',
                        'rounds',
                        'concentration',
                        'permanent'
                    )),
    duration_value  INTEGER,         -- number of rounds if duration_type = 'rounds'
    source          TEXT,            -- "Rage", "Bait and Switch", "Bless"
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ─── Superiority Dice ────────────────────────────────────────────────────────
-- Used by Battle Master and Psi Warrior

CREATE TABLE IF NOT EXISTS superiority_dice (
    id              TEXT PRIMARY KEY,
    campaign_id     TEXT NOT NULL REFERENCES campaigns(id),
    player_id       TEXT NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    pool_name       TEXT NOT NULL,   -- "Battle Master", "Psi Warrior"
    die_size        INTEGER NOT NULL DEFAULT 8,   -- 6, 8, 10, 12
    current_dice    INTEGER NOT NULL DEFAULT 4,
    max_dice        INTEGER NOT NULL DEFAULT 4,
    refresh_type    TEXT NOT NULL DEFAULT 'short_rest'
                    CHECK(refresh_type IN ('short_rest', 'long_rest')),
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ─── Known Maneuvers ─────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS known_maneuvers (
    id              TEXT PRIMARY KEY,
    campaign_id     TEXT NOT NULL REFERENCES campaigns(id),
    player_id       TEXT NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    maneuver_name   TEXT NOT NULL,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(player_id, maneuver_name)
);

-- ─── Weapon Mastery ──────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS weapon_mastery (
    id              TEXT PRIMARY KEY,
    campaign_id     TEXT NOT NULL REFERENCES campaigns(id),
    player_id       TEXT NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    weapon_type     TEXT NOT NULL,   -- "longsword", "greataxe", "handaxe"
    mastery_property TEXT NOT NULL
                    CHECK(mastery_property IN (
                        'cleave', 'graze', 'nick', 'push',
                        'sap', 'slow', 'topple', 'vex'
                    )),
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(player_id, weapon_type)
);

-- ─── World ───────────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS locations (
    id              TEXT PRIMARY KEY,
    campaign_id     TEXT NOT NULL REFERENCES campaigns(id),
    name            TEXT NOT NULL,
    location_type   TEXT NOT NULL DEFAULT 'area',
    description     TEXT NOT NULL,
    state           TEXT,
    is_discovered   INTEGER NOT NULL DEFAULT 0,
    notes           TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS location_connections (
    id              TEXT PRIMARY KEY,
    campaign_id     TEXT NOT NULL REFERENCES campaigns(id),
    from_location   TEXT NOT NULL REFERENCES locations(id),
    to_location     TEXT NOT NULL REFERENCES locations(id),
    travel_notes    TEXT,
    is_hidden       INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS npcs (
    id              TEXT PRIMARY KEY,
    campaign_id     TEXT NOT NULL REFERENCES campaigns(id),
    name            TEXT NOT NULL,
    race            TEXT,
    occupation      TEXT,
    description     TEXT NOT NULL,
    personality     TEXT,
    disposition     TEXT NOT NULL DEFAULT 'neutral'
                    CHECK(disposition IN ('friendly', 'neutral', 'unfriendly', 'hostile', 'allied')),
    current_location_id TEXT REFERENCES locations(id),
    is_alive        INTEGER NOT NULL DEFAULT 1,
    is_hidden       INTEGER NOT NULL DEFAULT 0,
    notes           TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS world_facts (
    id              TEXT PRIMARY KEY,
    campaign_id     TEXT NOT NULL REFERENCES campaigns(id),
    category        TEXT,
    title           TEXT NOT NULL,
    content         TEXT NOT NULL,
    tags            TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ─── Items ───────────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS items (
    id              TEXT PRIMARY KEY,
    campaign_id     TEXT NOT NULL REFERENCES campaigns(id),
    owner_type      TEXT CHECK(owner_type IN ('player', 'companion', 'location', 'npc')),
    owner_id        TEXT,
    name            TEXT NOT NULL,
    description     TEXT NOT NULL,
    item_type       TEXT NOT NULL
                    CHECK(item_type IN ('weapon', 'armor', 'consumable', 'wondrous', 'quest', 'shield')),
    quantity        INTEGER NOT NULL DEFAULT 1,
    is_equipped     INTEGER NOT NULL DEFAULT 0,
    slot            TEXT CHECK(slot IN (
                        'main_hand', 'off_hand', 'armor', 'cloak',
                        'ring_1', 'ring_2', 'boots', 'helmet', 'amulet', 'shield'
                    )),
    -- Weapon stats
    damage_die      TEXT,
    damage_type     TEXT,
    weapon_range    TEXT,
    weapon_type     TEXT,            -- "longsword", "greataxe" — for mastery lookup
    -- Armor stats
    base_ac         INTEGER,
    armor_type      TEXT CHECK(armor_type IN ('light', 'medium', 'heavy', 'shield', null)),
    stealth_disadvantage INTEGER NOT NULL DEFAULT 0,
    -- Misc
    rarity          TEXT NOT NULL DEFAULT 'common'
                    CHECK(rarity IN ('common', 'uncommon', 'rare', 'very_rare', 'legendary')),
    notes           TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS item_effects (
    id              TEXT PRIMARY KEY,
    item_id         TEXT NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    effect_type     TEXT NOT NULL
                    CHECK(effect_type IN (
                        'ac_bonus', 'attack_bonus', 'damage_bonus',
                        'advantage_on', 'disadvantage_on', 'damage_die',
                        'resistance_to', 'immunity_to', 'speed_bonus'
                    )),
    value           INTEGER,
    target          TEXT
);

-- ─── Shop ────────────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS shop_sessions (
    id              TEXT PRIMARY KEY,
    campaign_id     TEXT NOT NULL REFERENCES campaigns(id),
    merchant_npc_id TEXT REFERENCES npcs(id),
    shop_name       TEXT NOT NULL,
    shop_type       TEXT NOT NULL DEFAULT 'general'
                    CHECK(shop_type IN ('general', 'blacksmith', 'alchemist', 'magic', 'fletcher', 'tailor', 'jeweler', 'inn', 'black_market')),
    status          TEXT NOT NULL DEFAULT 'open'
                    CHECK(status IN ('open', 'closed')),
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS shop_items (
    id              TEXT PRIMARY KEY,
    shop_session_id TEXT NOT NULL REFERENCES shop_sessions(id),
    campaign_id     TEXT NOT NULL REFERENCES campaigns(id),
    name            TEXT NOT NULL,
    description     TEXT NOT NULL,
    item_type       TEXT NOT NULL
                    CHECK(item_type IN ('weapon', 'armor', 'shield', 'consumable', 'wondrous', 'quest')),
    -- Weapon fields
    damage_die      TEXT,
    damage_type     TEXT,
    weapon_range    TEXT CHECK(weapon_range IN ('melee', 'ranged', null)),
    weapon_type     TEXT,
    -- Armor fields
    base_ac         INTEGER,
    armor_type      TEXT CHECK(armor_type IN ('light', 'medium', 'heavy', 'shield', null)),
    -- Pricing (all denominations)
    price_pp        INTEGER NOT NULL DEFAULT 0,
    price_gp        INTEGER NOT NULL DEFAULT 0,
    price_sp        INTEGER NOT NULL DEFAULT 0,
    price_cp        INTEGER NOT NULL DEFAULT 0,
    -- Stock
    quantity        INTEGER NOT NULL DEFAULT 1,
    quantity_sold   INTEGER NOT NULL DEFAULT 0,
    rarity          TEXT NOT NULL DEFAULT 'common'
                    CHECK(rarity IN ('common', 'uncommon', 'rare', 'very_rare', 'legendary')),
    notes           TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS shop_transactions (
    id              TEXT PRIMARY KEY,
    shop_session_id TEXT NOT NULL REFERENCES shop_sessions(id),
    campaign_id     TEXT NOT NULL REFERENCES campaigns(id),
    transaction_type TEXT NOT NULL CHECK(transaction_type IN ('buy', 'sell')),
    item_name       TEXT NOT NULL,
    shop_item_id    TEXT REFERENCES shop_items(id),  -- null for sell transactions
    player_item_id  TEXT REFERENCES items(id),        -- null for buy transactions
    quantity        INTEGER NOT NULL DEFAULT 1,
    price_pp        INTEGER NOT NULL DEFAULT 0,
    price_gp        INTEGER NOT NULL DEFAULT 0,
    price_sp        INTEGER NOT NULL DEFAULT 0,
    price_cp        INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ─── Companions ──────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS companions (
    id                  TEXT PRIMARY KEY,
    campaign_id         TEXT NOT NULL REFERENCES campaigns(id),
    name                TEXT NOT NULL,
    companion_type      TEXT NOT NULL
                        CHECK(companion_type IN ('ally', 'familiar', 'animal', 'hireling', 'summon')),
    description         TEXT NOT NULL,
    personality         TEXT,
    disposition         TEXT NOT NULL DEFAULT 'friendly'
                        CHECK(disposition IN ('friendly', 'neutral', 'unfriendly', 'hostile', 'allied')),
    current_hp          INTEGER NOT NULL DEFAULT 10,
    max_hp              INTEGER NOT NULL DEFAULT 10,
    armor_class         INTEGER NOT NULL DEFAULT 10,
    attack_bonus        INTEGER NOT NULL DEFAULT 2,
    damage_die          TEXT NOT NULL DEFAULT 'd6',
    damage_bonus        INTEGER NOT NULL DEFAULT 0,
    damage_type         TEXT NOT NULL DEFAULT 'slashing',
    is_alive            INTEGER NOT NULL DEFAULT 1,
    is_active           INTEGER NOT NULL DEFAULT 1,
    current_location_id TEXT REFERENCES locations(id),
    notes               TEXT,
    created_at          TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at          TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ─── Combat ──────────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS combat_encounters (
    id                          TEXT PRIMARY KEY,
    campaign_id                 TEXT NOT NULL REFERENCES campaigns(id),
    status                      TEXT NOT NULL DEFAULT 'active'
                                CHECK(status IN ('active', 'victory', 'defeat', 'fled')),
    round_number                INTEGER NOT NULL DEFAULT 1,
    turn_index                  INTEGER NOT NULL DEFAULT 0,
    turn_order_json             TEXT,
    current_target_id           TEXT,
    player_rolled_initiative    INTEGER NOT NULL DEFAULT 0,
    actions_remaining           INTEGER NOT NULL DEFAULT 1,
    bonus_actions_remaining     INTEGER NOT NULL DEFAULT 1,
    reactions_remaining         INTEGER NOT NULL DEFAULT 1,
    attacks_remaining           INTEGER NOT NULL DEFAULT 1,
    action_surge_available      INTEGER NOT NULL DEFAULT 0,
    action_surge_used           INTEGER NOT NULL DEFAULT 0,
    attacks_made_this_action    INTEGER NOT NULL DEFAULT 0,
    created_at                  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at                  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS combat_enemies (
    id                  TEXT PRIMARY KEY,
    encounter_id        TEXT NOT NULL REFERENCES combat_encounters(id),
    campaign_id         TEXT NOT NULL,
    name                TEXT NOT NULL,
    description         TEXT,
    participant_type     TEXT NOT NULL DEFAULT 'enemy'
                        CHECK(participant_type IN ('enemy', 'ally')),
    weapon_name         TEXT NOT NULL DEFAULT 'weapon',
    max_hp              INTEGER NOT NULL DEFAULT 10,
    current_hp          INTEGER NOT NULL DEFAULT 10,
    armor_class         INTEGER NOT NULL DEFAULT 11,
    attack_bonus        INTEGER NOT NULL DEFAULT 0,
    damage_die          TEXT NOT NULL DEFAULT 'd6',
    damage_bonus        INTEGER NOT NULL DEFAULT 0,
    damage_type         TEXT NOT NULL DEFAULT 'slashing',
    initiative_score    INTEGER NOT NULL DEFAULT 0,
    is_alive            INTEGER NOT NULL DEFAULT 1,
    is_bloodied         INTEGER NOT NULL DEFAULT 0,
    is_prone            INTEGER NOT NULL DEFAULT 0,
    is_frightened       INTEGER NOT NULL DEFAULT 0,
    is_disarmed         INTEGER NOT NULL DEFAULT 0,
    player_missed_last_attack INTEGER NOT NULL DEFAULT 0,
    threat_json         TEXT NOT NULL DEFAULT '{}',
    enemy_type          TEXT NOT NULL DEFAULT 'humanoid_basic',
    spell_list_json     TEXT NOT NULL DEFAULT '[]', 
    created_at          TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ─── Time ────────────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS campaign_time (
    id              TEXT PRIMARY KEY,
    campaign_id     TEXT NOT NULL REFERENCES campaigns(id) UNIQUE,
    time_of_day     TEXT NOT NULL DEFAULT 'morning'
                    CHECK(time_of_day IN (
                        'dawn', 'morning', 'midday',
                        'afternoon', 'dusk', 'evening', 'night', 'deep_night'
                    )),
    current_day     INTEGER NOT NULL DEFAULT 1,
    season          TEXT NOT NULL DEFAULT 'spring'
                    CHECK(season IN ('spring', 'summer', 'autumn', 'winter')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ─── Events ──────────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS event_tables (
    id              TEXT PRIMARY KEY,
    campaign_id     TEXT NOT NULL REFERENCES campaigns(id),
    name            TEXT NOT NULL,
    location_type   TEXT,
    trigger_type    TEXT NOT NULL
                    CHECK(trigger_type IN ('travel', 'rest', 'time', 'location_enter', 'manual')),
    trigger_chance  INTEGER NOT NULL DEFAULT 30,
    is_active       INTEGER NOT NULL DEFAULT 1,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS event_entries (
    id              TEXT PRIMARY KEY,
    table_id        TEXT NOT NULL REFERENCES event_tables(id) ON DELETE CASCADE,
    campaign_id     TEXT NOT NULL REFERENCES campaigns(id),
    weight          INTEGER NOT NULL DEFAULT 10,
    event_type      TEXT NOT NULL
                    CHECK(event_type IN ('encounter', 'environmental', 'world', 'discovery', 'personal')),
    title           TEXT NOT NULL,
    description     TEXT NOT NULL,
    conditions      TEXT,
    is_repeatable   INTEGER NOT NULL DEFAULT 1,
    times_triggered INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ─── Spells Reference Table ───────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS spells (
    id                  TEXT PRIMARY KEY,
    name                TEXT NOT NULL UNIQUE,
    level               INTEGER NOT NULL DEFAULT 0,
    school              TEXT NOT NULL CHECK(school IN (
                            'abjuration','conjuration','divination','enchantment',
                            'evocation','illusion','necromancy','transmutation'
                        )),
    casting_time        TEXT NOT NULL DEFAULT 'action',
    range_type          TEXT NOT NULL DEFAULT 'self',
    range_feet          INTEGER,
    has_verbal          INTEGER NOT NULL DEFAULT 1,
    has_somatic         INTEGER NOT NULL DEFAULT 1,
    has_material        INTEGER NOT NULL DEFAULT 0,
    material_component  TEXT,
    duration            TEXT NOT NULL DEFAULT 'instantaneous',
    concentration       INTEGER NOT NULL DEFAULT 0,
    ritual              INTEGER NOT NULL DEFAULT 0,
    description         TEXT NOT NULL,
    damage_die          TEXT,
    damage_die_count    INTEGER DEFAULT 1,
    damage_type         TEXT,
    scales_with_level   INTEGER NOT NULL DEFAULT 0,
    cantrip_dice_5      INTEGER,
    cantrip_dice_11     INTEGER,
    cantrip_dice_17     INTEGER,
    slot_scale_dice     INTEGER,
    save_type           TEXT,
    attack_type         TEXT,
    target_type         TEXT NOT NULL DEFAULT 'single'
                        CHECK(target_type IN ('self','single','area','multiple')),
    area_shape          TEXT,
    area_size_feet      INTEGER,
    has_backend_resolver INTEGER NOT NULL DEFAULT 0,
    is_wizard_spell     INTEGER NOT NULL DEFAULT 1,
    created_at          TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ─── Player Spell Slots ───────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS spell_slots (
    id              TEXT PRIMARY KEY,
    campaign_id     TEXT NOT NULL REFERENCES campaigns(id),
    player_id       TEXT NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    slot_level      INTEGER NOT NULL CHECK(slot_level BETWEEN 1 AND 9),
    current_slots   INTEGER NOT NULL DEFAULT 0,
    max_slots       INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(player_id, slot_level)
);

-- ─── Known / Prepared Spells ─────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS known_spells (
    id              TEXT PRIMARY KEY,
    campaign_id     TEXT NOT NULL REFERENCES campaigns(id),
    player_id       TEXT NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    spell_id        TEXT NOT NULL REFERENCES spells(id),
    spell_type      TEXT NOT NULL DEFAULT 'prepared'
                    CHECK(spell_type IN ('cantrip', 'prepared', 'ritual', 'always_prepared')),
    source          TEXT NOT NULL DEFAULT 'eldritch_knight',
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(player_id, spell_id)
);

-- ─── War Bond ─────────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS war_bonds (
    id              TEXT PRIMARY KEY,
    campaign_id     TEXT NOT NULL REFERENCES campaigns(id),
    player_id       TEXT NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    item_id         TEXT NOT NULL REFERENCES items(id),
    is_summoned     INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(player_id, item_id)
);

-- ─── Concentration Tracking ───────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS concentration (
    id              TEXT PRIMARY KEY,
    campaign_id     TEXT NOT NULL REFERENCES campaigns(id),
    player_id       TEXT NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    spell_id        TEXT NOT NULL REFERENCES spells(id),
    spell_name      TEXT NOT NULL,
    started_at      TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at      TEXT,
    UNIQUE(player_id)  -- can only concentrate on one spell at a time
);

-- ─── Feats Tables ─────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS feats (
    id                          TEXT PRIMARY KEY,
    name                        TEXT NOT NULL,
    category                    TEXT NOT NULL,  -- 'origin' | 'general' | 'fighting_style' | 'epic_boon'
    prerequisite_level          INTEGER NOT NULL DEFAULT 0,
    prerequisite_feature        TEXT,    -- 'spellcasting' | 'pact_magic' | 'fighting_style'
                                         -- | 'heavy_armor_training' | 'medium_armor_training'
                                         -- | 'light_armor_training' | 'shield_training'
    prerequisite_ability        TEXT,    -- stat abbrev with '/' for OR, e.g. 'str' or 'str/dex'
    prerequisite_ability_score  INTEGER NOT NULL DEFAULT 0,
    description                 TEXT NOT NULL,
    ability_score_options       TEXT,    -- JSON array of eligible stats, e.g. '["str","dex"]'
    ability_score_increase      INTEGER NOT NULL DEFAULT 0,  -- 0 = no ASI, 1 = +1, 2 = +2
    repeatable                  INTEGER NOT NULL DEFAULT 0,  -- 1 = can take multiple times
    has_choice                  INTEGER NOT NULL DEFAULT 0,  -- 1 = player must make a selection
    choice_description          TEXT,    -- what the player must choose (skill, spell, damage type, etc.)
    grants_spells               TEXT,    -- JSON: always-prepared spells granted by the feat
    created_at                  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS player_feats (
    id          TEXT PRIMARY KEY,
    campaign_id TEXT NOT NULL,
    player_id   TEXT NOT NULL,
    feat_id     TEXT NOT NULL,
    feat_name   TEXT NOT NULL,
    source      TEXT NOT NULL,   -- 'background' | 'asi' | 'epic_boon' | 'class'
    level_taken INTEGER,         -- which player level the feat was taken at
    choices     TEXT,            -- JSON: choices made (e.g. {"skill":"arcana","spell":"detect_magic"})
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (feat_id) REFERENCES feats(id)
);

-- ─── Indexes ─────────────────────────────────────────────────────────────────

CREATE INDEX IF NOT EXISTS idx_messages_session         ON messages(session_id);
CREATE INDEX IF NOT EXISTS idx_messages_campaign        ON messages(campaign_id);
CREATE INDEX IF NOT EXISTS idx_npcs_location            ON npcs(current_location_id);
CREATE INDEX IF NOT EXISTS idx_npcs_campaign            ON npcs(campaign_id);
CREATE INDEX IF NOT EXISTS idx_locations_campaign       ON locations(campaign_id);
CREATE INDEX IF NOT EXISTS idx_items_owner              ON items(owner_type, owner_id);
CREATE INDEX IF NOT EXISTS idx_world_facts_campaign     ON world_facts(campaign_id);
CREATE INDEX IF NOT EXISTS idx_companions_campaign      ON companions(campaign_id);
CREATE INDEX IF NOT EXISTS idx_event_entries_table      ON event_entries(table_id);
CREATE INDEX IF NOT EXISTS idx_combat_encounter         ON combat_encounters(campaign_id, status);
CREATE INDEX IF NOT EXISTS idx_combat_enemies           ON combat_enemies(encounter_id);
CREATE INDEX IF NOT EXISTS idx_proficiencies_player     ON proficiencies(player_id);
CREATE INDEX IF NOT EXISTS idx_active_effects_target    ON active_effects(target_type, target_id);
CREATE INDEX IF NOT EXISTS idx_superiority_player       ON superiority_dice(player_id);
CREATE INDEX IF NOT EXISTS idx_known_maneuvers_player   ON known_maneuvers(player_id);
CREATE INDEX IF NOT EXISTS idx_weapon_mastery_player    ON weapon_mastery(player_id);
CREATE INDEX IF NOT EXISTS idx_shop_sessions_campaign   ON shop_sessions(campaign_id, status);
CREATE INDEX IF NOT EXISTS idx_shop_items_session       ON shop_items(shop_session_id);
CREATE INDEX IF NOT EXISTS idx_shop_transactions        ON shop_transactions(shop_session_id);
CREATE INDEX IF NOT EXISTS idx_spells_level             ON spells(level);
CREATE INDEX IF NOT EXISTS idx_spells_school            ON spells(school);
CREATE INDEX IF NOT EXISTS idx_spells_name              ON spells(name);
CREATE INDEX IF NOT EXISTS idx_player_feats_player      ON player_feats(player_id);
CREATE INDEX IF NOT EXISTS idx_player_feats_campaign    ON player_feats(campaign_id);
