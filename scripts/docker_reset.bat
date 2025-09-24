@echo off
REM =====================================================
REM Docker Stack Reset Script
REM =====================================================
REM Completely resets the Docker stack with fresh database

echo =====================================================
echo ERP Docker Stack Reset
echo =====================================================
echo.
echo WARNING: This will delete ALL existing data!
echo This includes:
echo - PostgreSQL database and all data
echo - Redis cache and sessions
echo - All Docker volumes and containers
echo.

set /p CONFIRM="Type 'RESET' to continue: "
if not "%CONFIRM%"=="RESET" (
    echo Reset cancelled by user.
    pause
    exit /b 0
)

echo.
echo Stopping and removing ERP containers...

REM Stop all ERP containers
docker-compose -f docker-compose.yml down -v --remove-orphans
if %ERRORLEVEL% neq 0 (
    echo Warning: Some containers may not have been running
)

echo.
echo Removing ERP Docker volumes...

REM Remove named volumes
docker volume rm erp_postgres_data 2>nul
docker volume rm erp_redis_data 2>nul
docker volume rm erp_pgadmin_data 2>nul

echo.
echo Removing ERP Docker network...
docker network rm erp-network 2>nul

echo.
echo Cleaning up Docker system...
docker system prune -f

echo.
echo Starting fresh ERP stack...
docker-compose -f docker-compose.yml up -d postgres redis

echo.
echo Waiting for PostgreSQL to be ready...
:wait_postgres
timeout /t 3 >nul
docker exec erp-postgres pg_isready -U erp_admin -d erp_main >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo PostgreSQL not ready yet, waiting...
    goto wait_postgres
)

echo ✓ PostgreSQL is ready

echo.
echo Waiting for Redis to be ready...
:wait_redis
timeout /t 2 >nul
docker exec erp-redis redis-cli ping >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo Redis not ready yet, waiting...
    goto wait_redis
)

echo ✓ Redis is ready

echo.
echo =====================================================
echo Docker Stack Reset Complete!
echo =====================================================
echo.
echo Services running:
docker-compose -f docker-compose.yml ps

echo.
echo Database connection:
echo   Host: localhost
echo   Port: 5432
echo   Database: erp_main
echo   Username: erp_admin
echo   Password: erp_secure_password_change_in_production
echo.
echo Redis connection:
echo   Host: localhost
echo   Port: 6379
echo   Password: erp_redis_password_change_in_production
echo.
echo Optional admin tools:
echo   Start pgAdmin: docker-compose --profile admin up -d pgadmin
echo   Start Redis Commander: docker-compose --profile admin up -d redis-commander
echo   pgAdmin: http://localhost:8080 (admin@erp.local / admin123)
echo   Redis Commander: http://localhost:8081 (admin / admin123)
echo.
echo Your ERP stack is ready for development!
echo.
pause