-- Tenant-specific schema template
-- This will be executed for each new tenant with {{schema}} replaced by the actual schema name

-- Users table
CREATE TABLE {{schema}}.users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255),
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    is_active BOOLEAN NOT NULL DEFAULT true,
    locked_until TIMESTAMPTZ,
    email_verified_at TIMESTAMPTZ,
    two_factor_secret_encrypted TEXT,
    two_factor_enabled_at TIMESTAMPTZ,
    last_login_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Roles table
CREATE TABLE {{schema}}.roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    is_editable BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Permissions table
CREATE TABLE {{schema}}.permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    resource VARCHAR(100) NOT NULL,
    action VARCHAR(100) NOT NULL,
    description TEXT,
    UNIQUE (resource, action)
);

-- User roles junction table
CREATE TABLE {{schema}}.user_roles (
    user_id UUID NOT NULL REFERENCES {{schema}}.users(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES {{schema}}.roles(id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, role_id)
);

-- Role permissions junction table
CREATE TABLE {{schema}}.role_permissions (
    role_id UUID NOT NULL REFERENCES {{schema}}.roles(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES {{schema}}.permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

-- Audit log table
CREATE TABLE {{schema}}.audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT now(),
    user_id UUID,
    impersonator_id UUID,
    source_ip INET,
    event_type VARCHAR(100) NOT NULL,
    target_resource VARCHAR(100),
    target_id UUID,
    details JSONB,
    status VARCHAR(50) NOT NULL
);

-- Indexes
CREATE INDEX idx_{{schema}}_users_email ON {{schema}}.users(email);
CREATE INDEX idx_{{schema}}_users_is_active ON {{schema}}.users(is_active);
CREATE INDEX idx_{{schema}}_audit_log_timestamp ON {{schema}}.audit_log(timestamp);
CREATE INDEX idx_{{schema}}_audit_log_user_id ON {{schema}}.audit_log(user_id);
CREATE INDEX idx_{{schema}}_audit_log_event_type ON {{schema}}.audit_log(event_type);

-- Triggers for updated_at
CREATE TRIGGER update_{{schema}}_users_updated_at 
    BEFORE UPDATE ON {{schema}}.users 
    FOR EACH ROW 
    EXECUTE FUNCTION public.update_updated_at_column();

CREATE TRIGGER update_{{schema}}_roles_updated_at 
    BEFORE UPDATE ON {{schema}}.roles 
    FOR EACH ROW 
    EXECUTE FUNCTION public.update_updated_at_column();

-- Insert default roles
INSERT INTO {{schema}}.roles (name, description, is_editable) VALUES
    ('admin', 'Full system administrator with all permissions', false),
    ('user', 'Standard user with basic permissions', true),
    ('viewer', 'Read-only access to most resources', true);

-- Insert default permissions
INSERT INTO {{schema}}.permissions (resource, action, description) VALUES
    ('user', 'create', 'Create new users'),
    ('user', 'read', 'View user information'),
    ('user', 'update', 'Update user information'),
    ('user', 'delete', 'Delete users'),
    ('user', 'impersonate', 'Impersonate other users'),
    ('role', 'manage', 'Manage roles and permissions'),
    ('permission', 'read', 'View available permissions'),
    ('audit', 'read', 'View audit logs'),
    ('tenant', 'manage', 'Manage tenant settings');

-- Assign all permissions to admin role
INSERT INTO {{schema}}.role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM {{schema}}.roles r
CROSS JOIN {{schema}}.permissions p
WHERE r.name = 'admin';

-- Assign read permissions to viewer role
INSERT INTO {{schema}}.role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM {{schema}}.roles r
CROSS JOIN {{schema}}.permissions p
WHERE r.name = 'viewer' AND p.action = 'read';

-- Assign basic permissions to user role
INSERT INTO {{schema}}.role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM {{schema}}.roles r
CROSS JOIN {{schema}}.permissions p
WHERE r.name = 'user' AND p.resource = 'user' AND p.action IN ('read', 'update');