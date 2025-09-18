#!/bin/bash

# Test runner script that sets up proper environment for integration tests

set -e

echo "🧪 Setting up test environment..."

# Load test environment variables
if [ -f ".env.test" ]; then
    export $(cat .env.test | grep -v '^#' | xargs)
    echo "✅ Loaded test environment variables"
else
    echo "❌ .env.test file not found"
    exit 1
fi

# Check if PostgreSQL is running
echo "🔍 Checking PostgreSQL connection..."
if ! pg_isready -h localhost -p 5432 > /dev/null 2>&1; then
    echo "❌ PostgreSQL is not running on localhost:5432"
    echo "Please start PostgreSQL and try again"
    exit 1
fi

# Check if Redis is running
echo "🔍 Checking Redis connection..."
if ! redis-cli -h localhost -p 6379 ping > /dev/null 2>&1; then
    echo "❌ Redis is not running on localhost:6379"
    echo "Please start Redis and try again"
    exit 1
fi

# Setup test database
echo "🗃️ Setting up test database..."
PGPASSWORD=erp_secure_password_change_in_production psql -h localhost -U erp_admin -d postgres -f scripts/setup_test_db.sql

# Run migrations on test database
echo "🚀 Running migrations on test database..."
DATABASE_URL="postgresql://erp_admin:erp_secure_password_change_in_production@localhost:5432/erp_test" cargo sqlx migrate run

# Prepare sqlx queries for offline mode
echo "📋 Preparing SQL queries for offline mode..."
DATABASE_URL="postgresql://erp_admin:erp_secure_password_change_in_production@localhost:5432/erp_test" cargo sqlx prepare --workspace

# Run tests
echo "🧪 Running integration tests..."
if [ "$1" = "--package" ] || [ "$1" = "-p" ]; then
    # Run tests for specific package
    DATABASE_URL="postgresql://erp_admin:erp_secure_password_change_in_production@localhost:5432/erp_test" cargo test -p "$2" "$3"
else
    # Run all tests
    DATABASE_URL="postgresql://erp_admin:erp_secure_password_change_in_production@localhost:5432/erp_test" cargo test --workspace "$@"
fi

echo "✅ Tests completed!"

# Clean up test database (optional)
if [ "$CLEAN_AFTER_TESTS" = "true" ]; then
    echo "🧹 Cleaning up test database..."
    PGPASSWORD=erp_secure_password_change_in_production psql -h localhost -U erp_admin -d postgres -c "DROP DATABASE IF EXISTS erp_test;"
fi