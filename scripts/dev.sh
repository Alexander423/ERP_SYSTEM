#!/bin/bash

# Development startup script for ERP System

set -e

echo "🚀 Starting ERP System Development Environment"

# Check if Docker is running
if ! docker info >/dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker first."
    exit 1
fi

# Create .env if it doesn't exist
if [ ! -f .env ]; then
    echo "📝 Creating .env file from template..."
    cp .env.example .env
    echo "⚠️  Please edit .env with your specific configuration before continuing."
    exit 1
fi

# Start infrastructure services
echo "🐳 Starting infrastructure services (PostgreSQL, Redis)..."
docker-compose up -d postgres redis

# Wait for services to be ready
echo "⏳ Waiting for services to be ready..."
sleep 5

# Check PostgreSQL
until docker-compose exec postgres pg_isready -h localhost -p 5432 -U erp_admin; do
    echo "⏳ Waiting for PostgreSQL..."
    sleep 2
done

# Check Redis
until docker-compose exec redis redis-cli ping; do
    echo "⏳ Waiting for Redis..."
    sleep 2
done

echo "✅ Infrastructure services are ready!"

# Run database migrations
echo "📊 Running database migrations..."
# Note: In a real setup, you'd use sqlx migrate run here

# Start the application
echo "🏃‍♂️ Starting ERP API server..."
echo "📖 API Documentation will be available at: http://localhost:8080/swagger-ui"
echo "🏥 Health check available at: http://localhost:8080/health"
echo ""
echo "Press Ctrl+C to stop all services"

# Start the server
cargo run --bin erp-server