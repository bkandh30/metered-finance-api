use metered_finance_api::models::{
    requests::{CreateAccountRequest, UpdateAccountRequest},
    responses::AccountResponse,
};

#[test]
fn test_create_account_request_validation() {
    let req = CreateAccountRequest {
        account_id: "user_123".to_string(),
        metadata: None,
    };
    assert!(req.validate().is_ok());

    let req = CreateAccountRequest {
        account_id: "".to_string(),
        metadata: None,
    };
    assert!(req.validate().is_err());

    let req = CreateAccountRequest {
        account_id: "ab".to_string(),
        metadata: None,
    };
    assert!(req.validate().is_err());

    let req = CreateAccountRequest {
        account_id: "a".repeat(256),
        metadata: None,
    };
    assert!(req.validate().is_err());

    let req = CreateAccountRequest {
        account_id: "user@123".to_string(),
        metadata: None,
    };
    assert!(req.validate().is_err());

    let req = CreateAccountRequest {
        account_id: "user_test-123".to_string(),
        metadata: None,
    };
    assert!(req.validate().is_ok());
}

#[test]
fn test_create_account_request_with_metadata() {
    let metadata = serde_json::json!({
        "name": "John Doe",
        "email": "john@example.com"
    });

    let req = CreateAccountRequest {
        account_id: "user_123".to_string(),
        metadata: Some(metadata.clone()),
    };

    assert!(req.validate().is_ok());
    assert_eq!(req.metadata, Some(metadata));
}

#[test]
fn test_update_account_request() {
    let metadata = serde_json::json!({
        "updated": true,
        "tags": ["premium", "active"]
    });

    let req = UpdateAccountRequest {
        metadata: metadata.clone(),
    };

    assert_eq!(req.metadata, metadata);
}

#[test]
fn test_account_response_serialization() {
    let response = AccountResponse {
        account_id: "acc_123".to_string(),
        metadata: Some(serde_json::json!({"test": "value"})),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("acc_123"));
    assert!(json.contains("metadata"));
}
