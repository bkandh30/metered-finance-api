use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::models::common::{ErrorCode, ErrorDetail, ErrorResponse};

#[derive(Debug)]
pub enum AppError {
    Unauthorized(String),
    Forbidden(String),
    InvalidApiKey,
    
    ValidationError(String),
    InvalidInput(String),
    
    NotFound(String),
    
    RateLimitExceeded,
    QuotaExceeded,
    
    DatabaseError(sqlx::Error),
    
    InternalError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, message, details) = match self {
            AppError::Unauthorized(msg) => (
                StatusCode::UNAUTHORIZED,
                ErrorCode::Unauthorized,
                msg,
                None,
            ),
            AppError::Forbidden(msg) => (
                StatusCode::FORBIDDEN,
                ErrorCode::Forbidden,
                msg,
                None,
            ),
            AppError::InvalidApiKey => (
                StatusCode::UNAUTHORIZED,
                ErrorCode::InvalidApiKey,
                "Invalid API key".to_string(),
                None,
            ),
            
            AppError::ValidationError(msg) => (
                StatusCode::BAD_REQUEST,
                ErrorCode::ValidationError,
                msg,
                None,
            ),
            AppError::InvalidInput(msg) => (
                StatusCode::BAD_REQUEST,
                ErrorCode::InvalidInput,
                msg,
                None,
            ),
            
            AppError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                ErrorCode::NotFound,
                msg,
                None,
            ),
            
            AppError::RateLimitExceeded => (
                StatusCode::TOO_MANY_REQUESTS,
                ErrorCode::RateLimitExceeded,
                "Rate limit exceeded".to_string(),
                Some(json!({
                    "retry_after": "60s"
                })),
            ),
            AppError::QuotaExceeded => (
                StatusCode::TOO_MANY_REQUESTS,
                ErrorCode::QuotaExceeded,
                "Daily quota exceeded".to_string(),
                None,
            ),
            
            AppError::DatabaseError(e) => {
                tracing::error!("Database error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorCode::DatabaseError,
                    "Database error occurred".to_string(),
                    None,
                )
            }
            
            AppError::InternalError(msg) => {
                tracing::error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorCode::InternalError,
                    "An internal error occurred".to_string(),
                    None,
                )
            }
        };

        let error_response = ErrorResponse {
            error: ErrorDetail {
                code: error_code.to_string(),
                message,
                details,
            },
        };

        (status, Json(error_response)).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        AppError::DatabaseError(error)
    }
}

impl From<crate::models::common::ValidationError> for AppError {
    fn from(error: crate::models::common::ValidationError) -> Self {
        AppError::ValidationError(error.to_string())
    }
}

impl From<crate::models::finance::ValidationError> for AppError {
    fn from(error: crate::models::finance::ValidationError) -> Self {
        AppError::ValidationError(error.to_string())
    }
}

impl From<crate::models::keys::ValidationError> for AppError {
    fn from(error: crate::models::keys::ValidationError) -> Self {
        AppError::ValidationError(error.to_string())
    }
}

impl AppError {
    pub fn not_found(resource: &str, id: &str) -> Self {
        AppError::NotFound(format!("{} with id '{}' not found", resource, id))
    }
    
    pub fn account_not_found(account_id: &str) -> Self {
        AppError::NotFound(format!("Account '{}' not found", account_id))
    }
    
    pub fn transaction_not_found(txn_id: &str) -> Self {
        AppError::NotFound(format!("Transaction '{}' not found", txn_id))
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            AppError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            AppError::InvalidApiKey => write!(f, "Invalid API key"),
            AppError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            AppError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::RateLimitExceeded => write!(f, "Rate limit exceeded"),
            AppError::QuotaExceeded => write!(f, "Quota exceeded"),
            AppError::InternalError(msg) => write!(f, "Internal error: {}", msg),
            AppError::DatabaseError(e) => write!(f, "Database error: {}", e),
        }
    }
}