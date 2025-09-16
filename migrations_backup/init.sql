-- Initial database setup
-- This file is executed when Docker container starts

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Create application user if not exists
DO
$do$
BEGIN
   IF NOT EXISTS (
      SELECT FROM pg_catalog.pg_user
      WHERE usename = 'erp_app') THEN

      CREATE USER erp_app WITH PASSWORD 'erp_app_password';
   END IF;
END
$do$;

-- Grant privileges
GRANT CREATE ON DATABASE erp_main TO erp_app;
GRANT ALL PRIVILEGES ON SCHEMA public TO erp_app;