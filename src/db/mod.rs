use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::time::Duration;
use tracing::info;

pub type PgPool = Pool<Postgres>;

pub async fn init_pool(database_url: &str) -> Result<PgPool> {
    info!("Initializing database connection pool");
    
    let pool = PgPoolOptions::new()
        .max_connections(15)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(database_url)
        .await?;

    info!("Database connection pool initialized successfully");
    
    Ok(pool)
}

pub async fn check_health(pool: &PgPool) -> Result<()> {
    sqlx::query("SELECT 1")
        .fetch_one(pool)
        .await?;

    info!("Database connection pool is healthy");

    Ok(())
}