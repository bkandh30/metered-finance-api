use axum::{
    middleware,
    routing::{delete, get, patch, post},
    Router,
};
use std::sync::Arc;

use crate::{
    app::AppState,
    handlers::{accounts, health, keys, transactions, usage},
    middleware::{
        auth::{require_admin_auth, require_client_auth},
        rate_limit::check_rate_limit_and_quota,
    },
};


pub fn build_routes(state: Arc<AppState>) -> Router {
    // Health check routes (no authentication)
    let health_routes = Router::new()
        .route("/health/live", get(health::health_live))
        .route("/health/ready", get(health::health_ready));
    
    // Client routes (require authentication + rate limiting)
    let client_routes = Router::new()
        .route("/accounts", post(accounts::create_account))
        .route("/accounts", get(accounts::list_accounts))
        .route("/accounts/:account_id", get(accounts::get_account))
        .route("/accounts/:account_id", patch(accounts::update_account))

        .route("/transactions", post(transactions::create_transaction))
        .route("/transactions", get(transactions::list_transactions))
        .route("/transactions/:transaction_id", get(transactions::get_transaction))
        .route("/accounts/:account_id/transactions", get(transactions::get_account_transactions))
        .route("/accounts/:account_id/balance", get(transactions::get_account_balance))

        .route("/usage", get(usage::get_own_usage))

        .layer(middleware::from_fn_with_state(
            state.clone(),
            require_client_auth,
        ))
        
        .layer(middleware::from_fn_with_state(
            state.clone(),
            check_rate_limit_and_quota,
        ));

    // Admin routes (require admin authentication, no rate limiting)
    let admin_routes = Router::new()
        .route("/keys", post(keys::create_api_key))
        .route("/keys", get(keys::list_api_keys))
        .route("/keys/:key_id", get(keys::get_api_key))
        .route("/keys/:key_id", patch(keys::update_api_key))
        .route("/keys/:key_id", delete(keys::delete_api_key))
        
        .route("/usage/:key_id", get(usage::get_key_usage))

        .layer(middleware::from_fn(require_admin_auth));

    // Combine routes
    Router::new()
        .merge(health_routes)
        .nest("/api", client_routes)
        .nest("/api/admin", admin_routes)
        .with_state(state)
}