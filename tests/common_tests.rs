use metered_finance_api::models::common::*;

#[test]
fn test_cursor_encode_decode() {
    let timestamp = time::OffsetDateTime::now_utc();
    let id = "test_id_123";

    let cursor = Cursor::new(&timestamp, id);
    let (decoded_ts, decoded_id) = cursor.decode().unwrap();

    assert_eq!(decoded_id, id);
    assert_eq!(decoded_ts.unix_timestamp(), timestamp.unix_timestamp());
}

#[test]
fn test_cursor_invalid_decode() {
    let invalid_cursor = Cursor("invalid_base64!@#".to_string());
    assert!(invalid_cursor.decode().is_err());

    let invalid_format = Cursor("dmFsaWQ=".to_string());
    assert!(invalid_format.decode().is_err()); 
}

#[test]
fn test_pagination_validation() {
    let valid = PaginationParams {
        limit: 50,
        cursor: None,
    };
    assert!(valid.validate().is_ok());
    
    let invalid_low = PaginationParams {
        limit: 0,
        cursor: None,
    };
    assert!(invalid_low.validate().is_err());
    
    let invalid_high = PaginationParams {
        limit: 101,
        cursor: None,
    };
    assert!(invalid_high.validate().is_err());
}

#[test]
fn test_pagination_default_limit() {
    let params: PaginationParams = serde_json::from_str("{}").unwrap();
    assert_eq!(params.limit, 20);
}

#[test]
fn test_paginated_response_creation() {
    let data = vec!["item1".to_string(), "item2".to_string()];
    let response = PaginatedResponse::new(data.clone(), true, Some("cursor123".to_string()));
    
    assert_eq!(response.data.len(), 2);
    assert_eq!(response.has_more, true);
    assert_eq!(response.next_cursor, Some("cursor123".to_string()));
}

#[test]
fn test_error_response_serialization() {
    let error = ErrorResponse {
        error: ErrorDetail {
            code: ErrorCode::ValidationError,
            message: "Invalid input".to_string(),
            details: Some(serde_json::json!({"field": "amount"})),
        },
    };
    
    let json = serde_json::to_string(&error).unwrap();
    assert!(json.contains("validation_error"));
    assert!(json.contains("Invalid input"));
}

#[test]
fn test_timestamp_utilities() {
    let now = timestamp::now();
    let unix = timestamp::unix_timestamp();
    let from_unix = timestamp::from_unix(unix);
    
    assert!(now.unix_timestamp() > 0);
    assert!(unix > 0);
    assert_eq!(from_unix.unix_timestamp(), unix);
}