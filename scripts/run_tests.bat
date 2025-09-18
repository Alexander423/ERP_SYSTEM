@echo off
REM Test runner script for Windows that sets up proper environment for integration tests

echo 🧪 Setting up test environment...

REM Check if .env.test exists
if not exist ".env.test" (
    echo ❌ .env.test file not found
    exit /b 1
)

REM Load test environment variables
for /f "usebackq tokens=1,2 delims==" %%i in (".env.test") do (
    if not "%%i"=="" if not "%%i:~0,1%"=="#" set %%i=%%j
)
echo ✅ Loaded test environment variables

REM Check if PostgreSQL is running
echo 🔍 Checking PostgreSQL connection...
pg_isready -h localhost -p 5432 >nul 2>&1
if errorlevel 1 (
    echo ❌ PostgreSQL is not running on localhost:5432
    echo Please start PostgreSQL and try again
    exit /b 1
)

REM Check if Redis is running
echo 🔍 Checking Redis connection...
redis-cli -h localhost -p 6379 ping >nul 2>&1
if errorlevel 1 (
    echo ❌ Redis is not running on localhost:6379
    echo Please start Redis and try again
    exit /b 1
)

REM Setup test database
echo 🗃️ Setting up test database...
set PGPASSWORD=erp_secure_password_change_in_production
psql -h localhost -U erp_admin -d postgres -f scripts/setup_test_db.sql

REM Run migrations on test database
echo 🚀 Running migrations on test database...
set DATABASE_URL=postgresql://erp_admin:erp_secure_password_change_in_production@localhost:5432/erp_test
cargo sqlx migrate run

REM Prepare sqlx queries for offline mode
echo 📋 Preparing SQL queries for offline mode...
cargo sqlx prepare --workspace

REM Run tests
echo 🧪 Running integration tests...
if "%1"=="-p" (
    cargo test -p %2 %3
) else (
    cargo test --workspace %*
)

echo ✅ Tests completed!

REM Clean up test database (optional)
if "%CLEAN_AFTER_TESTS%"=="true" (
    echo 🧹 Cleaning up test database...
    psql -h localhost -U erp_admin -d postgres -c "DROP DATABASE IF EXISTS erp_test;"
)