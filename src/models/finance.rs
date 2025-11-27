use serde::{Deserialize, Serialize};
use sqlx::Type;
use utoipa::ToSchema;
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, Type, ToSchema, PartialEq, Eq)]
#[sqlx(type_name = "TEXT")]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Payment,
    Refund,
    Payout,
    Transfer,
    Authorization,
    Capture,
    Reversal,
}

impl std::fmt::Display for TransactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionType::Payment => write!(f, "payment"),
            TransactionType::Refund => write!(f, "refund"),
            TransactionType::Payout => write!(f, "payout"),
            TransactionType::Transfer => write!(f, "transfer"),
            TransactionType::Authorization => write!(f, "authorization"),
            TransactionType::Capture => write!(f, "capture"),
            TransactionType::Reversal => write!(f, "reversal"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, ToSchema, PartialEq, Eq)]
#[sqlx(type_name = "TEXT")]
#[serde(rename_all = "lowercase")]
pub enum TransactionStatus {
    Pending,
    Succeeded,
    Failed,
    Reversed,
    Canceled,
}

impl std::fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionStatus::Pending => write!(f, "pending"),
            TransactionStatus::Succeeded => write!(f, "succeeded"),
            TransactionStatus::Failed => write!(f, "failed"),
            TransactionStatus::Reversed => write!(f, "reversed"),
            TransactionStatus::Canceled => write!(f, "canceled"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, ToSchema, PartialEq, Eq)]
#[sqlx(type_name = "TEXT")]
#[serde(rename_all = "snake_case")]
pub enum FailureReason {
    InsufficientFunds,
    CardDeclined,
    RiskBlocked,
    Duplicate,
    InternalError,
}

impl std::fmt::Display for FailureReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FailureReason::InsufficientFunds => write!(f, "insufficient_funds"),
            FailureReason::CardDeclined => write!(f, "card_declined"),
            FailureReason::RiskBlocked => write!(f, "risk_blocked"),
            FailureReason::Duplicate => write!(f, "duplicate"),
            FailureReason::InternalError => write!(f, "internal_error"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, ToSchema, PartialEq, Eq)]
pub struct Currency(pub String);

impl Currency {
    pub fn new(code: &str) -> Result<Self, CurrencyError> {
        if code.len() != 3 || !code.chars().all(|c| c.is_ascii_uppercase()) {
            return Err(CurrencyError::Invalid);
        }
        Ok(Self(code.to_string()))
    }

    pub fn usd() -> Self {
        Self("USD".to_string())
    }

    pub fn eur() -> Self {
        Self("EUR".to_string())
    }

    pub fn gbp() -> Self {
        Self("GBP".to_string())
    }
}

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CurrencyError {
    #[error("Invalid currency code")]
    Invalid,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Account {
    pub account_id: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Transaction {
    pub txn_id: String,
    pub account_id: String,
    pub transaction_type: TransactionType,
    pub status: TransactionStatus,
    pub amount_cents: i64,
    pub currency: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<FailureReason>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTransactionRequest {
    pub account_id: String,
    pub transaction_type: TransactionType,
    pub amount_cents: i64,
    pub currency: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl CreateTransactionRequest {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.account_id.is_empty() {
            return Err(ValidationError::EmptyAccountId);
        }
        
        if self.amount_cents < 0 {
            return Err(ValidationError::NegativeAmount);
        }
        
        Currency::new(&self.currency)
            .map_err(|_| ValidationError::InvalidCurrency)?;
        
        if let Some(desc) = &self.description {
            if desc.len() > 500 {
                return Err(ValidationError::DescriptionTooLong);
            }
        }
        
        Ok(())
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CreateTransactionResponse {
    pub txn_id: String,
    pub account_id: String,
    pub transaction_type: TransactionType,
    pub status: TransactionStatus,
    pub amount_cents: i64,
    pub currency: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct TransactionFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_type: Option<TransactionType>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<TransactionStatus>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(with = "time::serde::rfc3339::option")]
    pub from_timestamp: Option<OffsetDateTime>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(with = "time::serde::rfc3339::option")]
    pub to_timestamp: Option<OffsetDateTime>,
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Account ID cannot be empty")]
    EmptyAccountId,
    
    #[error("Amount cannot be negative")]
    NegativeAmount,
    
    #[error("Invalid currency code")]
    InvalidCurrency,
    
    #[error("Description too long: maximum 500 characters")]
    DescriptionTooLong,
}

pub fn generate_txn_id() -> String {
    format!("txn_{}", uuid::Uuid::new_v4())
}

pub fn generate_account_id() -> String {
    format!("acc_{}", uuid::Uuid::new_v4())
}