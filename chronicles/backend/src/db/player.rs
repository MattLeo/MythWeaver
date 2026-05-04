use anyhow::Result;
use sqlx::SqlitePool;
use uuid::Uuid;
use crate::models::*;

pub async fn create_player(
    pool: &SqlitePool,
    campaign_id: &str,
    req: &CreateCampaignRequest,
) -> Result<Player> {
    let id = Uuid::new_v4().to_string();
    let hit_die = hit_die_for_class(&req.player_class);
    let con_mod = Player::modifier(req.player_stats.con);
    let max_hp = hit_die + con_mod;
    let current_hp = max_hp.max(1);

    // Apply background ASI to stats
    let str = req.player_stats.str + req.player_background_asi.str.unwrap_or(0);
    let dex = req.player_stats.dex + req.player_background_asi.dex.unwrap_or(0);
    let con = req.player_stats.con + req.player_background_asi.con.unwrap_or(0);
    let int = req.player_stats.int + req.player_background_asi.int.unwrap_or(0);
    let wis = req.player_stats.wis + req.player_background_asi.wis.unwrap_or(0);
    let cha = req.player_stats.cha + req.player_background_asi.cha.unwrap_or(0);

    // Cap stats at 20
    let str = str.min(20);
    let dex = dex.min(20);
    let con = con.min(20);
    let int = int.min(20);
    let wis = wis.min(20);
    let cha = cha.min(20);

    // Recalculate HP with final CON modifier after ASI
    let final_con_mod = Player::modifier(con);
    let max_hp = (hit_die + final_con_mod).max(1);
    let current_hp = max_hp;

    sqlx::query(
        "INSERT INTO players (
            id, campaign_id, name, race, species_subtype, sex,
            class, background, background_feat,
            current_hp, max_hp, str, dex, con, int, wis, cha,
            gold, platinum, silver, copper, backstory
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(campaign_id)
    .bind(&req.player_name)
    .bind(&req.player_race)
    .bind(&req.player_species_subtype)
    .bind(&req.player_sex)
    .bind(&req.player_class)
    .bind(&req.player_background)
    .bind(&req.player_background_feat)
    .bind(current_hp)
    .bind(max_hp)
    .bind(str)
    .bind(dex)
    .bind(con)
    .bind(int)
    .bind(wis)
    .bind(cha)
    .bind(0i64)  // gold — set by equipment choice
    .bind(0i64)  // platinum
    .bind(0i64)  // silver
    .bind(0i64)  // copper
    .bind(&req.player_backstory)
    .execute(pool)
    .await?;

    get_player(pool, &id).await?.ok_or_else(|| anyhow::anyhow!("Player not found after creation"))
}

pub async fn get_player(pool: &SqlitePool, id: &str) -> Result<Option<Player>> {
    Ok(sqlx::query_as::<_, Player>("SELECT * FROM players WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

pub async fn get_player_by_campaign(pool: &SqlitePool, campaign_id: &str) -> Result<Option<Player>> {
    Ok(sqlx::query_as::<_, Player>(
        "SELECT * FROM players WHERE campaign_id = ? LIMIT 1"
    )
    .bind(campaign_id)
    .fetch_optional(pool)
    .await?)
}

pub async fn update_player_hp(pool: &SqlitePool, player_id: &str, new_hp: i64) -> Result<()> {
    sqlx::query(
        "UPDATE players SET current_hp = ?, updated_at = datetime('now') WHERE id = ?"
    )
    .bind(new_hp)
    .bind(player_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_player_location(
    pool: &SqlitePool,
    player_id: &str,
    location_id: &str,
) -> Result<()> {
    sqlx::query(
        "UPDATE players SET current_location_id = ?, updated_at = datetime('now') WHERE id = ?"
    )
    .bind(location_id)
    .bind(player_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_player_gold(pool: &SqlitePool, player_id: &str, new_gold: i64) -> Result<()> {
    sqlx::query(
        "UPDATE players SET gold = ?, updated_at = datetime('now') WHERE id = ?"
    )
    .bind(new_gold)
    .bind(player_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_player_xp(pool: &SqlitePool, player_id: &str, new_xp: i64) -> Result<()> {
    sqlx::query(
        "UPDATE players SET experience = ?, updated_at = datetime('now') WHERE id = ?"
    )
    .bind(new_xp)
    .bind(player_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn set_subclass(pool: &SqlitePool, player_id: &str, subclass: &str) -> Result<()> {
    sqlx::query(
        "UPDATE players SET subclass = ?, updated_at = datetime('now') WHERE id = ?"
    )
    .bind(subclass)
    .bind(player_id)
    .execute(pool)
    .await?;
    Ok(())
}

// ─── Currency ─────────────────────────────────────────────────────────────────

pub async fn update_currency(
    pool: &SqlitePool,
    player_id: &str,
    delta_pp: i64,
    delta_gp: i64,
    delta_sp: i64,
    delta_cp: i64,
) -> Result<(i64, i64, i64, i64)> {
    let player = get_player(pool, player_id).await?
        .ok_or_else(|| anyhow::anyhow!("Player not found"))?;

    normalize_and_save_currency(
        pool, player_id,
        player.platinum + delta_pp,
        player.gold     + delta_gp,
        player.silver   + delta_sp,
        player.copper   + delta_cp,
    ).await
}

pub async fn normalize_and_save_currency(
    pool: &SqlitePool,
    player_id: &str,
    pp: i64,
    gp: i64,
    sp: i64,
    cp: i64,
) -> Result<(i64, i64, i64, i64)> {
    // Convert everything to copper, floor at 0
    let mut total_cp = ((pp * 1000) + (gp * 100) + (sp * 10) + cp).max(0);

    let final_pp = total_cp / 1000; total_cp %= 1000;
    let final_gp = total_cp / 100;  total_cp %= 100;
    let final_sp = total_cp / 10;
    let final_cp = total_cp % 10;

    sqlx::query(
        "UPDATE players SET
            platinum = ?, gold = ?, silver = ?, copper = ?,
            updated_at = datetime('now')
         WHERE id = ?"
    )
    .bind(final_pp)
    .bind(final_gp)
    .bind(final_sp)
    .bind(final_cp)
    .bind(player_id)
    .execute(pool)
    .await?;

    Ok((final_pp, final_gp, final_sp, final_cp))
}

// ─── Background proficiencies ─────────────────────────────────────────────────

pub async fn seed_background_proficiencies(
    pool: &SqlitePool,
    campaign_id: &str,
    player_id: &str,
    skill_1: &str,
    skill_2: &str,
    tool: &str,
) -> Result<()> {
    let proficiencies = vec![
        ("skill", skill_1, "background"),
        ("skill", skill_2, "background"),
        ("tool",  tool,    "background"),
    ];

    for (prof_type, name, source) in proficiencies {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT OR IGNORE INTO proficiencies
             (id, campaign_id, player_id, proficiency_type, name, source)
             VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(campaign_id)
        .bind(player_id)
        .bind(prof_type)
        .bind(name)
        .bind(source)
        .execute(pool)
        .await?;
    }

    Ok(())
}

// ─── Level up ─────────────────────────────────────────────────────────────────

pub async fn level_up_player(
    pool: &SqlitePool,
    player_id: &str,
    player: &Player,
) -> Result<LevelUpResult> {
    let new_level = player.level + 1;
    let con_mod   = Player::modifier(player.con);
    let cha_mod   = Player::modifier(player.cha);
    let hp_gained = hp_gained_on_level(&player.class, con_mod);
    let new_max_hp = player.max_hp + hp_gained;
    let new_prof  = Player::proficiency_for_level(new_level);
 
    // ── Per-class derived stats ───────────────────────────────────────────────
    let (extra_attacks, indomitable_max, action_surge_uses, second_wind_uses, weapon_mastery_count) =
        match player.class.as_str() {
            "Fighter" => (
                fighter_extra_attacks(new_level),
                fighter_indomitable_max(new_level),
                fighter_action_surge_uses(new_level),
                fighter_second_wind_uses(new_level),
                fighter_weapon_mastery_count(new_level),
            ),
            "Barbarian" => (
                barbarian_extra_attacks(new_level),
                0i64, 0i64, 0i64,
                barbarian_weapon_mastery(new_level),
            ),
            "Bard" => {
                let valor_extra = if player.subclass.as_deref() == Some("College of Valor") && new_level >= 6 { 2 } else { 1 };
                (valor_extra, 0i64, 0i64, 0i64, 0i64)
            },
            "Cleric" => (1i64, 0i64, 0i64, 0i64, 0i64),
            "Druid"  => (1i64, 0i64, 0i64, 0i64, 0i64),
            _ => (player.extra_attacks, player.indomitable_max, 0, 2, 0),
        };
 
    // ── Class-specific scalars ────────────────────────────────────────────────
    let rage_uses   = if player.class == "Barbarian" { barbarian_rage_uses(new_level)   } else { 0 };
    let rage_damage = if player.class == "Barbarian" { barbarian_rage_damage(new_level) } else { 0 };
 
    let bardic_die              = if player.class == "Bard" { bard_inspiration_die(new_level) } else { 0 };
    let bardic_inspiration_uses = if player.class == "Bard" { cha_mod.max(1) } else { 0 };
    let bard_prepared_spells_n  = if player.class == "Bard" { bard_prepared_spells(new_level) } else { 0 };
    let bard_cantrips_n         = if player.class == "Bard" { bard_cantrips(new_level) } else { 0 };
 
    let channel_divinity_uses    = if player.class == "Cleric" { cleric_channel_divinity_uses(new_level) } else { 0 };
    let cleric_cantrips_n        = if player.class == "Cleric" { cleric_cantrips(new_level) } else { 0 };
    let cleric_prepared_spells_n = if player.class == "Cleric" { cleric_prepared_spells(new_level) } else { 0 };
 
    let wild_shape_uses_n      = if player.class == "Druid" { druid_wild_shape_uses(new_level) } else { 0 };
    let druid_cantrips_n       = if player.class == "Druid" { druid_cantrips(new_level) } else { 0 };
    let druid_prepared_n       = if player.class == "Druid" { druid_prepared_spells(new_level) } else { 0 };
 
    // ── ASI availability ─────────────────────────────────────────────────────
    let asi_available = match player.class.as_str() {
        "Fighter"   => matches!(new_level, 4 | 6 | 8 | 12 | 14 | 16),
        "Barbarian" => matches!(new_level, 4 | 8 | 12 | 16 | 19),
        "Bard"      => matches!(new_level, 4 | 8 | 12 | 16 | 19),
        "Cleric"    => matches!(new_level, 4 | 8 | 12 | 16 | 19),
        "Druid"     => matches!(new_level, 4 | 8 | 12 | 16 | 19),
        _           => Player::is_asi_level(new_level),
    };
 
    let subclass_choice_required = new_level == 3 && player.subclass.is_none();
 
    // ── Update core player row ────────────────────────────────────────────────
    sqlx::query(
        "UPDATE players SET
            level = ?, max_hp = ?, current_hp = ?,
            proficiency_bonus = ?, extra_attacks = ?,
            indomitable_max = ?, indomitable_uses = ?,
            updated_at = datetime('now')
         WHERE id = ?"
    )
    .bind(new_level).bind(new_max_hp)
    .bind((player.current_hp + hp_gained).min(new_max_hp))
    .bind(new_prof).bind(extra_attacks)
    .bind(indomitable_max).bind(indomitable_max)
    .bind(player_id)
    .execute(pool).await?;
 
    // ── Fighter ───────────────────────────────────────────────────────────────
    if player.class == "Fighter" {
        sqlx::query("UPDATE abilities SET max_uses = ?, current_uses = ? WHERE owner_id = ? AND name = 'Second Wind'")
            .bind(second_wind_uses).bind(second_wind_uses).bind(player_id).execute(pool).await?;
        if action_surge_uses > 0 {
            sqlx::query("UPDATE abilities SET max_uses = ?, current_uses = ? WHERE owner_id = ? AND name = 'Action Surge'")
                .bind(action_surge_uses).bind(action_surge_uses).bind(player_id).execute(pool).await?;
        }
        if player.subclass.as_deref() == Some("Battle Master") {
            let (dice_count, die_size) = battle_master_superiority_dice(new_level);
            sqlx::query("UPDATE superiority_dice SET max_dice = ?, current_dice = ?, die_size = ? WHERE player_id = ? AND pool_name = 'Battle Master'")
                .bind(dice_count).bind(dice_count).bind(die_size).bind(player_id).execute(pool).await?;
        }
        if player.subclass.as_deref() == Some("Psi Warrior") {
            let (dice_count, die_size) = psi_warrior_energy_dice(new_level);
            sqlx::query("UPDATE superiority_dice SET max_dice = ?, current_dice = ?, die_size = ? WHERE player_id = ? AND pool_name = 'Psi Warrior'")
                .bind(dice_count).bind(dice_count).bind(die_size).bind(player_id).execute(pool).await?;
        }
        if player.subclass.as_deref() == Some("Champion") {
            let crit_range = match new_level { 3..=14 => 19, 15..=20 => 18, _ => 20 };
            sqlx::query("UPDATE players SET crit_range_min = ? WHERE id = ?")
                .bind(crit_range).bind(player_id).execute(pool).await?;
        }
    }
 
    // ── Barbarian ─────────────────────────────────────────────────────────────
    if player.class == "Barbarian" {
        let rage_desc = format!(
            "Bonus Action: enter a Rage (lasts 1 min). While raging: Resistance to \
             Bludgeoning/Piercing/Slashing; +{} damage on STR-based attacks and Unarmed \
             Strikes; Advantage on STR checks and saves. Can't concentrate or cast spells. \
             Regain 1 use on Short Rest, all on Long Rest.",
            rage_damage
        );
        sqlx::query("UPDATE abilities SET max_uses = ?, current_uses = ?, description = ? WHERE owner_id = ? AND name = 'Rage'")
            .bind(rage_uses).bind(rage_uses).bind(&rage_desc).bind(player_id).execute(pool).await?;
        if new_level == 20 {
            sqlx::query("UPDATE players SET str = MIN(str + 4, 25), con = MIN(con + 4, 25), updated_at = datetime('now') WHERE id = ?")
                .bind(player_id).execute(pool).await?;
        }
    }
 
    // ── Bard ──────────────────────────────────────────────────────────────────
    if player.class == "Bard" {
        let insp_desc = format!(
            "Bonus Action: grant one Bardic Inspiration die (d{}) to a creature within 60 ft. \
             They can add it to one failed D20 Test within the next hour. \
             Refreshes on Long Rest{}.",
            bardic_die,
            if new_level >= 5 { " (Short Rest from level 5)" } else { "" }
        );
        sqlx::query("UPDATE abilities SET max_uses = ?, current_uses = ?, description = ? WHERE owner_id = ? AND name = 'Bardic Inspiration'")
            .bind(bardic_inspiration_uses).bind(bardic_inspiration_uses).bind(&insp_desc).bind(player_id).execute(pool).await?;
        if new_level == 5 {
            sqlx::query("UPDATE abilities SET refresh_type = 'short_rest' WHERE owner_id = ? AND name = 'Bardic Inspiration'")
                .bind(player_id).execute(pool).await?;
        }
    }
 
    // ── Cleric ────────────────────────────────────────────────────────────────
    if player.class == "Cleric" {
        if matches!(new_level, 2 | 6 | 18) {
            let dice = match new_level { 2..=6 => 1, 7..=12 => 2, 13..=17 => 3, _ => 4 };
            let cd_desc = format!(
                "Use Channel Divinity {} time{} per rest (regain 1 on Short Rest, all on Long Rest). \
                 Effects: Divine Spark (heal or deal {}d8+WIS Necrotic/Radiant to a target within 30 ft; \
                 CON save for half on damage), Turn Undead (WIS save or Frightened+Incapacitated 1 min), \
                 plus your domain feature.",
                channel_divinity_uses,
                if channel_divinity_uses == 1 { "" } else { "s" },
                dice
            );
            sqlx::query("UPDATE abilities SET max_uses = ?, current_uses = ?, description = ? WHERE owner_id = ? AND name = 'Channel Divinity'")
                .bind(channel_divinity_uses).bind(channel_divinity_uses).bind(&cd_desc).bind(player_id).execute(pool).await?;
        }
        if matches!(new_level, 7 | 13) {
            let dice = if new_level >= 13 { 3 } else { 2 };
            let cd_desc = format!(
                "Use Channel Divinity {} times per rest (regain 1 on Short Rest, all on Long Rest). \
                 Effects: Divine Spark (heal or deal {}d8+WIS), Turn Undead (WIS save or Frightened+Incapacitated), \
                 plus your domain feature.",
                channel_divinity_uses, dice
            );
            sqlx::query("UPDATE abilities SET description = ? WHERE owner_id = ? AND name = 'Channel Divinity'")
                .bind(&cd_desc).bind(player_id).execute(pool).await?;
        }
    }
 
    // ── Druid ─────────────────────────────────────────────────────────────────
    if player.class == "Druid" {
        // Update Wild Shape uses at milestone levels
        if matches!(new_level, 2 | 6 | 17) {
            let ws_desc = format!(
                "Bonus Action: shapeshift into a known Beast form (CR {} max{}). \
                 You retain your HP, hit dice, INT/WIS/CHA, class features, languages, feats, \
                 and skill/saving throw proficiencies. Gain Temporary HP equal to your Druid level \
                 when shifting. You can't cast spells while shifted. \
                 Uses: {}. Regain 1 on Short Rest, all on Long Rest.",
                match new_level {
                    2..=3  => "1/4",
                    4..=7  => "1/2",
                    _      => "1",
                },
                if new_level >= 8 { ", Fly Speed available" } else { "" },
                wild_shape_uses_n
            );
            sqlx::query("UPDATE abilities SET max_uses = ?, current_uses = ?, description = ? WHERE owner_id = ? AND name = 'Wild Shape'")
                .bind(wild_shape_uses_n).bind(wild_shape_uses_n).bind(&ws_desc).bind(player_id).execute(pool).await?;
        }
        // Update Wild Shape CR description at levels 4 and 8
        if matches!(new_level, 4 | 8) {
            let ws_desc = format!(
                "Bonus Action: shapeshift into a known Beast form (CR {} max{}). \
                 Retain HP, hit dice, INT/WIS/CHA, class features, languages, feats, proficiencies. \
                 Gain Temporary HP equal to Druid level when shifting. Can't cast spells while shifted. \
                 Uses: {}. Regain 1 on Short Rest, all on Long Rest.",
                if new_level >= 8 { "1" } else { "1/2" },
                if new_level >= 8 { ", Fly Speed available" } else { "" },
                wild_shape_uses_n
            );
            sqlx::query("UPDATE abilities SET description = ? WHERE owner_id = ? AND name = 'Wild Shape'")
                .bind(&ws_desc).bind(player_id).execute(pool).await?;
        }
    }
 
    // ── Feature list ──────────────────────────────────────────────────────────
    let new_features = match player.class.as_str() {
        "Fighter"   => fighter_features_at_level(&player.class, new_level, player.subclass.as_deref()),
        "Barbarian" => barbarian_features_at_level(new_level, player.subclass.as_deref()),
        "Bard"      => bard_features_at_level(new_level, player.subclass.as_deref()),
        "Cleric"    => cleric_features_at_level(new_level, player.subclass.as_deref()),
        "Druid"     => druid_features_at_level(new_level, player.subclass.as_deref()),
        _           => class_features_generic(&player.class, new_level),
    };
 
    // ── Spell slots ───────────────────────────────────────────────────────────
    let spell_slots = match player.class.as_str() {
        "Bard"   => bard_spell_slots(new_level),
        "Cleric" => cleric_spell_slots(new_level),
        "Druid"  => cleric_spell_slots(new_level), // identical full-caster table
        _        => eldritch_knight_spell_slots(player.subclass.as_deref(), new_level),
    };
 
    Ok(LevelUpResult {
        new_level,
        hp_gained,
        new_max_hp,
        new_proficiency_bonus: new_prof,
        asi_available,
        subclass_choice_required,
        new_features,
        spell_slots,
        second_wind_uses,
        weapon_mastery_count,
        extra_attacks,
        indomitable_max,
        action_surge_uses,
        rage_uses,
        rage_damage,
        bardic_die,
        bardic_inspiration_uses,
        bard_prepared_spells: bard_prepared_spells_n,
        bard_cantrips: bard_cantrips_n,
        channel_divinity_uses,
        cleric_cantrips: cleric_cantrips_n,
        cleric_prepared_spells: cleric_prepared_spells_n,
        wild_shape_uses: wild_shape_uses_n,
        druid_cantrips: druid_cantrips_n,
        druid_prepared_spells: druid_prepared_n,
    })
}

pub async fn update_death_saves(
    pool: &SqlitePool,
    player_id: &str,
    successes: i64,
    failures: i64,
    is_stable: bool,
    is_dead: bool,
) -> Result<()> {
    sqlx::query(
        "UPDATE players SET
            death_save_successes = ?, death_save_failures = ?,
            is_stable = ?, is_dead = ?,
            updated_at = datetime('now')
         WHERE id = ?"
    )
    .bind(successes)
    .bind(failures)
    .bind(is_stable)
    .bind(is_dead)
    .bind(player_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_armor_class(pool: &SqlitePool, player_id: &str, new_ac: i64) -> Result<()> {
    sqlx::query(
        "UPDATE players SET armor_class = ?, updated_at = datetime('now') WHERE id = ?"
    )
    .bind(new_ac)
    .bind(player_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn apply_asi(
    pool: &SqlitePool,
    player_id: &str,
    stat1: &str,
    stat2: Option<&str>,
) -> Result<()> {
    let col1 = stat_to_column(stat1)?;
    let is_same = stat2.map(|s| s.eq_ignore_ascii_case(stat1)).unwrap_or(false);

    if let Some(s2) = stat2 {
        if !is_same {
            let col2 = stat_to_column(s2)?;
            let query = format!(
                "UPDATE players SET {col1} = MIN({col1} + 1, 20), {col2} = MIN({col2} + 1, 20), updated_at = datetime('now') WHERE id = ?"
            );
            sqlx::query(&query).bind(player_id).execute(pool).await?;
            return Ok(());
        }
    }

    let query = format!(
        "UPDATE players SET {col1} = MIN({col1} + 2, 20), updated_at = datetime('now') WHERE id = ?"
    );
    sqlx::query(&query).bind(player_id).execute(pool).await?;
    Ok(())
}

pub async fn use_indomitable(
    pool: &SqlitePool,
    player_id: &str,
    player: &Player,
) -> Result<bool> {
    if player.indomitable_uses <= 0 { return Ok(false); }
    sqlx::query(
        "UPDATE players SET indomitable_uses = indomitable_uses - 1,
         updated_at = datetime('now') WHERE id = ?"
    )
    .bind(player_id)
    .execute(pool)
    .await?;
    Ok(true)
}

fn stat_to_column(stat: &str) -> Result<&'static str> {
    match stat.to_lowercase().as_str() {
        "str" | "strength"     => Ok("str"),
        "dex" | "dexterity"   => Ok("dex"),
        "con" | "constitution" => Ok("con"),
        "int" | "intelligence" => Ok("int"),
        "wis" | "wisdom"       => Ok("wis"),
        "cha" | "charisma"     => Ok("cha"),
        _ => Err(anyhow::anyhow!("Unknown stat: {}", stat)),
    }
}

// ─── Barbarian progression ────────────────────────────────────────────────────
 
pub fn barbarian_rage_uses(level: i64) -> i64 {
    // PHB table: 2/2/3/3/3/4/4/4/4/4/4/5/5/5/5/5/6/6/6/6
    match level {
        1..=2   => 2,
        3..=5   => 3,
        6..=11  => 4,
        12..=16 => 5,
        _       => 6,  // 17–20
    }
}
 
pub fn barbarian_rage_damage(level: i64) -> i64 {
    match level {
        1..=8   => 2,
        9..=15  => 3,
        _       => 4,
    }
}
 
pub fn barbarian_weapon_mastery(level: i64) -> i64 {
    match level {
        1..=3  => 2,
        4..=9  => 3,
        _      => 4,
    }
}
 
pub fn barbarian_extra_attacks(level: i64) -> i64 {
    if level >= 5 { 2 } else { 1 }
}

// ─── Bard progression ─────────────────────────────────────────────────────────
 
pub fn bard_inspiration_die(level: i64) -> i64 {
    // d6 (L1-4), d8 (L5-9), d10 (L10-14), d12 (L15-20)
    match level {
        1..=4   => 6,
        5..=9   => 8,
        10..=14 => 10,
        _       => 12,
    }
}
 
pub fn bard_prepared_spells(level: i64) -> i64 {
    let table = [0i64,4,5,6,7,9,10,11,12,14,15,16,16,17,17,18,18,19,20,21,22];
    table.get(level as usize).copied().unwrap_or(22)
}
 
pub fn bard_cantrips(level: i64) -> i64 {
    match level {
        1..=3  => 2,
        4..=9  => 3,
        _      => 4,
    }
}
 
// Full caster spell slots — same progression as Wizard
fn bard_spell_slots(level: i64) -> Option<SpellSlots> {
    Some(match level {
        1  => SpellSlots { level_1: Some(2), level_2: None,    level_3: None,    level_4: None,    level_5: None,    level_6: None,    level_7: None,    level_8: None,    level_9: None },
        2  => SpellSlots { level_1: Some(3), level_2: None,    level_3: None,    level_4: None,    level_5: None,    level_6: None,    level_7: None,    level_8: None,    level_9: None },
        3  => SpellSlots { level_1: Some(4), level_2: Some(2), level_3: None,    level_4: None,    level_5: None,    level_6: None,    level_7: None,    level_8: None,    level_9: None },
        4  => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: None,    level_4: None,    level_5: None,    level_6: None,    level_7: None,    level_8: None,    level_9: None },
        5  => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: Some(2), level_4: None,    level_5: None,    level_6: None,    level_7: None,    level_8: None,    level_9: None },
        6  => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: Some(3), level_4: None,    level_5: None,    level_6: None,    level_7: None,    level_8: None,    level_9: None },
        7  => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: Some(3), level_4: Some(1), level_5: None,    level_6: None,    level_7: None,    level_8: None,    level_9: None },
        8  => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: Some(3), level_4: Some(2), level_5: None,    level_6: None,    level_7: None,    level_8: None,    level_9: None },
        9  => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: Some(3), level_4: Some(3), level_5: Some(1), level_6: None,    level_7: None,    level_8: None,    level_9: None },
        10 => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: Some(3), level_4: Some(3), level_5: Some(2), level_6: None,    level_7: None,    level_8: None,    level_9: None },
        11 => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: Some(3), level_4: Some(3), level_5: Some(2), level_6: Some(1), level_7: None,    level_8: None,    level_9: None },
        12 => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: Some(3), level_4: Some(3), level_5: Some(2), level_6: Some(1), level_7: None,    level_8: None,    level_9: None },
        13 => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: Some(3), level_4: Some(3), level_5: Some(2), level_6: Some(1), level_7: Some(1), level_8: None,    level_9: None },
        14 => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: Some(3), level_4: Some(3), level_5: Some(2), level_6: Some(1), level_7: Some(1), level_8: None,    level_9: None },
        15 => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: Some(3), level_4: Some(3), level_5: Some(2), level_6: Some(1), level_7: Some(1), level_8: Some(1), level_9: None },
        16 => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: Some(3), level_4: Some(3), level_5: Some(2), level_6: Some(1), level_7: Some(1), level_8: Some(1), level_9: None },
        17 => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: Some(3), level_4: Some(3), level_5: Some(2), level_6: Some(1), level_7: Some(1), level_8: Some(1), level_9: Some(1) },
        18 => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: Some(3), level_4: Some(3), level_5: Some(3), level_6: Some(1), level_7: Some(1), level_8: Some(1), level_9: Some(1) },
        19 => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: Some(3), level_4: Some(3), level_5: Some(3), level_6: Some(2), level_7: Some(1), level_8: Some(1), level_9: Some(1) },
        20 => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: Some(3), level_4: Some(3), level_5: Some(3), level_6: Some(2), level_7: Some(2), level_8: Some(1), level_9: Some(1) },
        _  => return None,
    })
}
 
fn bard_features_at_level(level: i64, subclass: Option<&str>) -> Vec<String> {
    let mut features = vec![];
    match level {
        1  => features.extend(["Bardic Inspiration".to_string(), "Spellcasting".to_string()]),
        2  => features.extend(["Expertise".to_string(), "Jack of All Trades".to_string()]),
        3  => features.push("Bard Subclass".to_string()),
        4  => features.push("Ability Score Improvement".to_string()),
        5  => features.push("Font of Inspiration".to_string()),
        6  => features.push("Subclass Feature".to_string()),
        7  => features.push("Countercharm".to_string()),
        8  => features.push("Ability Score Improvement".to_string()),
        9  => features.push("Expertise (2 more skills)".to_string()),
        10 => features.push("Magical Secrets".to_string()),
        12 => features.push("Ability Score Improvement".to_string()),
        14 => features.push("Subclass Feature".to_string()),
        16 => features.push("Ability Score Improvement".to_string()),
        18 => features.push("Superior Inspiration".to_string()),
        19 => features.push("Epic Boon".to_string()),
        20 => features.push("Words of Creation".to_string()),
        _  => {}
    }
    match subclass {
        Some("College of Dance") => match level {
            3  => features.push("Dazzling Footwork".to_string()),
            6  => features.extend(["Inspiring Movement".to_string(), "Tandem Footwork".to_string()]),
            14 => features.push("Leading Evasion".to_string()),
            _  => {}
        },
        Some("College of Glamour") => match level {
            3  => features.extend(["Beguiling Magic".to_string(), "Mantle of Inspiration".to_string()]),
            6  => features.push("Mantle of Majesty".to_string()),
            14 => features.push("Unbreakable Majesty".to_string()),
            _  => {}
        },
        Some("College of Lore") => match level {
            3  => features.extend(["Bonus Proficiencies".to_string(), "Cutting Words".to_string()]),
            6  => features.push("Magical Discoveries".to_string()),
            14 => features.push("Peerless Skill".to_string()),
            _  => {}
        },
        Some("College of Valor") => match level {
            3  => features.extend(["Combat Inspiration".to_string(), "Martial Training".to_string()]),
            6  => features.push("Extra Attack".to_string()),
            14 => features.push("Battle Magic".to_string()),
            _  => {}
        },
        _ => {}
    }
    features
}

// ─── Cleric progression ───────────────────────────────────────────────────────
 
pub fn cleric_channel_divinity_uses(level: i64) -> i64 {
    // PHB table: — (L1), 2 (L2-5), 3 (L6-17), 4 (L18+)
    match level {
        1     => 0,
        2..=5 => 2,
        6..=17 => 3,
        _     => 4,
    }
}
 
pub fn cleric_cantrips(level: i64) -> i64 {
    // PHB table: 3 (L1-3), 4 (L4-9), 5 (L10+)
    match level {
        1..=3  => 3,
        4..=9  => 4,
        _      => 5,
    }
}
 
pub fn cleric_prepared_spells(level: i64) -> i64 {
    // Same table as Bard: 4/5/6/7/9/10/11/12/14/15/16/16/17/17/18/18/19/20/21/22
    let table = [0i64,4,5,6,7,9,10,11,12,14,15,16,16,17,17,18,18,19,20,21,22];
    table.get(level as usize).copied().unwrap_or(22)
}
 
// Cleric is a full caster — identical slot table to Bard
fn cleric_spell_slots(level: i64) -> Option<SpellSlots> {
    Some(match level {
        1  => SpellSlots { level_1: Some(2), level_2: None,    level_3: None,    level_4: None,    level_5: None,    level_6: None,    level_7: None,    level_8: None,    level_9: None },
        2  => SpellSlots { level_1: Some(3), level_2: None,    level_3: None,    level_4: None,    level_5: None,    level_6: None,    level_7: None,    level_8: None,    level_9: None },
        3  => SpellSlots { level_1: Some(4), level_2: Some(2), level_3: None,    level_4: None,    level_5: None,    level_6: None,    level_7: None,    level_8: None,    level_9: None },
        4  => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: None,    level_4: None,    level_5: None,    level_6: None,    level_7: None,    level_8: None,    level_9: None },
        5  => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: Some(2), level_4: None,    level_5: None,    level_6: None,    level_7: None,    level_8: None,    level_9: None },
        6  => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: Some(3), level_4: None,    level_5: None,    level_6: None,    level_7: None,    level_8: None,    level_9: None },
        7  => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: Some(3), level_4: Some(1), level_5: None,    level_6: None,    level_7: None,    level_8: None,    level_9: None },
        8  => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: Some(3), level_4: Some(2), level_5: None,    level_6: None,    level_7: None,    level_8: None,    level_9: None },
        9  => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: Some(3), level_4: Some(3), level_5: Some(1), level_6: None,    level_7: None,    level_8: None,    level_9: None },
        10 => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: Some(3), level_4: Some(3), level_5: Some(2), level_6: None,    level_7: None,    level_8: None,    level_9: None },
        11 => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: Some(3), level_4: Some(3), level_5: Some(2), level_6: Some(1), level_7: None,    level_8: None,    level_9: None },
        12 => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: Some(3), level_4: Some(3), level_5: Some(2), level_6: Some(1), level_7: None,    level_8: None,    level_9: None },
        13 => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: Some(3), level_4: Some(3), level_5: Some(2), level_6: Some(1), level_7: Some(1), level_8: None,    level_9: None },
        14 => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: Some(3), level_4: Some(3), level_5: Some(2), level_6: Some(1), level_7: Some(1), level_8: None,    level_9: None },
        15 => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: Some(3), level_4: Some(3), level_5: Some(2), level_6: Some(1), level_7: Some(1), level_8: Some(1), level_9: None },
        16 => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: Some(3), level_4: Some(3), level_5: Some(2), level_6: Some(1), level_7: Some(1), level_8: Some(1), level_9: None },
        17 => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: Some(3), level_4: Some(3), level_5: Some(2), level_6: Some(1), level_7: Some(1), level_8: Some(1), level_9: Some(1) },
        18 => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: Some(3), level_4: Some(3), level_5: Some(3), level_6: Some(1), level_7: Some(1), level_8: Some(1), level_9: Some(1) },
        19 => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: Some(3), level_4: Some(3), level_5: Some(3), level_6: Some(2), level_7: Some(1), level_8: Some(1), level_9: Some(1) },
        20 => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: Some(3), level_4: Some(3), level_5: Some(3), level_6: Some(2), level_7: Some(2), level_8: Some(1), level_9: Some(1) },
        _  => return None,
    })
}
 
fn cleric_features_at_level(level: i64, subclass: Option<&str>) -> Vec<String> {
    let mut features = vec![];
    match level {
        1  => features.extend(["Spellcasting".to_string(), "Divine Order".to_string()]),
        2  => features.push("Channel Divinity".to_string()),
        3  => features.push("Cleric Subclass".to_string()),
        4  => features.push("Ability Score Improvement".to_string()),
        5  => features.push("Sear Undead".to_string()),
        6  => features.push("Subclass Feature".to_string()),
        7  => features.push("Blessed Strikes".to_string()),
        8  => features.push("Ability Score Improvement".to_string()),
        10 => features.push("Divine Intervention".to_string()),
        12 => features.push("Ability Score Improvement".to_string()),
        14 => features.push("Improved Blessed Strikes".to_string()),
        16 => features.push("Ability Score Improvement".to_string()),
        17 => features.push("Subclass Feature".to_string()),
        19 => features.push("Epic Boon".to_string()),
        20 => features.push("Greater Divine Intervention".to_string()),
        _  => {}
    }
    match subclass {
        Some("Life Domain") => match level {
            3  => features.extend(["Disciple of Life".to_string(), "Life Domain Spells".to_string(), "Preserve Life".to_string()]),
            6  => features.push("Blessed Healer".to_string()),
            17 => features.push("Supreme Healing".to_string()),
            _  => {}
        },
        Some("Light Domain") => match level {
            3  => features.extend(["Light Domain Spells".to_string(), "Radiance of the Dawn".to_string(), "Warding Flare".to_string()]),
            6  => features.push("Improved Warding Flare".to_string()),
            17 => features.push("Corona of Light".to_string()),
            _  => {}
        },
        Some("Trickery Domain") => match level {
            3  => features.extend(["Blessing of the Trickster".to_string(), "Trickery Domain Spells".to_string(), "Invoke Duplicity".to_string()]),
            6  => features.push("Trickster's Transposition".to_string()),
            17 => features.push("Improved Duplicity".to_string()),
            _  => {}
        },
        Some("War Domain") => match level {
            3  => features.extend(["Guided Strike".to_string(), "War Domain Spells".to_string(), "War Priest".to_string()]),
            6  => features.push("War God's Blessing".to_string()),
            17 => features.push("Avatar of Battle".to_string()),
            _  => {}
        },
        _ => {}
    }
    features
}

// ─── Druid progression ────────────────────────────────────────────────────────
 
pub fn druid_wild_shape_uses(level: i64) -> i64 {
    // PHB Wild Shape column: — (L1), 2 (L2-5), 3 (L6-16), 4 (L17+)
    match level {
        1      => 0,
        2..=5  => 2,
        6..=16 => 3,
        _      => 4,
    }
}
 
pub fn druid_cantrips(level: i64) -> i64 {
    // PHB cantrips column: 2 (L1-3), 3 (L4-9), 4 (L10+)
    match level {
        1..=3  => 2,
        4..=9  => 3,
        _      => 4,
    }
}
 
pub fn druid_prepared_spells(level: i64) -> i64 {
    // Same table as Bard and Cleric
    let table = [0i64,4,5,6,7,9,10,11,12,14,15,16,16,17,17,18,18,19,20,21,22];
    table.get(level as usize).copied().unwrap_or(22)
}
 
fn druid_features_at_level(level: i64, subclass: Option<&str>) -> Vec<String> {
    let mut features = vec![];
    match level {
        1  => features.extend(["Spellcasting".to_string(), "Druidic".to_string(), "Primal Order".to_string()]),
        2  => features.extend(["Wild Shape".to_string(), "Wild Companion".to_string()]),
        3  => features.push("Druid Subclass".to_string()),
        4  => features.extend(["Ability Score Improvement".to_string(), "Wild Shape (CR 1/2, 6 forms)".to_string()]),
        5  => features.push("Wild Resurgence".to_string()),
        6  => features.extend(["Subclass Feature".to_string(), "Wild Shape (3 uses)".to_string()]),
        7  => features.push("Elemental Fury".to_string()),
        8  => features.extend(["Ability Score Improvement".to_string(), "Wild Shape (CR 1, 8 forms, Fly Speed)".to_string()]),
        10 => features.push("Subclass Feature".to_string()),
        12 => features.push("Ability Score Improvement".to_string()),
        14 => features.push("Subclass Feature".to_string()),
        15 => features.push("Improved Elemental Fury".to_string()),
        16 => features.push("Ability Score Improvement".to_string()),
        17 => features.push("Wild Shape (4 uses)".to_string()),
        18 => features.push("Beast Spells".to_string()),
        19 => features.push("Epic Boon".to_string()),
        20 => features.push("Archdruid".to_string()),
        _  => {}
    }
    match subclass {
        Some("Circle of the Land") => match level {
            3  => features.extend(["Circle of the Land Spells".to_string(), "Land's Aid".to_string()]),
            6  => features.push("Natural Recovery".to_string()),
            10 => features.push("Nature's Ward".to_string()),
            14 => features.push("Nature's Sanctuary".to_string()),
            _  => {}
        },
        Some("Circle of the Moon") => match level {
            3  => features.extend(["Circle Forms".to_string(), "Circle of the Moon Spells".to_string()]),
            6  => features.push("Improved Circle Forms".to_string()),
            10 => features.push("Moonlight Step".to_string()),
            14 => features.push("Lunar Form".to_string()),
            _  => {}
        },
        Some("Circle of the Sea") => match level {
            3  => features.extend(["Circle of the Sea Spells".to_string(), "Wrath of the Sea".to_string()]),
            6  => features.push("Aquatic Affinity".to_string()),
            10 => features.push("Stormborn".to_string()),
            14 => features.push("Oceanic Gift".to_string()),
            _  => {}
        },
        Some("Circle of the Stars") => match level {
            3  => features.extend(["Star Map".to_string(), "Starry Form".to_string()]),
            6  => features.push("Cosmic Omen".to_string()),
            10 => features.push("Twinkling Constellations".to_string()),
            14 => features.push("Full of Stars".to_string()),
            _  => {}
        },
        _ => {}
    }
    features
}

// ─── Feature tables ───────────────────────────────────────────────────────────

fn fighter_features_at_level(class: &str, level: i64, subclass: Option<&str>) -> Vec<String> {
    if class != "Fighter" {
        return class_features_generic(class, level);
    }
    let mut features = vec![];
    match level {
        1  => features.extend(["Fighting Style".to_string(), "Second Wind".to_string(), "Weapon Mastery".to_string()]),
        2  => features.extend(["Action Surge".to_string(), "Tactical Mind".to_string()]),
        3  => features.push("Fighter Subclass".to_string()),
        4  => features.push("Ability Score Improvement".to_string()),
        5  => features.extend(["Extra Attack".to_string(), "Tactical Shift".to_string()]),
        6  => features.push("Ability Score Improvement".to_string()),
        7  => features.push("Subclass Feature".to_string()),
        8  => features.push("Ability Score Improvement".to_string()),
        9  => features.extend(["Indomitable".to_string(), "Tactical Master".to_string()]),
        10 => features.push("Subclass Feature".to_string()),
        11 => features.push("Two Extra Attacks".to_string()),
        12 => features.push("Ability Score Improvement".to_string()),
        13 => features.extend(["Indomitable (two uses)".to_string(), "Studied Attacks".to_string()]),
        14 => features.push("Ability Score Improvement".to_string()),
        15 => features.push("Subclass Feature".to_string()),
        16 => features.push("Ability Score Improvement".to_string()),
        17 => features.extend(["Action Surge (two uses)".to_string(), "Indomitable (three uses)".to_string()]),
        18 => features.push("Subclass Feature".to_string()),
        19 => features.push("Epic Boon".to_string()),
        20 => features.push("Three Extra Attacks".to_string()),
        _  => {}
    }
    match subclass {
        Some("Champion") => match level {
            3  => features.extend(["Improved Critical".to_string(), "Remarkable Athlete".to_string()]),
            7  => features.push("Additional Fighting Style".to_string()),
            10 => features.push("Heroic Warrior".to_string()),
            15 => features.push("Superior Critical".to_string()),
            18 => features.push("Survivor".to_string()),
            _  => {}
        },
        Some("Battle Master") => match level {
            3  => features.extend(["Combat Superiority".to_string(), "Student of War".to_string()]),
            7  => features.push("Know Your Enemy".to_string()),
            10 => features.push("Improved Combat Superiority (d10)".to_string()),
            15 => features.extend(["Relentless".to_string(), "Improved Combat Superiority (d12)".to_string()]),
            18 => features.push("Ultimate Combat Superiority".to_string()),
            _  => {}
        },
        Some("Psi Warrior") => match level {
            3  => features.push("Psionic Power".to_string()),
            7  => features.push("Telekinetic Adept".to_string()),
            10 => features.push("Guarded Mind".to_string()),
            15 => features.push("Bulwark of Force".to_string()),
            18 => features.push("Telekinetic Master".to_string()),
            _  => {}
        },
        _ => {}
    }
    features
}

fn barbarian_features_at_level(level: i64, subclass: Option<&str>) -> Vec<String> {
    let mut features = vec![];
    match level {
        1  => features.extend(["Rage".to_string(), "Unarmored Defense".to_string(), "Weapon Mastery".to_string()]),
        2  => features.extend(["Danger Sense".to_string(), "Reckless Attack".to_string()]),
        3  => features.extend(["Barbarian Subclass".to_string(), "Primal Knowledge".to_string()]),
        4  => features.push("Ability Score Improvement".to_string()),
        5  => features.extend(["Extra Attack".to_string(), "Fast Movement".to_string()]),
        6  => features.push("Subclass Feature".to_string()),
        7  => features.extend(["Feral Instinct".to_string(), "Instinctive Pounce".to_string()]),
        8  => features.push("Ability Score Improvement".to_string()),
        9  => features.push("Brutal Strike".to_string()),
        10 => features.push("Subclass Feature".to_string()),
        11 => features.push("Relentless Rage".to_string()),
        12 => features.push("Ability Score Improvement".to_string()),
        13 => features.push("Improved Brutal Strike".to_string()),
        14 => features.push("Subclass Feature".to_string()),
        15 => features.push("Persistent Rage".to_string()),
        16 => features.push("Ability Score Improvement".to_string()),
        17 => features.push("Improved Brutal Strike (upgrade)".to_string()),
        18 => features.push("Indomitable Might".to_string()),
        19 => features.push("Epic Boon".to_string()),
        20 => features.push("Primal Champion".to_string()),
        _  => {}
    }
    match subclass {
        Some("Path of the Berserker") => match level {
            3  => features.push("Frenzy".to_string()),
            6  => features.push("Mindless Rage".to_string()),
            10 => features.push("Retaliation".to_string()),
            14 => features.push("Intimidating Presence".to_string()),
            _  => {}
        },
        Some("Path of the Wild Heart") => match level {
            3  => features.extend(["Animal Speaker".to_string(), "Rage of the Wilds".to_string()]),
            6  => features.push("Aspect of the Wilds".to_string()),
            10 => features.push("Nature Speaker".to_string()),
            14 => features.push("Power of the Wilds".to_string()),
            _  => {}
        },
        Some("Path of the World Tree") => match level {
            3  => features.push("Vitality of the Tree".to_string()),
            6  => features.push("Branches of the Tree".to_string()),
            10 => features.push("Battering Roots".to_string()),
            14 => features.push("Travel along the Tree".to_string()),
            _  => {}
        },
        Some("Path of the Zealot") => match level {
            3  => features.extend(["Divine Fury".to_string(), "Warrior of the Gods".to_string()]),
            6  => features.push("Fanatical Focus".to_string()),
            10 => features.push("Zealous Presence".to_string()),
            14 => features.push("Rage of the Gods".to_string()),
            _  => {}
        },
        _ => {}
    }
    features
}

fn class_features_generic(class: &str, level: i64) -> Vec<String> {
    let mut features = vec![];
    match class {
        "Barbarian" => match level {
            1  => features.extend(["Rage".to_string(), "Unarmored Defense".to_string(), "Weapon Mastery".to_string()]),
            2  => features.extend(["Danger Sense".to_string(), "Reckless Attack".to_string()]),
            3  => features.extend(["Barbarian Subclass".to_string(), "Primal Knowledge".to_string()]),
            4  => features.push("Ability Score Improvement".to_string()),
            5  => features.extend(["Extra Attack".to_string(), "Fast Movement".to_string()]),
            6  => features.push("Subclass Feature".to_string()),
            7  => features.extend(["Feral Instinct".to_string(), "Instinctive Pounce".to_string()]),
            8  => features.push("Ability Score Improvement".to_string()),
            9  => features.push("Brutal Strike".to_string()),
            10 => features.push("Subclass Feature".to_string()),
            11 => features.push("Relentless Rage".to_string()),
            12 => features.push("Ability Score Improvement".to_string()),
            13 => features.push("Improved Brutal Strike".to_string()),
            14 => features.push("Subclass Feature".to_string()),
            15 => features.push("Persistent Rage".to_string()),
            16 => features.push("Ability Score Improvement".to_string()),
            17 => features.push("Improved Brutal Strike (upgrade)".to_string()),
            18 => features.push("Indomitable Might".to_string()),
            19 => features.push("Epic Boon".to_string()),
            20 => features.push("Primal Champion".to_string()),
            _  => {}
        },
        "Rogue" => match level {
            2 => features.push("Cunning Action".to_string()),
            3 => features.push("Roguish Archetype".to_string()),
            5 => features.push("Uncanny Dodge".to_string()),
            7 => features.push("Evasion".to_string()),
            _ => {}
        },
        "Wizard" => match level {
            2 => features.push("Arcane Tradition".to_string()),
            _ => {}
        },
        "Cleric" => match level {
            1  => features.extend(["Spellcasting".to_string(), "Divine Order".to_string()]),
            2  => features.push("Channel Divinity".to_string()),
            3  => features.push("Cleric Subclass".to_string()),
            4  => features.push("Ability Score Improvement".to_string()),
            5  => features.push("Sear Undead".to_string()),
            6  => features.push("Subclass Feature".to_string()),
            7  => features.push("Blessed Strikes".to_string()),
            8  => features.push("Ability Score Improvement".to_string()),
            10 => features.push("Divine Intervention".to_string()),
            12 => features.push("Ability Score Improvement".to_string()),
            14 => features.push("Improved Blessed Strikes".to_string()),
            16 => features.push("Ability Score Improvement".to_string()),
            17 => features.push("Subclass Feature".to_string()),
            19 => features.push("Epic Boon".to_string()),
            20 => features.push("Greater Divine Intervention".to_string()),
            _  => {}
        },
        "Druid" => match level {
            1  => features.extend(["Spellcasting".to_string(), "Druidic".to_string(), "Primal Order".to_string()]),
            2  => features.extend(["Wild Shape".to_string(), "Wild Companion".to_string()]),
            3  => features.push("Druid Subclass".to_string()),
            4  => features.extend(["Ability Score Improvement".to_string(), "Wild Shape (CR 1/2, 6 forms)".to_string()]),
            5  => features.push("Wild Resurgence".to_string()),
            6  => features.extend(["Subclass Feature".to_string(), "Wild Shape (3 uses)".to_string()]),
            7  => features.push("Elemental Fury".to_string()),
            8  => features.extend(["Ability Score Improvement".to_string(), "Wild Shape (CR 1, 8 forms, Fly Speed)".to_string()]),
            10 => features.push("Subclass Feature".to_string()),
            12 => features.push("Ability Score Improvement".to_string()),
            14 => features.push("Subclass Feature".to_string()),
            15 => features.push("Improved Elemental Fury".to_string()),
            16 => features.push("Ability Score Improvement".to_string()),
            17 => features.push("Wild Shape (4 uses)".to_string()),
            18 => features.push("Beast Spells".to_string()),
            19 => features.push("Epic Boon".to_string()),
            20 => features.push("Archdruid".to_string()),
            _  => {}
        },
        _ => {}
    }
    features
}

fn eldritch_knight_spell_slots(subclass: Option<&str>, level: i64) -> Option<SpellSlots> {
    if subclass != Some("Eldritch Knight") { return None; }
    Some(match level {
        3..=4   => SpellSlots { level_1: Some(2), level_2: None, level_3: None, level_4: None, level_5: None, level_6: None, level_7: None, level_8: None, level_9: None },
        5..=6   => SpellSlots { level_1: Some(3), level_2: None, level_3: None, level_4: None, level_5: None, level_6: None, level_7: None, level_8: None, level_9: None },
        7..=9   => SpellSlots { level_1: Some(4), level_2: Some(2), level_3: None, level_4: None, level_5: None, level_6: None, level_7: None, level_8: None, level_9: None },
        10..=12 => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: None, level_4: None, level_5: None, level_6: None, level_7: None, level_8: None, level_9: None },
        13..=15 => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: Some(2), level_4: None, level_5: None, level_6: None, level_7: None, level_8: None, level_9: None },
        16..=18 => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: Some(3), level_4: None, level_5: None, level_6: None, level_7: None, level_8: None, level_9: None },
        19..=20 => SpellSlots { level_1: Some(4), level_2: Some(3), level_3: Some(3), level_4: Some(1), level_5: None, level_6: None, level_7: None, level_8: None, level_9: None },
        _ => return None,
    })
}

pub fn full_caster_slot_table(level: i64) -> [i64; 9] {
    match level {
        1  => [2,0,0,0,0,0,0,0,0],
        2  => [3,0,0,0,0,0,0,0,0],
        3  => [4,2,0,0,0,0,0,0,0],
        4  => [4,3,0,0,0,0,0,0,0],
        5  => [4,3,2,0,0,0,0,0,0],
        6  => [4,3,3,0,0,0,0,0,0],
        7  => [4,3,3,1,0,0,0,0,0],
        8  => [4,3,3,2,0,0,0,0,0],
        9  => [4,3,3,3,1,0,0,0,0],
        10 => [4,3,3,3,2,0,0,0,0],
        11 => [4,3,3,3,2,1,0,0,0],
        12 => [4,3,3,3,2,1,0,0,0],
        13 => [4,3,3,3,2,1,1,0,0],
        14 => [4,3,3,3,2,1,1,0,0],
        15 => [4,3,3,3,2,1,1,1,0],
        16 => [4,3,3,3,2,1,1,1,0],
        17 => [4,3,3,3,2,1,1,1,1],
        18 => [4,3,3,3,3,1,1,1,1],
        19 => [4,3,3,3,3,2,1,1,1],
        20 => [4,3,3,3,3,2,2,1,1],
        _  => [0,0,0,0,0,0,0,0,0],
    }
}

ub async fn seed_full_caster_spell_slots(
    pool: &SqlitePool,
    campaign_id: &str,
    player_id: &str,
    class_level: i64,
) -> Result<()> {
    let slots = full_caster_slot_table(class_level);
 
    for (i, &max) in slots.iter().enumerate() {
        let level = (i + 1) as i64;
 
        if max == 0 {
            // Remove any row that no longer has slots (shouldn't happen in normal play)
            sqlx::query!(
                "DELETE FROM spell_slots WHERE player_id = ? AND slot_level = ?",
                player_id, level
            )
            .execute(pool)
            .await?;
            continue;
        }
 
        let existing = sqlx::query!(
            "SELECT id, current_slots FROM spell_slots WHERE player_id = ? AND slot_level = ?",
            player_id, level
        )
        .fetch_optional(pool)
        .await?;
 
        if let Some(row) = existing {
            // Raise max without touching current (player may have expended some)
            let new_current = row.current_slots.min(max);
            sqlx::query!(
                "UPDATE spell_slots SET max_slots = ?, current_slots = ?, updated_at = datetime('now') WHERE id = ?",
                max, new_current, row.id
            )
            .execute(pool)
            .await?;
        } else {
            let id = Uuid::new_v4().to_string();
            sqlx::query!(
                "INSERT INTO spell_slots
                 (id, campaign_id, player_id, slot_level, current_slots, max_slots)
                 VALUES (?, ?, ?, ?, ?, ?)",
                id, campaign_id, player_id, level, max, max
            )
            .execute(pool)
            .await?;
        }
    }
 
    Ok(())
}
