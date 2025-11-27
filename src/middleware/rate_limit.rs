use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

use crate::{
    app::AppState,
    middleware::auth::ClientAuth,
    models::quota::{QuotaService, RateLimitService},
};

pub async fn check_rate_limit_and_quota(
    State(state): State<Arc<AppState>>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    let auth = req
        .extensions()
        .get::<ClientAuth>()
        .ok_or((
            StatusCode::UNAUTHORIZED,
            "Authentication required".to_string(),
        ))?;

    let key_id = match &auth.context {
        crate::models::keys::AuthContext::Client { key_id, .. } => key_id,
        crate::models::keys::AuthContext::Admin => {
            return Ok(next.run(req).await);
        }
    };

    let limits = QuotaService::get_limits(&state.pool, key_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get quota limits: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to check rate limit".to_string(),
            )
        })?;

    let within_rate_limit = RateLimitService::check_rate_limit(
        &state.pool,
        key_id,
        limits.rate_limit_per_minute,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to check rate limit: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to check rate limit".to_string(),
        )
    })?;

    if !within_rate_limit {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            format!("Rate limit exceeded. Limit: {} requests per minute", limits.rate_limit_per_minute),
        ));
    }

    let within_daily_quota = QuotaService::check_daily_quota(&state.pool, key_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to check daily quota: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to check quota".to_string(),
            )
        })?;

    if !within_daily_quota {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            "Daily quota exceeded".to_string(),
        ));
    }

    let within_monthly_quota = QuotaService::check_monthly_quota(&state.pool, key_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to check monthly quota: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to check quota".to_string(),
            )
        })?;

    if !within_monthly_quota {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            "Monthly quota exceeded".to_string(),
        ));
    }

    QuotaService::increment_usage(&state.pool, key_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to increment quota: {}", e);
        })
        .ok();

    Ok(next.run(req).await)
}

pub async fn check_rate_limit_only(
    State(state): State<Arc<AppState>>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    let auth = req
        .extensions()
        .get::<ClientAuth>()
        .ok_or((
            StatusCode::UNAUTHORIZED,
            "Authentication required".to_string(),
        ))?;

    let key_id = match &auth.context {
        crate::models::keys::AuthContext::Client { key_id, .. } => key_id,
        crate::models::keys::AuthContext::Admin => {
            return Ok(next.run(req).await);
        }
    };

    let limits = QuotaService::get_limits(&state.pool, key_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get quota limits: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to check rate limit".to_string(),
            )
        })?;

    let within_rate_limit = RateLimitService::check_rate_limit(
        &state.pool,
        key_id,
        limits.rate_limit_per_minute,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to check rate limit: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to check rate limit".to_string(),
        )
    })?;

    if !within_rate_limit {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            format!("Rate limit exceeded. Limit: {} requests per minute", limits.rate_limit_per_minute),
        ));
    }

    Ok(next.run(req).await)
}