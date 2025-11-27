use metered_finance_api::models::finance::*;

#[test]
fn test_currency_validation() {
    assert!(Currency::new("USD").is_ok());
    assert!(Currency::new("EUR").is_ok());
    assert!(Currency::new("GBP").is_ok());
    
    assert!(Currency::new("usd").is_err());
    assert!(Currency::new("US").is_err());
    assert!(Currency::new("USDT").is_err());
    assert!(Currency::new("U$D").is_err());
    assert!(Currency::new("123").is_err());
}

#[test]
fn test_currency_helpers() {
    let usd = Currency::usd();
    assert_eq!(usd.0, "USD");

    let eur = Currency::eur();
    assert_eq!(eur.0, "EUR");

    let gbp = Currency::gbp();
    assert_eq!(gbp.0, "GBP");
}

#[test]
fn test_transaction_type_serialization() {
    let payment = TransactionType::Payment;
    let json = serde_json::to_string(&payment).unwrap();
    assert_eq!(json, "\"payment\"");

    let deserialized: TransactionType = serde_json::from_str("\"payment\"").unwrap();
    assert_eq!(deserialized, payment);
}

#[test]
fn test_transaction_status_serialization() {
    let status = TransactionStatus::Succeeded;
    let json = serde_json::to_string(&status).unwrap();
    assert_eq!(json, "\"succeeded\"");
    
    let deserialized: TransactionStatus = serde_json::from_str("\"failed\"").unwrap();
    assert_eq!(deserialized, TransactionStatus::Failed);
}

#[test]
fn test_failure_reason_serialization() {
    let reason = FailureReason::InsufficientFunds;
    let json = serde_json::to_string(&reason).unwrap();
    assert_eq!(json, "\"insufficient_funds\"");
}

#[test]
fn test_transaction_request_validation_valid() {
    let valid = CreateTransactionRequest {
        account_id: "acc_123".to_string(),
        transaction_type: TransactionType::Payment,
        amount_cents: 1000,
        currency: "USD".to_string(),
        description: None,
        metadata: None,
    };
    assert!(valid.validate().is_ok());
}

#[test]
fn test_transaction_request_validation_empty_account() {
    let invalid = CreateTransactionRequest {
        account_id: "".to_string(),
        transaction_type: TransactionType::Payment,
        amount_cents: 1000,
        currency: "USD".to_string(),
        description: None,
        metadata: None,
    };
    assert!(matches!(invalid.validate(), Err(ValidationError::EmptyAccountId)));
}

#[test]
fn test_transaction_request_validation_negative_amount() {
    let invalid = CreateTransactionRequest {
        account_id: "acc_123".to_string(),
        transaction_type: TransactionType::Payment,
        amount_cents: -100,
        currency: "USD".to_string(),
        description: None,
        metadata: None,
    };
    assert!(matches!(invalid.validate(), Err(ValidationError::NegativeAmount)));
}

#[test]
fn test_transaction_request_validation_invalid_currency() {
    let invalid = CreateTransactionRequest {
        account_id: "acc_123".to_string(),
        transaction_type: TransactionType::Payment,
        amount_cents: 1000,
        currency: "invalid".to_string(),
        description: None,
        metadata: None,
    };
    assert!(matches!(invalid.validate(), Err(ValidationError::InvalidCurrency)));
}

#[test]
fn test_transaction_request_validation_description_too_long() {
    let long_description = "a".repeat(501);
    let invalid = CreateTransactionRequest {
        account_id: "acc_123".to_string(),
        transaction_type: TransactionType::Payment,
        amount_cents: 1000,
        currency: "USD".to_string(),
        description: Some(long_description),
        metadata: None,
    };
    assert!(matches!(invalid.validate(), Err(ValidationError::DescriptionTooLong)));
}

#[test]
fn test_transaction_request_with_metadata() {
    let metadata = serde_json::json!({
        "customer_id": "cust_123",
        "invoice_id": "inv_456"
    });
    
    let request = CreateTransactionRequest {
        account_id: "acc_123".to_string(),
        transaction_type: TransactionType::Payment,
        amount_cents: 1000,
        currency: "USD".to_string(),
        description: Some("Test payment".to_string()),
        metadata: Some(metadata),
    };
    
    assert!(request.validate().is_ok());
}

#[test]
fn test_id_generation() {
    let txn_id = generate_txn_id();
    assert!(txn_id.starts_with("txn_"));
    assert!(txn_id.len() > 4);
    
    let txn_id2 = generate_txn_id();
    assert_ne!(txn_id, txn_id2);
}

#[test]
fn test_account_id_generation() {
    let acc_id = generate_account_id();
    assert!(acc_id.starts_with("acc_"));
    assert!(acc_id.len() > 4);
    
    let acc_id2 = generate_account_id();
    assert_ne!(acc_id, acc_id2);
}

#[test]
fn test_transaction_filters_deserialization() {
    let json = r#"{
        "account_id": "acc_123",
        "transaction_type": "payment",
        "status": "succeeded",
        "currency": "USD"
    }"#;
    
    let filters: TransactionFilters = serde_json::from_str(json).unwrap();
    assert_eq!(filters.account_id, Some("acc_123".to_string()));
    assert_eq!(filters.transaction_type, Some(TransactionType::Payment));
    assert_eq!(filters.status, Some(TransactionStatus::Succeeded));
    assert_eq!(filters.currency, Some("USD".to_string()));
    assert_eq!(filters.from_timestamp, None);
    assert_eq!(filters.to_timestamp, None);
}