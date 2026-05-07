use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
mod api;
mod db;
mod llm;
mod models;
mod tools;
use llm::LlmClient;
pub struct AppState {
    pub pool: sqlx::SqlitePool,
    pub llm: LlmClient,
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "mythweaver=debug,tower_http=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:./data/mythweaver.db".to_string());
    let api_key = String::new(); // llama.cpp doesn't need one
    let model = std::env::var("LLM_MODEL")
        .unwrap_or_else(|_| "local".to_string()); // llama.cpp ignores this field
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3001".to_string())
        .parse::<u16>()
        .unwrap_or(3001);
    let pool = db::connect(&database_url).await?;
    db::run_migrations(&pool).await?;
    let llm = LlmClient::new(api_key, model);
    let app_state = Arc::new(AppState { pool, llm });
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    let app = Router::new()
        .route("/health", get(health_check))

        // ── Campaigns ────────────────────────────────────────────────────────
        .route("/api/campaigns", get(api::list_campaigns).post(api::create_campaign))
        .route("/api/campaigns/:id", get(api::get_campaign_state).delete(api::delete_campaign))
        .route("/api/campaigns/:id/player-state", get(api::get_player_state))
        .route("/api/campaigns/:id/level-up", post(api::level_up))

        // ── Sessions ─────────────────────────────────────────────────────────
        .route("/api/campaigns/:id/session", post(api::start_session))
        .route("/api/campaigns/:campaign_id/sessions/:session_id/end", post(api::end_session))
        .route("/api/campaigns/:campaign_id/sessions/:session_id/messages", get(api::get_session_messages))

        // ── Messaging ────────────────────────────────────────────────────────
        .route("/api/message", post(api::send_message))

        // ── Combat ───────────────────────────────────────────────────────────
        .route("/api/campaigns/:id/combat", get(api::get_combat_state_handler))
        .route("/api/campaigns/:id/combat/initiative", post(api::submit_initiative))
        .route("/api/campaigns/:id/combat/target", post(api::set_combat_target))
        .route("/api/campaigns/:id/combat/attack", post(api::resolve_attack))
        .route("/api/campaigns/:id/combat/damage", post(api::resolve_damage))
        .route("/api/campaigns/:id/combat/ability", post(api::use_combat_ability))
        .route("/api/campaigns/:id/combat/end-turn", post(api::end_combat_turn))
        .route("/api/campaigns/:id/combat/flee", post(api::flee_combat))
        .route("/api/campaigns/:id/combat/end", post(api::end_combat_handler))
        .route("/api/campaigns/:id/combat/process-start", post(api::process_initial_turns))

        // ── Shop ─────────────────────────────────────────────────────────────
        .route("/api/campaigns/:id/shop", get(api::get_shop_state))
        .route("/api/campaigns/:id/shop/buy", post(api::buy_item))
        .route("/api/campaigns/:id/shop/sell", post(api::sell_item))
        .route("/api/campaigns/:id/shop/close", post(api::close_shop))

        // ── Inventory ────────────────────────────────────────────────────────
        .route("/api/campaigns/:id/inventory/equip", post(api::equip_item_handler))
        .route("/api/campaigns/:id/inventory/unequip", post(api::unequip_item_handler))
        .route("/api/campaigns/:id/inventory/delete", post(api::delete_item_handler))

        // ── Spells ───────────────────────────────────────────────────────────
        .route("/api/campaigns/:id/spells", get(api::get_known_spells_handler))
        .route("/api/campaigns/:id/spells/castable", get(api::get_castable_spells_handler))
        .route("/api/campaigns/:id/spells/learn", post(api::learn_spell_handler))
        .route("/api/campaigns/:id/spells/forget", post(api::forget_spell_handler))
        .route("/api/campaigns/:id/spells/cast", post(api::cast_spell_handler))
        .route("/api/campaigns/:id/spells/search", post(api::search_spells_handler))
        .route("/api/campaigns/:id/spells/slots", get(api::get_spell_slots_handler))
        .route("/api/campaigns/:id/spells/slots/seed", post(api::seed_ek_slots_handler))

        // ── Concentration ────────────────────────────────────────────────────
        .route("/api/campaigns/:id/concentration", get(api::get_concentration_handler))
        .route("/api/campaigns/:id/concentration/drop", post(api::drop_concentration_handler))

        // ── War Bond ─────────────────────────────────────────────────────────
        .route("/api/campaigns/:id/war-bonds", get(api::get_war_bonds_handler))
        .route("/api/campaigns/:id/war-bonds/create", post(api::create_war_bond_handler))
        .route("/api/campaigns/:id/war-bonds/break", post(api::break_war_bond_handler))
        .route("/api/campaigns/:id/war-bonds/summon", post(api::summon_bonded_weapon_handler))

        .layer(cors)
        .with_state(app_state);
    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("MythWeaver backend listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
async fn health_check() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({"status": "ok", "service": "mythweaver-backend"}))
}