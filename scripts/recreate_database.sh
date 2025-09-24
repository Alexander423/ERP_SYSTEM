#!/bin/bash
# Complete database recreation script
# This bypasses all SQLx migration tracking and creates everything fresh

set -e

echo "==========================================="
echo "ERP Database Complete Recreation"
echo "==========================================="

# Database connection details
export DATABASE_URL="postgresql://erp_admin:erp_secure_password_change_in_production@localhost:5432/erp_main"
export BASE_URL="postgresql://erp_admin:erp_secure_password_change_in_production@localhost:5432/postgres"

echo "Step 1: Terminating all connections to database..."
# Try to terminate connections (will fail if database doesn't exist, that's ok)
sqlx query "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = 'erp_main' AND pid <> pg_backend_pid();" --database-url "$BASE_URL" 2>/dev/null || echo "No connections to terminate"

echo "Step 2: Dropping database if exists..."
sqlx query "DROP DATABASE IF EXISTS erp_main;" --database-url "$BASE_URL" 2>/dev/null || echo "Database didn't exist"

echo "Step 3: Creating fresh database..."
sqlx query "CREATE DATABASE erp_main;" --database-url "$BASE_URL"

echo "Step 4: Applying unified schema..."
sqlx migrate run --source migrations
if [ $? -ne 0 ]; then
    echo "ERROR: Failed to apply migrations"
    exit 1
fi

echo "Step 5: Clearing SQLx cache..."
rm -f .sqlx/*.json

echo "Step 6: Generating new SQLx cache..."
cargo sqlx prepare --workspace
if [ $? -ne 0 ]; then
    echo "ERROR: Failed to generate SQLx cache"
    exit 1
fi

echo "Step 7: Testing compilation..."
cargo check --all
if [ $? -ne 0 ]; then
    echo "ERROR: Compilation failed"
    exit 1
fi

echo ""
echo "==========================================="
echo "Database recreation completed successfully!"
echo "==========================================="
echo ""
echo "Database URL: $DATABASE_URL"
echo "Ready for development!"
echo ""