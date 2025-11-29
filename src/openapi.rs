use axum::{
    Router,
    http::StatusCode,
    routing::get,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers::{accounts, keys, transactions, usage};
use crate::models::{
    common::{Cursor, ErrorCode, ErrorDetail, ErrorResponse, PaginatedResponse, PaginationParams},
    finance::{Currency, FailureReason, TransactionFilters, TransactionStatus, TransactionType},
    keys::Scope,
    quota::{QuotaLimits, QuotaUsage, QuotaUsageStats, QuotaStatus},
    requests::{CreateAccountRequest, CreateApiKeyRequest, CreateTransactionRequest, UpdateAccountRequest, UpdateApiKeyRequest},
    responses::{AccountResponse, BalanceResponse, KeyCreatedResponse, KeyInfoResponse, TransactionResponse, UsageResponse},
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Metered Finance API",
        version = "1.0.0",
        description = r#"# Metered Finance API

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
```"#,
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
        // Account endpoints
        accounts::create_account,
        accounts::get_account,
        accounts::list_accounts,
        accounts::update_account,
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
        )
    ),
    tags(
        (name = "health", description = "Health Check Endpoints"),
        (name = "accounts", description = "Account management operations"),
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