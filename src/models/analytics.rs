use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RequestStats {
    pub total_requests: i64,
    
    pub successful_requests: i64,
    
    pub failed_requests: i64,
    
    pub avg_latency_ms: f64,
    
    pub median_latency_ms: Option<f64>,
    
    pub p95_latency_ms: Option<f64>,
    
    pub p99_latency_ms: Option<f64>,
    
    #[schema(value_type = String, format = DateTime)]
    pub period_start: DateTime<Utc>,
    
    #[schema(value_type = String, format = DateTime)]
    pub period_end: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EndpointStats {
    pub path: String,
    
    pub method: String,
    
    pub request_count: i64,
    
    pub avg_latency_ms: f64,
    
    pub error_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StatusCodeStats {
    pub status_code: i32,
    
    pub count: i64,
    
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HourlyVolume {
    #[schema(value_type = String, format = DateTime)]
    pub hour: DateTime<Utc>,
    
    pub request_count: i64,
    
    pub avg_latency_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AnalyticsResponse {
    pub overview: RequestStats,
    
    pub top_endpoints: Vec<EndpointStats>,
    
    pub status_codes: Vec<StatusCodeStats>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hourly_volume: Option<Vec<HourlyVolume>>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct TimeRangeFilter {
    #[serde(default)]
    #[schema(value_type = Option<String>, format = DateTime)]
    pub start: Option<DateTime<Utc>>,
    
    #[serde(default)]
    #[schema(value_type = Option<String>, format = DateTime)]
    pub end: Option<DateTime<Utc>>,
}

impl Default for TimeRangeFilter {
    fn default() -> Self {
        Self {
            start: Some(Utc::now() - chrono::Duration::days(7)),
            end: Some(Utc::now()),
        }
    }
}

pub struct AnalyticsService;

impl AnalyticsService {
    pub async fn get_request_stats(
        pool: &sqlx::PgPool,
        key_id: Option<&str>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<RequestStats, sqlx::Error> {
        let stats = if let Some(key_id) = key_id {
            sqlx::query_as::<_, (i64, i64, i64, Option<f64>)>(
                r#"
                SELECT 
                    COUNT(*) as total_requests,
                    COUNT(*) FILTER (WHERE status >= 200 AND status < 300) as successful_requests,
                    COUNT(*) FILTER (WHERE status >= 400) as failed_requests,
                    AVG(latency_ms) as avg_latency_ms
                FROM requests
                WHERE key_id = $1 
                    AND ts >= $2 
                    AND ts <= $3
                "#
            )
            .bind(key_id)
            .bind(start)
            .bind(end)
            .fetch_one(pool)
            .await?
        } else {
            sqlx::query_as::<_, (i64, i64, i64, Option<f64>)>(
                r#"
                SELECT 
                    COUNT(*) as total_requests,
                    COUNT(*) FILTER (WHERE status >= 200 AND status < 300) as successful_requests,
                    COUNT(*) FILTER (WHERE status >= 400) as failed_requests,
                    AVG(latency_ms) as avg_latency_ms
                FROM requests
                WHERE ts >= $1 
                    AND ts <= $2
                "#
            )
            .bind(start)
            .bind(end)
            .fetch_one(pool)
            .await?
        };

        Ok(RequestStats {
            total_requests: stats.0,
            successful_requests: stats.1,
            failed_requests: stats.2,
            avg_latency_ms: stats.3.unwrap_or(0.0),
            median_latency_ms: None, // TODO: Calculate with percentile_cont
            p95_latency_ms: None,
            p99_latency_ms: None,
            period_start: start,
            period_end: end,
        })
    }

    pub async fn get_endpoint_stats(
        pool: &sqlx::PgPool,
        key_id: Option<&str>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<EndpointStats>, sqlx::Error> {
        let stats = if let Some(key_id) = key_id {
            sqlx::query_as::<_, (String, String, i64, Option<f64>, i64)>(
                r#"
                SELECT 
                    path,
                    method,
                    COUNT(*) as request_count,
                    AVG(latency_ms) as avg_latency_ms,
                    COUNT(*) FILTER (WHERE status >= 400) as error_count
                FROM requests
                WHERE key_id = $1 
                    AND ts >= $2 
                    AND ts <= $3
                GROUP BY path, method
                ORDER BY request_count DESC
                LIMIT $4
                "#
            )
            .bind(key_id)
            .bind(start)
            .bind(end)
            .bind(limit)
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as::<_, (String, String, i64, Option<f64>, i64)>(
                r#"
                SELECT 
                    path,
                    method,
                    COUNT(*) as request_count,
                    AVG(latency_ms) as avg_latency_ms,
                    COUNT(*) FILTER (WHERE status >= 400) as error_count
                FROM requests
                WHERE ts >= $1 
                    AND ts <= $2
                GROUP BY path, method
                ORDER BY request_count DESC
                LIMIT $3
                "#
            )
            .bind(start)
            .bind(end)
            .bind(limit)
            .fetch_all(pool)
            .await?
        };

        Ok(stats
            .into_iter()
            .map(|(path, method, count, avg_latency, error_count)| {
                let error_rate = if count > 0 {
                    (error_count as f64 / count as f64) * 100.0
                } else {
                    0.0
                };
                
                EndpointStats {
                    path,
                    method,
                    request_count: count,
                    avg_latency_ms: avg_latency.unwrap_or(0.0),
                    error_rate,
                }
            })
            .collect())
    }

    pub async fn get_status_code_stats(
        pool: &sqlx::PgPool,
        key_id: Option<&str>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<StatusCodeStats>, sqlx::Error> {
        let total_count = if let Some(key_id) = key_id {
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM requests WHERE key_id = $1 AND ts >= $2 AND ts <= $3"
            )
            .bind(key_id)
            .bind(start)
            .bind(end)
            .fetch_one(pool)
            .await?
        } else {
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM requests WHERE ts >= $1 AND ts <= $2"
            )
            .bind(start)
            .bind(end)
            .fetch_one(pool)
            .await?
        };

        let stats = if let Some(key_id) = key_id {
            sqlx::query_as::<_, (i32, i64)>(
                r#"
                SELECT status, COUNT(*) as count
                FROM requests
                WHERE key_id = $1 
                    AND ts >= $2 
                    AND ts <= $3
                GROUP BY status
                ORDER BY count DESC
                "#
            )
            .bind(key_id)
            .bind(start)
            .bind(end)
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as::<_, (i32, i64)>(
                r#"
                SELECT status, COUNT(*) as count
                FROM requests
                WHERE ts >= $1 
                    AND ts <= $2
                GROUP BY status
                ORDER BY count DESC
                "#
            )
            .bind(start)
            .bind(end)
            .fetch_all(pool)
            .await?
        };

        Ok(stats
            .into_iter()
            .map(|(status, count)| {
                let percentage = if total_count > 0 {
                    (count as f64 / total_count as f64) * 100.0
                } else {
                    0.0
                };
                
                StatusCodeStats {
                    status_code: status,
                    count,
                    percentage,
                }
            })
            .collect())
    }

    pub async fn get_hourly_volume(
        pool: &sqlx::PgPool,
        key_id: Option<&str>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<HourlyVolume>, sqlx::Error> {
        let stats = if let Some(key_id) = key_id {
            sqlx::query_as::<_, (DateTime<Utc>, i64, Option<f64>)>(
                r#"
                SELECT 
                    date_trunc('hour', ts) as hour,
                    COUNT(*) as request_count,
                    AVG(latency_ms) as avg_latency_ms
                FROM requests
                WHERE key_id = $1 
                    AND ts >= $2 
                    AND ts <= $3
                GROUP BY hour
                ORDER BY hour ASC
                "#
            )
            .bind(key_id)
            .bind(start)
            .bind(end)
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as::<_, (DateTime<Utc>, i64, Option<f64>)>(
                r#"
                SELECT 
                    date_trunc('hour', ts) as hour,
                    COUNT(*) as request_count,
                    AVG(latency_ms) as avg_latency_ms
                FROM requests
                WHERE ts >= $1 
                    AND ts <= $2
                GROUP BY hour
                ORDER BY hour ASC
                "#
            )
            .bind(start)
            .bind(end)
            .fetch_all(pool)
            .await?
        };

        Ok(stats
            .into_iter()
            .map(|(hour, count, avg_latency)| HourlyVolume {
                hour,
                request_count: count,
                avg_latency_ms: avg_latency.unwrap_or(0.0),
            })
            .collect())
    }
}