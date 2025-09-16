-- Final migration to add all missing database columns and tables

-- Add missing columns to customers table
DO $$
BEGIN
    -- Add communication_preferences column if not exists
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='customers' AND column_name='communication_preferences') THEN
        ALTER TABLE customers ADD COLUMN communication_preferences JSONB DEFAULT '{}';
    END IF;

    -- Add version column if not exists
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='customers' AND column_name='version') THEN
        ALTER TABLE customers ADD COLUMN version INTEGER NOT NULL DEFAULT 1;
    END IF;
END $$;

-- Add missing columns to roles table
DO $$
BEGIN
    -- Add priority column if not exists
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='roles' AND column_name='priority') THEN
        ALTER TABLE roles ADD COLUMN priority SMALLINT DEFAULT 0;
    END IF;

    -- Add is_system_role column if not exists
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='roles' AND column_name='is_system_role') THEN
        ALTER TABLE roles ADD COLUMN is_system_role BOOLEAN NOT NULL DEFAULT FALSE;
    END IF;

    -- Add tenant_id column if not exists
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='roles' AND column_name='tenant_id') THEN
        ALTER TABLE roles ADD COLUMN tenant_id UUID;
    END IF;
END $$;

-- Add missing columns to user_roles table
DO $$
BEGIN
    -- Add assigned_by column if not exists
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='user_roles' AND column_name='assigned_by') THEN
        ALTER TABLE user_roles ADD COLUMN assigned_by UUID;
    END IF;

    -- Add assigned_at column if not exists
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='user_roles' AND column_name='assigned_at') THEN
        ALTER TABLE user_roles ADD COLUMN assigned_at TIMESTAMP WITH TIME ZONE DEFAULT NOW();
    END IF;

    -- Add tenant_id column if not exists
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='user_roles' AND column_name='tenant_id') THEN
        ALTER TABLE user_roles ADD COLUMN tenant_id UUID;
    END IF;
END $$;

-- Add missing columns to role_permissions table
DO $$
BEGIN
    -- Add resource_type column if not exists
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='role_permissions' AND column_name='resource_type') THEN
        ALTER TABLE role_permissions ADD COLUMN resource_type TEXT;
    END IF;

    -- Add action column if not exists
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='role_permissions' AND column_name='action') THEN
        ALTER TABLE role_permissions ADD COLUMN action TEXT;
    END IF;

    -- Add scope column if not exists
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='role_permissions' AND column_name='scope') THEN
        ALTER TABLE role_permissions ADD COLUMN scope TEXT;
    END IF;
END $$;

-- Add missing columns to data_masking_policies table if table exists
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name='data_masking_policies') THEN
        -- Add exemptions column if not exists
        IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='data_masking_policies' AND column_name='exemptions') THEN
            ALTER TABLE data_masking_policies ADD COLUMN exemptions JSONB;
        END IF;
    END IF;
END $$;

-- Create remediation_actions table if not exists
CREATE TABLE IF NOT EXISTS remediation_actions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    finding_id UUID NOT NULL,
    action_type VARCHAR(100) NOT NULL,
    description TEXT NOT NULL,
    priority VARCHAR(20) NOT NULL,
    assigned_to UUID,
    due_date TIMESTAMP WITH TIME ZONE,
    status VARCHAR(50) NOT NULL DEFAULT 'OPEN',
    estimated_effort_hours INTEGER,
    actual_effort_hours INTEGER,
    cost_estimate DECIMAL(15,2),
    actual_cost DECIMAL(15,2),
    approval_required BOOLEAN NOT NULL DEFAULT FALSE,
    approved_by UUID,
    approved_at TIMESTAMP WITH TIME ZONE,
    implementation_notes TEXT,
    verification_steps TEXT[],
    tenant_id UUID,
    created_by UUID NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    modified_by UUID NOT NULL,
    modified_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMP WITH TIME ZONE,
    completed_by UUID
);

-- Add missing columns to contacts table if table exists
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name='contacts') THEN
        -- Add communication_preferences column if not exists
        IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='contacts' AND column_name='communication_preferences') THEN
            ALTER TABLE contacts ADD COLUMN communication_preferences JSONB DEFAULT '{}';
        END IF;

        -- Add fax column if not exists
        IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='contacts' AND column_name='fax') THEN
            ALTER TABLE contacts ADD COLUMN fax VARCHAR(50);
        END IF;

        -- Add version column if not exists
        IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='contacts' AND column_name='version') THEN
            ALTER TABLE contacts ADD COLUMN version INTEGER NOT NULL DEFAULT 1;
        END IF;
    END IF;
END $$;

-- Create essential indexes for performance
CREATE INDEX IF NOT EXISTS idx_customers_communication_preferences ON customers USING GIN(communication_preferences) WHERE communication_preferences IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_roles_priority ON roles(priority) WHERE priority IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_user_roles_assigned_by ON user_roles(assigned_by) WHERE assigned_by IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_remediation_actions_finding_id ON remediation_actions(finding_id);
CREATE INDEX IF NOT EXISTS idx_remediation_actions_status ON remediation_actions(status);