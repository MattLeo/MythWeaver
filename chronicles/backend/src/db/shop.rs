use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::SqlitePool;
use uuid::Uuid;

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ShopSession {
    pub id: String,
    pub campaign_id: String,
    pub merchant_npc_id: Option<String>,
    pub shop_name: String,
    pub shop_type: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ShopItem {
    pub id: String,
    pub shop_session_id: String,
    pub campaign_id: String,
    pub name: String,
    pub description: String,
    pub item_type: String,
    pub damage_die: Option<String>,
    pub damage_type: Option<String>,
    pub weapon_range: Option<String>,
    pub weapon_type: Option<String>,
    pub base_ac: Option<i64>,
    pub armor_type: Option<String>,
    pub price_pp: i64,
    pub price_gp: i64,
    pub price_sp: i64,
    pub price_cp: i64,
    pub quantity: i64,
    pub quantity_sold: i64,
    pub rarity: String,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ShopTransaction {
    pub id: String,
    pub shop_session_id: String,
    pub campaign_id: String,
    pub transaction_type: String,
    pub item_name: String,
    pub shop_item_id: Option<String>,
    pub player_item_id: Option<String>,
    pub quantity: i64,
    pub price_pp: i64,
    pub price_gp: i64,
    pub price_sp: i64,
    pub price_cp: i64,
    pub created_at: String,
}

// ─── Open shop ────────────────────────────────────────────────────────────────

pub async fn open_shop(
    pool: &SqlitePool,
    campaign_id: &str,
    shop_name: &str,
    shop_type: &str,
    merchant_npc_id: Option<&str>,
    items: Vec<Value>,
) -> Result<Value> {
    // Close any existing open shop first
    sqlx::query(
        "UPDATE shop_sessions SET status = 'closed', updated_at = datetime('now')
         WHERE campaign_id = ? AND status = 'open'"
    )
    .bind(campaign_id)
    .execute(pool).await?;

    let session_id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO shop_sessions (id, campaign_id, merchant_npc_id, shop_name, shop_type)
         VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&session_id)
    .bind(campaign_id)
    .bind(merchant_npc_id)
    .bind(shop_name)
    .bind(shop_type)
    .execute(pool).await?;

    let mut seeded_items = vec![];

    for item in &items {
        let item_id = Uuid::new_v4().to_string();
        let name = item["name"].as_str().unwrap_or("Unknown Item");
        let description = item["description"].as_str().unwrap_or("");
        let item_type = item["item_type"].as_str().unwrap_or("wondrous");
        let damage_die = item["damage_die"].as_str();
        let damage_type = item["damage_type"].as_str();
        let weapon_range = item["weapon_range"].as_str();
        let weapon_type = item["weapon_type"].as_str();
        let base_ac = item["base_ac"].as_i64();
        let armor_type = item["armor_type"].as_str();
        let price_pp = item["price_pp"].as_i64().unwrap_or(0);
        let price_gp = item["price_gp"].as_i64().unwrap_or(0);
        let price_sp = item["price_sp"].as_i64().unwrap_or(0);
        let price_cp = item["price_cp"].as_i64().unwrap_or(0);
        let quantity = item["quantity"].as_i64().unwrap_or(1);
        let rarity = item["rarity"].as_str().unwrap_or("common");
        let notes = item["notes"].as_str();

        sqlx::query(
            "INSERT INTO shop_items (
                id, shop_session_id, campaign_id, name, description, item_type,
                damage_die, damage_type, weapon_range, weapon_type,
                base_ac, armor_type, price_pp, price_gp, price_sp, price_cp,
                quantity, rarity, notes
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&item_id)
        .bind(&session_id)
        .bind(campaign_id)
        .bind(name)
        .bind(description)
        .bind(item_type)
        .bind(damage_die)
        .bind(damage_type)
        .bind(weapon_range)
        .bind(weapon_type)
        .bind(base_ac)
        .bind(armor_type)
        .bind(price_pp)
        .bind(price_gp)
        .bind(price_sp)
        .bind(price_cp)
        .bind(quantity)
        .bind(rarity)
        .bind(notes)
        .execute(pool).await?;

        seeded_items.push(json!({
            "id": item_id,
            "name": name,
            "item_type": item_type,
            "price_gp": price_gp,
            "price_sp": price_sp,
            "quantity": quantity,
        }));
    }

    Ok(json!({
        "session_id": session_id,
        "shop_name": shop_name,
        "shop_type": shop_type,
        "item_count": seeded_items.len(),
        "items": seeded_items,
    }))
}

// ─── Get shop state ───────────────────────────────────────────────────────────

pub async fn get_active_shop(
    pool: &SqlitePool,
    campaign_id: &str,
) -> Result<Option<Value>> {
    let session = sqlx::query_as::<_, ShopSession>(
        "SELECT * FROM shop_sessions WHERE campaign_id = ? AND status = 'open' LIMIT 1"
    )
    .bind(campaign_id)
    .fetch_optional(pool).await?;

    let session = match session {
        Some(s) => s,
        None => return Ok(None),
    };

    let items = sqlx::query_as::<_, ShopItem>(
        "SELECT * FROM shop_items WHERE shop_session_id = ? ORDER BY item_type, name"
    )
    .bind(&session.id)
    .fetch_all(pool).await?;

    Ok(Some(json!({
        "session": session,
        "items": items,
    })))
}

// ─── Buy item ─────────────────────────────────────────────────────────────────

pub async fn buy_item(
    pool: &SqlitePool,
    campaign_id: &str,
    player: &crate::models::Player,
    shop_item_id: &str,
    quantity: i64,
) -> Result<Value> {
    let item = sqlx::query_as::<_, ShopItem>(
        "SELECT * FROM shop_items WHERE id = ? AND shop_session_id IN (
            SELECT id FROM shop_sessions WHERE campaign_id = ? AND status = 'open'
        )"
    )
    .bind(shop_item_id)
    .bind(campaign_id)
    .fetch_optional(pool).await?;

    let item = match item {
        Some(i) => i,
        None => return Ok(json!({"error": "Item not found in active shop"})),
    };

    let available = item.quantity - item.quantity_sold;
    if available < quantity {
        return Ok(json!({
            "error": format!("Only {} available", available),
            "available": available
        }));
    }

    // Calculate total cost
    let total_pp = item.price_pp * quantity;
    let total_gp = item.price_gp * quantity;
    let total_sp = item.price_sp * quantity;
    let total_cp = item.price_cp * quantity;

    // Convert everything to copper for comparison
    let total_cost_cp = (total_pp * 1000) + (total_gp * 100) + (total_sp * 10) + total_cp;
    let player_total_cp = (player.platinum * 1000) + (player.gold * 100) + (player.silver * 10) + player.copper;

    if player_total_cp < total_cost_cp {
        return Ok(json!({
            "error": "Insufficient funds",
            "cost_gp": total_gp,
            "cost_sp": total_sp,
            "player_gold": player.gold,
        }));
    }

    // Deduct currency
    let (new_pp, new_gp, new_sp, new_cp) = crate::db::player::update_currency(
        pool, &player.id, -total_pp, -total_gp, -total_sp, -total_cp
    ).await?;

    // Create the item in the items table and give to player
    let item_data = json!({
        "name": item.name,
        "description": item.description,
        "item_type": item.item_type,
        "damage_die": item.damage_die,
        "damage_type": item.damage_type,
        "weapon_range": item.weapon_range,
        "weapon_type": item.weapon_type,
        "base_ac": item.base_ac,
        "armor_type": item.armor_type,
        "rarity": item.rarity,
        "quantity": quantity,
        "notes": item.notes,
    });

    let created_item = crate::db::items::create_item(pool, campaign_id, &item_data).await?;
    crate::db::items::give_item(pool, &created_item.id, "player", &player.id).await?;

    // Update shop stock
    sqlx::query(
        "UPDATE shop_items SET quantity_sold = quantity_sold + ? WHERE id = ?"
    )
    .bind(quantity)
    .bind(shop_item_id)
    .execute(pool).await?;

    // Record transaction
    let session_id: String = sqlx::query_scalar(
        "SELECT id FROM shop_sessions WHERE campaign_id = ? AND status = 'open'"
    )
    .bind(campaign_id)
    .fetch_one(pool).await?;

    let tx_id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO shop_transactions
         (id, shop_session_id, campaign_id, transaction_type, item_name,
          shop_item_id, quantity, price_pp, price_gp, price_sp, price_cp)
         VALUES (?, ?, ?, 'buy', ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&tx_id)
    .bind(&session_id)
    .bind(campaign_id)
    .bind(&item.name)
    .bind(shop_item_id)
    .bind(quantity)
    .bind(total_pp)
    .bind(total_gp)
    .bind(total_sp)
    .bind(total_cp)
    .execute(pool).await?;

    Ok(json!({
        "success": true,
        "item_purchased": item.name,
        "quantity": quantity,
        "total_cost": {
            "pp": total_pp,
            "gp": total_gp,
            "sp": total_sp,
            "cp": total_cp,
        },
        "new_balance": {
            "pp": new_pp,
            "gp": new_gp,
            "sp": new_sp,
            "cp": new_cp,
        },
        "item_id": created_item.id,
    }))
}

// ─── Sell item ────────────────────────────────────────────────────────────────

pub async fn sell_item(
    pool: &SqlitePool,
    campaign_id: &str,
    player: &crate::models::Player,
    player_item_id: &str,
) -> Result<Value> {
    let item = crate::db::items::get_item(pool, player_item_id).await?
        .ok_or_else(|| anyhow::anyhow!("Item not found"))?;

    // Verify player owns this item
    if item.owner_id.as_deref() != Some(&player.id) {
        return Ok(json!({"error": "You do not own this item"}));
    }

    // Calculate sell price — half of base value (in GP for simplicity)
    // Items without a clear price get a nominal amount
    let sell_gp = match item.rarity.as_str() {
        "common"    => 1,
        "uncommon"  => 25,
        "rare"      => 250,
        "very_rare" => 2500,
        "legendary" => 25000,
        _           => 1,
    };
    let sell_gp = (sell_gp / 2).max(1);

    // Add currency to player
    let (new_pp, new_gp, new_sp, new_cp) = crate::db::player::update_currency(
        pool, &player.id, 0, sell_gp, 0, 0
    ).await?;

    // Remove item from player
    crate::db::items::remove_item(pool, player_item_id, 1).await?;

    // Record transaction
    let session: Option<String> = sqlx::query_scalar(
        "SELECT id FROM shop_sessions WHERE campaign_id = ? AND status = 'open'"
    )
    .bind(campaign_id)
    .fetch_optional(pool).await?;

    if let Some(session_id) = session {
        let tx_id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO shop_transactions
             (id, shop_session_id, campaign_id, transaction_type, item_name,
              player_item_id, quantity, price_gp)
             VALUES (?, ?, ?, 'sell', ?, ?, 1, ?)"
        )
        .bind(&tx_id)
        .bind(&session_id)
        .bind(campaign_id)
        .bind(&item.name)
        .bind(player_item_id)
        .bind(sell_gp)
        .execute(pool).await?;
    }

    Ok(json!({
        "success": true,
        "item_sold": item.name,
        "gold_received": sell_gp,
        "new_balance": {
            "pp": new_pp,
            "gp": new_gp,
            "sp": new_sp,
            "cp": new_cp,
        }
    }))
}

// ─── Close shop ───────────────────────────────────────────────────────────────

pub async fn close_shop(
    pool: &SqlitePool,
    campaign_id: &str,
) -> Result<Value> {
    let session = sqlx::query_as::<_, ShopSession>(
        "SELECT * FROM shop_sessions WHERE campaign_id = ? AND status = 'open' LIMIT 1"
    )
    .bind(campaign_id)
    .fetch_optional(pool).await?;

    let session = match session {
        Some(s) => s,
        None => return Ok(json!({"error": "No active shop"})),
    };

    sqlx::query(
        "UPDATE shop_sessions SET status = 'closed', updated_at = datetime('now') WHERE id = ?"
    )
    .bind(&session.id)
    .execute(pool).await?;

    // Get transaction summary
    let transactions = sqlx::query_as::<_, ShopTransaction>(
        "SELECT * FROM shop_transactions WHERE shop_session_id = ?"
    )
    .bind(&session.id)
    .fetch_all(pool).await?;

    let purchased: Vec<&ShopTransaction> = transactions.iter()
        .filter(|t| t.transaction_type == "buy").collect();
    let sold: Vec<&ShopTransaction> = transactions.iter()
        .filter(|t| t.transaction_type == "sell").collect();

    Ok(json!({
        "session_closed": true,
        "shop_name": session.shop_name,
        "items_purchased": purchased.iter().map(|t| &t.item_name).collect::<Vec<_>>(),
        "items_sold": sold.iter().map(|t| &t.item_name).collect::<Vec<_>>(),
        "transaction_count": transactions.len(),
    }))
}