use anyhow::Result;
use axum::{
    http::HeaderValue,
    routing::get,
    Router,
};
use std::time::Duration;
use tower_http::{
    cors::CorsLayer, 
    timeout::TimeoutLayer, 
    trace::TraceLayer
};

use crate::handlers::{
    health, 
    metrics, 
    metrics::Metrics
};
use crate::{
    config::Config, 
    middleware::request_id::request_id_layers, 
    openapi
};

pub async fn build_router(_config: Config) -> Result<Router> {
    let Metrics {
        router: metrics_router,
        layer: prom_layer,
    } = metrics::init();

    let cors = CorsLayer::new()
        .allow_origin("*".parse::<HeaderValue>()?)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    let (propagate_xrid, set_xrid) = request_id_layers();

    let app = Router::new()
        .route("/health/live", get(health::health_live))
        .route("/health/ready", get(health::health_ready))
        .merge(metrics_router)
        .merge(openapi::openapi_routes())
        .nest("/v1", api_v1_routes())
        .layer(prom_layer)
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(cors)
        .layer(set_xrid)
        .layer(propagate_xrid)
        .layer(TraceLayer::new_for_http());

    Ok(app)
}

fn api_v1_routes() -> Router {
    Router::new().route("/", get(|| async { "API v1" }))
}
