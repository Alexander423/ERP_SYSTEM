@echo off
REM =====================================================
REM Docker Logs Viewer Script
REM =====================================================
REM View logs from ERP Docker services

echo =====================================================
echo ERP Docker Logs Viewer
echo =====================================================
echo.

if "%1"=="" (
    echo Available services:
    echo   postgres  - PostgreSQL database logs
    echo   redis     - Redis cache logs
    echo   pgadmin   - pgAdmin interface logs
    echo   redis-commander - Redis Commander logs
    echo   all       - All services logs
    echo.
    echo Usage: scripts\docker_logs.bat [service]
    echo Example: scripts\docker_logs.bat postgres
    echo.

    set /p SERVICE="Enter service name (or 'all' for all services): "
) else (
    set SERVICE=%1
)

echo.
echo Showing logs for: %SERVICE%
echo Press Ctrl+C to exit
echo.

if "%SERVICE%"=="all" (
    docker-compose -f docker-compose.yml logs -f
) else (
    docker-compose -f docker-compose.yml logs -f %SERVICE%
)

if %ERRORLEVEL% neq 0 (
    echo.
    echo Error: Service '%SERVICE%' not found or not running
    echo.
    echo Available running services:
    docker-compose -f docker-compose.yml ps --services
    pause
)