-- MythWeaver Database Schema
-- Migration 001: Initial schema

PRAGMA journal_mode=WAL;
PRAGMA foreign_keys=ON;

-- ─── Campaign ────────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS campaigns (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL DEFAULT 'MythWeaver Campaign',
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
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
    tool_calls  TEXT,  -- JSON array of tool calls if any
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ─── Player ──────────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS players (
    id                  TEXT PRIMARY KEY,
    campaign_id         TEXT NOT NULL REFERENCES campaigns(id),
    name                TEXT NOT NULL,
    race                TEXT NOT NULL,
    class               TEXT NOT NULL,
    background          TEXT NOT NULL,
    level               INTEGER NOT NULL DEFAULT 1,
    experience          INTEGER NOT NULL DEFAULT 0,
    current_hp          INTEGER NOT NULL DEFAULT 10,
    max_hp              INTEGER NOT NULL DEFAULT 10,
    temp_hp             INTEGER NOT NULL DEFAULT 0,
    armor_class         INTEGER NOT NULL DEFAULT 10,
    str                 INTEGER NOT NULL DEFAULT 10,
    dex                 INTEGER NOT NULL DEFAULT 10,
    con                 INTEGER NOT NULL DEFAULT 10,
    int                 INTEGER NOT NULL DEFAULT 10,
    wis                 INTEGER NOT NULL DEFAULT 10,
    cha                 INTEGER NOT NULL DEFAULT 10,
    proficiency_bonus   INTEGER NOT NULL DEFAULT 2,
    gold                INTEGER NOT NULL DEFAULT 0,
    current_location_id TEXT REFERENCES locations(id),
    backstory           TEXT,
    -- Death saves
    death_save_successes INTEGER NOT NULL DEFAULT 0,
    death_save_failures  INTEGER NOT NULL DEFAULT 0,
    is_stable           INTEGER NOT NULL DEFAULT 1,
    is_dead             INTEGER NOT NULL DEFAULT 0,
    created_at          TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at          TEXT NOT NULL DEFAULT (datetime('now'))
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

-- ─── World ───────────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS locations (
    id              TEXT PRIMARY KEY,
    campaign_id     TEXT NOT NULL REFERENCES campaigns(id),
    name            TEXT NOT NULL,
    location_type   TEXT NOT NULL DEFAULT 'area',
    description     TEXT NOT NULL,
    state           TEXT,           -- "burned down", "occupied", "abandoned"
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
    travel_notes    TEXT,           -- "rough road, half a day's travel"
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
    category        TEXT,           -- "faction", "history", "quest", "rumor"
    title           TEXT NOT NULL,
    content         TEXT NOT NULL,
    tags            TEXT,           -- JSON array of searchable tags
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
    damage_die      TEXT,           -- "1d8", "2d6"
    damage_type     TEXT,           -- "slashing", "piercing", "bludgeoning"
    weapon_range    TEXT,           -- "melee", "ranged (80/320)"
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
    value           INTEGER,        -- numeric bonus where applicable
    target          TEXT            -- "stealth", "fire", "all_saves", etc.
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
    is_alive            INTEGER NOT NULL DEFAULT 1,
    is_active           INTEGER NOT NULL DEFAULT 1,  -- traveling with player
    current_location_id TEXT REFERENCES locations(id),
    notes               TEXT,
    created_at          TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at          TEXT NOT NULL DEFAULT (datetime('now'))
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
    location_type   TEXT,           -- null = global
    trigger_type    TEXT NOT NULL
                    CHECK(trigger_type IN ('travel', 'rest', 'time', 'location_enter', 'manual')),
    trigger_chance  INTEGER NOT NULL DEFAULT 30,  -- percent chance per trigger
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
    description     TEXT NOT NULL,  -- context injected into DM prompt
    conditions      TEXT,           -- JSON: {"min_level": 3, "time_of_day": "night"}
    is_repeatable   INTEGER NOT NULL DEFAULT 1,
    times_triggered INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ─── Indexes ─────────────────────────────────────────────────────────────────

CREATE INDEX IF NOT EXISTS idx_messages_session    ON messages(session_id);
CREATE INDEX IF NOT EXISTS idx_messages_campaign   ON messages(campaign_id);
CREATE INDEX IF NOT EXISTS idx_npcs_location       ON npcs(current_location_id);
CREATE INDEX IF NOT EXISTS idx_npcs_campaign       ON npcs(campaign_id);
CREATE INDEX IF NOT EXISTS idx_locations_campaign  ON locations(campaign_id);
CREATE INDEX IF NOT EXISTS idx_items_owner         ON items(owner_type, owner_id);
CREATE INDEX IF NOT EXISTS idx_world_facts_campaign ON world_facts(campaign_id);
CREATE INDEX IF NOT EXISTS idx_companions_campaign ON companions(campaign_id);
CREATE INDEX IF NOT EXISTS idx_event_entries_table ON event_entries(table_id);