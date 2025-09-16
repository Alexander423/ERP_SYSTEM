-- Fix all remaining missing columns and tables
-- Date: 2024-12-16
-- Purpose: Complete database schema for full functionality

-- The security schema already has most columns, but let's verify they exist
-- The errors suggest these columns are missing, but they actually exist in the schema
-- Let's check if the columns weren't created properly

-- Add exemptions column to data_masking_policies if it doesn't exist
ALTER TABLE data_masking_policies
ADD COLUMN IF NOT EXISTS exemptions JSONB DEFAULT '{}';

-- Create remediation_actions table for compliance module
CREATE TABLE IF NOT EXISTS remediation_actions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    finding_id UUID NOT NULL,
    action_title VARCHAR(255) NOT NULL,
    action_description TEXT,
    action_type VARCHAR(100) NOT NULL,
    priority VARCHAR(20) NOT NULL DEFAULT 'MEDIUM',
    assigned_to UUID,
    target_completion_date DATE,
    actual_completion_date DATE,
    status VARCHAR(50) NOT NULL DEFAULT 'PENDING',
    progress_percentage INTEGER DEFAULT 0 CHECK (progress_percentage >= 0 AND progress_percentage <= 100),
    evidence_documents JSONB DEFAULT '[]',
    cost_estimate DECIMAL(15, 2),
    actual_cost DECIMAL(15, 2),
    effectiveness_rating INTEGER CHECK (effectiveness_rating >= 1 AND effectiveness_rating <= 5),
    notes TEXT,
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    created_by UUID NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    modified_by UUID NOT NULL,
    modified_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    version INTEGER NOT NULL DEFAULT 1
);

-- Create indexes for remediation_actions
CREATE INDEX IF NOT EXISTS idx_remediation_actions_finding_id ON remediation_actions(finding_id);
CREATE INDEX IF NOT EXISTS idx_remediation_actions_status ON remediation_actions(status);
CREATE INDEX IF NOT EXISTS idx_remediation_actions_assigned_to ON remediation_actions(assigned_to);
CREATE INDEX IF NOT EXISTS idx_remediation_actions_tenant_id ON remediation_actions(tenant_id);

-- Ensure all customer-related tables exist with proper structure
-- Create customer_performance_metrics table if missing
CREATE TABLE IF NOT EXISTS customer_performance_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id UUID NOT NULL,
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    total_revenue DECIMAL(15, 2),
    average_order_value DECIMAL(15, 2),
    total_orders INTEGER,
    last_order_date DATE,
    relationship_duration_days INTEGER,
    satisfaction_score DECIMAL(3, 2),
    net_promoter_score INTEGER,
    last_contact_date DATE,
    contact_frequency INTEGER,
    response_rate DECIMAL(3, 2),
    days_sales_outstanding INTEGER,
    payment_reliability_score DECIMAL(3, 2),
    support_ticket_count INTEGER,
    last_calculated TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    modified_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(customer_id, tenant_id)
);

-- Create customer_behavioral_data table if missing
CREATE TABLE IF NOT EXISTS customer_behavioral_data (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id UUID NOT NULL,
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    preferred_purchase_channels JSONB DEFAULT '[]',
    seasonal_purchase_patterns JSONB DEFAULT '{}',
    product_category_preferences JSONB DEFAULT '[]',
    preferred_contact_times JSONB DEFAULT '[]',
    channel_engagement_rates JSONB DEFAULT '{}',
    website_engagement_score DECIMAL(3, 2),
    mobile_app_usage DECIMAL(3, 2),
    social_media_sentiment DECIMAL(3, 2),
    propensity_to_buy DECIMAL(3, 2),
    upsell_probability DECIMAL(3, 2),
    cross_sell_probability DECIMAL(3, 2),
    last_updated TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    modified_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(customer_id, tenant_id)
);

-- Create indexes for customer metrics tables
CREATE INDEX IF NOT EXISTS idx_customer_performance_metrics_customer_id ON customer_performance_metrics(customer_id);
CREATE INDEX IF NOT EXISTS idx_customer_performance_metrics_tenant_id ON customer_performance_metrics(tenant_id);

CREATE INDEX IF NOT EXISTS idx_customer_behavioral_data_customer_id ON customer_behavioral_data(customer_id);
CREATE INDEX IF NOT EXISTS idx_customer_behavioral_data_tenant_id ON customer_behavioral_data(tenant_id);

-- Verify and fix the security_incidents table
-- The remediation_actions column should be JSONB
ALTER TABLE security_incidents
ALTER COLUMN remediation_actions TYPE JSONB USING remediation_actions::JSONB;

-- Add any missing columns to existing tables (defensive approach)
-- These should already exist but let's make sure
ALTER TABLE roles
ADD COLUMN IF NOT EXISTS priority SMALLINT NOT NULL DEFAULT 0,
ADD COLUMN IF NOT EXISTS is_system_role BOOLEAN NOT NULL DEFAULT FALSE;

ALTER TABLE role_permissions
ADD COLUMN IF NOT EXISTS resource_type TEXT;

-- Update role_permissions to set resource_type if NULL
UPDATE role_permissions
SET resource_type = 'general'
WHERE resource_type IS NULL;

-- Make resource_type NOT NULL after setting defaults
ALTER TABLE role_permissions
ALTER COLUMN resource_type SET NOT NULL;

-- Ensure user_roles has all required columns
ALTER TABLE user_roles
ADD COLUMN IF NOT EXISTS assigned_by UUID;

-- Update any NULL assigned_by values to a system user
UPDATE user_roles
SET assigned_by = '00000000-0000-0000-0000-000000000000'::UUID
WHERE assigned_by IS NULL;

-- Make assigned_by NOT NULL after setting defaults
ALTER TABLE user_roles
ALTER COLUMN assigned_by SET NOT NULL;

-- Grant necessary permissions
GRANT ALL ON ALL TABLES IN SCHEMA public TO erp_admin;
GRANT ALL ON ALL SEQUENCES IN SCHEMA public TO erp_admin;

COMMENT ON TABLE remediation_actions IS 'Tracks remediation actions for compliance findings and security incidents';
COMMENT ON COLUMN data_masking_policies.exemptions IS 'JSON object defining exemption rules for data masking';