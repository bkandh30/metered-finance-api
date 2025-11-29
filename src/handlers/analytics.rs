use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use chrono::Utc;
use std::sync::Arc;

use crate::{
    app::AppState,
    middleware::{auth::{AdminAuth, ClientAuth}, errors::AppError},
    models::{
        common::ErrorResponse,
        analytics::{AnalyticsResponse, AnalyticsService, TimeRangeFilter},
        keys::AuthContext,
    },
};

#[utoipa::path(
    get,
    path = "/api/analytics",
    tag = "analytics",
    params(
        TimeRangeFilter
    ),
    responses(
        (status = 200, description = "Analytics retrieved successfully", body = AnalyticsResponse,
            example = json!({
                "overview": {
                    "total_requests": 15234,
                    "successful_requests": 14891,
                    "failed_requests": 343,
                    "avg_latency_ms": 45.2,
                    "median_latency_ms": 38.5,
                    "p95_latency_ms": 125.0,
                    "p99_latency_ms": 289.0,
                    "period_start": "2024-01-08T00:00:00Z",
                    "period_end": "2024-01-15T23:59:59Z"
                },
                "top_endpoints": [
                    {
                        "path": "/api/transactions",
                        "method": "POST",
                        "request_count": 5234,
                        "avg_latency_ms": 52.3,
                        "error_rate": 2.1
                    }
                ],
                "status_codes": [
                    {
                        "status_code": 200,
                        "count": 12456,
                        "percentage": 81.8
                    },
                    {
                        "status_code": 201,
                        "count": 2435,
                        "percentage": 16.0
                    }
                ],
                "hourly_volume": []
            })
        ),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 429, description = "Rate limit exceeded", body = ErrorResponse),
    ),
    security(
        ("ApiKeyAuth" = [])
    )
)]
pub async fn get_own_analytics(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<ClientAuth>,
    Query(filter): Query<TimeRangeFilter>,
) -> Result<Json<AnalyticsResponse>, AppError> {
    let key_id = match &auth.context {
        AuthContext::Client { key_id, .. } => key_id,
        AuthContext::Admin => {
            return Err(AppError::InvalidInput(
                "Admin keys do not have analytics".to_string(),
            ));
        }
    };

    let start = filter.start.unwrap_or_else(|| Utc::now() - chrono::Duration::days(7));
    let end = filter.end.unwrap_or_else(|| Utc::now());

    let overview = AnalyticsService::get_request_stats(&state.pool, Some(key_id), start, end)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get request stats: {}", e);
            AppError::InternalError("Failed to retrieve analytics".to_string())
        })?;

    let top_endpoints = AnalyticsService::get_endpoint_stats(&state.pool, Some(key_id), start, end, 10)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get endpoint stats: {}", e);
            AppError::InternalError("Failed to retrieve analytics".to_string())
        })?;

    let status_codes = AnalyticsService::get_status_code_stats(&state.pool, Some(key_id), start, end)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get status code stats: {}", e);
            AppError::InternalError("Failed to retrieve analytics".to_string())
        })?;

    let hourly_volume = if end.signed_duration_since(start).num_hours() <= 168 {
        Some(
            AnalyticsService::get_hourly_volume(&state.pool, Some(key_id), start, end)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to get hourly volume: {}", e);
                    AppError::InternalError("Failed to retrieve analytics".to_string())
                })?,
        )
    } else {
        None
    };

    Ok(Json(AnalyticsResponse {
        overview,
        top_endpoints,
        status_codes,
        hourly_volume,
    }))
}

#[utoipa::path(
    get,
    path = "/api/admin/analytics/{key_id}",
    tag = "analytics",
    params(
        ("key_id" = String, Path, description = "API key identifier"),
        TimeRangeFilter
    ),
    responses(
        (status = 200, description = "Analytics retrieved successfully", body = AnalyticsResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "API key not found", body = ErrorResponse),
    ),
    security(
        ("AdminKeyAuth" = [])
    )
)]
pub async fn get_key_analytics(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AdminAuth>,
    Path(key_id): Path<String>,
    Query(filter): Query<TimeRangeFilter>,
) -> Result<Json<AnalyticsResponse>, AppError> {
    let exists = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM api_keys WHERE key_id = $1"
    )
    .bind(&key_id)
    .fetch_one(&state.pool)
    .await?;

    if exists == 0 {
        return Err(AppError::not_found("API Key", &key_id));
    }

    let start = filter.start.unwrap_or_else(|| Utc::now() - chrono::Duration::days(7));
    let end = filter.end.unwrap_or_else(|| Utc::now());

    let overview = AnalyticsService::get_request_stats(&state.pool, Some(&key_id), start, end)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get request stats: {}", e);
            AppError::InternalError("Failed to retrieve analytics".to_string())
        })?;

    let top_endpoints = AnalyticsService::get_endpoint_stats(&state.pool, Some(&key_id), start, end, 10)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get endpoint stats: {}", e);
            AppError::InternalError("Failed to retrieve analytics".to_string())
        })?;

    let status_codes = AnalyticsService::get_status_code_stats(&state.pool, Some(&key_id), start, end)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get status code stats: {}", e);
            AppError::InternalError("Failed to retrieve analytics".to_string())
        })?;

    let hourly_volume = if end.signed_duration_since(start).num_hours() <= 168 {
        Some(
            AnalyticsService::get_hourly_volume(&state.pool, Some(&key_id), start, end)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to get hourly volume: {}", e);
                    AppError::InternalError("Failed to retrieve analytics".to_string())
                })?,
        )
    } else {
        None
    };

    Ok(Json(AnalyticsResponse {
        overview,
        top_endpoints,
        status_codes,
        hourly_volume,
    }))
}

#[utoipa::path(
    get,
    path = "/api/admin/analytics",
    tag = "analytics",
    params(
        TimeRangeFilter
    ),
    responses(
        (status = 200, description = "System analytics retrieved successfully", body = AnalyticsResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    security(
        ("AdminKeyAuth" = [])
    )
)]
pub async fn get_system_analytics(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AdminAuth>,
    Query(filter): Query<TimeRangeFilter>,
) -> Result<Json<AnalyticsResponse>, AppError> {
    let start = filter.start.unwrap_or_else(|| Utc::now() - chrono::Duration::days(7));
    let end = filter.end.unwrap_or_else(|| Utc::now());

    let overview = AnalyticsService::get_request_stats(&state.pool, None, start, end)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get request stats: {}", e);
            AppError::InternalError("Failed to retrieve analytics".to_string())
        })?;

    let top_endpoints = AnalyticsService::get_endpoint_stats(&state.pool, None, start, end, 20)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get endpoint stats: {}", e);
            AppError::InternalError("Failed to retrieve analytics".to_string())
        })?;

    let status_codes = AnalyticsService::get_status_code_stats(&state.pool, None, start, end)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get status code stats: {}", e);
            AppError::InternalError("Failed to retrieve analytics".to_string())
        })?;

    let hourly_volume = if end.signed_duration_since(start).num_hours() <= 168 {
        Some(
            AnalyticsService::get_hourly_volume(&state.pool, None, start, end)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to get hourly volume: {}", e);
                    AppError::InternalError("Failed to retrieve analytics".to_string())
                })?,
        )
    } else {
        None
    };

    Ok(Json(AnalyticsResponse {
        overview,
        top_endpoints,
        status_codes,
        hourly_volume,
    }))
}