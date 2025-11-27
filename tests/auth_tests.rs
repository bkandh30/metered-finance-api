use metered_finance_api::middleware::auth::{require_scope, ClientAuth, AdminAuth};
use metered_finance_api::models::keys::{AuthContext, Scope};

#[test]
fn test_require_scope_admin_has_all_scopes() {
    let admin_ctx = AuthContext::Admin;
    
    assert!(require_scope(&admin_ctx, &Scope::Client).is_ok());
    assert!(require_scope(&admin_ctx, &Scope::Admin).is_ok());
    assert!(require_scope(&admin_ctx, &Scope::Reporting).is_ok());
}

#[test]
fn test_require_scope_client_with_scopes() {
    let client_ctx = AuthContext::Client {
        key_id: "key_test123".to_string(),
        scopes: vec![Scope::Client, Scope::Reporting],
    };
    
    assert!(require_scope(&client_ctx, &Scope::Client).is_ok());
    assert!(require_scope(&client_ctx, &Scope::Reporting).is_ok());
    
    assert!(require_scope(&client_ctx, &Scope::Admin).is_err());
}

#[test]
fn test_require_scope_client_without_scope() {
    let client_ctx = AuthContext::Client {
        key_id: "key_test456".to_string(),
        scopes: vec![Scope::Client],
    };
    
    assert!(require_scope(&client_ctx, &Scope::Client).is_ok());
    
    let result = require_scope(&client_ctx, &Scope::Admin);
    assert!(result.is_err());
    
    if let Err((status, msg)) = result {
        assert_eq!(status, axum::http::StatusCode::FORBIDDEN);
        assert!(msg.contains("admin"));
    }
}

#[test]
fn test_require_scope_error_message() {
    let client_ctx = AuthContext::Client {
        key_id: "key_test789".to_string(),
        scopes: vec![],
    };
    
    let result = require_scope(&client_ctx, &Scope::Reporting);
    assert!(result.is_err());
    
    if let Err((status, msg)) = result {
        assert_eq!(status, axum::http::StatusCode::FORBIDDEN);
        assert!(msg.contains("reporting"));
    }
}

#[test]
fn test_auth_context_has_scope() {
    let admin_ctx = AuthContext::Admin;
    assert!(admin_ctx.has_scope(&Scope::Client));
    assert!(admin_ctx.has_scope(&Scope::Admin));
    assert!(admin_ctx.has_scope(&Scope::Reporting));
    
    let client_ctx = AuthContext::Client {
        key_id: "key_abc".to_string(),
        scopes: vec![Scope::Client],
    };
    assert!(client_ctx.has_scope(&Scope::Client));
    assert!(!client_ctx.has_scope(&Scope::Admin));
}

#[test]
fn test_auth_context_is_admin() {
    let admin_ctx = AuthContext::Admin;
    assert!(admin_ctx.is_admin());
    
    let client_ctx = AuthContext::Client {
        key_id: "key_xyz".to_string(),
        scopes: vec![Scope::Admin],
    };
    assert!(!client_ctx.is_admin());
}

#[test]
fn test_auth_context_key_id() {
    let admin_ctx = AuthContext::Admin;
    assert_eq!(admin_ctx.key_id(), None);
    
    let client_ctx = AuthContext::Client {
        key_id: "key_test_123".to_string(),
        scopes: vec![Scope::Client],
    };
    assert_eq!(client_ctx.key_id(), Some("key_test_123"));
}

#[test]
fn test_multiple_scopes() {
    let ctx = AuthContext::Client {
        key_id: "key_multi".to_string(),
        scopes: vec![Scope::Client, Scope::Reporting],
    };
    
    assert!(require_scope(&ctx, &Scope::Client).is_ok());
    assert!(require_scope(&ctx, &Scope::Reporting).is_ok());
    assert!(require_scope(&ctx, &Scope::Admin).is_err());
}

#[test]
fn test_empty_scopes() {
    let ctx = AuthContext::Client {
        key_id: "key_empty".to_string(),
        scopes: vec![],
    };
    
    assert!(require_scope(&ctx, &Scope::Client).is_err());
    assert!(require_scope(&ctx, &Scope::Admin).is_err());
    assert!(require_scope(&ctx, &Scope::Reporting).is_err());
}