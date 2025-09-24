@echo off
REM =====================================================
REM Development Environment Setup Script
REM =====================================================
REM Complete setup for ERP development environment

echo =====================================================
echo ERP System Development Setup
echo =====================================================
echo.

REM Check prerequisites
echo Checking prerequisites...

REM Check Rust installation
cargo --version >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo ERROR: Rust/Cargo not found. Please install Rust from https://rustup.rs/
    pause
    exit /b 1
)
echo ✓ Rust/Cargo found

REM Check PostgreSQL installation
pg_isready --help >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo ERROR: PostgreSQL tools not found. Please install PostgreSQL
    pause
    exit /b 1
)
echo ✓ PostgreSQL tools found

REM Check SQLx CLI
sqlx --version >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo SQLx CLI not found. Installing...
    cargo install sqlx-cli --features postgres
    if %ERRORLEVEL% neq 0 (
        echo ERROR: Failed to install SQLx CLI
        pause
        exit /b 1
    )
    echo ✓ SQLx CLI installed
) else (
    echo ✓ SQLx CLI found
)

echo.
echo Prerequisites check completed.
echo.

REM Create environment configuration
echo Creating development environment configuration...

REM Create .env file if it doesn't exist
if not exist .env (
    echo Creating .env file...
    (
        echo # ERP System Environment Configuration
        echo.
        echo # Database Configuration
        echo DATABASE_URL=postgresql://erp_admin:erp_secure_password_change_in_production@localhost:5432/erp_main
        echo.
        echo # Application Configuration
        echo RUST_LOG=info
        echo ENVIRONMENT=development
        echo.
        echo # Server Configuration
        echo SERVER_HOST=127.0.0.1
        echo SERVER_PORT=8080
        echo.
        echo # Security Configuration
        echo JWT_SECRET=development-jwt-secret-change-in-production
        echo BCRYPT_COST=12
        echo.
        echo # Feature Flags
        echo ENABLE_ANALYTICS=true
        echo ENABLE_INVENTORY_TRACKING=true
        echo ENABLE_MULTI_LOCATION=true
    ) > .env
    echo ✓ .env file created
) else (
    echo ✓ .env file already exists
)

REM Create development database if it doesn't exist
echo.
echo Setting up development database...
PGPASSWORD=erp_secure_password_change_in_production createdb -h localhost -U erp_admin erp_main 2>nul
if %ERRORLEVEL% equ 0 (
    echo ✓ Database created
) else (
    echo ✓ Database already exists
)

REM Install Rust dependencies
echo.
echo Installing Rust dependencies...
cargo fetch
if %ERRORLEVEL% neq 0 (
    echo ERROR: Failed to fetch Rust dependencies
    pause
    exit /b 1
)
echo ✓ Rust dependencies installed

REM Create useful development scripts directory
if not exist scripts\dev (
    mkdir scripts\dev
    echo ✓ Development scripts directory created
)

REM Create quick development commands
echo Creating development helper scripts...

REM Quick test script
(
    echo @echo off
    echo REM Quick test runner for development
    echo set DATABASE_URL=postgresql://erp_admin:erp_secure_password_change_in_production@localhost:5432/erp_main
    echo echo Running tests...
    echo cargo test --lib --all --message-format=short
) > scripts\dev\quick_test.bat

REM Quick build script
(
    echo @echo off
    echo REM Quick build for development
    echo set DATABASE_URL=postgresql://erp_admin:erp_secure_password_change_in_production@localhost:5432/erp_main
    echo echo Building project...
    echo cargo check --all --message-format=short
) > scripts\dev\quick_build.bat

REM Development server script
(
    echo @echo off
    echo REM Start development server
    echo set DATABASE_URL=postgresql://erp_admin:erp_secure_password_change_in_production@localhost:5432/erp_main
    echo set RUST_LOG=debug
    echo echo Starting ERP API server in development mode...
    echo cargo run -p erp-api --bin erp-server
) > scripts\dev\start_server.bat

REM Database console script
(
    echo @echo off
    echo REM Connect to development database
    echo set PGPASSWORD=erp_secure_password_change_in_production
    echo echo Connecting to ERP development database...
    echo psql -h localhost -U erp_admin -d erp_main
) > scripts\dev\db_console.bat

echo ✓ Development helper scripts created

echo.
echo =====================================================
echo Development Setup Complete!
echo =====================================================
echo.
echo Available development commands:
echo   scripts\apply_migrations.bat    - Apply all migrations
echo   scripts\validate_system.bat     - Validate system health
echo   scripts\dev\quick_test.bat      - Run tests quickly
echo   scripts\dev\quick_build.bat     - Build project quickly
echo   scripts\dev\start_server.bat    - Start development server
echo   scripts\dev\db_console.bat      - Connect to database
echo.
echo Next steps:
echo   1. Run: scripts\apply_migrations.bat
echo   2. Run: scripts\validate_system.bat
echo   3. Start development: scripts\dev\start_server.bat
echo.
echo Your development environment is ready!
echo.
pause