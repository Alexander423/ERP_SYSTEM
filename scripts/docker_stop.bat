@echo off
REM =====================================================
REM Docker Stack Stop Script
REM =====================================================
REM Gracefully stops the ERP Docker stack

echo =====================================================
echo Stopping ERP Docker Stack
echo =====================================================
echo.

echo Stopping all ERP services...
docker-compose -f docker-compose.yml --profile admin down

if %ERRORLEVEL% equ 0 (
    echo ✓ All services stopped successfully
) else (
    echo Warning: Some services may not have been running
)

echo.
echo Current Docker status:
docker-compose -f docker-compose.yml ps

echo.
echo =====================================================
echo ERP Docker Stack Stopped
echo =====================================================
echo.
echo To restart: scripts\docker_start.bat
echo To reset completely: scripts\docker_reset.bat
echo.
pause