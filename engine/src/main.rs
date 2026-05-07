mod engine;
mod quantum_walk;
mod routes;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::engine::{Engine, EngineConfig};
use crate::routes::{get_status, post_speed, post_topology, ws_handler};

#[tokio::main]
async fn main() {
    // ── Logging ───────────────────────────────────────────────────────────────
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "qwalk_engine=debug,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // ── Engine ────────────────────────────────────────────────────────────────
    let config = EngineConfig::default(); // 6×6 grid, 120ms tick
    let engine = Engine::new(config);
    engine.start(); 

    // ── CORS (allow SvelteKit dev server) ─────────────────────────────────────
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // ── Router ────────────────────────────────────────────────────────────────
    let app = Router::new()
        .route("/api/status",   get(get_status))
        .route("/api/topology", post(post_topology))
        .route("/api/speed",    post(post_speed))
        .route("/ws",           get(ws_handler))
        .layer(cors)
        .with_state(engine);

    let addr = "0.0.0.0:4000";
    info!("qwalk-engine listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
