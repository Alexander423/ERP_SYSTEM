-- Default Admin User for New Tenants
-- This script creates a default admin user for a new tenant
-- Variables: {TENANT_SCHEMA}, {ADMIN_EMAIL}, {ADMIN_PASSWORD_HASH} will be replaced

-- Set search path to tenant schema
SET search_path TO {TENANT_SCHEMA}, public;

-- Create default admin user
-- Note: Password hash should be generated using bcrypt with cost 12
-- Default password: "AdminPass123!" (change immediately after first login)
INSERT INTO {TENANT_SCHEMA}.users (
    id,
    tenant_id,
    username,
    email,
    password_hash,
    first_name,
    last_name,
    status,
    email_verified,
    created_by,
    modified_by
) VALUES (
    '{ADMIN_USER_ID}'::uuid,
    '{TENANT_ID}'::uuid,
    'admin',
    '{ADMIN_EMAIL}',
    '{ADMIN_PASSWORD_HASH}',
    'System',
    'Administrator',
    'Active',
    true,
    '00000000-0000-0000-0000-000000000000'::uuid,
    '00000000-0000-0000-0000-000000000000'::uuid
);

-- Assign super_admin role to the admin user
INSERT INTO {TENANT_SCHEMA}.user_roles (
    user_id,
    role_id,
    granted_by
) VALUES (
    '{ADMIN_USER_ID}'::uuid,
    (SELECT id FROM {TENANT_SCHEMA}.roles WHERE name = 'super_admin'),
    '00000000-0000-0000-0000-000000000000'::uuid
);

-- Reset search path
SET search_path TO DEFAULT;