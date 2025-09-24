#!/bin/bash
# =====================================================
# ERP Database Setup Script for Linux/macOS
# =====================================================
# This script sets up a complete ERP database from scratch
# Can be run on any system with PostgreSQL and Rust/SQLx

set -e  # Exit on any error

echo ""
echo "====================================================="
echo "ERP Database Setup"
echo "====================================================="
echo ""

# Check if DATABASE_URL is set
if [ -z "$DATABASE_URL" ]; then
    echo "Setting default DATABASE_URL..."
    export DATABASE_URL="postgresql://erp_admin:erp_secure_password_change_in_production@localhost:5432/erp_main"
fi

echo "Using DATABASE_URL: $DATABASE_URL"
echo ""

# Step 1: Drop and recreate database (CAUTION!)
echo "Step 1: Recreating database..."
sqlx database drop --force 2>/dev/null || echo "Note: Database might not exist yet, continuing..."

sqlx database create
echo "Database created successfully."
echo ""

# Step 2: Run migrations
echo "Step 2: Running migrations..."
sqlx migrate run --source migrations
echo "Migrations completed successfully."
echo ""

# Step 3: Generate SQLx cache
echo "Step 3: Generating SQLx cache..."
rm -f .sqlx/*.json 2>/dev/null || true
cargo sqlx prepare --workspace
echo "SQLx cache generated successfully."
echo ""

# Step 4: Test compilation
echo "Step 4: Testing compilation..."
cargo check --all
echo ""

echo "====================================================="
echo "Database setup completed successfully!"
echo "====================================================="
echo ""
echo "You can now:"
echo "- Run the server: cargo run --bin erp-server"
echo "- Run tests: cargo test"
echo "- Deploy: cargo run --bin erp-deploy"
echo ""
echo "Database URL: $DATABASE_URL"
echo ""