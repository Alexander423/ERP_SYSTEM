-- Test Database Setup Script
-- Creates a separate test database for integration tests

-- Create test database if it doesn't exist
SELECT 'CREATE DATABASE erp_test'
WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'erp_test');

-- Connect to test database and set up extensions
\c erp_test;

-- Create required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";
CREATE EXTENSION IF NOT EXISTS "btree_gin";
CREATE EXTENSION IF NOT EXISTS "citext";

-- Create schemas
CREATE SCHEMA IF NOT EXISTS public;

-- Set up permissions
GRANT ALL PRIVILEGES ON DATABASE erp_test TO erp_admin;
GRANT ALL PRIVILEGES ON SCHEMA public TO erp_admin;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO erp_admin;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO erp_admin;
GRANT ALL PRIVILEGES ON ALL FUNCTIONS IN SCHEMA public TO erp_admin;

-- Default privileges for future objects
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON TABLES TO erp_admin;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON SEQUENCES TO erp_admin;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON FUNCTIONS TO erp_admin;