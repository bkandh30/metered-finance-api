use anyhow::Result;
use axum::{
    Router, 
    routing::get,
    http::HeaderValue,
};
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
    timeout::TimeoutLayer,
    set_header::SetResponseHeaderLayer,
};
use std::time::Duration;

use crate::{
    config::Config,
    handlers::{health, metrics},
    middleware::request_id::RequestIdLayer,
    openapi,
};

pub async fn build_router(_config: Config) -> Result<Router> {
    let metrics_handle = metrics::init_metrics();
    
    let cors = CorsLayer::new()
        .allow_origin("*".parse::<HeaderValue>()?)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    let app = Router::new()
        .route("/health/live", get(health::health_live))
        .route("/health/ready", get(health::health_ready))
        .route("/metrics", get(move || metrics::metrics_handler(metrics_handle.clone())))
        .merge(openapi::openapi_routes())
        .nest("/v1", api_v1_routes())
        .layer(
            ServiceBuilder::new()
                .layer(RequestIdLayer)
                .layer(TraceLayer::new_for_http())
                .layer(cors)
                .layer(TimeoutLayer::new(Duration::from_secs(30)))
                .layer(metrics::create_metrics_layer())
        );
    
    Ok(app)
}

fn api_v1_routes() -> Router {
    Router::new()
        .route("/", get(|| async { "API v1" }))
}