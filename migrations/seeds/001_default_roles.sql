-- Default roles for tenant setup
-- This file is included by the tenant creation process

-- Create default roles for the tenant
INSERT INTO roles (id, name, description, permissions, is_system, is_active, created_at, updated_at) VALUES
    (gen_random_uuid(), 'admin', 'System Administrator',
     '["users:read", "users:write", "users:delete", "products:read", "products:write", "products:delete", "inventory:read", "inventory:write", "customers:read", "customers:write", "suppliers:read", "suppliers:write", "reports:read", "settings:write"]',
     true, true, NOW(), NOW()),

    (gen_random_uuid(), 'manager', 'Manager',
     '["products:read", "products:write", "inventory:read", "inventory:write", "customers:read", "customers:write", "suppliers:read", "suppliers:write", "reports:read"]',
     true, true, NOW(), NOW()),

    (gen_random_uuid(), 'employee', 'Employee',
     '["products:read", "inventory:read", "customers:read", "suppliers:read"]',
     true, true, NOW(), NOW()),

    (gen_random_uuid(), 'readonly', 'Read Only User',
     '["products:read", "inventory:read", "customers:read", "suppliers:read", "reports:read"]',
     true, true, NOW(), NOW());

-- Create default permission groups
INSERT INTO permission_groups (id, name, description, permissions, is_active, created_at, updated_at) VALUES
    (gen_random_uuid(), 'product_management', 'Product Management Permissions',
     '["products:read", "products:write", "products:delete"]',
     true, NOW(), NOW()),

    (gen_random_uuid(), 'inventory_management', 'Inventory Management Permissions',
     '["inventory:read", "inventory:write", "inventory:adjust", "inventory:transfer"]',
     true, NOW(), NOW()),

    (gen_random_uuid(), 'customer_management', 'Customer Management Permissions',
     '["customers:read", "customers:write", "customers:delete"]',
     true, NOW(), NOW()),

    (gen_random_uuid(), 'supplier_management', 'Supplier Management Permissions',
     '["suppliers:read", "suppliers:write", "suppliers:delete"]',
     true, NOW(), NOW());