# Metered Finance API

A production-ready RESTful API for managing financial transactions, accounts, and metered API usage with Rust and Axum.

## Features

### Core Functionality

- **Account Management** - Create and manage customer accounts with metadata
- **Transaction Processing** - Handle payments, refunds, payouts, and adjustments
- **API Key Management** - Secure API key generation and rotation (admin only)
- **Rate Limiting** - Token bucket algorithm with per-key limits
- **Quota Management** - Daily and monthly usage quotas per API key
- **Request Analytics** - Track request patterns, latency, and error rates

### Technical Features

- **RESTful API** - Clean, well-documented REST endpoints
- **OpenAPI/Swagger** - Interactive API documentation at `/docs`
- **Authentication** - API key-based auth with client and admin scopes
- **Database** - PostgreSQL with SQLx for type-safe queries
- **Migrations** - Versioned database migrations
- **Observability** - Structured logging, request tracing
- **Health Checks** - Liveness and readiness probes for orchestration
- **Docker Ready** - Multi-stage builds with optimized images
- **Production Ready** - Comprehensive error handling, validation, security headers

## Table of Contents

- [Quick Start](#quick-start)
- [API Documentation](#api-documentation)
- [Architecture](#architecture)
- [Development](#development)
- [Deployment](#deployment)
- [Testing](#testing)
- [Contributing](#contributing)
- [License](#license)

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) 1.91 or higher
- [Docker](https://www.docker.com/) and Docker Compose
- [SQLx CLI](https://github.com/launchbadge/sqlx/tree/main/sqlx-cli)
- [Neon DB](https://neon.tech) account (free tier works)

### Installation

1. **Clone the repository**

   ```bash
   git clone https://github.com/bkandh30/metered-finance-api.git
   cd metered-finance-api
   ```

2. **Set up environment**

   ```bash
   # Copy environment template
   cp .env.development .env

   # Update with your Neon DB credentials
   vim .env
   ```

3. **Run migrations**

   ```bash
   export DATABASE_URL="your-neon-db-url"
   sqlx migrate run
   ```

4. **Start the API**

   ```bash
   # With Docker
   docker-compose up -d

   # Or locally
   cargo run
   ```

5. **Access the API**
   - API: http://localhost:3030
   - Docs: http://localhost:3030/docs
   - Health: http://localhost:3030/health/ready

## API Documentation

### Base URL

```
http://localhost:3030/api
```

### Authentication

**Client API Key** (for most operations):

```bash
curl -H "X-Api-Key: sk_live_your_key_here" http://localhost:3030/api/accounts
```

**Admin Key** (for key management):

```bash
curl -H "X-Admin-Key: your_admin_key" http://localhost:3030/api/admin/keys
```

### Core Endpoints

#### Accounts

```bash
# Create account
POST /api/accounts
{
  "account_id": "acc_customer_001",
  "metadata": {"name": "John Doe"}
}

# Get account
GET /api/accounts/{account_id}

# List accounts (with pagination)
GET /api/accounts?limit=10&cursor=xxx

# Update account
PATCH /api/accounts/{account_id}
```

#### Transactions

```bash
# Create transaction
POST /api/transactions
{
  "transaction_id": "txn_001",
  "account_id": "acc_customer_001",
  "amount": 10000,
  "currency": "USD",
  "transaction_type": "payment",
  "metadata": {}
}

# Get transaction
GET /api/transactions/{transaction_id}

# List transactions
GET /api/transactions?status=completed&limit=10

# Get account balance
GET /api/accounts/{account_id}/balance
```

#### API Keys (Admin Only)

```bash
# Create API key
POST /api/admin/keys
{
  "name": "Production Key",
  "scope": "client",
  "quota_limits": {
    "max_daily_requests": 10000,
    "max_monthly_requests": 300000
  }
}

# List keys
GET /api/admin/keys

# Get key details
GET /api/admin/keys/{key_id}

# Update key
PATCH /api/admin/keys/{key_id}

# Delete key
DELETE /api/admin/keys/{key_id}
```

#### Analytics

```bash
# Get own usage analytics
GET /api/analytics?start=2024-01-01T00:00:00Z&end=2024-01-31T23:59:59Z

# Get analytics for specific key (admin)
GET /api/admin/analytics/{key_id}

# Get system-wide analytics (admin)
GET /api/admin/analytics
```

### Interactive Documentation

Full interactive API documentation with request/response examples:
👉 **http://localhost:3030/docs**

## Architecture

### Technology Stack

**Backend**:

- **Language**: Rust 2021 Edition
- **Framework**: Axum 0.8 (async web framework)
- **Database**: PostgreSQL (via Neon DB)
- **ORM**: SQLx (compile-time verified queries)
- **Async Runtime**: Tokio

**Observability**:

- **Logging**: Tracing + `tracing-subscriber`
- **Documentation**: OpenAPI 3.0 (via `utoipa`)

**Security**:

- **API Keys**: Argon2 hashing
- **Rate Limiting**: Token bucket algorithm (`tower-governor`)
- **CORS**: Configurable origins
- **Request ID**: Distributed tracing support

### Database Schema

**Core Tables**:

- `accounts` - Customer accounts
- `transactions` - Financial transactions
- `api_keys` - API authentication keys
- `quota_usage` - Usage tracking per key
- `requests` - Request logs for analytics

**Migrations**: Versioned SQL migrations in `migrations/`

### Request Flow

```
Client Request
    ↓
[Request ID Middleware] → Generates unique ID
    ↓
[Auth Middleware] → Validates API key
    ↓
[Rate Limit Middleware] → Checks quota
    ↓
[Request Logging Middleware] → Logs request
    ↓
[Handler] → Process business logic
    ↓
[Database] → PostgreSQL via SQLx
    ↓
[Response] → JSON with proper status codes
```

## Development

### Local Setup

```bash
# Install dependencies
cargo build

# Run tests
cargo test

# Run with hot reload (cargo-watch)
cargo install cargo-watch
cargo watch -x run

# Format code
cargo fmt

# Run linter
cargo clippy

# Generate documentation
cargo doc --open
```

### Environment Variables

Required variables (see `.env.development`):

```bash
PORT=3030
RUST_LOG=debug,axum=debug
APP_ENV=development
DATABASE_URL=postgres://...
ADMIN_KEY=your_admin_key
RATE_LIMIT_PER_MINUTE=120
QUOTA_DAILY_REQUESTS=5000
```

### Database Migrations

```bash
# Create new migration
sqlx migrate add migration_name

# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert

# Check status
sqlx migrate info
```

### Testing

```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --test '*'

# With coverage
cargo tarpaulin --out Html

# Benchmarks
cargo bench

# Load testing
k6 run tests/load-test.js
```

## 🚢 Deployment

### Docker

```bash
# Build image
docker build -t metered-finance-api:latest .

# Run container
docker run -p 3030:3030 --env-file .env metered-finance-api:latest

# With Docker Compose
docker-compose up -d
```

### Environment Management

```bash
# Switch to development
./switch-env.sh dev
docker-compose restart api

# Switch to production
./switch-env.sh prod
docker-compose restart api
```

### Production Checklist

- [ ] Update `ADMIN_KEY` to secure value
- [ ] Configure `DATABASE_URL` for production Neon DB
- [ ] Set `RUST_LOG=warn,metered_finance_api=info`
- [ ] Set `APP_ENV=production`
- [ ] Run migrations on production database
- [ ] Configure monitoring and alerting
- [ ] Set up SSL/TLS certificates
- [ ] Configure CORS for your domain
- [ ] Set up database backups (Neon automated)
- [ ] Review rate limits and quotas
- [ ] Test health endpoints
- [ ] Configure log aggregation

### Neon DB Setup

See [NEON_SETUP.md](NEON_SETUP.md) for detailed instructions on:

- Creating development and production branches
- Running migrations
- Switching between environments
- Backup and restore procedures

## Monitoring

### Health Checks

```bash
# Liveness probe (is the service running?)
curl http://localhost:3030/health/live

# Readiness probe (is the service ready to accept traffic?)
curl http://localhost:3030/health/ready
```

**Key Metrics**:

- `http_requests_total` - Total HTTP requests
- `http_request_duration_seconds` - Request latency
- `http_requests_in_progress` - Active requests
- `quota_usage_total` - API quota consumption

### Logs

Structured JSON logging with request correlation:

```json
{
  "timestamp": "2024-01-15T10:30:45Z",
  "level": "INFO",
  "target": "metered_finance_api",
  "request_id": "req_abc123",
  "message": "Account created",
  "account_id": "acc_001"
}
```

## Security

### Best Practices Implemented

- API key authentication with Argon2 hashing
- Rate limiting per API key
- Input validation and sanitization
- SQL injection prevention (parameterized queries)
- CORS configuration
- Request size limits
- Secure headers
- TLS/SSL for database connections
- No secrets in code or logs
- Principle of least privilege (admin vs client scopes)

### Security Headers

```
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-XSS-Protection: 1; mode=block
Strict-Transport-Security: max-age=31536000
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
