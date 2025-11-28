use metered_finance_api::models::{
    finance::{Currency, TransactionType},
};

#[test]
fn test_currency_from_str() {
    use std::str::FromStr;

    assert!(Currency::from_str("USD").is_ok());
    assert!(Currency::from_str("usd").is_ok());
    assert!(Currency::from_str("EUR").is_ok());
    assert!(Currency::from_str("GBP").is_ok());
    assert!(Currency::from_str("INVALID").is_err());
}

#[test]
fn test_currency_to_string() {
    assert_eq!(Currency::USD.to_string(), "USD");
    assert_eq!(Currency::EUR.to_string(), "EUR");
    assert_eq!(Currency::GBP.to_string(), "GBP");
}

#[test]
fn test_transaction_type_from_str() {
    use std::str::FromStr;

    assert!(TransactionType::from_str("payment").is_ok());
    assert!(TransactionType::from_str("PAYMENT").is_ok());
    assert!(TransactionType::from_str("refund").is_ok());
    assert!(TransactionType::from_str("invalid").is_err());
}

#[test]
fn test_transaction_type_to_string() {
    assert_eq!(TransactionType::Payment.to_string(), "payment");
    assert_eq!(TransactionType::Refund.to_string(), "refund");
    assert_eq!(TransactionType::Fee.to_string(), "fee");
}

#[test]
fn test_transaction_status_from_str() {
    use std::str::FromStr;
    use metered_finance_api::models::finance::TransactionStatus;

    assert!(TransactionStatus::from_str("pending").is_ok());
    assert!(TransactionStatus::from_str("COMPLETED").is_ok());
    assert!(TransactionStatus::from_str("failed").is_ok());
    assert!(TransactionStatus::from_str("invalid").is_err());
}

#[test]
fn test_transaction_status_to_string() {
    use metered_finance_api::models::finance::TransactionStatus;

    assert_eq!(TransactionStatus::Pending.to_string(), "pending");
    assert_eq!(TransactionStatus::Completed.to_string(), "completed");
    assert_eq!(TransactionStatus::Failed.to_string(), "failed");
}

#[test]
fn test_defaults() {
    let currency = Currency::default();
    assert_eq!(currency.to_string(), "USD");

    let txn_type = TransactionType::default();
    assert_eq!(txn_type.to_string(), "payment");

    use metered_finance_api::models::finance::TransactionStatus;
    let status = TransactionStatus::default();
    assert_eq!(status.to_string(), "pending");
}