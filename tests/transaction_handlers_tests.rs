use metered_finance_api::models::{
    requests::CreateTransactionRequest,
    responses::{TransactionResponse, BalanceResponse},
    finance::{Currency, TransactionType},
};

#[test]
fn test_create_transaction_request_validation() {
    let req = CreateTransactionRequest {
        account_id: "user_123".to_string(),
        amount: 99.99,
        currency: Currency::USD,
        transaction_type: TransactionType::Payment,
        description: None,
        metadata: None,
    };
    assert!(req.validate().is_ok());

    let req = CreateTransactionRequest {
        account_id: "".to_string(),
        amount: 99.99,
        currency: Currency::USD,
        transaction_type: TransactionType::Payment,
        description: None,
        metadata: None,
    };
    assert!(req.validate().is_err());

    let req = CreateTransactionRequest {
        account_id: "user_123".to_string(),
        amount: 0.0,
        currency: Currency::USD,
        transaction_type: TransactionType::Payment,
        description: None,
        metadata: None,
    };
    assert!(req.validate().is_err());

    let req = CreateTransactionRequest {
        account_id: "user_123".to_string(),
        amount: -50.0,
        currency: Currency::USD,
        transaction_type: TransactionType::Payment,
        description: None,
        metadata: None,
    };
    assert!(req.validate().is_err());

    let req = CreateTransactionRequest {
        account_id: "user_123".to_string(),
        amount: f64::NAN,
        currency: Currency::USD,
        transaction_type: TransactionType::Payment,
        description: None,
        metadata: None,
    };
    assert!(req.validate().is_err());

    let req = CreateTransactionRequest {
        account_id: "user_123".to_string(),
        amount: f64::INFINITY,
        currency: Currency::USD,
        transaction_type: TransactionType::Payment,
        description: None,
        metadata: None,
    };
    assert!(req.validate().is_err());
}

#[test]
fn test_create_transaction_request_decimal_places() {
    let req = CreateTransactionRequest {
        account_id: "user_123".to_string(),
        amount: 99.99,
        currency: Currency::USD,
        transaction_type: TransactionType::Payment,
        description: None,
        metadata: None,
    };
    assert!(req.validate().is_ok());

    let req = CreateTransactionRequest {
        account_id: "user_123".to_string(),
        amount: 50.5,
        currency: Currency::USD,
        transaction_type: TransactionType::Payment,
        description: None,
        metadata: None,
    };
    assert!(req.validate().is_ok());

    let req = CreateTransactionRequest {
        account_id: "user_123".to_string(),
        amount: 100.0,
        currency: Currency::USD,
        transaction_type: TransactionType::Payment,
        description: None,
        metadata: None,
    };
    assert!(req.validate().is_ok());
}

#[test]
fn test_create_transaction_request_description_length() {
    let req = CreateTransactionRequest {
        account_id: "user_123".to_string(),
        amount: 99.99,
        currency: Currency::USD,
        transaction_type: TransactionType::Payment,
        description: Some("Payment for services".to_string()),
        metadata: None,
    };
    assert!(req.validate().is_ok());

    let req = CreateTransactionRequest {
        account_id: "user_123".to_string(),
        amount: 99.99,
        currency: Currency::USD,
        transaction_type: TransactionType::Payment,
        description: Some("a".repeat(1001)),
        metadata: None,
    };
    assert!(req.validate().is_err());

    let req = CreateTransactionRequest {
        account_id: "user_123".to_string(),
        amount: 99.99,
        currency: Currency::USD,
        transaction_type: TransactionType::Payment,
        description: Some("a".repeat(1000)),
        metadata: None,
    };
    assert!(req.validate().is_ok());
}

#[test]
fn test_transaction_response_serialization() {
    let response = TransactionResponse {
        transaction_id: "txn_123".to_string(),
        account_id: "acc_123".to_string(),
        amount: 99.99,
        currency: Currency::USD,
        transaction_type: TransactionType::Payment,
        status: metered_finance_api::models::finance::TransactionStatus::Completed,
        description: Some("Test transaction".to_string()),
        metadata: None,
        created_at: chrono::Utc::now(),
        processed_at: Some(chrono::Utc::now()),
    };

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("txn_123"));
    assert!(json.contains("99.99"));
}

#[test]
fn test_balance_response() {
    let response = BalanceResponse {
        account_id: "acc_123".to_string(),
        balance: 1500.50,
        currency: Currency::USD,
        as_of: chrono::Utc::now(),
    };

    assert_eq!(response.account_id, "acc_123");
    assert_eq!(response.balance, 1500.50);
}

#[test]
fn test_transaction_types() {
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
