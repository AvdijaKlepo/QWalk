/// HTTP and WebSocket route handlers.
///
/// Routes:
///   GET  /api/status          — current snapshot (JSON, one-shot)
///   POST /api/topology        — change topology  { kind: "grid", side: 6 }
///   POST /api/speed           — change tick ms   { tick_ms: 200 }
///   GET  /ws                  — WebSocket stream of Snapshot JSON

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::engine::{Engine, TopologyKind};

// ── Shared application state ──────────────────────────────────────────────────

pub type AppState = Engine;

// ── GET /api/status ───────────────────────────────────────────────────────────

pub async fn get_status(State(engine): State<AppState>) -> impl IntoResponse {
    let snap = engine.current_snapshot().await;
    Json(snap)
}

// ── POST /api/topology ────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TopologyRequest {
    Grid { side: usize },
    Ring { nodes: usize, chord_step: usize },
}

#[derive(Serialize)]
struct Ack { ok: bool, message: String }

pub async fn post_topology(
    State(engine): State<AppState>,
    Json(req): Json<TopologyRequest>,
) -> impl IntoResponse {
    let topo = match req {
        TopologyRequest::Grid { side } => {
            if side < 2 || side > 12 {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(Ack { ok: false, message: "side must be 2–12".into() }),
                );
            }
            TopologyKind::Grid { side }
        }
        TopologyRequest::Ring { nodes, chord_step } => {
            if nodes < 3 || nodes > 64 {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(Ack { ok: false, message: "nodes must be 3–64".into() }),
                );
            }
            TopologyKind::Ring { nodes, chord_step }
        }
    };
    engine.set_topology(topo).await;
    (StatusCode::OK, Json(Ack { ok: true, message: "topology updated".into() }))
}

// ── POST /api/speed ───────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SpeedRequest { pub tick_ms: u64 }

pub async fn post_speed(
    State(engine): State<AppState>,
    Json(req): Json<SpeedRequest>,
) -> impl IntoResponse {
    if req.tick_ms < 16 || req.tick_ms > 5_000 {
        return (
            StatusCode::BAD_REQUEST,
            Json(Ack { ok: false, message: "tick_ms must be 16–5000".into() }),
        );
    }
    engine.set_tick_ms(req.tick_ms).await;
    (StatusCode::OK, Json(Ack { ok: true, message: format!("tick set to {}ms", req.tick_ms) }))
}

// ── GET /ws  (WebSocket upgrade) ──────────────────────────────────────────────

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(engine): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, engine))
}

async fn handle_ws(mut socket: WebSocket, engine: Engine) {
    info!("WebSocket client connected");
    let mut rx = engine.subscribe();

    // Send the current state immediately so the client doesn't wait for the
    // next tick before showing anything.
    let initial = engine.current_snapshot().await;
    if let Ok(json) = serde_json::to_string(&initial) {
        if socket.send(Message::Text(json.into())).await.is_err() {
            return;
        }
    }

    loop {
        tokio::select! {
            // New snapshot from the engine tick loop.
            result = rx.recv() => {
                match result {
                    Ok(snap) => {
                        match serde_json::to_string(&snap) {
                            Ok(json) => {
                                if socket.send(Message::Text(json.into())).await.is_err() {
                                    info!("WebSocket client disconnected (send error)");
                                    break;
                                }
                                debug!("Sent snapshot step={}", snap.step);
                            }
                            Err(e) => warn!("Serialisation error: {e}"),
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        warn!("WebSocket receiver lagged by {n} messages");
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        info!("Engine broadcast channel closed");
                        break;
                    }
                }
            }

            // Client sent a message — only ping/pong / close expected, but
            // receive anyway to detect clean disconnects.
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => {
                        info!("WebSocket client disconnected");
                        break;
                    }
                    Some(Ok(Message::Ping(p))) => {
                        let _ = socket.send(Message::Pong(p)).await;
                    }
                    _ => {}
                }
            }
        }
    }
}
