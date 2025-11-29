use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use std::time::Instant;

use crate::{
    app::AppState,
    middleware::auth::ClientAuth,
    models::keys::AuthContext,
};

pub async fn log_request(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    
    let auth_context = request.extensions().get::<ClientAuth>().map(|a| &a.context);
    
    let key_id = auth_context.and_then(|ctx| match ctx {
        AuthContext::Client { key_id, .. } => Some(key_id.clone()),
        AuthContext::Admin => None,
    });
    
    let response = next.run(request).await;
    
    let latency_ms = start.elapsed().as_millis() as i32;
    let status = response.status().as_u16() as i32;
    
    let pool = state.pool.clone();
    let path_clone = path.clone();
    let method_clone = method.clone();
    
    tokio::spawn(async move {
        if let Err(e) = log_request_to_db(
            &pool,
            key_id.as_deref(),
            None,
            &path_clone,
            &method_clone,
            status,
            latency_ms,
        )
        .await
        {
            tracing::error!("Failed to log request: {}", e);
        }
    });
    
    response
}

async fn log_request_to_db(
    pool: &sqlx::PgPool,
    key_id: Option<&str>,
    account_id: Option<&str>,
    path: &str,
    method: &str,
    status: i32,
    latency_ms: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO requests (key_id, account_id, path, method, status, latency_ms, ts, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
        "#
    )
    .bind(key_id)
    .bind(account_id)
    .bind(path)
    .bind(method)
    .bind(status)
    .bind(latency_ms)
    .execute(pool)
    .await?;
    
    Ok(())
}

pub async fn extract_account_context(
    request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path();
    
    let _account_id = if let Some(captures) = extract_account_from_path(path) {
        Some(captures.to_string())
    } else {
        None
    };
    
    next.run(request).await
}

pub fn extract_account_from_path(path: &str) -> Option<&str> {
    if path.starts_with("/api/accounts/") {
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() >= 4 {
            return Some(parts[3]);
        }
    }
    
    None
}