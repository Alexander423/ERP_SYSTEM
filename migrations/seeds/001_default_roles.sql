-- Default Roles for New Tenants
-- This script inserts default roles and permissions for a new tenant
-- Variables: {TENANT_SCHEMA} will be replaced with actual tenant schema name

-- Set search path to tenant schema
SET search_path TO {TENANT_SCHEMA}, public;

-- Default system roles with comprehensive permissions
INSERT INTO {TENANT_SCHEMA}.roles (id, name, display_name, description, is_system_role, permissions, created_by, modified_by) VALUES
(
    gen_random_uuid(),
    'super_admin',
    'Super Administrator',
    'Full system access with all permissions',
    true,
    '["users:create", "users:read", "users:update", "users:delete", "users:manage_roles", "customers:create", "customers:read", "customers:update", "customers:delete", "customers:export", "orders:create", "orders:read", "orders:update", "orders:delete", "orders:fulfill", "reports:view", "reports:export", "settings:read", "settings:update", "audit:read", "system:backup", "system:restore", "system:configure"]'::jsonb,
    '00000000-0000-0000-0000-000000000000'::uuid,
    '00000000-0000-0000-0000-000000000000'::uuid
),
(
    gen_random_uuid(),
    'admin',
    'Administrator',
    'Administrative access with most permissions except system configuration',
    true,
    '["users:create", "users:read", "users:update", "users:delete", "customers:create", "customers:read", "customers:update", "customers:delete", "customers:export", "orders:create", "orders:read", "orders:update", "orders:delete", "orders:fulfill", "reports:view", "reports:export", "settings:read", "audit:read"]'::jsonb,
    '00000000-0000-0000-0000-000000000000'::uuid,
    '00000000-0000-0000-0000-000000000000'::uuid
),
(
    gen_random_uuid(),
    'manager',
    'Manager',
    'Management access for customers, orders and team oversight',
    true,
    '["users:read", "customers:create", "customers:read", "customers:update", "customers:export", "orders:create", "orders:read", "orders:update", "orders:fulfill", "reports:view", "reports:export"]'::jsonb,
    '00000000-0000-0000-0000-000000000000'::uuid,
    '00000000-0000-0000-0000-000000000000'::uuid
),
(
    gen_random_uuid(),
    'sales_rep',
    'Sales Representative',
    'Sales-focused access for customer and order management',
    true,
    '["customers:create", "customers:read", "customers:update", "orders:create", "orders:read", "orders:update", "reports:view"]'::jsonb,
    '00000000-0000-0000-0000-000000000000'::uuid,
    '00000000-0000-0000-0000-000000000000'::uuid
),
(
    gen_random_uuid(),
    'support_agent',
    'Support Agent',
    'Customer support access for viewing and updating customer information',
    true,
    '["customers:read", "customers:update", "orders:read", "orders:update"]'::jsonb,
    '00000000-0000-0000-0000-000000000000'::uuid,
    '00000000-0000-0000-0000-000000000000'::uuid
),
(
    gen_random_uuid(),
    'viewer',
    'Viewer',
    'Read-only access to customers and orders',
    true,
    '["customers:read", "orders:read", "reports:view"]'::jsonb,
    '00000000-0000-0000-0000-000000000000'::uuid,
    '00000000-0000-0000-0000-000000000000'::uuid
),
(
    gen_random_uuid(),
    'accountant',
    'Accountant',
    'Financial and reporting access',
    true,
    '["customers:read", "orders:read", "reports:view", "reports:export", "customers:export"]'::jsonb,
    '00000000-0000-0000-0000-000000000000'::uuid,
    '00000000-0000-0000-0000-000000000000'::uuid
);

-- Reset search path
SET search_path TO DEFAULT;