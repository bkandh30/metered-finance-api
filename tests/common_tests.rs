use metered_finance_api::models::common::{
    Cursor, PaginationParams, PaginatedResponse, ErrorResponse, ErrorDetail, ErrorCode,
};

#[test]
fn test_cursor_encode_decode() {
    let cursor = Cursor::encode("test_id_123");
    let decoded = cursor.decode_string().unwrap();
    assert_eq!(decoded, "test_id_123");
}

#[test]
fn test_pagination_params_valid() {
    let params = PaginationParams {
        cursor: None,
        limit: Some(50),
    };
    assert!(params.validate().is_ok());
}

#[test]
fn test_pagination_params_invalid_limit_low() {
    let params = PaginationParams {
        cursor: None,
        limit: Some(0),
    };
    assert!(params.validate().is_err());
}

#[test]
fn test_pagination_params_invalid_limit_high() {
    let params = PaginationParams {
        cursor: None,
        limit: Some(101),
    };
    assert!(params.validate().is_err());
}

#[test]
fn test_pagination_params_default_limit() {
    let params = PaginationParams {
        cursor: None,
        limit: None,
    };
    assert_eq!(params.limit, None);
}

#[test]
fn test_paginated_response() {
    let data = vec!["item1".to_string(), "item2".to_string()];
    let response = PaginatedResponse {
        data: data.clone(),
        has_more: true,
        next_cursor: Some(Cursor::encode("cursor123")),
    };
    
    assert_eq!(response.data, data);
    assert!(response.has_more);
    assert!(response.next_cursor.is_some());
}

#[test]
fn test_error_response_serialization() {
    let error = ErrorResponse {
        error: ErrorDetail {
            code: ErrorCode::ValidationError.to_string(),
            message: "Invalid input".to_string(),
            details: None,
        },
    };
    
    let json = serde_json::to_string(&error).unwrap();
    assert!(json.contains("validation_error"));
    assert!(json.contains("Invalid input"));
}