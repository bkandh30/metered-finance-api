use metered_finance_api::models::{
    finance::{Currency, TransactionType, TransactionStatus, TransactionFilters},
    requests::CreateTransactionRequest,
};

#[test]
fn test_currency_validation() {
    use std::str::FromStr;
    
    assert!(Currency::from_str("USD").is_ok());
    assert!(Currency::from_str("EUR").is_ok());
    assert!(Currency::from_str("GBP").is_ok());
    assert!(Currency::from_str("INVALID").is_err());
}

#[test]
fn test_currency_values() {
    assert_eq!(Currency::USD.to_string(), "USD");
    assert_eq!(Currency::EUR.to_string(), "EUR");
    assert_eq!(Currency::GBP.to_string(), "GBP");
}

#[test]
fn test_transaction_status() {
    use std::str::FromStr;
    
    assert!(TransactionStatus::from_str("pending").is_ok());
    assert!(TransactionStatus::from_str("completed").is_ok());
    assert!(TransactionStatus::from_str("failed").is_ok());
}

#[test]
fn test_create_transaction_request_validation() {
    let valid = CreateTransactionRequest {
        account_id: "user_123".to_string(),
        amount: 99.99,
        currency: Currency::USD,
        transaction_type: TransactionType::Payment,
        description: None,
        metadata: None,
    };
    assert!(valid.validate().is_ok());

    let invalid = CreateTransactionRequest {
        account_id: "".to_string(),
        amount: 99.99,
        currency: Currency::USD,
        transaction_type: TransactionType::Payment,
        description: None,
        metadata: None,
    };
    assert!(invalid.validate().is_err());

    let invalid = CreateTransactionRequest {
        account_id: "user_123".to_string(),
        amount: -50.0,
        currency: Currency::USD,
        transaction_type: TransactionType::Payment,
        description: None,
        metadata: None,
    };
    assert!(invalid.validate().is_err());

    let invalid = CreateTransactionRequest {
        account_id: "user_123".to_string(),
        amount: 0.0,
        currency: Currency::USD,
        transaction_type: TransactionType::Payment,
        description: None,
        metadata: None,
    };
    assert!(invalid.validate().is_err());

    let invalid = CreateTransactionRequest {
        account_id: "user_123".to_string(),
        amount: 99.99,
        currency: Currency::USD,
        transaction_type: TransactionType::Payment,
        description: Some("a".repeat(1001)),
        metadata: None,
    };
    assert!(invalid.validate().is_err());
}

#[test]
fn test_transaction_id_generation() {
    let txn_id = metered_finance_api::models::finance::generate_transaction_id();
    assert!(txn_id.starts_with("txn_"));
    
    let txn_id2 = metered_finance_api::models::finance::generate_transaction_id();
    assert_ne!(txn_id, txn_id2);
}

#[test]
fn test_account_id_generation() {
    let acc_id = metered_finance_api::models::finance::generate_account_id();
    assert!(acc_id.starts_with("acc_"));
}

#[test]
fn test_transaction_filters() {
    let filters = TransactionFilters {
        account_id: Some("user_123".to_string()),
        status: Some(TransactionStatus::Completed),
        transaction_type: Some(TransactionType::Payment),
        currency: Some(Currency::USD),
        created_after: None,
        created_before: None,
    };
    
    assert_eq!(filters.account_id, Some("user_123".to_string()));
    assert_eq!(filters.status, Some(TransactionStatus::Completed));
    assert_eq!(filters.currency, Some(Currency::USD));
    assert_eq!(filters.created_after, None);
    assert_eq!(filters.created_before, None);
}

#[test]
fn test_all_transaction_types() {
    let types = vec![
        TransactionType::Payment,
        TransactionType::Refund,
        TransactionType::Adjustment,
        TransactionType::Fee,
        TransactionType::Payout,
        TransactionType::Chargeback,
        TransactionType::Transfer,
    ];
    
    for txn_type in types {
        let req = CreateTransactionRequest {
            account_id: "user_123".to_string(),
            amount: 50.0,
            currency: Currency::USD,
            transaction_type: txn_type,
            description: None,
            metadata: None,
        };
        assert!(req.validate().is_ok());
    }
}

#[test]
fn test_all_currencies() {
    let currencies = vec![
        Currency::USD,
        Currency::EUR,
        Currency::GBP,
        Currency::JPY,
        Currency::CAD,
        Currency::AUD,
    ];
    
    for currency in currencies {
        let req = CreateTransactionRequest {
            account_id: "user_123".to_string(),
            amount: 100.0,
            currency,
            transaction_type: TransactionType::Payment,
            description: None,
            metadata: None,
        };
        assert!(req.validate().is_ok());
    }
}