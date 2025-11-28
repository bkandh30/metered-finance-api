use axum::{
    extract::{Path, State},
    Extension, Json,
};
use std::sync::Arc;

use crate::{
    app::AppState,
    middleware::{auth::{AdminAuth, ClientAuth}, errors::AppError},
    models::{
        keys::AuthContext,
        quota::QuotaService,
        responses::UsageResponse,
    },
};

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