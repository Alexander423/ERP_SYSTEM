# Database Scripts

## Folder Structure

### setup/
Contains scripts for initial database setup and schema configuration:
- `fix_schema.sql` - Complete database schema setup with all tables and enums

### fixes/
Contains scripts for fixing specific database issues:
- `fix_industry_classification.sql` - Fixes industry classification enum
- `quick_fix.sql` - Quick database fixes and patches

### maintenance/
Reserved for ongoing database maintenance scripts.

## Usage

### Initial Setup
```bash
# Run schema setup
psql -h localhost -U erp_admin -d erp_main -f scripts/setup/fix_schema.sql
```

### Applying Fixes
```bash
# Apply industry classification fix
psql -h localhost -U erp_admin -d erp_main -f scripts/fixes/fix_industry_classification.sql

# Apply quick fixes
psql -h localhost -U erp_admin -d erp_main -f scripts/fixes/quick_fix.sql
```

## Migration vs Scripts
- **migrations/**: Versioned database migrations managed by sqlx
- **scripts/**: Manual utility scripts for setup and maintenance