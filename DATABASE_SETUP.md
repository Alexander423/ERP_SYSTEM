# ERP Database Setup Guide

This guide explains how to set up the ERP database system on any development environment.

## Quick Start

### Option 1: Automated Setup (Recommended)

**Windows:**
```cmd
scripts\setup_database.bat
```

**Linux/macOS:**
```bash
./scripts/setup_database.sh
```

### Option 2: Docker Development Environment

```bash
# Start PostgreSQL and Redis
docker-compose -f docker-compose.dev.yml up -d

# Wait for services to be ready
docker-compose -f docker-compose.dev.yml ps

# Run setup script
./scripts/setup_database.sh  # or setup_database.bat on Windows
```

### Option 3: Manual Setup

1. **Set environment variable:**
   ```bash
   export DATABASE_URL="postgresql://erp_admin:password@localhost:5432/erp_main"
   ```

2. **Create database:**
   ```bash
   sqlx database create
   ```

3. **Run migrations:**
   ```bash
   sqlx migrate run --source migrations
   ```

4. **Generate SQLx cache:**
   ```bash
   cargo sqlx prepare --workspace
   ```

5. **Test compilation:**
   ```bash
   cargo check --all
   ```

## Database Schema Overview

The ERP system uses a comprehensive PostgreSQL schema with the following main components:

### Core Tables

- **`products`** - Master product catalog with full ERP capabilities
- **`customers`** - Customer master data with hierarchical support
- **`suppliers`** - Supplier information and performance metrics
- **`addresses`** - Geographic address information for all entities
- **`contact_info`** - Contact details with support for multiple contacts per entity

### Inventory Management

- **`location_items`** - Multi-location inventory tracking with ABC classification
- **`inventory_movements`** - Complete movement history with full traceability
- **`inventory_analytics`** - Pre-calculated analytics for performance

### Financial Tracking

- **`sales_transactions`** - Sales history for analytics and reporting
- **`cash_flows`** - Financial flow tracking
- **`customer_feedback`** - Customer satisfaction and ratings

### Enum Types

The system uses PostgreSQL enums for data integrity:

- `product_status` - active, inactive, development, discontinued, planned
- `product_type` - physical, digital, service, bundle, subscription
- `unit_of_measure` - piece, kg, gram, liter, etc.
- `location_type` - warehouse, store, distribution_center, etc.
- `abc_classification` - A, B, C (inventory classification)
- `movement_velocity` - fast, medium, slow
- `customer_type` - individual, business, government, non_profit
- `entity_status` - active, inactive, suspended, pending

## Directory Structure

```
migrations/
├── 001_complete_schema.sql    # Master schema (authoritative)
├── 001_public_schema.sql      # Basic public schema
├── 002_enums_only.sql         # Legacy enum definitions
└── ...                        # Other foundational migrations

migrations_backup/             # Archived fragmented migrations
scripts/
├── setup_database.sh         # Linux/macOS setup script
└── setup_database.bat        # Windows setup script

docker-compose.dev.yml         # Development environment
```

## Troubleshooting

### Common Issues

1. **Migration conflicts:**
   ```bash
   # Reset everything and start fresh
   sqlx database drop --force
   sqlx database create
   sqlx migrate run --source migrations
   ```

2. **SQLx cache out of sync:**
   ```bash
   # Clear cache and regenerate
   rm .sqlx/*.json
   cargo sqlx prepare --workspace
   ```

3. **Permission errors:**
   - Ensure database user has CREATE/DROP privileges
   - Check connection string credentials

4. **Docker issues:**
   ```bash
   # Reset Docker environment
   docker-compose -f docker-compose.dev.yml down -v
   docker-compose -f docker-compose.dev.yml up -d
   ```

### Database Connection

Default connection string:
```
postgresql://erp_admin:erp_secure_password_change_in_production@localhost:5432/erp_main
```

**⚠️ Security Note:** Change the default password in production environments!

## Development Workflow

1. **Initial setup:** Run setup script once
2. **Code changes:** Modify Rust code as needed
3. **Schema changes:** Create new migration files
4. **Testing:** Use `cargo test` for validation
5. **Deployment:** Use `cargo run --bin erp-deploy`

## Schema Maintenance

### Adding New Tables

1. Create migration file: `migrations/XXX_new_feature.sql`
2. Define table with proper constraints and indexes
3. Update Rust models to match
4. Regenerate SQLx cache: `cargo sqlx prepare`

### Modifying Existing Tables

1. Create migration with `ALTER TABLE` statements
2. Update corresponding Rust structs
3. Test with `cargo check --all`
4. Regenerate SQLx cache

## Performance Considerations

The schema includes optimized indexes for:
- Primary key lookups
- Foreign key relationships
- Common query patterns
- Date-based filtering
- Multi-tenant isolation

Monitor query performance and add indexes as needed based on actual usage patterns.

## Backup and Recovery

### Backup
```bash
pg_dump $DATABASE_URL > backup.sql
```

### Restore
```bash
psql $DATABASE_URL < backup.sql
```

For production environments, implement automated backup strategies with point-in-time recovery capabilities.