-- Fix missing database columns and tables for compilation

-- Add missing communication_preferences column to customers table
ALTER TABLE customers ADD COLUMN IF NOT EXISTS communication_preferences JSONB DEFAULT '{}';

-- Add missing priority column to roles table
ALTER TABLE roles ADD COLUMN IF NOT EXISTS priority SMALLINT DEFAULT 0;

-- Add missing exemptions column to data_masking_policies table
ALTER TABLE data_masking_policies ADD COLUMN IF NOT EXISTS exemptions JSONB;

-- Create missing remediation_actions table for compliance tracking
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
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    created_by UUID NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    modified_by UUID NOT NULL,
    modified_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMP WITH TIME ZONE,
    completed_by UUID
);

-- Create indexes for remediation_actions table
CREATE INDEX IF NOT EXISTS idx_remediation_actions_finding_id ON remediation_actions(finding_id);
CREATE INDEX IF NOT EXISTS idx_remediation_actions_status ON remediation_actions(status);
CREATE INDEX IF NOT EXISTS idx_remediation_actions_assigned_to ON remediation_actions(assigned_to);
CREATE INDEX IF NOT EXISTS idx_remediation_actions_due_date ON remediation_actions(due_date);
CREATE INDEX IF NOT EXISTS idx_remediation_actions_tenant_id ON remediation_actions(tenant_id);

-- Add missing contacts table columns for communication_preferences
ALTER TABLE contacts ADD COLUMN IF NOT EXISTS communication_preferences JSONB DEFAULT '{}';
ALTER TABLE contacts ADD COLUMN IF NOT EXISTS linkedin_profile VARCHAR(255);
ALTER TABLE contacts ADD COLUMN IF NOT EXISTS fax VARCHAR(50);

-- Ensure all required indexes exist for performance
CREATE INDEX IF NOT EXISTS idx_customers_communication_preferences ON customers USING GIN(communication_preferences);
CREATE INDEX IF NOT EXISTS idx_contacts_communication_preferences ON contacts USING GIN(communication_preferences);
CREATE INDEX IF NOT EXISTS idx_roles_priority ON roles(priority);

-- Update roles table to ensure all columns exist
ALTER TABLE roles ADD COLUMN IF NOT EXISTS metadata JSONB DEFAULT '{}';
ALTER TABLE roles ADD COLUMN IF NOT EXISTS is_system_role BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE roles ADD COLUMN IF NOT EXISTS is_active BOOLEAN NOT NULL DEFAULT TRUE;

-- Update role_permissions table to ensure all columns exist
ALTER TABLE role_permissions ADD COLUMN IF NOT EXISTS resource_type TEXT;
ALTER TABLE role_permissions ADD COLUMN IF NOT EXISTS action TEXT;
ALTER TABLE role_permissions ADD COLUMN IF NOT EXISTS scope TEXT;

-- Update data_masking_policies table
ALTER TABLE data_masking_policies ADD COLUMN IF NOT EXISTS masking_type VARCHAR(50) NOT NULL DEFAULT 'REDACTION';
ALTER TABLE data_masking_policies ADD COLUMN IF NOT EXISTS masking_config JSONB NOT NULL DEFAULT '{}';
ALTER TABLE data_masking_policies ADD COLUMN IF NOT EXISTS conditions JSONB;

-- Ensure tenant_id columns exist where needed
ALTER TABLE user_roles ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE;

-- Add version columns where missing
ALTER TABLE customers ADD COLUMN IF NOT EXISTS version INTEGER NOT NULL DEFAULT 1;
ALTER TABLE contacts ADD COLUMN IF NOT EXISTS version INTEGER NOT NULL DEFAULT 1;
ALTER TABLE addresses ADD COLUMN IF NOT EXISTS version INTEGER NOT NULL DEFAULT 1;

-- Add missing assigned_by column to user_roles table
ALTER TABLE user_roles ADD COLUMN IF NOT EXISTS assigned_by UUID;
ALTER TABLE user_roles ADD COLUMN IF NOT EXISTS assigned_at TIMESTAMP WITH TIME ZONE DEFAULT NOW();

-- Add missing tenant_id column to security tables where needed
ALTER TABLE roles ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE;

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_user_roles_assigned_by ON user_roles(assigned_by);
CREATE INDEX IF NOT EXISTS idx_user_roles_tenant_id ON user_roles(tenant_id);
CREATE INDEX IF NOT EXISTS idx_roles_tenant_id ON roles(tenant_id);