use metered_finance_api::middleware::request_logging::extract_account_from_path;

#[test]
fn test_extract_account_from_path() {
    assert_eq!(
        extract_account_from_path("/api/accounts/user_123"),
        Some("user_123")
    );
    assert_eq!(
        extract_account_from_path("/api/accounts/user_123/transactions"),
        Some("user_123")
    );
    assert_eq!(
        extract_account_from_path("/api/accounts/user_123/balance"),
        Some("user_123")
    );
    assert_eq!(
        extract_account_from_path("/api/transactions"),
        None
    );
    assert_eq!(
        extract_account_from_path("/health/live"),
        None
    );
}

#[test]
fn test_extract_account_from_path_edge_cases() {
    assert_eq!(extract_account_from_path(""), None);
    
    assert_eq!(extract_account_from_path("/"), None);
    
    assert_eq!(extract_account_from_path("/api"), None);
    assert_eq!(extract_account_from_path("/api/accounts"), None);
    
    assert_eq!(
        extract_account_from_path("/api/accounts/acc_test_456"),
        Some("acc_test_456")
    );
    assert_eq!(
        extract_account_from_path("/api/accounts/user-with-dashes"),
        Some("user-with-dashes")
    );
}