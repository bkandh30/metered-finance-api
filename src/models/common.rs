use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Cursor(pub String);

impl Cursor {
    pub fn new(timestamp: &time::OffsetDateTime, id: &str) -> Self {
        let payload = format!("{}|{}", timestamp.unix_timestamp_nanos(), id);
        let encoded = URL_SAFE_NO_PAD.encode(payload.as_bytes());
        Self(encoded)
    }

    pub fn decode(&self) -> Result<(time::OffsetDateTime, String), CursorError> {
        let decoded = URL_SAFE_NO_PAD
            .decode(self.0.as_bytes())
            .map_err(|_| CursorError::Invalid)?;
        
        let payload = String::from_utf8(decoded)
            .map_err(|_| CursorError::Invalid)?;
        
        let parts: Vec<&str> = payload.split('|').collect();
        if parts.len() != 2 {
            return Err(CursorError::Invalid);
        }
        
        let timestamp_nanos: i128 = parts[0]
            .parse()
            .map_err(|_| CursorError::Invalid)?;
        
        let timestamp = time::OffsetDateTime::from_unix_timestamp_nanos(timestamp_nanos)
            .map_err(|_| CursorError::Invalid)?;
        
        Ok((timestamp, parts[1].to_string()))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CursorError {
    #[error("Invalid cursor format")]
    Invalid,
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct PaginationParams {
    #[serde(default = "default_limit")]
    #[param(minimum = 1, maximum = 100)]
    pub limit: u32,
    
    #[serde(default)]
    pub cursor: Option<String>,
}

fn default_limit() -> u32 {
    20
}

impl PaginationParams {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.limit < 1 || self.limit > 100 {
            return Err(ValidationError::InvalidLimit);
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub has_more: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

impl<T: Serialize> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, has_more: bool, next_cursor: Option<String>) -> Self {
        Self {
            data,
            has_more,
            next_cursor,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorDetail {
    pub code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    Unauthorized,
    Forbidden,
    InvalidApiKey,
    InvalidAdminKey,
    
    ValidationError,
    InvalidInput,
    InvalidCursor,
    InvalidLimit,
    
    NotFound,
    ResourceNotFound,
    AccountNotFound,
    TransactionNotFound,
    
    RateLimitExceeded,
    QuotaExceeded,
    
    IdempotencyMismatch,
    
    InsufficientFunds,
    DuplicateTransaction,
    InvalidTransactionState,
    
    InternalError,
    DatabaseError,
    ServiceUnavailable,
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Invalid limit: must be between 1 and 100")]
    InvalidLimit,
    
    #[error("Invalid cursor format")]
    InvalidCursor,
    
    #[error("Missing required field: {0}")]
    MissingField(String),
    
    #[error("Invalid field value: {0}")]
    InvalidField(String),
}

pub mod timestamp {
    use time::OffsetDateTime;
    
    pub fn now() -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }
    
    pub fn unix_timestamp() -> i64 {
        OffsetDateTime::now_utc().unix_timestamp()
    }
    
    pub fn from_unix(timestamp: i64) -> OffsetDateTime {
        OffsetDateTime::from_unix_timestamp(timestamp)
            .unwrap_or_else(|_| OffsetDateTime::now_utc())
    }
}