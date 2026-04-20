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
    // Load .env
    dotenvy::dotenv().ok();

    // Init logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "mythweaver=debug,tower_http=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting MythWeaver backend...");

    // Config
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:./data/mythweaver.db".to_string());

    let ollama_url = std::env::var("OLLAMA_URL")
        .unwrap_or_else(|_| "http://localhost:11434".to_string());

    let ollama_model = std::env::var("OLLAMA_MODEL")
        .unwrap_or_else(|_| "lukey03/qwen3.5-9b-abliterated".to_string());

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3001".to_string())
        .parse::<u16>()
        .unwrap_or(3001);

    // Database
    tracing::info!("Connecting to database: {}", database_url);
    let pool = db::connect(&database_url).await?;
    db::run_migrations(&pool).await?;

    // LLM client
    tracing::info!("Connecting to Ollama: {} (model: {})", ollama_url, ollama_model);
    let llm = LlmClient::new(ollama_url, ollama_model);

    let app_state = Arc::new(AppState { pool, llm });

    // CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/campaigns", post(api::create_campaign))
        .route("/api/campaigns/:id", get(api::get_campaign_state))
        .route("/api/campaigns/:id/session", post(api::start_session))
        .route("/api/campaigns/:campaign_id/sessions/:session_id/end", post(api::end_session))
        .route("/api/message", post(api::send_message))
        .route("/api/campaigns/:id/player-state", get(api::get_player_state))
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
