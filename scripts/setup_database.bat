@echo off
REM =====================================================
REM ERP Database Setup Script for Windows
REM =====================================================
REM This script sets up a complete ERP database from scratch
REM Can be run on any system with PostgreSQL and Rust/SQLx

echo.
echo =====================================================
echo ERP Database Setup
echo =====================================================
echo.

REM Check if DATABASE_URL is set
if "%DATABASE_URL%"=="" (
    echo Setting default DATABASE_URL...
    set DATABASE_URL=postgresql://erp_admin:erp_secure_password_change_in_production@localhost:5432/erp_main
)

echo Using DATABASE_URL: %DATABASE_URL%
echo.

REM Step 1: Drop and recreate database (CAUTION!)
echo Step 1: Recreating database...
sqlx database drop --force 2>nul
if %errorlevel% neq 0 (
    echo Note: Database might not exist yet, continuing...
)

sqlx database create
if %errorlevel% neq 0 (
    echo ERROR: Failed to create database
    exit /b 1
)

echo Database created successfully.
echo.

REM Step 2: Run migrations
echo Step 2: Running migrations...
sqlx migrate run --source migrations
if %errorlevel% neq 0 (
    echo ERROR: Migration failed
    exit /b 1
)

echo Migrations completed successfully.
echo.

REM Step 3: Generate SQLx cache
echo Step 3: Generating SQLx cache...
del .sqlx\*.json 2>nul
cargo sqlx prepare --workspace
if %errorlevel% neq 0 (
    echo ERROR: Failed to generate SQLx cache
    exit /b 1
)

echo SQLx cache generated successfully.
echo.

REM Step 4: Test compilation
echo Step 4: Testing compilation...
cargo check --all
if %errorlevel% neq 0 (
    echo ERROR: Compilation failed
    exit /b 1
)

echo.
echo =====================================================
echo Database setup completed successfully!
echo =====================================================
echo.
echo You can now:
echo - Run the server: cargo run --bin erp-server
echo - Run tests: cargo test
echo - Deploy: cargo run --bin erp-deploy
echo.
echo Database URL: %DATABASE_URL%
echo.

pause