use axum::{
    Router,
    http::StatusCode,
    routing::get,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::models::{
    common::{Cursor, ErrorCode, ErrorDetail, ErrorResponse, PaginatedResponse, PaginationParams},
    finance::{Currency, FailureReason, TransactionFilters, TransactionStatus, TransactionType},
    keys::Scope,
    quota::{QuotaLimits, QuotaUsage, QuotaUsageStats, QuotaStatus},
    requests::{CreateAccountRequest, CreateApiKeyRequest, CreateTransactionRequest, UpdateAccountRequest, UpdateApiKeyRequest},
    responses::{AccountResponse, BalanceResponse, KeyCreatedResponse, KeyInfoResponse, TransactionResponse, UsageResponse},
    analytics::{AnalyticsResponse, EndpointStats, HourlyVolume, RequestStats, StatusCodeStats, TimeRangeFilter},
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Metered Finance API",
        version = "1.0.0",
        description = r#"
A production-ready financial transactions API with built-in metering, rate limiting, and quota management.

## Features

- **Account Management**: Create and manage customer accounts
- **Transaction Processing**: Handle payments, refunds, payouts, and more
- **API Key Authentication**: Secure authentication with scoped API keys
- **Rate Limiting**: Per-minute request rate limiting
- **Quota Management**: Daily and monthly request quotas
- **Usage Tracking**: Monitor API usage and limits
- **Idempotency**: Safe request retry with idempotency keys

## Authentication

All API endpoints (except health checks) require authentication using API keys.

### Client Authentication
Include your API key in the `X-Api-Key` header:
```
X-Api-Key: sk_live_your_api_key_here
```

### Admin Authentication
For administrative endpoints, use the admin key in the `X-Admin-Key` header:
```
X-Admin-Key: your_admin_key_here
```

## Rate Limits

- **Default**: 60 requests per minute
- **Quota**: 10,000 requests per day, 300,000 per month
- Limits can be customized per API key

## Error Handling

All errors follow a consistent format:
```json
{
  "error": {
    "code": "error_code",
    "message": "Human readable message",
    "details": {}
  }
}
```
"#,
        contact(
            name = "API Support",
            email = "support@financely.com",
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    servers(
        (url = "http://localhost:3030", description = "Local Development Server"),
        (url = "https://staging.api.financely.com", description = "Staging Server"),
        (url = "https://api.financely.com", description = "Production Server"),
    ),
    paths(
        // Health check endpoints
        crate::handlers::health::health_live,
        crate::handlers::health::health_ready,
        
        // Account endpoints
        crate::handlers::accounts::create_account,
        crate::handlers::accounts::get_account,
        crate::handlers::accounts::list_accounts,
        crate::handlers::accounts::update_account,
        
        // Transaction endpoints
        crate::handlers::transactions::create_transaction,
        crate::handlers::transactions::get_transaction,
        crate::handlers::transactions::list_transactions,
        crate::handlers::transactions::get_account_transactions,
        crate::handlers::transactions::get_account_balance,
        
        // API Key management endpoints (Admin only)
        crate::handlers::keys::create_api_key,
        crate::handlers::keys::list_api_keys,
        crate::handlers::keys::get_api_key,
        crate::handlers::keys::update_api_key,
        crate::handlers::keys::delete_api_key,
        
        // Usage endpoints
        crate::handlers::usage::get_own_usage,
        crate::handlers::usage::get_key_usage,
        
        // Analytics endpoints
        crate::handlers::analytics::get_own_analytics,
        crate::handlers::analytics::get_key_analytics,
        crate::handlers::analytics::get_system_analytics,
    ),
    components(
        schemas(
            // Common schemas
            Cursor,
            PaginationParams,
            PaginatedResponse<AccountResponse>,
            PaginatedResponse<TransactionResponse>,
            PaginatedResponse<KeyInfoResponse>,
            ErrorResponse,
            ErrorDetail,
            ErrorCode,
            
            // Finance schemas
            Currency,
            TransactionType,
            TransactionStatus,
            FailureReason,
            TransactionFilters,
            
            // Request schemas
            CreateAccountRequest,
            UpdateAccountRequest,
            CreateTransactionRequest,
            CreateApiKeyRequest,
            UpdateApiKeyRequest,
            
            // Response schemas
            AccountResponse,
            TransactionResponse,
            BalanceResponse,
            KeyCreatedResponse,
            KeyInfoResponse,
            UsageResponse,
            
            // Key schemas
            Scope,
            
            // Quota schemas
            QuotaLimits,
            QuotaUsage,
            QuotaUsageStats,
            QuotaStatus,
            
            // Analytics schemas
            AnalyticsResponse,
            RequestStats,
            EndpointStats,
            StatusCodeStats,
            HourlyVolume,
            TimeRangeFilter,
        )
    ),
    tags(
        (name = "accounts", description = "Account management operations"),
        (name = "transactions", description = "Transaction processing and retrieval"),
        (name = "keys", description = "API key management (Admin only)"),
        (name = "usage", description = "Usage and quota monitoring"),
        (name = "analytics", description = "Request analytics and statistics"),
        (name = "health", description = "Health check endpoints"),
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "ApiKeyAuth",
                utoipa::openapi::security::SecurityScheme::ApiKey(
                    utoipa::openapi::security::ApiKey::Header(
                        utoipa::openapi::security::ApiKeyValue::new("X-Api-Key")
                    )
                )
            );
            components.add_security_scheme(
                "AdminKeyAuth",
                utoipa::openapi::security::SecurityScheme::ApiKey(
                    utoipa::openapi::security::ApiKey::Header(
                        utoipa::openapi::security::ApiKeyValue::new("X-Admin-Key")
                    )
                )
            );
        }
    }
}

pub fn openapi_routes() -> Router {
    Router::new()
        .merge(SwaggerUi::new("/docs").url("/openapi.json", ApiDoc::openapi()))
        .route("/openapi.yaml", get(openapi_yaml))
}

async fn openapi_yaml() -> Result<String, StatusCode> {
    serde_yaml::to_string(&ApiDoc::openapi())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}