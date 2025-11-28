use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use std::sync::Arc;

use crate::{
    app::AppState,
    middleware::{auth::AdminAuth, errors::AppError},
    models::{
        common::{PaginatedResponse, PaginationParams},
        keys::{ApiKeyGenerator, Scope},
        requests::{CreateApiKeyRequest, UpdateApiKeyRequest},
        responses::{KeyCreatedResponse, KeyInfoResponse},
    },
};

pub async fn create_api_key(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AdminAuth>,
    Json(req): Json<CreateApiKeyRequest>,
) -> Result<(StatusCode, Json<KeyCreatedResponse>), AppError> {
    req.validate()
        .map_err(|e| AppError::ValidationError(e))?;

    let (api_key, key_id, prefix, secret_hash) = ApiKeyGenerator::generate_full();

    let rate_limit = req.rate_limit_per_minute.unwrap_or(60);
    let daily_quota = req.daily_quota.unwrap_or(10_000);
    let monthly_quota = req.monthly_quota.unwrap_or(300_000);

    let scopes_str: Vec<String> = req.scopes.iter().map(|s| s.to_string()).collect();

    let key = sqlx::query_as::<_, (
        String,
        String,
        String,
        Vec<String>,
        bool,
        i32,
        i32,
        i32,
        chrono::DateTime<chrono::Utc>,
    )>(
        r#"
        INSERT INTO api_keys (
            key_id, prefix, name, secret_hash, scopes, active,
            rate_limit_per_minute, daily_quota, monthly_quota,
            created_at, last_used_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NULL)
        RETURNING 
            key_id, prefix, name, scopes, active,
            rate_limit_per_minute, daily_quota, monthly_quota,
            created_at
        "#
    )
    .bind(&key_id)
    .bind(&prefix)
    .bind(&req.name)
    .bind(&secret_hash)
    .bind(&scopes_str)
    .bind(true)
    .bind(rate_limit)
    .bind(daily_quota)
    .bind(monthly_quota)
    .fetch_one(&state.pool)
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(KeyCreatedResponse {
            key_id: key.0,
            api_key,
            prefix: key.1,
            name: key.2,
            scopes: key.3.iter().filter_map(|s| Scope::from_str(s)).collect(),
            active: key.4,
            rate_limit_per_minute: key.5,
            daily_quota: key.6,
            monthly_quota: key.7,
            created_at: key.8,
        }),
    ))
}

pub async fn list_api_keys(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AdminAuth>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<KeyInfoResponse>>, AppError> {
    params.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let limit = params.limit.unwrap_or(20);

    let keys = if let Some(cursor) = &params.cursor {
        let decoded = cursor.decode_string()
            .map_err(|e| AppError::InvalidInput(format!("Invalid cursor: {}", e)))?;

        sqlx::query_as::<_, (
            String,
            String,
            String,
            Vec<String>,
            bool,
            i32,
            i32,
            i32,
            chrono::DateTime<chrono::Utc>,
            Option<chrono::DateTime<chrono::Utc>>,
        )>(
            r#"
            SELECT 
                key_id, prefix, name, scopes, active,
                rate_limit_per_minute, daily_quota, monthly_quota,
                created_at, last_used_at
            FROM api_keys
            WHERE key_id > $1
            ORDER BY key_id ASC
            LIMIT $2
            "#
        )
        .bind(&decoded)
        .bind(limit + 1)
        .fetch_all(&state.pool)
        .await?
    } else {
        sqlx::query_as::<_, (
            String,
            String,
            String,
            Vec<String>,
            bool,
            i32,
            i32,
            i32,
            chrono::DateTime<chrono::Utc>,
            Option<chrono::DateTime<chrono::Utc>>,
        )>(
            r#"
            SELECT 
                key_id, prefix, name, scopes, active,
                rate_limit_per_minute, daily_quota, monthly_quota,
                created_at, last_used_at
            FROM api_keys
            ORDER BY key_id ASC
            LIMIT $1
            "#
        )
        .bind(limit + 1)
        .fetch_all(&state.pool)
        .await?
    };

    let has_more = keys.len() > limit as usize;
    let items: Vec<KeyInfoResponse> = keys
        .into_iter()
        .take(limit as usize)
        .map(|k| KeyInfoResponse {
            key_id: k.0,
            prefix: k.1,
            name: k.2,
            scopes: k.3.iter().filter_map(|s| Scope::from_str(s)).collect(),
            active: k.4,
            rate_limit_per_minute: k.5,
            daily_quota: k.6,
            monthly_quota: k.7,
            created_at: k.8,
            last_used_at: k.9,
        })
        .collect();

    let next_cursor = if has_more {
        items.last().map(|item| {
            crate::models::common::Cursor::encode(&item.key_id)
        })
    } else {
        None
    };

    Ok(Json(PaginatedResponse {
        data: items,
        has_more,
        next_cursor,
    }))
}

pub async fn get_api_key(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AdminAuth>,
    Path(key_id): Path<String>,
) -> Result<Json<KeyInfoResponse>, AppError> {
    let key = sqlx::query_as::<_, (
        String,
        String,
        String,
        Vec<String>,
        bool,
        i32,
        i32,
        i32,
        chrono::DateTime<chrono::Utc>,
        Option<chrono::DateTime<chrono::Utc>>,
    )>(
        r#"
        SELECT 
            key_id, prefix, name, scopes, active,
            rate_limit_per_minute, daily_quota, monthly_quota,
            created_at, last_used_at
        FROM api_keys
        WHERE key_id = $1
        "#
    )
    .bind(&key_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("API Key", &key_id))?;

    Ok(Json(KeyInfoResponse {
        key_id: key.0,
        prefix: key.1,
        name: key.2,
        scopes: key.3.iter().filter_map(|s| Scope::from_str(s)).collect(),
        active: key.4,
        rate_limit_per_minute: key.5,
        daily_quota: key.6,
        monthly_quota: key.7,
        created_at: key.8,
        last_used_at: key.9,
    }))
}

pub async fn update_api_key(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AdminAuth>,
    Path(key_id): Path<String>,
    Json(req): Json<UpdateApiKeyRequest>,
) -> Result<Json<KeyInfoResponse>, AppError> {
    req.validate()
        .map_err(|e| AppError::ValidationError(e))?;

    let exists = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM api_keys WHERE key_id = $1"
    )
    .bind(&key_id)
    .fetch_one(&state.pool)
    .await?;

    if exists == 0 {
        return Err(AppError::not_found("API Key", &key_id));
    }

    let mut updates = vec![];
    let mut param_count = 1;

    if req.active.is_some() {
        param_count += 1;
        updates.push(format!("active = ${}", param_count));
    }

    if req.scopes.is_some() {
        param_count += 1;
        updates.push(format!("scopes = ${}", param_count));
    }

    if req.rate_limit_per_minute.is_some() {
        param_count += 1;
        updates.push(format!("rate_limit_per_minute = ${}", param_count));
    }

    if req.daily_quota.is_some() {
        param_count += 1;
        updates.push(format!("daily_quota = ${}", param_count));
    }

    if req.monthly_quota.is_some() {
        param_count += 1;
        updates.push(format!("monthly_quota = ${}", param_count));
    }

    if updates.is_empty() {
        return Err(AppError::ValidationError(
            "No fields to update".to_string(),
        ));
    }

    let query = format!(
        r#"
        UPDATE api_keys
        SET {}
        WHERE key_id = $1
        RETURNING 
            key_id, prefix, name, scopes, active,
            rate_limit_per_minute, daily_quota, monthly_quota,
            created_at, last_used_at
        "#,
        updates.join(", ")
    );

    let mut sql_query = sqlx::query_as::<_, (
        String,
        String,
        String,
        Vec<String>,
        bool,
        i32,
        i32,
        i32,
        chrono::DateTime<chrono::Utc>,
        Option<chrono::DateTime<chrono::Utc>>,
    )>(&query)
    .bind(&key_id);

    if let Some(active) = req.active {
        sql_query = sql_query.bind(active);
    }

    if let Some(scopes) = req.scopes {
        let scopes_str: Vec<String> = scopes.iter().map(|s| s.to_string()).collect();
        sql_query = sql_query.bind(scopes_str);
    }

    if let Some(rate) = req.rate_limit_per_minute {
        sql_query = sql_query.bind(rate);
    }

    if let Some(daily) = req.daily_quota {
        sql_query = sql_query.bind(daily);
    }

    if let Some(monthly) = req.monthly_quota {
        sql_query = sql_query.bind(monthly);
    }

    let key = sql_query.fetch_one(&state.pool).await?;

    Ok(Json(KeyInfoResponse {
        key_id: key.0,
        prefix: key.1,
        name: key.2,
        scopes: key.3.iter().filter_map(|s| Scope::from_str(s)).collect(),
        active: key.4,
        rate_limit_per_minute: key.5,
        daily_quota: key.6,
        monthly_quota: key.7,
        created_at: key.8,
        last_used_at: key.9,
    }))
}

pub async fn delete_api_key(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AdminAuth>,
    Path(key_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query("DELETE FROM api_keys WHERE key_id = $1")
        .bind(&key_id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::not_found("API Key", &key_id));
    }

    Ok(StatusCode::NO_CONTENT)
}