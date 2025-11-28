use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use std::sync::Arc;

use crate::{
    app::AppState,
    middleware::{auth::ClientAuth, errors::AppError},
    models::{
        common::{PaginatedResponse, PaginationParams},
        requests::{CreateAccountRequest, UpdateAccountRequest},
        responses::AccountResponse,
    },
};

pub async fn create_account(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<ClientAuth>,
    Json(req): Json<CreateAccountRequest>,
) -> Result<(StatusCode, Json<AccountResponse>), AppError> {
    req.validate()
        .map_err(|e| AppError::ValidationError(e))?;

    let existing = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM accounts WHERE account_id = $1"
    )
    .bind(&req.account_id)
    .fetch_one(&state.pool)
    .await?;

    if existing > 0 {
        return Err(AppError::InvalidInput(format!(
            "Account with ID '{}' already exists",
            req.account_id
        )));
    }

    let account = sqlx::query_as::<_, (String, Option<serde_json::Value>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
        r#"
        INSERT INTO accounts (account_id, metadata, created_at, updated_at)
        VALUES ($1, $2, NOW(), NOW())
        RETURNING account_id, metadata, created_at, updated_at
        "#
    )
    .bind(&req.account_id)
    .bind(&req.metadata)
    .fetch_one(&state.pool)
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(AccountResponse {
            account_id: account.0,
            metadata: account.1,
            created_at: account.2,
            updated_at: account.3,
        }),
    ))
}

pub async fn get_account(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<ClientAuth>,
    Path(account_id): Path<String>,
) -> Result<Json<AccountResponse>, AppError> {
    let account = sqlx::query_as::<_, (String, Option<serde_json::Value>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
        r#"
        SELECT account_id, metadata, created_at, updated_at
        FROM accounts
        WHERE account_id = $1
        "#
    )
    .bind(&account_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::account_not_found(&account_id))?;

    Ok(Json(AccountResponse {
        account_id: account.0,
        metadata: account.1,
        created_at: account.2,
        updated_at: account.3,
    }))
}

pub async fn list_accounts(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<ClientAuth>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<AccountResponse>>, AppError> {
    params.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let limit = params.limit.unwrap_or(20) as i64;

    let accounts = if let Some(cursor) = &params.cursor {
        let decoded = cursor.decode_string()
            .map_err(|e| AppError::InvalidInput(format!("Invalid cursor: {}", e)))?;

        sqlx::query_as::<_, (String, Option<serde_json::Value>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
            r#"
            SELECT account_id, metadata, created_at, updated_at
            FROM accounts
            WHERE account_id > $1
            ORDER BY account_id ASC
            LIMIT $2
            "#
        )
        .bind(&decoded)
        .bind(limit + 1)
        .fetch_all(&state.pool)
        .await?
    } else {
        sqlx::query_as::<_, (String, Option<serde_json::Value>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
            r#"
            SELECT account_id, metadata, created_at, updated_at
            FROM accounts
            ORDER BY account_id ASC
            LIMIT $1
            "#
        )
        .bind(limit + 1)
        .fetch_all(&state.pool)
        .await?
    };

    let has_more = accounts.len() > limit as usize;
    let items: Vec<AccountResponse> = accounts
        .into_iter()
        .take(limit as usize)
        .map(|a| AccountResponse {
            account_id: a.0,
            metadata: a.1,
            created_at: a.2,
            updated_at: a.3,
        })
        .collect();

    let next_cursor = if has_more {
        items.last().map(|item| {
            crate::models::common::Cursor::encode(&item.account_id)
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

pub async fn update_account(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<ClientAuth>,
    Path(account_id): Path<String>,
    Json(req): Json<UpdateAccountRequest>,
) -> Result<Json<AccountResponse>, AppError> {
    let exists = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM accounts WHERE account_id = $1"
    )
    .bind(&account_id)
    .fetch_one(&state.pool)
    .await?;

    if exists == 0 {
        return Err(AppError::account_not_found(&account_id));
    }

    let account = sqlx::query_as::<_, (String, Option<serde_json::Value>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
        r#"
        UPDATE accounts
        SET metadata = $1, updated_at = NOW()
        WHERE account_id = $2
        RETURNING account_id, metadata, created_at, updated_at
        "#
    )
    .bind(&req.metadata)
    .bind(&account_id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(AccountResponse {
        account_id: account.0,
        metadata: account.1,
        created_at: account.2,
        updated_at: account.3,
    }))
}