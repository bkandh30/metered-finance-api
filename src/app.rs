use anyhow::Result;
use axum::{
    http::HeaderValue,
    routing::get,
    Router,
};
use std::sync::Arc;
use std::time::Duration;
use tower_http::{cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer};

use crate::handlers::health;
use crate::{config::Config, db::PgPool, middleware::request_id::request_id_layers, openapi};

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

    let cors = CorsLayer::new()
        .allow_origin("*".parse::<HeaderValue>()?)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    let (propagate_xrid, set_xrid) = request_id_layers();

    let openapi_router = openapi::openapi_routes().with_state::<Arc<AppState>>(());
    let v1_router = api_v1_routes().with_state::<Arc<AppState>>(());

    let app = Router::new()
        .with_state::<Arc<AppState>>(())
        .merge(openapi_router)
        .nest("/v1", v1_router)
        .route(
            "/health/live",
            get(health::health_live).with_state::<Arc<AppState>>(()),
        )
        .route("/health/ready", get(health::health_ready))
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(cors)
        .layer(set_xrid)
        .layer(propagate_xrid)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    Ok(app)
}

fn api_v1_routes() -> Router {
    Router::new().route("/", get(|| async { "API v1" }))
}