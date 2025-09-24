@echo off
REM =====================================================
REM Professional Migration Application Script
REM =====================================================
REM Applies all structured migrations in correct order

echo Starting ERP System Migration Process...
echo.

REM Set environment variables
set DATABASE_URL=postgresql://erp_admin:erp_secure_password_change_in_production@localhost:5432/erp_main
set RUST_LOG=info

REM Check if PostgreSQL is running
pg_isready -h localhost -p 5432 -U erp_admin
if %ERRORLEVEL% neq 0 (
    echo ERROR: PostgreSQL is not running or not accessible
    echo Please start PostgreSQL and ensure it's accessible
    pause
    exit /b 1
)

echo PostgreSQL connection confirmed.
echo.

REM Reset the database (WARNING: This deletes all data!)
echo WARNING: This will delete ALL existing data in the database!
set /p CONFIRM="Type 'YES' to continue with database reset: "
if not "%CONFIRM%"=="YES" (
    echo Migration cancelled by user.
    pause
    exit /b 0
)

echo.
echo Resetting database...
PGPASSWORD=erp_secure_password_change_in_production psql -h localhost -U erp_admin -d postgres -f scripts/reset_database.sql
if %ERRORLEVEL% neq 0 (
    echo ERROR: Database reset failed
    pause
    exit /b 1
)

echo Database reset completed successfully.
echo.

REM Apply migrations in order using SQLx
echo Applying structured migrations...

echo.
echo [1/5] Applying foundation migrations...
sqlx migrate run --source migrations/001_foundation
if %ERRORLEVEL% neq 0 (
    echo ERROR: Foundation migrations failed
    pause
    exit /b 1
)

echo [2/5] Applying core table migrations...
sqlx migrate run --source migrations/002_core_tables
if %ERRORLEVEL% neq 0 (
    echo ERROR: Core table migrations failed
    pause
    exit /b 1
)

echo [3/5] Applying inventory system migrations...
sqlx migrate run --source migrations/003_inventory
if %ERRORLEVEL% neq 0 (
    echo ERROR: Inventory migrations failed
    pause
    exit /b 1
)

echo [4/5] Applying performance indexes...
sqlx migrate run --source migrations/004_indexes
if %ERRORLEVEL% neq 0 (
    echo ERROR: Index migrations failed
    pause
    exit /b 1
)

echo [5/5] Applying seed data...
sqlx migrate run --source migrations/005_seed_data
if %ERRORLEVEL% neq 0 (
    echo ERROR: Seed data migrations failed
    pause
    exit /b 1
)

echo.
echo =====================================================
echo Migration Process Completed Successfully!
echo =====================================================
echo.
echo Database structure has been created with:
echo - Foundation: Extensions, enums, functions
echo - Core Tables: Products, customers, suppliers, addresses
echo - Inventory: Locations, movements, analytics
echo - Indexes: Performance optimizations
echo - Seed Data: Demo data for testing
echo.

REM Prepare SQLx for Rust compilation
echo Preparing SQLx metadata for Rust compilation...
timeout 60 cargo sqlx prepare --workspace
if %ERRORLEVEL% neq 0 (
    echo WARNING: SQLx prepare failed - some Rust compilation may not work offline
) else (
    echo SQLx metadata prepared successfully.
)

echo.
echo Migration process complete. Database is ready for use.
pause