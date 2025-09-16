use axum::{
    response::Json,
    http::StatusCode,
};
use serde_json::json;

pub async fn health_live() -> Json<serde_json::Value> {
    Json(json!({
        "status": "live"
    }))
}

pub async fn health_ready() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(json!({
        "status": "ready"
    })))
}