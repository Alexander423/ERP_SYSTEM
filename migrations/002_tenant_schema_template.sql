-- Tenant schema template for multi-tenant setup
-- This file is referenced by the Rust deploy module

-- Create schema-specific tables for tenant isolation
CREATE SCHEMA IF NOT EXISTS {TENANT_SCHEMA};

-- Set search path for this schema
SET search_path TO {TENANT_SCHEMA}, public;

-- Copy all tables from public schema to tenant schema
CREATE TABLE {TENANT_SCHEMA}.users (LIKE public.users INCLUDING ALL);
CREATE TABLE {TENANT_SCHEMA}.roles (LIKE public.roles INCLUDING ALL);
CREATE TABLE {TENANT_SCHEMA}.user_permissions (LIKE public.user_permissions INCLUDING ALL);
CREATE TABLE {TENANT_SCHEMA}.products (LIKE public.products INCLUDING ALL);
CREATE TABLE {TENANT_SCHEMA}.customers (LIKE public.customers INCLUDING ALL);
CREATE TABLE {TENANT_SCHEMA}.suppliers (LIKE public.suppliers INCLUDING ALL);
CREATE TABLE {TENANT_SCHEMA}.locations (LIKE public.locations INCLUDING ALL);
CREATE TABLE {TENANT_SCHEMA}.location_items (LIKE public.location_items INCLUDING ALL);
CREATE TABLE {TENANT_SCHEMA}.inventory_transactions (LIKE public.inventory_transactions INCLUDING ALL);
CREATE TABLE {TENANT_SCHEMA}.currencies (LIKE public.currencies INCLUDING ALL);
CREATE TABLE {TENANT_SCHEMA}.countries (LIKE public.countries INCLUDING ALL);
CREATE TABLE {TENANT_SCHEMA}.units_of_measure (LIKE public.units_of_measure INCLUDING ALL);

-- Reset search path
SET search_path TO public;