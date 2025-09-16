use anyhow::Result;
use axum::{Router, routing::get};
use tower_http::trace::TraceLayer;

use crate::config::Config;

pub async fn build_router(_config: Config) -> Result<Router> {
    let app = Router::new()
        .route("/health/live", get(health_live))
        .layer(TraceLayer::new_for_http());
    
    Ok(app)
}

async fn health_live() -> &'static str {
    r#"{"status":"live"}"#
}