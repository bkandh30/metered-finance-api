use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

pub type PgPool = Pool<Postgres>;

pub async fn init_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(15)
        .acquire_timeout(std::time::Duration::from_secs(10))
        .connect(database_url)
        .await?;

    Ok(pool)
}