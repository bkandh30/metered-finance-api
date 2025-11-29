use axum::{
    extract::{Path, State},
    Extension, Json,
};
use std::sync::Arc;

use crate::{
    app::AppState,
    middleware::{auth::{AdminAuth, ClientAuth}, errors::AppError},
    models::{
        common::ErrorResponse,
        keys::AuthContext,
        quota::QuotaService,
        responses::UsageResponse,
    },
};

/// Get own usage statistics
///
/// Retrieves usage statistics and quota information for the authenticated API key.
/// Shows current rate limits, daily and monthly quotas, and remaining allocations.
#[utoipa::path(
    get,
    path = "/api/usage",
    tag = "usage",
    responses(
        (status = 200, description = "Usage statistics retrieved successfully", body = UsageResponse,
            example = json!({
                "key_id": "key_a1b2c3d4",
                "limits": {
                    "rate_limit_per_minute": 100,
                    "daily_quota": 10000,
                    "monthly_quota": 300000
                },
                "usage": {
                    "today": 1234,
                    "this_month": 45678,
                    "daily_remaining": 8766,
                    "monthly_remaining": 254322
                }
            })
        ),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 429, description = "Rate limit exceeded", body = ErrorResponse),
    ),
    security(
        ("ApiKeyAuth" = [])
    )
)]
pub async fn get_own_usage(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<ClientAuth>,
) -> Result<Json<UsageResponse>, AppError> {
    let key_id = match &auth.context {
        AuthContext::Client { key_id, .. } => key_id,
        AuthContext::Admin => {
            return Err(AppError::InvalidInput(
                "Admin keys do not have quota usage".to_string(),
            ));
        }
    };

    let status = QuotaService::get_status(&state.pool, key_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get quota status: {}", e);
            AppError::InternalError("Failed to retrieve usage data".to_string())
        })?;

    Ok(Json(UsageResponse {
        key_id: status.key_id,
        limits: status.limits,
        usage: status.usage,
    }))
}

/// Get usage statistics for a specific API key
///
/// Retrieves usage statistics and quota information for any API key by its ID.
/// This endpoint is only accessible with admin authentication.
#[utoipa::path(
    get,
    path = "/api/admin/usage/{key_id}",
    tag = "usage",
    params(
        ("key_id" = String, Path, description = "API key identifier")
    ),
    responses(
        (status = 200, description = "Usage statistics retrieved successfully", body = UsageResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "API key not found", body = ErrorResponse),
    ),
    security(
        ("AdminKeyAuth" = [])
    )
)]
pub async fn get_key_usage(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AdminAuth>,
    Path(key_id): Path<String>,
) -> Result<Json<UsageResponse>, AppError> {
    let exists = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM api_keys WHERE key_id = $1"
    )
    .bind(&key_id)
    .fetch_one(&state.pool)
    .await?;

    if exists == 0 {
        return Err(AppError::not_found("API Key", &key_id));
    }

    let status = QuotaService::get_status(&state.pool, &key_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get quota status: {}", e);
            AppError::InternalError("Failed to retrieve usage data".to_string())
        })?;

    Ok(Json(UsageResponse {
        key_id: status.key_id,
        limits: status.limits,
        usage: status.usage,
    }))
}