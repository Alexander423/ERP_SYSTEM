# ERP System - Deployment Guide

**Status**: ⚠️ **Development Setup Guide** - Not production ready

This guide covers how to set up the ERP system locally for development and provides future deployment options.

## 📋 Table of Contents

1. [Current Reality](#current-reality)
2. [Local Development Setup](#local-development-setup)
3. [Docker Development Environment](#docker-development-environment)
4. [Database Setup](#database-setup)
5. [Configuration Guide](#configuration-guide)
6. [Common Issues & Troubleshooting](#common-issues--troubleshooting)
7. [Future Deployment Options](#future-deployment-options)

## 🔍 Current Reality

### What This System Actually Is

- **Development Stage**: Foundation layer with basic HTTP server
- **Local Development Only**: Not ready for production deployment
- **Mock Implementation**: Most APIs return placeholder data
- **Learning Platform**: Good for development and Rust learning

### What Works For Deployment

✅ **Currently Working:**
- Local development setup with Docker Compose
- HTTP API server (basic functionality)
- PostgreSQL database with migrations
- Basic configuration system

❌ **Not Production Ready:**
- No load balancing or high availability
- Limited security implementation
- No monitoring or observability
- Mock authentication and business logic

## 🚀 Local Development Setup

### Prerequisites

**Required Software:**
- **Rust 1.70+**: https://rustup.rs/
- **Docker & Docker Compose**: https://docker.com/
- **Git**: Version control

**Optional Tools:**
- **PostgreSQL Client** (psql): Database management
- **curl or Postman**: API testing

### Quick Start

1. **Clone Repository**
   ```bash
   git clone <repository-url>
   cd ERP
   ```

2. **Start Infrastructure**
   ```bash
   # Start PostgreSQL and Redis containers
   docker-compose up -d

   # Verify containers are running
   docker-compose ps
   ```

3. **Configure Environment**
   ```bash
   # Set environment variables
   export DATABASE_URL="postgresql://erp_admin:erp_secure_password_change_in_production@localhost:5432/erp_main"
   export REDIS_URL="redis://localhost:6379"
   export JWT_SECRET="your-jwt-secret-min-32-characters-long"
   export AES_ENCRYPTION_KEY="exactly-32-character-aes-key-here"
   ```

4. **Initialize Database**
   ```bash
   # Run database migrations
   cargo sqlx migrate run

   # Verify database setup
   psql $DATABASE_URL -c "SELECT version();"
   ```

5. **Build and Start**
   ```bash
   # Build the application
   cargo build --all

   # Start the API server
   cargo run -p erp-api

   # Server should be running on http://localhost:3000
   ```

6. **Verify Setup**
   ```bash
   # Test health endpoint
   curl http://localhost:3000/health

   # Expected response:
   # {"status":"healthy","timestamp":"2024-12-16T..."}
   ```

## 🐳 Docker Development Environment

### Docker Compose Setup

The project includes a Docker Compose configuration for development infrastructure:

```yaml
# docker-compose.yml (overview)
version: '3.8'
services:
  postgres:
    image: postgres:15
    ports:
      - "5432:5432"
    environment:
      POSTGRES_DB: erp_main
      POSTGRES_USER: erp_admin
      POSTGRES_PASSWORD: erp_secure_password_change_in_production

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"

  pgadmin:
    image: dpage/pgadmin4
    ports:
      - "5050:80"
    profiles:
      - debug
```

### Docker Commands

```bash
# Start all services
docker-compose up -d

# Start with pgAdmin for database management
docker-compose --profile debug up -d

# View logs
docker-compose logs -f postgres
docker-compose logs -f redis

# Stop all services
docker-compose down

# Reset everything (delete volumes)
docker-compose down -v

# Restart individual service
docker-compose restart postgres
```

### Accessing Services

- **PostgreSQL**: localhost:5432
  - Username: `erp_admin`
  - Password: `erp_secure_password_change_in_production`
  - Database: `erp_main`

- **Redis**: localhost:6379 (no password)

- **pgAdmin**: http://localhost:5050 (when using debug profile)
  - Email: `admin@admin.com`
  - Password: `admin`

## 💾 Database Setup

### Initial Database Creation

```bash
# Connect to PostgreSQL
psql "postgresql://erp_admin:erp_secure_password_change_in_production@localhost:5432/erp_main"

# List databases
\l

# Connect to erp_main database
\c erp_main

# List tables
\dt

# Check migration status
SELECT * FROM _sqlx_migrations ORDER BY version;
```

### Running Migrations

```bash
# Install sqlx-cli (if not installed)
cargo install sqlx-cli --no-default-features --features postgres

# Run all migrations
cargo sqlx migrate run

# Check migration status
cargo sqlx migrate info

# Revert last migration (if needed)
cargo sqlx migrate revert
```

### Database Schema Overview

```sql
-- Core tables (after migrations)
\dt public.*

-- Example tables:
-- - users: User accounts and authentication
-- - tenants: Multi-tenant organization data
-- - customers: Customer master data (main business entity)
-- - _sqlx_migrations: Migration tracking
```

### Database Backup & Restore

```bash
# Create backup
pg_dump $DATABASE_URL > backup_$(date +%Y%m%d_%H%M%S).sql

# Restore from backup
psql $DATABASE_URL < backup_file.sql

# Reset database to clean state
docker-compose down -v
docker-compose up -d
cargo sqlx migrate run
```

## ⚙️ Configuration Guide

### Environment Variables

**Required Configuration:**
```bash
# Database connection
export DATABASE_URL="postgresql://user:pass@host:port/database"

# Redis connection (optional, for future features)
export REDIS_URL="redis://host:port"

# Security settings
export JWT_SECRET="minimum-32-character-secret-for-jwt-signing"
export AES_ENCRYPTION_KEY="exactly-32-character-encryption-key-here"

# Server settings
export SERVER_PORT="3000"
export ENVIRONMENT="development"
```

**Optional Configuration:**
```bash
# Logging
export RUST_LOG="info,erp_api=debug,erp_auth=debug,erp_core=debug"

# CORS (for frontend development)
export CORS_ALLOWED_ORIGINS="http://localhost:3000,http://127.0.0.1:3000"
```

### Configuration Files

The system supports TOML configuration files in the `config/` directory:

```toml
# config/development.toml
[server]
host = "127.0.0.1"
port = 3000

[database]
max_connections = 5
min_connections = 1

[cors]
allowed_origins = ["*"]  # Development only
allowed_methods = ["GET", "POST", "PUT", "DELETE", "OPTIONS"]
```

### Validation

The system validates configuration at startup:

```bash
# Test configuration loading
cargo run -p erp-api --help

# If configuration is invalid, you'll see detailed error messages
# Example: "JWT secret must be at least 32 characters"
```

## 🔧 Common Issues & Troubleshooting

### Build Issues

**Problem**: Cargo build fails with dependency errors
```bash
# Solution: Clean and rebuild
cargo clean
cargo update
cargo build --all
```

**Problem**: SQLX compilation errors
```bash
# Solution: Prepare queries against database
cargo sqlx prepare --workspace

# Or run with offline mode
SQLX_OFFLINE=true cargo build
```

### Database Issues

**Problem**: Cannot connect to database
```bash
# Check if PostgreSQL is running
docker-compose ps

# Check connection manually
psql "postgresql://erp_admin:erp_secure_password_change_in_production@localhost:5432/erp_main"

# Restart database
docker-compose restart postgres
```

**Problem**: Migration errors
```bash
# Check current migration state
cargo sqlx migrate info

# Reset database (CAUTION: deletes all data)
docker-compose down -v
docker-compose up -d postgres
cargo sqlx migrate run
```

### Runtime Issues

**Problem**: Server won't start
```bash
# Check if port 3000 is already in use
lsof -i :3000
# Or on Windows: netstat -an | findstr :3000

# Use different port
SERVER_PORT=3001 cargo run -p erp-api
```

**Problem**: Health check returns error
```bash
# Check server logs for detailed error messages
RUST_LOG=debug cargo run -p erp-api

# Test with curl verbose mode
curl -v http://localhost:3000/health
```

### Docker Issues

**Problem**: Docker containers won't start
```bash
# Check Docker daemon is running
docker info

# Check for port conflicts
docker ps -a

# Remove conflicting containers
docker-compose down
docker system prune -f
```

### Environment Variable Issues

**Problem**: Configuration validation fails
```bash
# Check all required environment variables are set
env | grep -E "(DATABASE_URL|JWT_SECRET|AES_ENCRYPTION_KEY)"

# Generate secure secrets
export JWT_SECRET=$(openssl rand -base64 32)
export AES_ENCRYPTION_KEY=$(openssl rand -base64 32 | cut -c1-32)
```

## 🔮 Future Deployment Options

*Note: These are planned features, not currently implemented*

### Production Docker Deployment

```dockerfile
# Future Dockerfile structure
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/erp-api /usr/local/bin/
EXPOSE 3000
CMD ["erp-api"]
```

### Kubernetes Deployment

```yaml
# Future k8s deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: erp-api
spec:
  replicas: 3
  selector:
    matchLabels:
      app: erp-api
  template:
    metadata:
      labels:
        app: erp-api
    spec:
      containers:
      - name: erp-api
        image: erp-system:latest
        ports:
        - containerPort: 3000
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: erp-secrets
              key: database-url
```

### Cloud Deployment Options

**AWS:**
- ECS/Fargate for containerized deployment
- RDS PostgreSQL for managed database
- ElastiCache Redis for session storage
- Application Load Balancer
- CloudWatch for monitoring

**Azure:**
- Container Instances or App Service
- Azure Database for PostgreSQL
- Azure Cache for Redis
- Application Gateway
- Azure Monitor

**Google Cloud:**
- Cloud Run for serverless containers
- Cloud SQL PostgreSQL
- Memorystore Redis
- Cloud Load Balancing
- Cloud Monitoring

### Monitoring & Observability (Planned)

```bash
# Future monitoring stack
- Prometheus: Metrics collection
- Grafana: Dashboards and alerting
- Jaeger: Distributed tracing
- ELK Stack: Log aggregation and search
```

---

## 📝 Deployment Checklist

### Development Setup Verification

- [ ] Rust toolchain installed (1.70+)
- [ ] Docker and Docker Compose working
- [ ] PostgreSQL container accessible
- [ ] Database migrations completed
- [ ] Environment variables configured
- [ ] API server starts without errors
- [ ] Health check endpoint responds
- [ ] Can connect to database with psql

### Before Production (Future)

- [ ] Security configuration reviewed
- [ ] All default passwords changed
- [ ] HTTPS/TLS configured
- [ ] Database backups automated
- [ ] Monitoring and alerting set up
- [ ] Load testing completed
- [ ] Disaster recovery plan ready

---

## ⚠️ Important Notes

### Current Limitations

1. **Development Only**: This setup is for development, not production
2. **Mock Implementation**: Most business logic is placeholder code
3. **Single Instance**: No high availability or clustering
4. **Basic Security**: Authentication is mock implementation
5. **No Monitoring**: No production-ready monitoring included

### Security Warnings

- **Change Default Passwords**: Never use development passwords in production
- **Use HTTPS**: Only HTTP is configured for local development
- **Secret Management**: Use proper secret management systems for production
- **Network Security**: Configure firewalls and network security

### Performance Considerations

- **Resource Usage**: Current implementation is not optimized for performance
- **Connection Pooling**: Database connections are limited for development
- **Caching**: Redis is configured but not actively used yet
- **Scaling**: No horizontal scaling implemented

---

**Status**: Updated December 2024 | **Version**: 0.1.0-alpha | **Deployment**: Local Development Ready