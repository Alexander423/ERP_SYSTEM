-- Fix missing schema elements for SQLX compilation
-- This applies the necessary schema without conflicting with existing types

-- First ensure we have all required enums
CREATE TYPE IF NOT EXISTS industry_classification AS ENUM (
    'agriculture', 'automotive', 'banking', 'construction', 'education',
    'energy', 'finance', 'government', 'healthcare', 'hospitality',
    'insurance', 'logistics', 'manufacturing', 'media', 'nonprofit',
    'professional_services', 'real_estate', 'retail', 'technology',
    'telecommunications', 'transportation', 'utilities', 'other'
);

CREATE TYPE IF NOT EXISTS address_type AS ENUM (
    'billing', 'shipping', 'mailing', 'physical', 'headquarters',
    'branch_office', 'warehouse', 'other'
);

CREATE TYPE IF NOT EXISTS contact_type AS ENUM (
    'primary', 'billing', 'technical', 'sales', 'purchasing',
    'support', 'executive', 'decision_maker', 'influencer', 'user', 'other'
);

CREATE TYPE IF NOT EXISTS data_source AS ENUM (
    'manual', 'import', 'api', 'system', 'migration'
);

CREATE TYPE IF NOT EXISTS sync_status AS ENUM (
    'synced', 'pending', 'failed', 'in_progress'
);

-- Create tenants table if not exists (required for foreign keys)
CREATE TABLE IF NOT EXISTS tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Insert default tenant if none exists
INSERT INTO tenants (id, name)
VALUES ('00000000-0000-0000-0000-000000000001', 'Default Tenant')
ON CONFLICT (id) DO NOTHING;

-- Create customers table with all required columns
CREATE TABLE IF NOT EXISTS customers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    customer_number VARCHAR(50) NOT NULL,
    external_ids JSONB DEFAULT '{}',
    legal_name VARCHAR(255) NOT NULL,
    trade_names JSONB DEFAULT '[]',
    customer_type customer_type NOT NULL,
    industry_classification industry_classification,
    business_size business_size,
    parent_customer_id UUID,
    corporate_group_id UUID,
    customer_hierarchy_level SMALLINT DEFAULT 0,
    consolidation_group VARCHAR(100),
    lifecycle_stage customer_lifecycle_stage NOT NULL,
    status entity_status NOT NULL DEFAULT 'active',
    credit_status credit_status,
    primary_address_id UUID,
    billing_address_id UUID,
    shipping_address_ids UUID[],
    primary_contact_id UUID,
    tax_jurisdictions JSONB DEFAULT '[]',
    tax_numbers JSONB DEFAULT '{}',
    regulatory_classifications JSONB DEFAULT '[]',
    compliance_status compliance_status,
    kyc_status kyc_status,
    aml_risk_rating risk_rating,
    currency_code VARCHAR(3),
    credit_limit DECIMAL(15, 2),
    payment_terms JSONB,
    tax_exempt BOOLEAN DEFAULT FALSE,
    price_group_id UUID,
    discount_group_id UUID,
    sales_representative_id UUID,
    account_manager_id UUID,
    customer_segments JSONB DEFAULT '[]',
    acquisition_channel acquisition_channel,
    customer_lifetime_value DECIMAL(15, 2),
    churn_probability DECIMAL(3, 2),
    master_data_source data_source DEFAULT 'manual',
    external_id VARCHAR(255),
    sync_status sync_status DEFAULT 'synced',
    last_sync TIMESTAMP WITH TIME ZONE,
    sync_source VARCHAR(100),
    sync_version INTEGER DEFAULT 1,
    custom_fields JSONB DEFAULT '{}',
    contract_ids UUID[],
    created_by UUID NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    modified_by UUID NOT NULL,
    modified_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    version INTEGER NOT NULL DEFAULT 1,
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    deleted_at TIMESTAMP WITH TIME ZONE,
    deleted_by UUID,
    UNIQUE(tenant_id, customer_number)
);

-- Create customer_performance_metrics table
CREATE TABLE IF NOT EXISTS customer_performance_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id UUID NOT NULL,
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    revenue_last_12_months DECIMAL(15, 2),
    average_order_value DECIMAL(15, 2),
    order_frequency DECIMAL(10, 2),
    total_orders INTEGER,
    total_revenue DECIMAL(15, 2),
    profit_margin DECIMAL(5, 2),
    last_purchase_date DATE,
    first_purchase_date DATE,
    customer_lifetime_value DECIMAL(15, 2),
    predicted_churn_probability DECIMAL(3, 2),
    last_updated TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    modified_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(customer_id)
);

-- Create customer_behavioral_data table
CREATE TABLE IF NOT EXISTS customer_behavioral_data (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id UUID NOT NULL,
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    purchase_frequency DECIMAL(10, 2),
    preferred_categories JSONB DEFAULT '[]',
    seasonal_trends JSONB DEFAULT '{}',
    price_sensitivity DECIMAL(3, 2),
    brand_loyalty DECIMAL(3, 2),
    communication_preferences JSONB DEFAULT '{}',
    support_ticket_frequency DECIMAL(10, 2),
    product_return_rate DECIMAL(3, 2),
    referral_activity DECIMAL(3, 2),
    product_category_preferences JSONB DEFAULT '[]',
    preferred_contact_times JSONB DEFAULT '[]',
    channel_engagement_rates JSONB DEFAULT '{}',
    website_engagement_score DECIMAL(3, 2),
    mobile_app_usage DECIMAL(3, 2),
    social_media_sentiment DECIMAL(3, 2),
    propensity_to_buy DECIMAL(3, 2),
    upsell_probability DECIMAL(3, 2),
    cross_sell_probability DECIMAL(3, 2),
    last_updated TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    modified_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(customer_id)
);

-- Create customer_events table for event store
CREATE TABLE IF NOT EXISTS customer_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    aggregate_id UUID NOT NULL,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    sequence_number BIGINT NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    event_data JSONB NOT NULL,
    metadata JSONB DEFAULT '{}',
    event_timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(aggregate_id, sequence_number)
);

-- Create roles table for access control
CREATE TABLE IF NOT EXISTS roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    priority SMALLINT NOT NULL DEFAULT 0,
    is_system_role BOOLEAN NOT NULL DEFAULT FALSE,
    created_by UUID NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    modified_by UUID NOT NULL,
    modified_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, name)
);

-- Create user_roles table
CREATE TABLE IF NOT EXISTS user_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    assigned_by UUID NOT NULL,
    assigned_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE,
    UNIQUE(user_id, role_id)
);

-- Create role_permissions table
CREATE TABLE IF NOT EXISTS role_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL,
    resource_type TEXT NOT NULL DEFAULT 'general',
    action VARCHAR(100) NOT NULL,
    scope JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(role_id, permission_id, action)
);

-- Create security_audit_log table
CREATE TABLE IF NOT EXISTS security_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type TEXT NOT NULL,
    event_category TEXT NOT NULL,
    user_id UUID,
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    resource_type TEXT,
    resource_id UUID,
    action TEXT NOT NULL,
    outcome TEXT NOT NULL,
    risk_level TEXT NOT NULL,
    event_data JSONB DEFAULT '{}',
    ip_address INET,
    user_agent TEXT,
    session_id TEXT,
    correlation_id UUID,
    source_system TEXT NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    retention_until TIMESTAMP WITH TIME ZONE
);

-- Create data_masking_policies table
CREATE TABLE IF NOT EXISTS data_masking_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    table_name VARCHAR(100) NOT NULL,
    column_name VARCHAR(100) NOT NULL,
    masking_rule JSONB NOT NULL,
    exemptions JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    modified_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, table_name, column_name)
);

-- Create compliance_findings table
CREATE TABLE IF NOT EXISTS compliance_findings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    finding_type VARCHAR(100) NOT NULL,
    severity VARCHAR(20) NOT NULL,
    description TEXT NOT NULL,
    remediation_actions JSONB DEFAULT '[]',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    modified_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_customers_tenant_id ON customers(tenant_id);
CREATE INDEX IF NOT EXISTS idx_customers_customer_number ON customers(customer_number);
CREATE INDEX IF NOT EXISTS idx_customer_events_aggregate_id ON customer_events(aggregate_id);
CREATE INDEX IF NOT EXISTS idx_customer_events_tenant_id ON customer_events(tenant_id);
CREATE INDEX IF NOT EXISTS idx_user_roles_user_id ON user_roles(user_id);
CREATE INDEX IF NOT EXISTS idx_user_roles_role_id ON user_roles(role_id);
CREATE INDEX IF NOT EXISTS idx_security_audit_log_tenant_id ON security_audit_log(tenant_id);
CREATE INDEX IF NOT EXISTS idx_security_audit_log_timestamp ON security_audit_log(timestamp);

-- Grant permissions
GRANT ALL ON ALL TABLES IN SCHEMA public TO erp_admin;
GRANT ALL ON ALL SEQUENCES IN SCHEMA public TO erp_admin;