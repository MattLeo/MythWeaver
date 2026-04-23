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

    tracing::info!("Starting MythWeaver backend...");

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:./data/mythweaver.db".to_string());

    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY must be set");

    let model = std::env::var("ANTHROPIC_MODEL")
        .unwrap_or_else(|_| "claude-haiku-4-5-20251001".to_string());

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3001".to_string())
        .parse::<u16>()
        .unwrap_or(3001);

    tracing::info!("Connecting to database: {}", database_url);
    let pool = db::connect(&database_url).await?;
    db::run_migrations(&pool).await?;

    tracing::info!("Using Anthropic model: {}", model);
    let llm = LlmClient::new(api_key, model);

    let app_state = Arc::new(AppState { pool, llm });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/campaigns", get(api::list_campaigns).post(api::create_campaign))
        .route("/api/campaigns/:id", get(api::get_campaign_state))
        .route("/api/campaigns/:id/player-state", get(api::get_player_state))
        .route("/api/campaigns/:id/session", post(api::start_session))
        .route("/api/campaigns/:campaign_id/sessions/:session_id/end", post(api::end_session))
        .route("/api/campaigns/:campaign_id/sessions/:session_id/messages", get(api::get_session_messages))
        .route("/api/message", post(api::send_message))
        .layer(cors)
        .with_state(app_state);

    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("MythWeaver backend listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "ok",
        "service": "mythweaver-backend"
    }))
}