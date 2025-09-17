use axum::{
    response::Json,
    http::StatusCode,
    extract::State,
};
use serde_json::json;
use std::sync::Arc;
use tracing::error;
use chrono::Utc;

use crate::app::AppState;

pub async fn health_live() -> Json<serde_json::Value> {
    Json(json!({
        "status": "live",
        "timestamp": Utc::now().to_rfc3339()
    }))
}

pub async fn health_ready(
    State(state): State<Arc<AppState>>
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match crate::db::check_health(&state.pool).await {
        Ok(_) => {
            Ok(Json(json!({
                "status": "ready",
                "database": "healthy",
                "timestamp": Utc::now().to_rfc3339()
            })))
        }
        Err(e) => {
            error!("Database health check failed: {}", e);
            Err((
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "status": "not_ready",
                    "database": "unhealthy",
                    "error": "Database connection failed",
                    "timestamp": Utc::now().to_rfc3339()
                }))
            ))
        }
    }
}