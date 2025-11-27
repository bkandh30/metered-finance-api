use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct QuotaUsage {
    pub key_id: String,
    pub usage_date: NaiveDate,
    pub request_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct QuotaLimits {
    pub rate_limit_per_minute: i32,
    pub daily_quota: i32,
    pub monthly_quota: i32,
}

impl Default for QuotaLimits {
    fn default() -> Self {
        Self {
            rate_limit_per_minute: 60,
            daily_quota: 10_000,
            monthly_quota: 300_000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct QuotaStatus {
    pub key_id: String,
    pub limits: QuotaLimits,
    pub usage: QuotaUsageStats,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct QuotaUsageStats {
    pub today: i32,
    pub this_month: i32,
    pub daily_remaining: i32,
    pub monthly_remaining: i32,
}

pub struct QuotaService;

impl QuotaService {
    pub async fn increment_usage(
        pool: &PgPool,
        key_id: &str,
    ) -> Result<i32, sqlx::Error> {
        let result = sqlx::query_scalar::<_, i32>(
            "SELECT increment_quota_usage($1, CURRENT_DATE)"
        )
        .bind(key_id)
        .fetch_one(pool)
        .await?;

        Ok(result)
    }

    pub async fn get_usage(
        pool: &PgPool,
        key_id: &str,
    ) -> Result<QuotaUsageStats, sqlx::Error> {
        let today = sqlx::query_scalar::<_, Option<i32>>(
            r#"
            SELECT request_count
            FROM quota_usage
            WHERE key_id = $1 AND usage_date = CURRENT_DATE
            "#
        )
        .bind(key_id)
        .fetch_optional(pool)
        .await?
        .flatten()
        .unwrap_or(0);

        let this_month = sqlx::query_scalar::<_, Option<i32>>(
            r#"
            SELECT COALESCE(SUM(request_count), 0)
            FROM quota_usage
            WHERE key_id = $1
            AND usage_date >= DATE_TRUNC('month', CURRENT_DATE)
            AND usage_date < DATE_TRUNC('month', CURRENT_DATE) + INTERVAL '1 month'
            "#
        )
        .bind(key_id)
        .fetch_optional(pool)
        .await?
        .flatten()
        .unwrap_or(0);

        let limits = Self::get_limits(pool, key_id).await?;

        Ok(QuotaUsageStats {
            today,
            this_month,
            daily_remaining: (limits.daily_quota - today).max(0),
            monthly_remaining: (limits.monthly_quota - this_month).max(0),
        })
    }

    pub async fn get_limits(
        pool: &PgPool,
        key_id: &str,
    ) -> Result<QuotaLimits, sqlx::Error> {
        let result = sqlx::query_as::<_, (i32, i32, i32)>(
            r#"
            SELECT rate_limit_per_minute, daily_quota, monthly_quota
            FROM api_keys
            WHERE key_id = $1
            "#
        )
        .bind(key_id)
        .fetch_optional(pool)
        .await?;

        match result {
            Some((rate_limit, daily, monthly)) => Ok(QuotaLimits {
                rate_limit_per_minute: rate_limit,
                daily_quota: daily,
                monthly_quota: monthly,
            }),
            None => Ok(QuotaLimits::default()),
        }
    }

    pub async fn check_daily_quota(
        pool: &PgPool,
        key_id: &str,
    ) -> Result<bool, sqlx::Error> {
        let usage = Self::get_usage(pool, key_id).await?;
        let limits = Self::get_limits(pool, key_id).await?;
        
        Ok(usage.today < limits.daily_quota)
    }

    pub async fn check_monthly_quota(
        pool: &PgPool,
        key_id: &str,
    ) -> Result<bool, sqlx::Error> {
        let usage = Self::get_usage(pool, key_id).await?;
        let limits = Self::get_limits(pool, key_id).await?;
        
        Ok(usage.this_month < limits.monthly_quota)
    }

    pub async fn get_status(
        pool: &PgPool,
        key_id: &str,
    ) -> Result<QuotaStatus, sqlx::Error> {
        let limits = Self::get_limits(pool, key_id).await?;
        let usage = Self::get_usage(pool, key_id).await?;

        Ok(QuotaStatus {
            key_id: key_id.to_string(),
            limits,
            usage,
        })
    }
}

pub struct RateLimitService;

impl RateLimitService {
    pub async fn check_rate_limit(
        pool: &PgPool,
        key_id: &str,
        limit: i32,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar::<_, bool>(
            "SELECT check_rate_limit($1, $2, 1)"
        )
        .bind(key_id)
        .bind(limit)
        .fetch_one(pool)
        .await?;

        Ok(result)
    }

    pub async fn cleanup(pool: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query("SELECT cleanup_rate_limits()")
            .execute(pool)
            .await?;
        Ok(())
    }
}