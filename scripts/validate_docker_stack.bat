@echo off
REM =====================================================
REM Docker Stack Validation Script
REM =====================================================
REM Validates the ERP Docker stack is working correctly

echo =====================================================
echo ERP Docker Stack Validation
echo =====================================================
echo.

echo [1/6] Checking Docker availability...
docker --version >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo ❌ ERROR: Docker is not installed or not running
    pause
    exit /b 1
)
echo ✓ Docker is available

echo.
echo [2/6] Checking Docker Compose availability...
docker-compose --version >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo ❌ ERROR: Docker Compose is not available
    pause
    exit /b 1
)
echo ✓ Docker Compose is available

echo.
echo [3/6] Checking container status...
docker-compose -f docker-compose.yml ps

echo.
echo [4/6] Testing PostgreSQL connection...
docker exec erp-postgres pg_isready -U erp_admin -d erp_main
if %ERRORLEVEL% neq 0 (
    echo ❌ ERROR: PostgreSQL is not responding
    echo Trying to start PostgreSQL...
    docker-compose -f docker-compose.yml up -d postgres
    timeout /t 10 >nul
    docker exec erp-postgres pg_isready -U erp_admin -d erp_main
    if %ERRORLEVEL% neq 0 (
        echo ❌ ERROR: PostgreSQL failed to start
        pause
        exit /b 1
    )
)
echo ✓ PostgreSQL connection OK

echo.
echo [5/6] Testing Redis connection...
docker exec erp-redis redis-cli ping >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo ❌ ERROR: Redis is not responding
    echo Trying to start Redis...
    docker-compose -f docker-compose.yml up -d redis
    timeout /t 5 >nul
    docker exec erp-redis redis-cli ping >nul 2>&1
    if %ERRORLEVEL% neq 0 (
        echo ❌ ERROR: Redis failed to start
        pause
        exit /b 1
    )
)
echo ✓ Redis connection OK

echo.
echo [6/6] Validating database schema...
docker exec erp-postgres psql -U erp_admin -d erp_main -c "SELECT table_name FROM information_schema.tables WHERE table_schema='public' AND table_type='BASE TABLE' ORDER BY table_name LIMIT 5;"
if %ERRORLEVEL% neq 0 (
    echo ❌ ERROR: Database schema validation failed
    echo The database may not be properly initialized
    echo Try running: scripts\docker_reset.bat
    pause
    exit /b 1
)
echo ✓ Database schema validation OK

echo.
echo =====================================================
echo Stack Validation Results
echo =====================================================
echo.
echo ✅ All validations passed!
echo.

echo System Information:
echo ===================
echo Docker version:
docker --version

echo.
echo Docker Compose version:
docker-compose --version

echo.
echo Running containers:
docker-compose -f docker-compose.yml ps

echo.
echo Database information:
docker exec erp-postgres psql -U erp_admin -d erp_main -c "SELECT 'Tables' as type, count(*) as count FROM information_schema.tables WHERE table_schema='public' AND table_type='BASE TABLE' UNION ALL SELECT 'Products', count(*) FROM products UNION ALL SELECT 'Customers', count(*) FROM customers UNION ALL SELECT 'Suppliers', count(*) FROM suppliers;"

echo.
echo =====================================================
echo ERP Docker Stack is healthy and ready!
echo =====================================================
echo.
echo Connection details:
echo   PostgreSQL: localhost:5432
echo   Redis: localhost:6379
echo.
echo Admin tools (optional):
echo   pgAdmin: http://localhost:8080 (start with: docker-compose --profile admin up -d pgadmin)
echo   Redis Commander: http://localhost:8081 (start with: docker-compose --profile admin up -d redis-commander)
echo.
pause