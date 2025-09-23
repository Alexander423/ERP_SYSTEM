-- Create default admin user for tenant
-- Password: admin123 (should be changed on first login)

DO $$
DECLARE
    admin_role_id UUID;
    admin_user_id UUID;
BEGIN
    -- Get the admin role ID
    SELECT id INTO admin_role_id FROM roles WHERE name = 'admin' LIMIT 1;

    -- Create admin user
    INSERT INTO users (
        id, email, username, password_hash, first_name, last_name,
        is_active, email_verified, must_change_password,
        created_at, updated_at
    ) VALUES (
        gen_random_uuid(),
        'admin@company.com',
        'admin',
        '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBBX1CdL4xfj8.',  -- Hash for 'admin123'
        'System',
        'Administrator',
        true,
        true,
        true,  -- Force password change on first login
        NOW(),
        NOW()
    ) RETURNING id INTO admin_user_id;

    -- Assign admin role to user
    INSERT INTO user_roles (
        id, user_id, role_id, assigned_at, assigned_by, is_active,
        created_at, updated_at
    ) VALUES (
        gen_random_uuid(),
        admin_user_id,
        admin_role_id,
        NOW(),
        admin_user_id,  -- Self-assigned during setup
        true,
        NOW(),
        NOW()
    );

    -- Create user profile
    INSERT INTO user_profiles (
        id, user_id, phone, department, job_title, language,
        timezone, date_format, number_format,
        created_at, updated_at
    ) VALUES (
        gen_random_uuid(),
        admin_user_id,
        NULL,
        'IT',
        'System Administrator',
        'en',
        'UTC',
        'YYYY-MM-DD',
        'en-US',
        NOW(),
        NOW()
    );

    RAISE NOTICE 'Admin user created successfully with email: admin@company.com';
    RAISE NOTICE 'Default password: admin123 (MUST BE CHANGED)';
END $$;