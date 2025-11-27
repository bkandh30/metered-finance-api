use metered_finance_api::models::quota::{QuotaLimits, QuotaUsageStats, QuotaStatus};

#[test]
fn test_quota_limits_default() {
    let limits = QuotaLimits::default();
    
    assert_eq!(limits.rate_limit_per_minute, 60);
    assert_eq!(limits.daily_quota, 10_000);
    assert_eq!(limits.monthly_quota, 300_000);
}

#[test]
fn test_quota_limits_custom() {
    let limits = QuotaLimits {
        rate_limit_per_minute: 100,
        daily_quota: 50_000,
        monthly_quota: 1_000_000,
    };
    
    assert_eq!(limits.rate_limit_per_minute, 100);
    assert_eq!(limits.daily_quota, 50_000);
    assert_eq!(limits.monthly_quota, 1_000_000);
}

#[test]
fn test_quota_usage_stats_calculations() {
    let stats = QuotaUsageStats {
        today: 500,
        this_month: 5_000,
        daily_remaining: 9_500,
        monthly_remaining: 295_000,
    };
    
    assert_eq!(stats.today, 500);
    assert_eq!(stats.this_month, 5_000);
    assert_eq!(stats.daily_remaining, 9_500);
    assert_eq!(stats.monthly_remaining, 295_000);
}

#[test]
fn test_quota_usage_stats_zero_remaining() {
    let stats = QuotaUsageStats {
        today: 10_000,
        this_month: 300_000,
        daily_remaining: 0,
        monthly_remaining: 0,
    };
    
    assert_eq!(stats.daily_remaining, 0);
    assert_eq!(stats.monthly_remaining, 0);
}

#[test]
fn test_quota_status_structure() {
    let limits = QuotaLimits::default();
    let usage = QuotaUsageStats {
        today: 100,
        this_month: 1_000,
        daily_remaining: 9_900,
        monthly_remaining: 299_000,
    };
    
    let status = QuotaStatus {
        key_id: "key_test123".to_string(),
        limits: limits.clone(),
        usage: usage.clone(),
    };
    
    assert_eq!(status.key_id, "key_test123");
    assert_eq!(status.limits.daily_quota, limits.daily_quota);
    assert_eq!(status.usage.today, usage.today);
}

#[test]
fn test_quota_serialization() {
    let limits = QuotaLimits::default();
    let json = serde_json::to_string(&limits).unwrap();
    
    assert!(json.contains("rate_limit_per_minute"));
    assert!(json.contains("daily_quota"));
    assert!(json.contains("monthly_quota"));
}

#[test]
fn test_quota_deserialization() {
    let json = r#"{
        "rate_limit_per_minute": 120,
        "daily_quota": 20000,
        "monthly_quota": 500000
    }"#;
    
    let limits: QuotaLimits = serde_json::from_str(json).unwrap();
    
    assert_eq!(limits.rate_limit_per_minute, 120);
    assert_eq!(limits.daily_quota, 20_000);
    assert_eq!(limits.monthly_quota, 500_000);
}

#[test]
fn test_quota_usage_stats_serialization() {
    let stats = QuotaUsageStats {
        today: 50,
        this_month: 500,
        daily_remaining: 9_950,
        monthly_remaining: 299_500,
    };
    
    let json = serde_json::to_string(&stats).unwrap();
    let deserialized: QuotaUsageStats = serde_json::from_str(&json).unwrap();
    
    assert_eq!(deserialized.today, stats.today);
    assert_eq!(deserialized.this_month, stats.this_month);
}

#[test]
fn test_quota_limits_clone() {
    let limits = QuotaLimits::default();
    let cloned = limits.clone();
    
    assert_eq!(cloned.rate_limit_per_minute, limits.rate_limit_per_minute);
    assert_eq!(cloned.daily_quota, limits.daily_quota);
    assert_eq!(cloned.monthly_quota, limits.monthly_quota);
}

#[test]
fn test_high_usage_scenario() {
    let stats = QuotaUsageStats {
        today: 9_999,
        this_month: 299_999,
        daily_remaining: 1,
        monthly_remaining: 1,
    };
    
    assert_eq!(stats.daily_remaining, 1);
    assert_eq!(stats.monthly_remaining, 1);
}

#[test]
fn test_over_quota_scenario() {
    let stats = QuotaUsageStats {
        today: 11_000,
        this_month: 350_000,
        daily_remaining: 0,
        monthly_remaining: 0,
    };
    
    assert_eq!(stats.daily_remaining, 0);
    assert_eq!(stats.monthly_remaining, 0);
}

#[test]
fn test_quota_status_with_different_limits() {
    let custom_limits = QuotaLimits {
        rate_limit_per_minute: 200,
        daily_quota: 100_000,
        monthly_quota: 2_000_000,
    };
    
    let usage = QuotaUsageStats {
        today: 50_000,
        this_month: 1_000_000,
        daily_remaining: 50_000,
        monthly_remaining: 1_000_000,
    };
    
    let status = QuotaStatus {
        key_id: "key_premium".to_string(),
        limits: custom_limits,
        usage,
    };
    
    assert_eq!(status.limits.rate_limit_per_minute, 200);
    assert_eq!(status.usage.today, 50_000);
}