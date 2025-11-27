use axum::{http::StatusCode, response::IntoResponse};
use metered_finance_api::middleware::errors::AppError;
use metered_finance_api::models::common::ValidationError as CommonValidationError;
use metered_finance_api::models::finance::ValidationError as FinanceValidationError;
use metered_finance_api::models::keys::ValidationError as KeysValidationError;

#[test]
fn test_unauthorized_error_response() {
    let err = AppError::Unauthorized("Invalid credentials".to_string());
    let response = err.into_response();
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[test]
fn test_forbidden_error_response() {
    let err = AppError::Forbidden("Access denied".to_string());
    let response = err.into_response();
    
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[test]
fn test_invalid_api_key_error_response() {
    let err = AppError::InvalidApiKey;
    let response = err.into_response();
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[test]
fn test_validation_error_response() {
    let err = AppError::ValidationError("Field is required".to_string());
    let response = err.into_response();
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[test]
fn test_invalid_input_error_response() {
    let err = AppError::InvalidInput("Invalid format".to_string());
    let response = err.into_response();
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[test]
fn test_not_found_error_response() {
    let err = AppError::NotFound("Resource not found".to_string());
    let response = err.into_response();
    
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[test]
fn test_rate_limit_exceeded_error_response() {
    let err = AppError::RateLimitExceeded;
    let response = err.into_response();
    
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[test]
fn test_quota_exceeded_error_response() {
    let err = AppError::QuotaExceeded;
    let response = err.into_response();
    
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[test]
fn test_internal_error_response() {
    let err = AppError::InternalError("Something went wrong".to_string());
    let response = err.into_response();
    
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn test_helper_not_found() {
    let err = AppError::not_found("Transaction", "txn_123");
    
    if let AppError::NotFound(msg) = err {
        assert!(msg.contains("Transaction"));
        assert!(msg.contains("txn_123"));
    } else {
        panic!("Expected NotFound error");
    }
}

#[test]
fn test_helper_account_not_found() {
    let err = AppError::account_not_found("acc_456");
    
    if let AppError::NotFound(msg) = err {
        assert!(msg.contains("Account"));
        assert!(msg.contains("acc_456"));
    } else {
        panic!("Expected NotFound error");
    }
}

#[test]
fn test_helper_transaction_not_found() {
    let err = AppError::transaction_not_found("txn_789");
    
    if let AppError::NotFound(msg) = err {
        assert!(msg.contains("Transaction"));
        assert!(msg.contains("txn_789"));
    } else {
        panic!("Expected NotFound error");
    }
}

#[test]
fn test_from_common_validation_error() {
    let validation_err = CommonValidationError::InvalidLimit;
    let app_err: AppError = validation_err.into();
    
    if let AppError::ValidationError(msg) = app_err {
        assert!(msg.contains("limit"));
    } else {
        panic!("Expected ValidationError");
    }
}

#[test]
fn test_from_finance_validation_error() {
    let validation_err = FinanceValidationError::EmptyAccountId;
    let app_err: AppError = validation_err.into();
    
    if let AppError::ValidationError(msg) = app_err {
        assert!(msg.contains("Account ID"));
    } else {
        panic!("Expected ValidationError");
    }
}

#[test]
fn test_from_keys_validation_error() {
    let validation_err = KeysValidationError::EmptyScopes;
    let app_err: AppError = validation_err.into();
    
    if let AppError::ValidationError(msg) = app_err {
        assert!(msg.contains("Scopes"));
    } else {
        panic!("Expected ValidationError");
    }
}

#[test]
fn test_error_chain_validation() {
    // Test that we can convert from domain validation errors
    let finance_err = FinanceValidationError::NegativeAmount;
    let app_err: AppError = finance_err.into();
    let response = app_err.into_response();
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[test]
fn test_error_debug_format() {
    let err = AppError::Unauthorized("Test".to_string());
    let debug_str = format!("{:?}", err);
    
    assert!(debug_str.contains("Unauthorized"));
    assert!(debug_str.contains("Test"));
}

#[test]
fn test_multiple_error_types() {
    let errors = vec![
        AppError::Unauthorized("auth".to_string()),
        AppError::ValidationError("validation".to_string()),
        AppError::NotFound("not found".to_string()),
        AppError::RateLimitExceeded,
    ];
    
    let statuses: Vec<StatusCode> = errors
        .into_iter()
        .map(|e| e.into_response().status())
        .collect();
    
    assert_eq!(statuses[0], StatusCode::UNAUTHORIZED);
    assert_eq!(statuses[1], StatusCode::BAD_REQUEST);
    assert_eq!(statuses[2], StatusCode::NOT_FOUND);
    assert_eq!(statuses[3], StatusCode::TOO_MANY_REQUESTS);
}

#[test]
fn test_error_message_content() {
    let err = AppError::ValidationError("Email is invalid".to_string());
    
    if let AppError::ValidationError(msg) = err {
        assert_eq!(msg, "Email is invalid");
    } else {
        panic!("Expected ValidationError");
    }
}

#[test]
fn test_not_found_with_specific_id() {
    let account_err = AppError::account_not_found("acc_test_123");
    let txn_err = AppError::transaction_not_found("txn_test_456");
    
    if let AppError::NotFound(msg) = account_err {
        assert!(msg.contains("acc_test_123"));
    }
    
    if let AppError::NotFound(msg) = txn_err {
        assert!(msg.contains("txn_test_456"));
    }
}