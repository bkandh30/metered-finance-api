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
        finance::{TransactionFilters, TransactionStatus},
        requests::CreateTransactionRequest,
        responses::{BalanceResponse, TransactionResponse},
    },
};

pub async fn create_transaction(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<ClientAuth>,
    Json(req): Json<CreateTransactionRequest>,
) -> Result<(StatusCode, Json<TransactionResponse>), AppError> {
    req.validate()
        .map_err(|e| AppError::ValidationError(e))?;

    let account_exists = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM accounts WHERE account_id = $1"
    )
    .bind(&req.account_id)
    .fetch_one(&state.pool)
    .await?;

    if account_exists == 0 {
        return Err(AppError::account_not_found(&req.account_id));
    }

    let transaction_id = crate::models::finance::generate_transaction_id();

    let transaction = sqlx::query_as::<_, (
        String,
        String,
        f64,
        String,
        String,
        String,
        Option<String>,
        Option<serde_json::Value>,
        chrono::DateTime<chrono::Utc>,
        Option<chrono::DateTime<chrono::Utc>>,
    )>(
        r#"
        INSERT INTO transactions (
            transaction_id, account_id, amount, currency, 
            transaction_type, status, description, metadata,
            created_at, processed_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
        RETURNING 
            transaction_id, account_id, amount, currency,
            transaction_type, status, description, metadata,
            created_at, processed_at
        "#
    )
    .bind(&transaction_id)
    .bind(&req.account_id)
    .bind(req.amount)
    .bind(req.currency.to_string())
    .bind(req.transaction_type.to_string())
    .bind(TransactionStatus::Completed.to_string())
    .bind(&req.description)
    .bind(&req.metadata)
    .fetch_one(&state.pool)
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(TransactionResponse {
            transaction_id: transaction.0,
            account_id: transaction.1,
            amount: transaction.2,
            currency: transaction.3.parse().unwrap_or_default(),
            transaction_type: transaction.4.parse().unwrap_or_default(),
            status: transaction.5.parse().unwrap_or_default(),
            description: transaction.6,
            metadata: transaction.7,
            created_at: transaction.8,
            processed_at: transaction.9,
        }),
    ))
}

pub async fn get_transaction(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<ClientAuth>,
    Path(transaction_id): Path<String>,
) -> Result<Json<TransactionResponse>, AppError> {
    let transaction = sqlx::query_as::<_, (
        String,
        String,
        f64,
        String,
        String,
        String,
        Option<String>,
        Option<serde_json::Value>,
        chrono::DateTime<chrono::Utc>,
        Option<chrono::DateTime<chrono::Utc>>,
    )>(
        r#"
        SELECT 
            transaction_id, account_id, amount, currency,
            transaction_type, status, description, metadata,
            created_at, processed_at
        FROM transactions
        WHERE transaction_id = $1
        "#
    )
    .bind(&transaction_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::transaction_not_found(&transaction_id))?;

    Ok(Json(TransactionResponse {
        transaction_id: transaction.0,
        account_id: transaction.1,
        amount: transaction.2,
        currency: transaction.3.parse().unwrap_or_default(),
        transaction_type: transaction.4.parse().unwrap_or_default(),
        status: transaction.5.parse().unwrap_or_default(),
        description: transaction.6,
        metadata: transaction.7,
        created_at: transaction.8,
        processed_at: transaction.9,
    }))
}

pub async fn list_transactions(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<ClientAuth>,
    Query(mut params): Query<PaginationParams>,
    Query(filters): Query<TransactionFilters>,
) -> Result<Json<PaginatedResponse<TransactionResponse>>, AppError> {
    params.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let limit = params.limit.unwrap_or(20) as i64;

    let mut query = String::from(
        r#"
        SELECT 
            transaction_id, account_id, amount, currency,
            transaction_type, status, description, metadata,
            created_at, processed_at
        FROM transactions
        WHERE 1=1
        "#
    );

    let mut bind_values: Vec<String> = vec![];
    let mut param_count = 0;

    if let Some(account_id) = &filters.account_id {
        param_count += 1;
        query.push_str(&format!(" AND account_id = ${}", param_count));
        bind_values.push(account_id.clone());
    }

    if let Some(status) = &filters.status {
        param_count += 1;
        query.push_str(&format!(" AND status = ${}", param_count));
        bind_values.push(status.to_string());
    }

    if let Some(txn_type) = &filters.transaction_type {
        param_count += 1;
        query.push_str(&format!(" AND transaction_type = ${}", param_count));
        bind_values.push(txn_type.to_string());
    }

    if let Some(currency) = &filters.currency {
        param_count += 1;
        query.push_str(&format!(" AND currency = ${}", param_count));
        bind_values.push(currency.to_string());
    }

    if let Some(start) = &filters.created_after {
        param_count += 1;
        query.push_str(&format!(" AND created_at >= ${}", param_count));
        bind_values.push(start.to_rfc3339());
    }

    if let Some(end) = &filters.created_before {
        param_count += 1;
        query.push_str(&format!(" AND created_at <= ${}", param_count));
        bind_values.push(end.to_rfc3339());
    }

    if let Some(cursor) = &params.cursor {
        let decoded = cursor.decode_string()
            .map_err(|e| AppError::InvalidInput(format!("Invalid cursor: {}", e)))?;
        param_count += 1;
        query.push_str(&format!(" AND transaction_id > ${}", param_count));
        bind_values.push(decoded);
    }

    param_count += 1;
    query.push_str(&format!(" ORDER BY transaction_id ASC LIMIT ${}", param_count));
    bind_values.push((limit + 1).to_string());

    let mut sql_query = sqlx::query_as::<_, (
        String,
        String,
        f64,
        String,
        String,
        String,
        Option<String>,
        Option<serde_json::Value>,
        chrono::DateTime<chrono::Utc>,
        Option<chrono::DateTime<chrono::Utc>>,
    )>(&query);

    for value in &bind_values {
        sql_query = sql_query.bind(value);
    }

    let transactions = sql_query.fetch_all(&state.pool).await?;

    let has_more = transactions.len() > limit as usize;
    let items: Vec<TransactionResponse> = transactions
        .into_iter()
        .take(limit as usize)
        .map(|t| TransactionResponse {
            transaction_id: t.0,
            account_id: t.1,
            amount: t.2,
            currency: t.3.parse().unwrap_or_default(),
            transaction_type: t.4.parse().unwrap_or_default(),
            status: t.5.parse().unwrap_or_default(),
            description: t.6,
            metadata: t.7,
            created_at: t.8,
            processed_at: t.9,
        })
        .collect();

    let next_cursor = if has_more {
        items.last().map(|item| {
            crate::models::common::Cursor::encode(&item.transaction_id)
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

pub async fn get_account_transactions(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<ClientAuth>,
    Path(account_id): Path<String>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<TransactionResponse>>, AppError> {
    let account_exists = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM accounts WHERE account_id = $1"
    )
    .bind(&account_id)
    .fetch_one(&state.pool)
    .await?;

    if account_exists == 0 {
        return Err(AppError::account_not_found(&account_id));
    }

    let mut filters = TransactionFilters::default();
    filters.account_id = Some(account_id);

    list_transactions(
        State(state),
        Extension(_auth),
        Query(params),
        Query(filters),
    )
    .await
}

pub async fn get_account_balance(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<ClientAuth>,
    Path(account_id): Path<String>,
) -> Result<Json<BalanceResponse>, AppError> {
    let account_exists = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM accounts WHERE account_id = $1"
    )
    .bind(&account_id)
    .fetch_one(&state.pool)
    .await?;

    if account_exists == 0 {
        return Err(AppError::account_not_found(&account_id));
    }

    let balance = sqlx::query_as::<_, (Option<f64>, Option<String>)>(
        r#"
        SELECT 
            COALESCE(SUM(amount), 0.0) as balance,
            MAX(currency) as currency
        FROM transactions
        WHERE account_id = $1 AND status = 'completed'
        "#
    )
    .bind(&account_id)
    .fetch_one(&state.pool)
    .await?;

    let balance_amount = balance.0.unwrap_or(0.0);
    let currency = balance.1.unwrap_or_else(|| "USD".to_string());

    Ok(Json(BalanceResponse {
        account_id,
        balance: balance_amount,
        currency: currency.parse().unwrap_or_default(),
        as_of: chrono::Utc::now(),
    }))
}