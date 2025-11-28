use metered_finance_api::models::{
    responses::{KeyCreatedResponse, KeyInfoResponse, UsageResponse},
    keys::Scope,
    quota::{QuotaLimits, QuotaUsageStats},
};

#[test]
fn test_key_created_response_serialization() {
    let response = KeyCreatedResponse {
        key_id: "key_123".to_string(),
        api_key: "sk_live_abcdef123456".to_string(),
        prefix: "sk_live_abc".to_string(),
        name: "Production Key".to_string(),
        scopes: vec![Scope::Client, Scope::Reporting],
        active: true,
        rate_limit_per_minute: 100,
        daily_quota: 10000,
        monthly_quota: 300000,
        created_at: chrono::Utc::now(),
    };

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("key_123"));
    assert!(json.contains("sk_live_abcdef123456"));
    assert!(json.contains("Production Key"));
}

#[test]
fn test_key_info_response_serialization() {
    let response = KeyInfoResponse {
        key_id: "key_456".to_string(),
        prefix: "sk_test_xyz".to_string(),
        name: "Test Key".to_string(),
        scopes: vec![Scope::Client],
        active: false,
        rate_limit_per_minute: 60,
        daily_quota: 5000,
        monthly_quota: 150000,
        created_at: chrono::Utc::now(),
        last_used_at: Some(chrono::Utc::now()),
    };

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("key_456"));
    assert!(json.contains("Test Key"));
    assert!(!json.contains("secret"));
}

#[test]
fn test_usage_response() {
    let limits = QuotaLimits {
        rate_limit_per_minute: 100,
        daily_quota: 10000,
        monthly_quota: 300000,
    };

    let usage = QuotaUsageStats {
        today: 500,
        this_month: 5000,
        daily_remaining: 9500,
        monthly_remaining: 295000,
    };

    let response = UsageResponse {
        key_id: "key_789".to_string(),
        limits,
        usage,
    };

    assert_eq!(response.key_id, "key_789");
    assert_eq!(response.limits.rate_limit_per_minute, 100);
    assert_eq!(response.usage.today, 500);
}

#[test]
fn test_usage_response_serialization() {
    let limits = QuotaLimits::default();
    let usage = QuotaUsageStats {
        today: 100,
        this_month: 1000,
        daily_remaining: 9900,
        monthly_remaining: 299000,
    };

    let response = UsageResponse {
        key_id: "key_test".to_string(),
        limits,
        usage,
    };

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("key_test"));
    assert!(json.contains("daily_remaining"));
    assert!(json.contains("monthly_remaining"));
}