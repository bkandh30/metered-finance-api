use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

use crate::{
    app::AppState,
    models::keys::{ApiKeyGenerator, AuthContext, Scope},
};

#[derive(Debug, Clone)]
pub struct ClientAuth {
    pub context: AuthContext,
}

impl ClientAuth {
    pub async fn from_request(
        state: &Arc<AppState>,
        headers: &axum::http::HeaderMap,
    ) -> Result<Self, (StatusCode, String)> {
        let api_key = headers
            .get("X-Api-Key")
            .and_then(|h| h.to_str().ok())
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "Missing X-Api-Key header".to_string(),
            ))?;

        let prefix = ApiKeyGenerator::extract_prefix(api_key).ok_or((
            StatusCode::UNAUTHORIZED,
            "Invalid API key format".to_string(),
        ))?;

        let result = sqlx::query_as::<_, (String, String, Vec<String>, bool)>(
            r#"
            SELECT key_id, secret_hash, scopes, active
            FROM api_keys
            WHERE prefix = $1 AND active = TRUE
            "#,
        )
        .bind(&prefix)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error during API key lookup: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Authentication failed".to_string(),
            )
        })?;

        let (key_id, secret_hash, scopes_raw, active) = result.ok_or((
            StatusCode::UNAUTHORIZED,
            "Invalid API key".to_string(),
        ))?;

        if !ApiKeyGenerator::verify_secret(api_key, &secret_hash) {
            return Err((
                StatusCode::UNAUTHORIZED,
                "Invalid API key".to_string(),
            ));
        }

        if !active {
            return Err((
                StatusCode::UNAUTHORIZED,
                "API key is inactive".to_string(),
            ));
        }

        let scopes: Vec<Scope> = scopes_raw
            .iter()
            .filter_map(|s| Scope::from_str(s))
            .collect();

        let _ = sqlx::query(
            r#"
            UPDATE api_keys
            SET last_used_at = NOW()
            WHERE key_id = $1
            "#,
        )
        .bind(&key_id)
        .execute(&state.pool)
        .await;

        Ok(ClientAuth {
            context: AuthContext::Client {
                key_id,
                scopes,
            },
        })
    }
}

#[derive(Debug, Clone)]
pub struct AdminAuth {
    pub context: AuthContext,
}

impl AdminAuth {
    pub fn from_request(headers: &axum::http::HeaderMap) -> Result<Self, (StatusCode, String)> {
        let admin_key = headers
            .get("X-Admin-Key")
            .and_then(|h| h.to_str().ok())
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "Missing X-Admin-Key header".to_string(),
            ))?;

        let expected_admin_key = std::env::var("ADMIN_KEY").map_err(|_| {
            tracing::error!("ADMIN_KEY not configured");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Server configuration error".to_string(),
            )
        })?;

        if admin_key != expected_admin_key {
            return Err((
                StatusCode::UNAUTHORIZED,
                "Invalid admin key".to_string(),
            ));
        }

        Ok(AdminAuth {
            context: AuthContext::Admin,
        })
    }
}

#[derive(Debug, Clone)]
pub struct OptionalClientAuth {
    pub context: Option<AuthContext>,
}

impl OptionalClientAuth {
    pub async fn from_request(
        state: &Arc<AppState>,
        headers: &axum::http::HeaderMap,
    ) -> Result<Self, (StatusCode, String)> {
        if headers.get("X-Api-Key").is_none() {
            return Ok(OptionalClientAuth { context: None });
        }

        match ClientAuth::from_request(state, headers).await {
            Ok(auth) => Ok(OptionalClientAuth {
                context: Some(auth.context),
            }),
            Err(e) => Err(e),
        }
    }
}

pub async fn require_client_auth(
    State(state): State<Arc<AppState>>,
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    let auth = ClientAuth::from_request(&state, req.headers()).await?;
    req.extensions_mut().insert(auth);
    Ok(next.run(req).await)
}

pub async fn require_admin_auth(
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    let auth = AdminAuth::from_request(req.headers())?;
    req.extensions_mut().insert(auth);
    Ok(next.run(req).await)
}

pub fn require_scope(context: &AuthContext, scope: &Scope) -> Result<(), (StatusCode, String)> {
    if context.has_scope(scope) {
        Ok(())
    } else {
        Err((
            StatusCode::FORBIDDEN,
            format!("Missing required scope: {}", scope),
        ))
    }
}