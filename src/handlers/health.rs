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

/// Health check - Liveness probe
///
/// Returns a simple health check response indicating the service is alive.
/// This endpoint is typically used by orchestrators (like Kubernetes) to determine
/// if the service should be restarted.
#[utoipa::path(
    get,
    path = "/health/live",
    tag = "health",
    responses(
        (status = 200, description = "Service is alive", 
            body = serde_json::Value,
            example = json!({
                "status": "live",
                "timestamp": "2024-01-15T10:30:00Z"
            })
        )
    )
)]
pub async fn health_live() -> Json<serde_json::Value> {
    Json(json!({
        "status": "live",
        "timestamp": Utc::now().to_rfc3339()
    }))
}

/// Health check - Readiness probe
///
/// Returns the health status of the service including database connectivity.
/// This endpoint is used by orchestrators to determine if the service is ready
/// to accept traffic. Returns 503 if dependencies are unhealthy.
#[utoipa::path(
    get,
    path = "/health/ready",
    tag = "health",
    responses(
        (status = 200, description = "Service is ready", 
            body = serde_json::Value,
            example = json!({
                "status": "ready",
                "database": "healthy",
                "timestamp": "2024-01-15T10:30:00Z"
            })
        ),
        (status = 503, description = "Service is not ready",
            body = serde_json::Value,
            example = json!({
                "status": "not_ready",
                "database": "unhealthy",
                "error": "Database connection failed",
                "timestamp": "2024-01-15T10:30:00Z"
            })
        )
    )
)]
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