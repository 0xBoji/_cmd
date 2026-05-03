//! REST API handlers for VIEW web server.

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde_json::{json, Value};

use crate::SharedState;

/// `GET /` — server info, useful as a health check from any device.
pub async fn root_info() -> Json<Value> {
    Json(json!({
        "service": "web",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "VIEW LAN remote access server",
        "endpoints": {
            "agents":   "/api/agents",
            "events":   "/api/events",
            "snapshot": "/api/snapshot",
            "ws":       "/ws"
        }
    }))
}

/// `GET /api/agents` — all agents as JSON array.
pub async fn agents(State(state): State<SharedState>) -> impl IntoResponse {
    let app = state.read();
    let agents: Vec<_> = app.registry.agents.values().cloned().collect();
    (StatusCode::OK, Json(agents))
}

/// `GET /api/events` — last 20 events as JSON array.
pub async fn events(State(state): State<SharedState>) -> impl IntoResponse {
    let app = state.read();
    let events: Vec<_> = app.registry.events.iter().take(20).cloned().collect();
    (StatusCode::OK, Json(events))
}

/// `GET /api/snapshot` — complete `WebSnapshot` (agents + events + terminals).
pub async fn snapshot(State(state): State<SharedState>) -> impl IntoResponse {
    let app = state.read();
    let snap = app.web_snapshot();
    (StatusCode::OK, Json(snap))
}
