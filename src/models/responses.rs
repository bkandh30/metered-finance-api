use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::finance::{Currency, TransactionStatus, TransactionType};
use super::keys::Scope;
use super::quota::{QuotaLimits, QuotaUsageStats};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AccountResponse {
    pub account_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TransactionResponse {
    pub transaction_id: String,
    pub account_id: String,
    pub amount: f64,
    pub currency: Currency,
    pub transaction_type: TransactionType,
    pub status: TransactionStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub processed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BalanceResponse {
    pub account_id: String,
    pub balance: f64,
    pub currency: Currency,
    #[schema(value_type = String, format = DateTime)]
    pub as_of: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct KeyCreatedResponse {
    pub key_id: String,
    #[schema(example = "sk_live_1234567890abcdef")]
    pub api_key: String,
    pub prefix: String,
    pub name: String,
    pub scopes: Vec<Scope>,
    pub active: bool,
    pub rate_limit_per_minute: i32,
    pub daily_quota: i32,
    pub monthly_quota: i32,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct KeyInfoResponse {
    pub key_id: String,
    pub prefix: String,
    pub name: String,
    pub scopes: Vec<Scope>,
    pub active: bool,
    pub rate_limit_per_minute: i32,
    pub daily_quota: i32,
    pub monthly_quota: i32,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UsageResponse {
    pub key_id: String,
    pub limits: QuotaLimits,
    pub usage: QuotaUsageStats,
}