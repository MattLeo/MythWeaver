// ─── items.rs ─────────────────────────────────────────────────────────────────
// Items database operations

use anyhow::Result;
use sqlx::SqlitePool;
use uuid::Uuid;
use crate::models::*;

pub async fn create_item(pool: &SqlitePool, campaign_id: &str, item: &serde_json::Value) -> Result<Item> {
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO items (
            id, campaign_id, owner_type, owner_id, name, description, item_type,
            quantity, damage_die, damage_type, weapon_range, base_ac, armor_type,
            stealth_disadvantage, rarity, notes
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(campaign_id)
    .bind(item["owner_type"].as_str())
    .bind(item["owner_id"].as_str())
    .bind(item["name"].as_str().unwrap_or("Unknown Item"))
    .bind(item["description"].as_str().unwrap_or(""))
    .bind(item["item_type"].as_str().unwrap_or("wondrous"))
    .bind(item["quantity"].as_i64().unwrap_or(1))
    .bind(item["damage_die"].as_str())
    .bind(item["damage_type"].as_str())
    .bind(item["weapon_range"].as_str())
    .bind(item["base_ac"].as_i64())
    .bind(item["armor_type"].as_str())
    .bind(item["stealth_disadvantage"].as_bool().unwrap_or(false))
    .bind(item["rarity"].as_str().unwrap_or("common"))
    .bind(item["notes"].as_str())
    .execute(pool)
    .await?;

    // Add effects if present
    if let Some(effects) = item["effects"].as_array() {
        for effect in effects {
            let effect_id = Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT INTO item_effects (id, item_id, effect_type, value, target)
                 VALUES (?, ?, ?, ?, ?)"
            )
            .bind(&effect_id)
            .bind(&id)
            .bind(effect["effect_type"].as_str().unwrap_or(""))
            .bind(effect["value"].as_i64())
            .bind(effect["target"].as_str())
            .execute(pool)
            .await?;
        }
    }

    get_item(pool, &id).await?.ok_or_else(|| anyhow::anyhow!("Item not found after creation"))
}

pub async fn get_item(pool: &SqlitePool, id: &str) -> Result<Option<Item>> {
    Ok(sqlx::query_as::<_, Item>("SELECT * FROM items WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

pub async fn get_player_items(pool: &SqlitePool, player_id: &str) -> Result<Vec<Item>> {
    Ok(sqlx::query_as::<_, Item>(
        "SELECT * FROM items WHERE owner_type = 'player' AND owner_id = ?"
    )
    .bind(player_id)
    .fetch_all(pool)
    .await?)
}

pub async fn get_item_effects(pool: &SqlitePool, item_id: &str) -> Result<Vec<ItemEffect>> {
    Ok(sqlx::query_as::<_, ItemEffect>(
        "SELECT * FROM item_effects WHERE item_id = ?"
    )
    .bind(item_id)
    .fetch_all(pool)
    .await?)
}

pub async fn equip_item(
    pool: &SqlitePool,
    item_id: &str,
    slot: &str,
    player_id: &str,
) -> Result<()> {
    // Unequip anything currently in that slot
    sqlx::query(
        "UPDATE items SET is_equipped = 0, slot = NULL
         WHERE owner_type = 'player' AND owner_id = ? AND slot = ?"
    )
    .bind(player_id)
    .bind(slot)
    .execute(pool)
    .await?;

    sqlx::query(
        "UPDATE items SET is_equipped = 1, slot = ? WHERE id = ?"
    )
    .bind(slot)
    .bind(item_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn unequip_item(pool: &SqlitePool, item_id: &str) -> Result<()> {
    sqlx::query(
        "UPDATE items SET is_equipped = 0, slot = NULL WHERE id = ?"
    )
    .bind(item_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn give_item(
    pool: &SqlitePool,
    item_id: &str,
    owner_type: &str,
    owner_id: &str,
) -> Result<()> {
    sqlx::query(
        "UPDATE items SET owner_type = ?, owner_id = ? WHERE id = ?"
    )
    .bind(owner_type)
    .bind(owner_id)
    .bind(item_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn remove_item(pool: &SqlitePool, item_id: &str, quantity: i64) -> Result<()> {
    let item = get_item(pool, item_id).await?;
    if let Some(item) = item {
        let new_qty = item.quantity - quantity;
        if new_qty <= 0 {
            sqlx::query("DELETE FROM items WHERE id = ?")
                .bind(item_id)
                .execute(pool)
                .await?;
        } else {
            sqlx::query("UPDATE items SET quantity = ? WHERE id = ?")
                .bind(new_qty)
                .bind(item_id)
                .execute(pool)
                .await?;
        }
    }
    Ok(())
}

/// Recalculate player AC based on equipped armor + DEX + magic bonuses
pub async fn recalculate_ac(pool: &SqlitePool, player_id: &str) -> Result<i64> {
    let player = sqlx::query_as::<_, Player>("SELECT * FROM players WHERE id = ?")
        .bind(player_id)
        .fetch_one(pool)
        .await?;

    let dex_mod = Player::modifier(player.dex);

    // Find equipped armor
    let armor = sqlx::query_as::<_, Item>(
        "SELECT * FROM items WHERE owner_type = 'player' AND owner_id = ?
         AND is_equipped = 1 AND item_type = 'armor'"
    )
    .bind(player_id)
    .fetch_optional(pool)
    .await?;

    // Find equipped shield
    let shield = sqlx::query_as::<_, Item>(
        "SELECT * FROM items WHERE owner_type = 'player' AND owner_id = ?
         AND is_equipped = 1 AND item_type = 'shield'"
    )
    .bind(player_id)
    .fetch_optional(pool)
    .await?;

    let base_ac = if let Some(armor) = &armor {
        let base = armor.base_ac.unwrap_or(10);
        match armor.armor_type.as_deref() {
            Some("light") => base + dex_mod,
            Some("medium") => base + dex_mod.min(2),
            Some("heavy") => base,
            _ => 10 + dex_mod,
        }
    } else {
        10 + dex_mod
    };

    let shield_bonus = if shield.is_some() { 2 } else { 0 };

    // Sum up magic AC bonuses from all equipped items
    let magic_bonus: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(ie.value), 0)
         FROM item_effects ie
         JOIN items i ON i.id = ie.item_id
         WHERE i.owner_type = 'player'
           AND i.owner_id = ?
           AND i.is_equipped = 1
           AND ie.effect_type = 'ac_bonus'"
    )
    .bind(player_id)
    .fetch_one(pool)
    .await?;

    let total_ac = base_ac + shield_bonus + magic_bonus;

    sqlx::query("UPDATE players SET armor_class = ? WHERE id = ?")
        .bind(total_ac)
        .bind(player_id)
        .execute(pool)
        .await?;

    Ok(total_ac)
}