@echo off
REM =====================================================
REM Docker Stack Start Script
REM =====================================================
REM Starts the ERP Docker stack

echo =====================================================
echo Starting ERP Docker Stack
echo =====================================================
echo.

echo Starting core services (PostgreSQL and Redis)...
docker-compose -f docker-compose.yml up -d postgres redis

if %ERRORLEVEL% neq 0 (
    echo ERROR: Failed to start core services
    pause
    exit /b 1
)

echo.
echo Waiting for services to be ready...

REM Wait for PostgreSQL
echo Checking PostgreSQL...
:wait_postgres
timeout /t 2 >nul
docker exec erp-postgres pg_isready -U erp_admin -d erp_main >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo PostgreSQL starting...
    goto wait_postgres
)
echo ✓ PostgreSQL is ready

REM Wait for Redis
echo Checking Redis...
:wait_redis
timeout /t 1 >nul
docker exec erp-redis redis-cli ping >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo Redis starting...
    goto wait_redis
)
echo ✓ Redis is ready

echo.
echo =====================================================
echo ERP Docker Stack Started Successfully!
echo =====================================================
echo.

REM Show running services
echo Running services:
docker-compose -f docker-compose.yml ps

echo.
echo Connection details:
echo   PostgreSQL: localhost:5432 (erp_admin / erp_secure_password_change_in_production)
echo   Redis: localhost:6379 (password: erp_redis_password_change_in_production)
echo.
echo Optional commands:
echo   Start admin tools: docker-compose --profile admin up -d
echo   View logs: docker-compose logs -f
echo   Stop stack: docker-compose down
echo.
echo Ready for development!
pause