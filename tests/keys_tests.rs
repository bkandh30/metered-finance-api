use metered_finance_api::models::keys::*;

#[test]
fn test_scope_serialization() {
    let client = Scope::Client;
    let json = serde_json::to_string(&client).unwrap();
    assert_eq!(json, "\"client\"");
    
    let deserialized: Scope = serde_json::from_str("\"admin\"").unwrap();
    assert_eq!(deserialized, Scope::Admin);
}

#[test]
fn test_scope_from_str() {
    assert_eq!(Scope::from_str("client"), Some(Scope::Client));
    assert_eq!(Scope::from_str("admin"), Some(Scope::Admin));
    assert_eq!(Scope::from_str("reporting"), Some(Scope::Reporting));
    assert_eq!(Scope::from_str("invalid"), None);
    assert_eq!(Scope::from_str("CLIENT"), None);
}

#[test]
fn test_api_key_generation() {
    let (key, prefix) = ApiKeyGenerator::generate("sk_test");
    assert!(key.starts_with("sk_test_"));
    assert_eq!(prefix, "sk_test");
    assert!(key.len() > 40);
    
    let (key2, _) = ApiKeyGenerator::generate("sk_test");
    assert_ne!(key, key2);
}

#[test]
fn test_api_key_generation_different_prefixes() {
    let (live_key, live_prefix) = ApiKeyGenerator::generate("sk_live");
    assert!(live_key.starts_with("sk_live_"));
    assert_eq!(live_prefix, "sk_live");
    
    let (test_key, test_prefix) = ApiKeyGenerator::generate("sk_test");
    assert!(test_key.starts_with("sk_test_"));
    assert_eq!(test_prefix, "sk_test");
}

#[test]
fn test_secret_hashing_and_verification() {
    let secret = "test_secret_key_12345";
    let hash = ApiKeyGenerator::hash_secret(secret).unwrap();
    
    assert!(ApiKeyGenerator::verify_secret(secret, &hash));
    
    assert!(!ApiKeyGenerator::verify_secret("wrong_secret", &hash));
    assert!(!ApiKeyGenerator::verify_secret("", &hash));
}

#[test]
fn test_secret_hashing_produces_different_hashes() {
    let secret = "same_secret";
    let hash1 = ApiKeyGenerator::hash_secret(secret).unwrap();
    let hash2 = ApiKeyGenerator::hash_secret(secret).unwrap();
    
    assert_ne!(hash1, hash2);
    
    assert!(ApiKeyGenerator::verify_secret(secret, &hash1));
    assert!(ApiKeyGenerator::verify_secret(secret, &hash2));
}

#[test]
fn test_prefix_extraction() {
    let key = "sk_test_abc123def456";
    let prefix = ApiKeyGenerator::extract_prefix(key);
    assert_eq!(prefix, Some("sk_test".to_string()));
    
    let live_key = "sk_live_xyz789";
    let live_prefix = ApiKeyGenerator::extract_prefix(live_key);
    assert_eq!(live_prefix, Some("sk_live".to_string()));
}

#[test]
fn test_prefix_extraction_invalid() {
    let invalid = "invalidkey";
    assert_eq!(ApiKeyGenerator::extract_prefix(invalid), None);
    
    let single_part = "sk_test";
    assert_eq!(ApiKeyGenerator::extract_prefix(single_part), None);
}

#[test]
fn test_auth_context_client_permissions() {
    let client_ctx = AuthContext::Client {
        key_id: "key_123".to_string(),
        scopes: vec![Scope::Client, Scope::Reporting],
    };
    
    assert!(client_ctx.has_scope(&Scope::Client));
    assert!(client_ctx.has_scope(&Scope::Reporting));
    assert!(!client_ctx.has_scope(&Scope::Admin));
    assert!(!client_ctx.is_admin());
    assert_eq!(client_ctx.key_id(), Some("key_123"));
}

#[test]
fn test_auth_context_admin_permissions() {
    let admin_ctx = AuthContext::Admin;
    
    assert!(admin_ctx.has_scope(&Scope::Client));
    assert!(admin_ctx.has_scope(&Scope::Admin));
    assert!(admin_ctx.has_scope(&Scope::Reporting));
    assert!(admin_ctx.is_admin());
    assert_eq!(admin_ctx.key_id(), None);
}

#[test]
fn test_auth_context_empty_scopes() {
    let ctx = AuthContext::Client {
        key_id: "key_456".to_string(),
        scopes: vec![],
    };
    
    assert!(!ctx.has_scope(&Scope::Client));
    assert!(!ctx.has_scope(&Scope::Admin));
    assert!(!ctx.has_scope(&Scope::Reporting));
}

#[test]
fn test_create_api_key_validation_valid() {
    let valid = CreateApiKeyRequest {
        scopes: vec![Scope::Client, Scope::Reporting],
    };
    assert!(valid.validate().is_ok());
}

#[test]
fn test_create_api_key_validation_empty_scopes() {
    let empty_scopes = CreateApiKeyRequest {
        scopes: vec![],
    };
    assert!(matches!(empty_scopes.validate(), Err(ValidationError::EmptyScopes)));
}

#[test]
fn test_create_api_key_validation_duplicate_scopes() {
    let duplicate_scopes = CreateApiKeyRequest {
        scopes: vec![Scope::Client, Scope::Client],
    };
    assert!(matches!(duplicate_scopes.validate(), Err(ValidationError::DuplicateScopes)));
}

#[test]
fn test_create_api_key_validation_multiple_duplicates() {
    let request = CreateApiKeyRequest {
        scopes: vec![Scope::Client, Scope::Admin, Scope::Client, Scope::Admin],
    };
    assert!(matches!(request.validate(), Err(ValidationError::DuplicateScopes)));
}

#[test]
fn test_key_id_generation() {
    let key_id = generate_key_id();
    assert!(key_id.starts_with("key_"));
    assert!(key_id.len() > 4);
    
    let key_id2 = generate_key_id();
    assert_ne!(key_id, key_id2);
}

#[test]
fn test_verify_with_invalid_hash() {
    let secret = "test_secret";
    let invalid_hash = "not_a_valid_hash";
    
    assert!(!ApiKeyGenerator::verify_secret(secret, invalid_hash));
}