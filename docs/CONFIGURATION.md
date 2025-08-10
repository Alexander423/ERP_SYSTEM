# ⚙️ Configuration Management Guide

## Table of Contents

- [Overview](#overview)
- [Configuration Hierarchy](#configuration-hierarchy)
- [Environment-Specific Settings](#environment-specific-settings)
- [Security Configuration](#security-configuration)
- [Database Configuration](#database-configuration)
- [Redis Configuration](#redis-configuration)
- [Email Configuration](#email-configuration)
- [CORS Configuration](#cors-configuration)
- [Monitoring Configuration](#monitoring-configuration)
- [Environment Variables Reference](#environment-variables-reference)

## Overview

The ERP System uses a hierarchical configuration system that supports multiple environments and secure secret management. Configuration is loaded from:

1. **TOML files** - Base configuration and environment-specific overrides
2. **Environment variables** - Runtime secrets and deployment-specific values
3. **Command-line arguments** - Runtime parameters (future enhancement)

### Configuration Philosophy

- **Security First**: Sensitive values never stored in files
- **Environment Specific**: Different settings for dev/test/prod
- **Runtime Flexible**: Environment variables override file settings
- **Type Safe**: Compile-time validation of configuration schema
- **Hierarchical**: Logical grouping of related settings

## Configuration Hierarchy

### Loading Order (Increasing Precedence)

```text
1. config/default.toml      (Base settings)
     ↓
2. config/{environment}.toml (Environment overrides)  
     ↓
3. Environment Variables    (Runtime secrets)
```

### Directory Structure

```text
config/
├── default.toml           # Base configuration for all environments
├── development.toml       # Development-specific overrides
├── testing.toml          # Test environment settings
└── production.toml       # Production environment settings
```

### Configuration Sections

```rust
pub struct Config {
    pub database: DatabaseConfig,        // PostgreSQL settings
    pub redis: RedisConfig,             // Cache and session store
    pub jwt: JwtConfig,                 // Token signing and expiry
    pub security: SecurityConfig,       // Crypto parameters
    pub server: ServerConfig,           // HTTP server settings
    pub rate_limit: RateLimitConfig,    // API throttling
    pub email: EmailConfig,             // Email service providers
    pub app: AppConfig,                 // Application-level settings
    pub metrics: MetricsConfig,         // Prometheus monitoring
    pub cors: CorsConfig,               // Cross-origin policies
}
```

## Environment-Specific Settings

### Environment Selection

The active environment is determined by the `ENVIRONMENT` environment variable:

```bash
export ENVIRONMENT=production    # Use production.toml
export ENVIRONMENT=development   # Use development.toml (default)
export ENVIRONMENT=testing       # Use testing.toml
```

### Development Environment

**File**: `config/development.toml`

```toml
# Development-focused settings
[app]
base_url = "http://localhost:3000"
log_level = "debug"

[database]
url = "postgresql://erp_dev:dev_password@localhost:5432/erp_dev"
max_connections = 5
min_connections = 1

[jwt]
access_token_expiry = 7200    # 2 hours for longer development sessions

[security]
argon2_memory_cost = 32768    # 32 MB for faster dev cycles
argon2_time_cost = 2

[email]
provider = "mock"             # No real emails in development

[cors]
allowed_origins = ["*"]       # Permissive for development
allowed_headers = ["*"]

[metrics]
enabled = true
port = 9091                   # Different port to avoid conflicts
```

### Testing Environment

**File**: `config/testing.toml`

```toml
# Optimized for fast test execution
[app]
base_url = "http://localhost:3001"
environment = "testing"
log_level = "warn"            # Minimal logging for faster tests

[server]
port = 3001                   # Different port for test isolation
workers = 2

[database]
url = "postgresql://erp_test:test_password@localhost:5432/erp_test"
max_connections = 3           # Minimal connections for tests
min_connections = 1

[redis]
url = "redis://localhost:6380"  # Separate Redis instance
max_connections = 3

[jwt]
access_token_expiry = 300     # 5 minutes for quick tests
refresh_token_expiry = 900    # 15 minutes

[security]
argon2_memory_cost = 16384    # 16 MB for faster test execution
argon2_time_cost = 1
argon2_parallelism = 1

[rate_limit]
requests_per_minute = 1000    # Relaxed for testing
burst_size = 100

[email]
provider = "mock"             # Always mock in tests

[metrics]
enabled = false               # Disabled for testing
```

### Production Environment

**File**: `config/production.toml`

```toml
# Production security and performance settings
[app]
environment = "production"
log_level = "info"
# base_url set via BASE_URL environment variable
# company_name set via COMPANY_NAME environment variable

[server]
host = "0.0.0.0"             # Bind to all interfaces
workers = 8                   # More workers for production load

[database]
# URL set via DATABASE_URL environment variable
max_connections = 20          # Higher connection pool
min_connections = 5

[redis]
# URL set via REDIS_URL environment variable  
max_connections = 20

[jwt]
# secret set via JWT_SECRET environment variable
access_token_expiry = 1800    # 30 minutes (shorter for security)
refresh_token_expiry = 604800 # 7 days

[security]
argon2_memory_cost = 131072   # 128 MB for maximum security
argon2_time_cost = 4
argon2_parallelism = 4
# aes_encryption_key set via AES_ENCRYPTION_KEY environment variable

[rate_limit]
requests_per_minute = 300     # Production rate limiting
burst_size = 50

[email]
provider = "smtp"             # Real email service
# All SMTP credentials set via environment variables
use_tls = true
timeout_seconds = 60
max_retries = 5

[cors]
# allowed_origins set via CORS_ALLOWED_ORIGINS (no wildcards!)
allowed_methods = ["GET", "POST", "PUT", "DELETE", "OPTIONS"]
allowed_headers = ["authorization", "content-type", "x-request-id", "accept"]
allow_credentials = true
max_age = 7200               # 2 hours

[metrics]
enabled = true
port = 9090
path = "/metrics"
namespace = "erp_production"
```

## Security Configuration

### Cryptographic Settings

```toml
[security]
# Argon2id password hashing (memory-hard, side-channel resistant)
argon2_memory_cost = 65536    # Memory usage in KiB (64 MB)
argon2_time_cost = 3          # Iteration count
argon2_parallelism = 2        # Thread count

# AES-GCM encryption for sensitive data at rest
aes_encryption_key = "32-char-key"  # Must be exactly 32 characters

# JWT token settings
[jwt]
secret = "jwt-signing-secret"    # Minimum 32 characters
access_token_expiry = 1800       # 30 minutes (seconds)
refresh_token_expiry = 604800    # 7 days (seconds)
```

### Security Best Practices

#### Development
- Use mock services where possible
- Lower cryptographic parameters for speed
- Permissive CORS for easy frontend development

#### Production  
- **Never use wildcards** in CORS origins
- Set maximum cryptographic parameters for security
- Use managed secrets (Kubernetes secrets, AWS Secrets Manager)
- Enable comprehensive audit logging

### Environment Variable Security

```bash
# ✅ GOOD: Strong, random secrets
JWT_SECRET=$(openssl rand -base64 32)
AES_ENCRYPTION_KEY=$(openssl rand -base64 32 | cut -c1-32)

# ✅ GOOD: Database with strong authentication
DATABASE_URL="postgresql://user:$(openssl rand -base64 20)@host:5432/db"

# ❌ BAD: Weak or predictable secrets
JWT_SECRET="secret123"
DATABASE_URL="postgresql://user:password@host:5432/db"
```

## Database Configuration

### Connection Pool Settings

```toml
[database]
url = "postgresql://username:password@host:port/database"
max_connections = 20          # Maximum pool size
min_connections = 5           # Minimum maintained connections
```

### Multi-Tenant Considerations

- **Main Pool**: Used for global operations and tenant management
- **Tenant Pools**: Created dynamically with 1/4 of main pool size
- **Schema Isolation**: Each tenant uses a separate PostgreSQL schema
- **Connection Efficiency**: Pools are cached and reused

### Environment-Specific Tuning

| Environment | Max Connections | Min Connections | Reasoning |
|-------------|----------------|-----------------|-----------|
| **Development** | 5 | 1 | Resource efficiency on dev machines |
| **Testing** | 3 | 1 | Minimal for test isolation |
| **Production** | 20-50 | 5 | High concurrency handling |

### Health Monitoring

The system performs automatic database health checks:

```sql
-- Simple connectivity test
SELECT 1;

-- Connection pool status monitoring
SELECT 
    state,
    count(*) 
FROM pg_stat_activity 
WHERE datname = current_database() 
GROUP BY state;
```

## Redis Configuration

### Basic Settings

```toml
[redis]
url = "redis://[:password@]host:port[/db]"
max_connections = 10          # Connection pool size
```

### Redis Usage Patterns

1. **Session Storage**: JWT refresh tokens and user sessions
2. **Rate Limiting**: Request counters and rate limit tracking
3. **Caching**: Frequently accessed user permissions and tenant data
4. **Job Queue**: Background task processing (future enhancement)

### Redis Data Structure

```text
Keys Pattern:
- session:{user_id}:{token_id}     # User session data
- rate_limit:{tenant_id}:{endpoint} # Rate limiting counters
- cache:permissions:{user_id}       # Cached user permissions
- job_queue:auth:{job_id}          # Background jobs
```

### High Availability Setup

For production deployments:

```toml
[redis]
# Redis Cluster or Sentinel setup
url = "redis://redis-cluster:6379"
max_connections = 20

# Alternative: Redis Sentinel
# url = "redis-sentinel://sentinel1:26379,sentinel2:26379/mymaster"
```

## Email Configuration

### Provider Selection

```toml
[email]
provider = "smtp"  # Options: "mock", "smtp", "sendgrid", "aws_ses"
```

### SMTP Configuration

```toml
[email]
provider = "smtp"
smtp_host = "smtp.gmail.com"
smtp_port = 587
smtp_username = "noreply@company.com"
smtp_password = "app-password"      # Use app passwords, not account passwords
smtp_from_email = "noreply@company.com"
smtp_from_name = "ERP System"
use_tls = true
use_starttls = true
timeout_seconds = 30
max_retries = 3
```

### SendGrid Configuration

```toml
[email]
provider = "sendgrid"
sendgrid_api_key = "SG.xxx"        # Set via SENDGRID_API_KEY env var
smtp_from_email = "noreply@company.com"
smtp_from_name = "ERP System"
timeout_seconds = 30
max_retries = 3
```

### AWS SES Configuration

```toml
[email]
provider = "aws_ses"
aws_region = "us-east-1"
aws_access_key_id = "AKIA..."      # Set via AWS_ACCESS_KEY_ID env var
aws_secret_access_key = "xxx"      # Set via AWS_SECRET_ACCESS_KEY env var
smtp_from_email = "noreply@company.com"
smtp_from_name = "ERP System"
timeout_seconds = 30
max_retries = 3
```

### Email Templates

The system uses embedded email templates for:
- **Account verification**: Welcome email with verification link
- **Password reset**: Secure password reset with time-limited token
- **Two-factor setup**: QR code for TOTP authenticator setup
- **Security alerts**: Account lockout and suspicious activity notifications

## CORS Configuration

### Security Levels by Environment

#### Development (Permissive)
```toml
[cors]
allowed_origins = ["*"]             # Allow all origins
allowed_methods = ["*"]             # Allow all methods  
allowed_headers = ["*"]             # Allow all headers
allow_credentials = true
max_age = 86400                     # 24 hours
```

#### Production (Restrictive)
```toml
[cors]
allowed_origins = [                 # Specific domains only
    "https://app.company.com",
    "https://admin.company.com"
]
allowed_methods = [
    "GET", "POST", "PUT", "DELETE", "OPTIONS"
]
allowed_headers = [
    "authorization", "content-type", "x-request-id", "accept"
]
expose_headers = ["x-request-id"]
allow_credentials = true
max_age = 7200                      # 2 hours
```

### CORS Security Best Practices

1. **Never use wildcards in production**
2. **Specify exact domains**: Include protocol (https/http)
3. **Limit headers**: Only include necessary headers
4. **Monitor CORS errors**: Log blocked requests for security analysis
5. **Regular audits**: Review and update allowed origins regularly

## Monitoring Configuration

### Prometheus Metrics

```toml
[metrics]
enabled = true
port = 9090
path = "/metrics"
namespace = "erp_system"
```

### Available Metrics

The system exposes comprehensive metrics:

#### Authentication Metrics
- `erp_login_attempts_total` - Login attempt counters
- `erp_login_failures_total` - Failed login counters  
- `erp_2fa_verifications_total` - 2FA verification counters
- `erp_account_lockouts_total` - Account lockout counters

#### Performance Metrics
- `erp_request_duration_seconds` - Request processing times
- `erp_database_query_duration_seconds` - Database query times
- `erp_redis_operation_duration_seconds` - Redis operation times

#### System Metrics
- `erp_active_connections` - Database connection counts
- `erp_memory_usage_bytes` - Memory usage tracking
- `erp_goroutines_total` - Active goroutine count

### Grafana Dashboard

Example Grafana queries for monitoring:

```promql
# Request rate
rate(erp_http_requests_total[5m])

# Error rate
rate(erp_http_requests_total{status=~"5.."}[5m]) / 
rate(erp_http_requests_total[5m])

# Response time percentiles  
histogram_quantile(0.95, rate(erp_request_duration_seconds_bucket[5m]))

# Database connection pool utilization
erp_database_connections_active / erp_database_connections_max
```

## Environment Variables Reference

### Complete Environment Variables List

```bash
# ==========================================
# Core Application
# ==========================================
ENVIRONMENT=development                    # Environment selection
LOG_LEVEL=info                            # Logging verbosity
RUST_LOG=info,erp_api=debug               # Rust-specific logging

# ==========================================
# Server Configuration
# ==========================================
SERVER_HOST=127.0.0.1                    # Bind address
SERVER_PORT=3000                          # HTTP port
SERVER_WORKERS=4                          # Worker thread count

# ==========================================
# Database (Critical - Set via Env Vars)
# ==========================================
DATABASE_URL=postgresql://...             # PostgreSQL connection URL
DATABASE_MAX_CONNECTIONS=20               # Connection pool max size
DATABASE_MIN_CONNECTIONS=5                # Connection pool min size

# ==========================================
# Redis (Critical - Set via Env Vars)
# ==========================================
REDIS_URL=redis://...                    # Redis connection URL
REDIS_MAX_CONNECTIONS=10                  # Redis connection pool size

# ==========================================
# Security (Critical - Never in Files)
# ==========================================
JWT_SECRET=...                           # JWT signing secret (32+ chars)
AES_ENCRYPTION_KEY=...                   # AES encryption key (32 chars)
ARGON2_MEMORY_COST=65536                 # Argon2 memory parameter
ARGON2_TIME_COST=3                       # Argon2 iteration count
ARGON2_PARALLELISM=2                     # Argon2 thread count

# ==========================================
# Application Settings
# ==========================================
APP_COMPANY_NAME="Your Company"           # Branding
APP_BASE_URL=https://api.company.com     # Base URL for emails/links
APP_ENABLE_REGISTRATION=true             # Feature flags
APP_ENABLE_2FA=true
APP_ENABLE_EMAIL_VERIFICATION=true

# ==========================================
# CORS (Production - Never Wildcards)
# ==========================================
CORS_ALLOWED_ORIGINS=https://app.com,https://admin.com
CORS_ALLOWED_METHODS=GET,POST,PUT,DELETE,OPTIONS
CORS_ALLOWED_HEADERS=authorization,content-type
CORS_ALLOW_CREDENTIALS=true
CORS_MAX_AGE=3600

# ==========================================
# Email Service (Provider-Specific)
# ==========================================
EMAIL_PROVIDER=smtp                       # smtp, sendgrid, aws_ses, mock

# SMTP Settings
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=noreply@company.com
SMTP_PASSWORD=...                        # App password
SMTP_FROM_EMAIL=noreply@company.com
SMTP_FROM_NAME="ERP System"
SMTP_USE_TLS=true
SMTP_USE_STARTTLS=true

# SendGrid Settings  
SENDGRID_API_KEY=SG....                  # SendGrid API key

# AWS SES Settings
AWS_REGION=us-east-1
AWS_ACCESS_KEY_ID=AKIA...                # AWS credentials
AWS_SECRET_ACCESS_KEY=...

# ==========================================
# Rate Limiting
# ==========================================
RATE_LIMIT_REQUESTS_PER_MINUTE=60
RATE_LIMIT_BURST_SIZE=10

# ==========================================
# Monitoring & Metrics
# ==========================================
METRICS_ENABLED=true                     # Enable Prometheus metrics
METRICS_PORT=9090                        # Metrics server port  
METRICS_PATH=/metrics                    # Metrics endpoint
METRICS_NAMESPACE=erp_production         # Metric name prefix
```

### Environment Variable Validation

The system validates environment variables at startup:

```rust
// Example validation logic
pub fn validate_config(config: &Config) -> Result<(), ConfigError> {
    // JWT secret must be at least 32 characters
    if config.jwt.secret.len() < 32 {
        return Err(ConfigError::ValidationError(
            "JWT_SECRET must be at least 32 characters".to_string()
        ));
    }
    
    // AES key must be exactly 32 characters
    if config.security.aes_encryption_key.len() != 32 {
        return Err(ConfigError::ValidationError(
            "AES_ENCRYPTION_KEY must be exactly 32 characters".to_string()
        ));
    }
    
    // Production CORS cannot use wildcards
    if config.app.environment == "production" {
        if config.cors.allowed_origins.contains(&"*".to_string()) {
            return Err(ConfigError::ValidationError(
                "CORS wildcards not allowed in production".to_string()
            ));
        }
    }
    
    Ok(())
}
```

---

## Quick Reference

### Development Setup
```bash
cp .env.example .env
export ENVIRONMENT=development
cargo run --bin erp-api
```

### Production Deployment
```bash
export ENVIRONMENT=production
export DATABASE_URL="postgresql://..."
export JWT_SECRET=$(openssl rand -base64 32)
export AES_ENCRYPTION_KEY=$(openssl rand -base64 32 | cut -c1-32)
cargo run --bin erp-api --release
```

### Configuration Testing
```bash
# Test configuration loading
ENVIRONMENT=testing cargo test test_config_loading

# Validate production config
ENVIRONMENT=production cargo run --bin validate-config
```

This configuration system provides the flexibility needed for different deployment scenarios while maintaining security and type safety throughout the application lifecycle.