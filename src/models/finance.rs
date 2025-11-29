use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    Payment,
    Refund,
    Adjustment,
    Fee,
    Payout,
    Chargeback,
    Transfer,
}

impl Default for TransactionType {
    fn default() -> Self {
        TransactionType::Payment
    }
}

impl std::str::FromStr for TransactionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "payment" => Ok(TransactionType::Payment),
            "refund" => Ok(TransactionType::Refund),
            "adjustment" => Ok(TransactionType::Adjustment),
            "fee" => Ok(TransactionType::Fee),
            "payout" => Ok(TransactionType::Payout),
            "chargeback" => Ok(TransactionType::Chargeback),
            "transfer" => Ok(TransactionType::Transfer),
            _ => Err(format!("Invalid transaction type: {}", s)),
        }
    }
}

impl std::fmt::Display for TransactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionType::Payment => write!(f, "payment"),
            TransactionType::Refund => write!(f, "refund"),
            TransactionType::Adjustment => write!(f, "adjustment"),
            TransactionType::Fee => write!(f, "fee"),
            TransactionType::Payout => write!(f, "payout"),
            TransactionType::Chargeback => write!(f, "chargeback"),
            TransactionType::Transfer => write!(f, "transfer"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TransactionStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

impl Default for TransactionStatus {
    fn default() -> Self {
        TransactionStatus::Pending
    }
}

impl std::str::FromStr for TransactionStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(TransactionStatus::Pending),
            "processing" => Ok(TransactionStatus::Processing),
            "completed" => Ok(TransactionStatus::Completed),
            "failed" => Ok(TransactionStatus::Failed),
            "cancelled" => Ok(TransactionStatus::Cancelled),
            _ => Err(format!("Invalid transaction status: {}", s)),
        }
    }
}

impl std::fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionStatus::Pending => write!(f, "pending"),
            TransactionStatus::Processing => write!(f, "processing"),
            TransactionStatus::Completed => write!(f, "completed"),
            TransactionStatus::Failed => write!(f, "failed"),
            TransactionStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum FailureReason {
    InsufficientFunds,
    InvalidAccount,
    NetworkError,
    Timeout,
    Fraud,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum Currency {
    USD,
    EUR,
    GBP,
    JPY,
    CAD,
    AUD,
}

impl Default for Currency {
    fn default() -> Self {
        Currency::USD
    }
}

impl std::str::FromStr for Currency {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "USD" => Ok(Currency::USD),
            "EUR" => Ok(Currency::EUR),
            "GBP" => Ok(Currency::GBP),
            "JPY" => Ok(Currency::JPY),
            "CAD" => Ok(Currency::CAD),
            "AUD" => Ok(Currency::AUD),
            _ => Err(format!("Invalid currency: {}", s)),
        }
    }
}

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Currency::USD => write!(f, "USD"),
            Currency::EUR => write!(f, "EUR"),
            Currency::GBP => write!(f, "GBP"),
            Currency::JPY => write!(f, "JPY"),
            Currency::CAD => write!(f, "CAD"),
            Currency::AUD => write!(f, "AUD"),
        }
    }
}

impl Currency {
    pub fn validate(code: &str) -> Result<Self, ValidationError> {
        code.parse().map_err(|_| ValidationError::InvalidCurrency)
    }

    pub fn is_valid(code: &str) -> bool {
        matches!(
            code.to_uppercase().as_str(),
            "USD" | "EUR" | "GBP" | "JPY" | "CAD" | "AUD"
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Account {
    pub account_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Transaction {
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

#[derive(Debug, Clone, Serialize, Deserialize, IntoParams, ToSchema)]
pub struct TransactionFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<TransactionStatus>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_type: Option<TransactionType>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<Currency>,
    
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[param(value_type = Option<String>, format = DateTime)]
    #[schema(value_type = Option<String>, format = DateTime)]
    pub created_after: Option<DateTime<Utc>>,
    
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[param(value_type = Option<String>, format = DateTime)]
    #[schema(value_type = Option<String>, format = DateTime)]
    pub created_before: Option<DateTime<Utc>>,
}

impl Default for TransactionFilters {
    fn default() -> Self {
        Self {
            account_id: None,
            status: None,
            transaction_type: None,
            currency: None,
            created_after: None,
            created_before: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    EmptyAccountId,
    InvalidAccountId,
    NegativeAmount,
    InvalidCurrency,
    InvalidTransactionType,
    InvalidStatus,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::EmptyAccountId => write!(f, "Account ID cannot be empty"),
            ValidationError::InvalidAccountId => write!(f, "Invalid account ID format"),
            ValidationError::NegativeAmount => write!(f, "Amount cannot be negative"),
            ValidationError::InvalidCurrency => write!(f, "Invalid currency code"),
            ValidationError::InvalidTransactionType => write!(f, "Invalid transaction type"),
            ValidationError::InvalidStatus => write!(f, "Invalid transaction status"),
        }
    }
}

impl std::error::Error for ValidationError {}

pub fn generate_transaction_id() -> String {
    let timestamp = Utc::now().timestamp();
    let random: u32 = rand::rng().random();
    format!("txn_{}_{:08x}", timestamp, random)
}

pub fn generate_account_id() -> String {
    let timestamp = Utc::now().timestamp();
    let random: u32 = rand::rng().random();
    format!("acc_{}_{:08x}", timestamp, random)
}