use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Cursor(pub String);

impl Cursor {
    pub fn new(timestamp: &time::OffsetDateTime, id: &str) -> Self {
        let encoded = format!("{}|{}", timestamp.unix_timestamp(), id);
        Self(BASE64_STANDARD.encode(encoded.as_bytes()))
    }

    pub fn encode(id: &str) -> Self {
        Self(BASE64_STANDARD.encode(id.as_bytes()))
    }

    pub fn decode_string(&self) -> Result<String, CursorError> {
        let decoded = BASE64_STANDARD
            .decode(self.0.as_bytes())
            .map_err(|_| CursorError::InvalidFormat)?;
        
        String::from_utf8(decoded)
            .map_err(|_| CursorError::InvalidFormat)
    }

    pub fn decode(&self) -> Result<(time::OffsetDateTime, String), CursorError> {
        let decoded = BASE64_STANDARD
            .decode(self.0.as_bytes())
            .map_err(|_| CursorError::InvalidFormat)?;

        let decoded_str = std::str::from_utf8(&decoded)
            .map_err(|_| CursorError::InvalidFormat)?;

        let parts: Vec<&str> = decoded_str.split('|').collect();
        if parts.len() != 2 {
            return Err(CursorError::InvalidFormat);
        }

        let timestamp = parts[0]
            .parse::<i64>()
            .map_err(|_| CursorError::InvalidFormat)?;
        
        let dt = time::OffsetDateTime::from_unix_timestamp(timestamp)
            .map_err(|_| CursorError::InvalidFormat)?;

        Ok((dt, parts[1].to_string()))
    }
}

#[derive(Debug, Clone)]
pub enum CursorError {
    InvalidFormat,
}

impl std::fmt::Display for CursorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CursorError::InvalidFormat => write!(f, "Invalid cursor format"),
        }
    }
}

impl std::error::Error for CursorError {}

#[derive(Debug, Clone, Serialize, Deserialize, IntoParams, ToSchema)]
pub struct PaginationParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<Cursor>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[param(minimum = 1, maximum = 100)]
    pub limit: Option<u32>,
}

impl PaginationParams {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if let Some(limit) = self.limit {
            if limit < 1 || limit > 100 {
                return Err(ValidationError::InvalidLimit);
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub has_more: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<Cursor>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    InvalidLimit,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::InvalidLimit => write!(f, "Limit must be between 1 and 100"),
        }
    }
}

impl std::error::Error for ValidationError {}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    Unauthorized,
    Forbidden,
    InvalidApiKey,
    
    ValidationError,
    InvalidInput,
    
    NotFound,
    AlreadyExists,
    
    RateLimitExceeded,
    QuotaExceeded,
    
    InternalError,
    DatabaseError,
    ServiceUnavailable,
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCode::Unauthorized => write!(f, "unauthorized"),
            ErrorCode::Forbidden => write!(f, "forbidden"),
            ErrorCode::InvalidApiKey => write!(f, "invalid_api_key"),
            ErrorCode::ValidationError => write!(f, "validation_error"),
            ErrorCode::InvalidInput => write!(f, "invalid_input"),
            ErrorCode::NotFound => write!(f, "not_found"),
            ErrorCode::AlreadyExists => write!(f, "already_exists"),
            ErrorCode::RateLimitExceeded => write!(f, "rate_limit_exceeded"),
            ErrorCode::QuotaExceeded => write!(f, "quota_exceeded"),
            ErrorCode::InternalError => write!(f, "internal_error"),
            ErrorCode::DatabaseError => write!(f, "database_error"),
            ErrorCode::ServiceUnavailable => write!(f, "service_unavailable"),
        }
    }
}