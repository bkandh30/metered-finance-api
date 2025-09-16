use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub rate_limit_per_minute: u32,
    pub quota_daily_requests: u32,
}

pub fn load_config() -> Result<Config> {
    let config = Config {
        port: std::env::var("PORT")
            .unwrap_or_else(|_| "3030".to_string())
            .parse()?,
        database_url: std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/metered_finance".to_string()),
        rate_limit_per_minute: std::env::var("RATE_LIMIT_PER_MINUTE")
            .unwrap_or_else(|_| "120".to_string())
            .parse()?,
        quota_daily_requests: std::env::var("QUOTA_DAILY_REQUESTS")
            .unwrap_or_else(|_| "5000".to_string())
            .parse()?,
    };

    Ok(config)
}