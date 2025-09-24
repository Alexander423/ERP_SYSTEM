#!/bin/bash
# Reset database script that bypasses confirmation prompts

set -e

echo "Resetting ERP database..."

# Set DATABASE_URL
export DATABASE_URL="postgresql://erp_admin:erp_secure_password_change_in_production@localhost:5432/erp_main"

# Drop database using direct SQL connection to postgres database
PGPASSWORD=erp_secure_password_change_in_production psql -h localhost -U erp_admin -d postgres -c "DROP DATABASE IF EXISTS erp_main;" 2>/dev/null || echo "Database didn't exist"

# Create database
PGPASSWORD=erp_secure_password_change_in_production psql -h localhost -U erp_admin -d postgres -c "CREATE DATABASE erp_main;"

echo "Database reset completed."

# Run setup
echo "Running setup..."
sqlx migrate run --source migrations
echo "Migrations completed."

# Clear and regenerate SQLx cache
rm -f .sqlx/*.json
cargo sqlx prepare --workspace
echo "SQLx cache regenerated."

echo "Database setup completed successfully!"