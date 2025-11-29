use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use std::sync::Arc;
use utoipa;

use crate::{
    app::AppState,
    middleware::{auth::ClientAuth, errors::AppError},
    models::{
        common::{ErrorResponse, PaginatedResponse, PaginationParams},
        requests::{CreateAccountRequest, UpdateAccountRequest},
        responses::AccountResponse,
    },
};

/// Create a new account
///
/// Creates a new account with the specified account ID and optional metadata.
/// The account ID must be unique across the system.
#[utoipa::path(
    post,
    path = "/api/accounts",
    tag = "accounts",
    request_body = CreateAccountRequest,
    responses(
        (status = 201, description = "Account created successfully", body = AccountResponse),
        (status = 400, description = "Invalid input", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 409, description = "Account already exists", body = ErrorResponse),
        (status = 429, description = "Rate limit exceeded", body = ErrorResponse),
    ),
    security(
        ("ApiKeyAuth" = [])
    )
)]
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

/// Get account details
///
/// Retrieves detailed information about a specific account by its ID.
#[utoipa::path(
    get,
    path = "/api/accounts/{account_id}",
    tag = "accounts",
    params(
        ("account_id" = String, Path, description = "Account identifier")
    ),
    responses(
        (status = 200, description = "Account found", body = AccountResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Account not found", body = ErrorResponse),
        (status = 429, description = "Rate limit exceeded", body = ErrorResponse),
    ),
    security(
        ("ApiKeyAuth" = [])
    )
)]
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

/// List all accounts
///
/// Retrieves a paginated list of all accounts. Use the cursor parameter for pagination.
#[utoipa::path(
    get,
    path = "/api/accounts",
    tag = "accounts",
    params(
        PaginationParams
    ),
    responses(
        (status = 200, description = "List of accounts", body = PaginatedResponse<AccountResponse>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 429, description = "Rate limit exceeded", body = ErrorResponse),
    ),
    security(
        ("ApiKeyAuth" = [])
    )
)]
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

/// Update account metadata
///
/// Updates the metadata for an existing account. The metadata can contain any valid JSON.
#[utoipa::path(
    patch,
    path = "/api/accounts/{account_id}",
    tag = "accounts",
    params(
        ("account_id" = String, Path, description = "Account identifier")
    ),
    request_body = UpdateAccountRequest,
    responses(
        (status = 200, description = "Account updated successfully", body = AccountResponse),
        (status = 400, description = "Invalid input", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Account not found", body = ErrorResponse),
        (status = 429, description = "Rate limit exceeded", body = ErrorResponse),
    ),
    security(
        ("ApiKeyAuth" = [])
    )
)]
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