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
    db::PgPool,
    middleware::request_id::request_id_layers, 
    openapi
};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Config,
}

pub async fn build_router(config: Config) -> Result<Router> {
    let pool = crate::db::init_pool(&config.database_url).await?;

    let state = Arc::new(AppState {
        pool: pool.clone(),
        config: config.clone(),
    });
    
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
        .route("/health/ready", get({
            let state = state.clone();
            move |State(app_state): State<Arc<AppState>>| health::health_ready(State(app_state))
        }))
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
