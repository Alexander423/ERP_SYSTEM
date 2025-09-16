-- Master Data Management Database Schema
-- This migration creates comprehensive master data tables for customers, suppliers, products, etc.

-- Create custom types for master data
CREATE TYPE IF NOT EXISTS customer_type AS ENUM (
    'b2b', 'b2c', 'b2g', 'internal', 'reseller',
    'distributor', 'end_user', 'prospect'
);

CREATE TYPE IF NOT EXISTS customer_lifecycle_stage AS ENUM (
    'lead', 'prospect', 'new_customer', 'active_customer',
    'vip_customer', 'at_risk_customer', 'inactive_customer',
    'won_back_customer', 'former_customer'
);

CREATE TYPE IF NOT EXISTS credit_status AS ENUM (
    'excellent', 'good', 'fair', 'poor', 'on_hold',
    'blocked', 'cash_only', 'requires_prepayment'
);

CREATE TYPE IF NOT EXISTS compliance_status AS ENUM (
    'compliant', 'non_compliant', 'under_review',
    'pending_documents', 'exempt', 'unknown'
);

CREATE TYPE IF NOT EXISTS kyc_status AS ENUM (
    'not_started', 'in_progress', 'completed',
    'requires_update', 'failed', 'exempted'
);

CREATE TYPE IF NOT EXISTS entity_status AS ENUM (
    'active', 'inactive', 'pending', 'suspended',
    'blocked', 'archived', 'deleted'
);

CREATE TYPE IF NOT EXISTS risk_rating AS ENUM (
    'low', 'medium', 'high', 'critical'
);

CREATE TYPE IF NOT EXISTS business_size AS ENUM (
    'micro', 'small', 'medium', 'large', 'enterprise'
);

CREATE TYPE IF NOT EXISTS acquisition_channel AS ENUM (
    'direct_sales', 'website_inquiry', 'social_media',
    'email_marketing', 'search_engine', 'referral',
    'partner_channel', 'trade_show', 'cold_call',
    'advertisement', 'other'
);

CREATE TYPE IF NOT EXISTS address_type AS ENUM (
    'billing', 'shipping', 'mailing', 'physical',
    'headquarters', 'branch', 'warehouse', 'other'
);

CREATE TYPE IF NOT EXISTS contact_type AS ENUM (
    'primary', 'billing', 'technical', 'sales',
    'purchasing', 'support', 'executive', 'decision_maker',
    'influencer', 'user', 'other'
);

CREATE TYPE IF NOT EXISTS payment_method AS ENUM (
    'cash', 'check', 'bank_transfer', 'credit_card',
    'debit_card', 'digital_wallet', 'cryptocurrency',
    'trade_credit', 'letter_of_credit', 'other'
);

CREATE TYPE IF NOT EXISTS data_source AS ENUM (
    'manual', 'import', 'api', 'integration', 'migration', 'synchronization'
);

CREATE TYPE IF NOT EXISTS sync_status AS ENUM (
    'pending', 'in_progress', 'success', 'failed', 'conflict', 'skipped'
);

-- ==============================================================================
-- CUSTOMERS TABLE
-- ==============================================================================

CREATE TABLE customers (
    -- Core Identity
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES public.tenants(id) ON DELETE CASCADE,
    customer_number VARCHAR(50) NOT NULL,
    external_ids JSONB DEFAULT '{}',

    -- Business Information
    legal_name VARCHAR(255) NOT NULL,
    trade_names TEXT[] DEFAULT '{}',
    customer_type customer_type NOT NULL DEFAULT 'b2b',

    -- Industry Classification (stored as JSONB for flexibility)
    industry_classification JSONB DEFAULT '{}',
    business_size business_size,

    -- Hierarchy & Grouping
    parent_customer_id UUID REFERENCES customers(id),
    corporate_group_id UUID,
    customer_hierarchy_level SMALLINT DEFAULT 1 CHECK (customer_hierarchy_level > 0),
    consolidation_group VARCHAR(100),

    -- Status & Lifecycle
    lifecycle_stage customer_lifecycle_stage NOT NULL DEFAULT 'prospect',
    status entity_status NOT NULL DEFAULT 'active',
    credit_status credit_status NOT NULL DEFAULT 'good',

    -- Geographic Information (primary references)
    primary_address_id UUID,
    billing_address_id UUID,
    shipping_address_ids UUID[] DEFAULT '{}',

    -- Contact Information (primary reference)
    primary_contact_id UUID,

    -- Tax & Legal Information
    tax_jurisdictions JSONB DEFAULT '[]',
    tax_numbers JSONB DEFAULT '{}', -- {tax_type: tax_number}
    regulatory_classifications JSONB DEFAULT '[]',
    compliance_status compliance_status NOT NULL DEFAULT 'unknown',
    kyc_status kyc_status NOT NULL DEFAULT 'not_started',
    aml_risk_rating risk_rating NOT NULL DEFAULT 'medium',

    -- Financial Information
    currency_code CHAR(3) NOT NULL DEFAULT 'USD',
    credit_limit DECIMAL(15,2),
    payment_terms JSONB DEFAULT '{}',
    tax_exempt BOOLEAN DEFAULT FALSE,

    -- Commercial Information
    price_group_id UUID,
    discount_group_id UUID,

    -- Sales & Marketing
    sales_representative_id UUID,
    account_manager_id UUID,
    customer_segments JSONB DEFAULT '[]',
    acquisition_channel acquisition_channel,
    customer_lifetime_value DECIMAL(15,2),
    churn_probability DECIMAL(3,2) CHECK (churn_probability >= 0 AND churn_probability <= 1),

    -- Performance Metrics (stored as JSONB for flexibility)
    performance_metrics JSONB DEFAULT '{}',
    behavioral_data JSONB DEFAULT '{}',

    -- Integration & Sync
    master_data_source data_source NOT NULL DEFAULT 'manual',
    external_id VARCHAR(255),
    sync_status sync_status NOT NULL DEFAULT 'success',
    last_sync_timestamp TIMESTAMPTZ,
    sync_errors TEXT[] DEFAULT '{}',

    -- Audit Trail
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    modified_by UUID NOT NULL,
    modified_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    version INTEGER NOT NULL DEFAULT 1,

    -- Constraints
    UNIQUE(tenant_id, customer_number),
    UNIQUE(tenant_id, external_id) WHERE external_id IS NOT NULL
);

-- ==============================================================================
-- ADDRESSES TABLE
-- ==============================================================================

CREATE TABLE addresses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES public.tenants(id) ON DELETE CASCADE,

    -- Reference to parent entity (polymorphic)
    entity_type VARCHAR(50) NOT NULL, -- 'customer', 'supplier', 'location', etc.
    entity_id UUID NOT NULL,

    -- Address Information
    address_type address_type NOT NULL,
    street_line_1 VARCHAR(255) NOT NULL,
    street_line_2 VARCHAR(255),
    city VARCHAR(100) NOT NULL,
    state_province VARCHAR(100),
    postal_code VARCHAR(20) NOT NULL,
    country_code CHAR(3) NOT NULL, -- ISO 3166-1 alpha-3

    -- Geographic Coordinates
    latitude DECIMAL(10,8),
    longitude DECIMAL(11,8),
    coordinate_accuracy REAL, -- Accuracy in meters

    -- Additional Address Information
    what3words_address VARCHAR(255),
    plus_code VARCHAR(20),
    timezone VARCHAR(50), -- IANA timezone identifier

    -- Status
    is_primary BOOLEAN DEFAULT FALSE,
    is_active BOOLEAN DEFAULT TRUE,

    -- Audit Trail
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    modified_by UUID NOT NULL,
    modified_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    version INTEGER NOT NULL DEFAULT 1,

    -- Constraints
    CHECK (latitude IS NULL OR (latitude >= -90 AND latitude <= 90)),
    CHECK (longitude IS NULL OR (longitude >= -180 AND longitude <= 180))
);

-- ==============================================================================
-- CONTACTS TABLE
-- ==============================================================================

CREATE TABLE contacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES public.tenants(id) ON DELETE CASCADE,

    -- Reference to parent entity (polymorphic)
    entity_type VARCHAR(50) NOT NULL, -- 'customer', 'supplier', etc.
    entity_id UUID NOT NULL,

    -- Contact Information
    contact_type contact_type NOT NULL,
    first_name VARCHAR(100) NOT NULL,
    last_name VARCHAR(100) NOT NULL,
    title VARCHAR(100),
    department VARCHAR(100),

    -- Communication Details
    email VARCHAR(255),
    phone VARCHAR(50),
    mobile VARCHAR(50),
    fax VARCHAR(50),
    linkedin_profile VARCHAR(255),

    -- Preferences
    preferred_language CHAR(2), -- ISO 639-1
    communication_preferences JSONB DEFAULT '{}',

    -- Status
    is_primary BOOLEAN DEFAULT FALSE,
    is_active BOOLEAN DEFAULT TRUE,

    -- Audit Trail
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    modified_by UUID NOT NULL,
    modified_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    version INTEGER NOT NULL DEFAULT 1,

    -- Constraints
    CHECK (email IS NULL OR email ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$')
);

-- ==============================================================================
-- CUSTOMER PERFORMANCE METRICS TABLE
-- ==============================================================================

CREATE TABLE customer_performance_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES public.tenants(id) ON DELETE CASCADE,
    customer_id UUID NOT NULL REFERENCES customers(id) ON DELETE CASCADE,

    -- Financial Metrics
    total_revenue DECIMAL(15,2),
    average_order_value DECIMAL(15,2),
    total_orders BIGINT,
    last_order_date TIMESTAMPTZ,

    -- Relationship Metrics
    relationship_duration_days INTEGER,
    satisfaction_score DECIMAL(3,2) CHECK (satisfaction_score >= 1.0 AND satisfaction_score <= 5.0),
    net_promoter_score SMALLINT CHECK (net_promoter_score >= -100 AND net_promoter_score <= 100),

    -- Engagement Metrics
    last_contact_date TIMESTAMPTZ,
    contact_frequency DECIMAL(5,2), -- Contacts per month
    response_rate DECIMAL(3,2) CHECK (response_rate >= 0 AND response_rate <= 1),

    -- Risk Metrics
    days_sales_outstanding DECIMAL(5,2),
    payment_reliability_score DECIMAL(3,2) CHECK (payment_reliability_score >= 0 AND payment_reliability_score <= 1),
    support_ticket_count INTEGER DEFAULT 0,

    -- Calculation Info
    last_calculated TIMESTAMPTZ NOT NULL DEFAULT now(),
    calculation_method VARCHAR(50) DEFAULT 'automated',

    -- Audit Trail
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    modified_by UUID NOT NULL,
    modified_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Constraints
    UNIQUE(tenant_id, customer_id)
);

-- ==============================================================================
-- CUSTOMER BEHAVIORAL DATA TABLE
-- ==============================================================================

CREATE TABLE customer_behavioral_data (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES public.tenants(id) ON DELETE CASCADE,
    customer_id UUID NOT NULL REFERENCES customers(id) ON DELETE CASCADE,

    -- Purchase Behavior (stored as JSONB for flexibility)
    preferred_purchase_channels JSONB DEFAULT '[]',
    seasonal_purchase_patterns JSONB DEFAULT '{}',
    product_category_preferences JSONB DEFAULT '{}',

    -- Communication Behavior
    preferred_contact_times JSONB DEFAULT '[]',
    channel_engagement_rates JSONB DEFAULT '{}',

    -- Digital Behavior
    website_engagement_score DECIMAL(3,2) CHECK (website_engagement_score >= 0 AND website_engagement_score <= 1),
    mobile_app_usage DECIMAL(3,2) CHECK (mobile_app_usage >= 0 AND mobile_app_usage <= 1),
    social_media_sentiment DECIMAL(3,2) CHECK (social_media_sentiment >= -1 AND social_media_sentiment <= 1),

    -- Predictive Scores (AI/ML generated)
    propensity_to_buy DECIMAL(3,2) CHECK (propensity_to_buy >= 0 AND propensity_to_buy <= 1),
    upsell_probability DECIMAL(3,2) CHECK (upsell_probability >= 0 AND upsell_probability <= 1),
    cross_sell_probability DECIMAL(3,2) CHECK (cross_sell_probability >= 0 AND cross_sell_probability <= 1),

    -- Data Sources and Quality
    data_sources JSONB DEFAULT '[]', -- Track where behavioral data comes from
    confidence_score DECIMAL(3,2) CHECK (confidence_score >= 0 AND confidence_score <= 1),

    -- Update Info
    last_updated TIMESTAMPTZ NOT NULL DEFAULT now(),
    update_frequency VARCHAR(20) DEFAULT 'daily', -- daily, weekly, monthly, real-time

    -- Audit Trail
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    modified_by UUID NOT NULL,
    modified_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Constraints
    UNIQUE(tenant_id, customer_id)
);

-- ==============================================================================
-- INDEXES FOR PERFORMANCE
-- ==============================================================================

-- Customer indexes
CREATE INDEX idx_customers_tenant_id ON customers(tenant_id);
CREATE INDEX idx_customers_customer_number ON customers(tenant_id, customer_number);
CREATE INDEX idx_customers_legal_name ON customers(tenant_id, legal_name);
CREATE INDEX idx_customers_customer_type ON customers(tenant_id, customer_type);
CREATE INDEX idx_customers_status ON customers(tenant_id, status);
CREATE INDEX idx_customers_lifecycle_stage ON customers(tenant_id, lifecycle_stage);
CREATE INDEX idx_customers_parent_id ON customers(parent_customer_id) WHERE parent_customer_id IS NOT NULL;
CREATE INDEX idx_customers_corporate_group ON customers(corporate_group_id) WHERE corporate_group_id IS NOT NULL;
CREATE INDEX idx_customers_sales_rep ON customers(sales_representative_id) WHERE sales_representative_id IS NOT NULL;
CREATE INDEX idx_customers_account_manager ON customers(account_manager_id) WHERE account_manager_id IS NOT NULL;
CREATE INDEX idx_customers_created_at ON customers(tenant_id, created_at);
CREATE INDEX idx_customers_modified_at ON customers(tenant_id, modified_at);
CREATE INDEX idx_customers_external_id ON customers(tenant_id, external_id) WHERE external_id IS NOT NULL;
CREATE INDEX idx_customers_sync_status ON customers(tenant_id, sync_status);

-- Text search index for customer names and numbers
CREATE INDEX idx_customers_text_search ON customers USING gin (
    to_tsvector('english', coalesce(legal_name, '') || ' ' || coalesce(customer_number, '') || ' ' || coalesce(array_to_string(trade_names, ' '), ''))
);

-- Address indexes
CREATE INDEX idx_addresses_tenant_id ON addresses(tenant_id);
CREATE INDEX idx_addresses_entity ON addresses(tenant_id, entity_type, entity_id);
CREATE INDEX idx_addresses_type ON addresses(tenant_id, address_type);
CREATE INDEX idx_addresses_country ON addresses(tenant_id, country_code);
CREATE INDEX idx_addresses_city ON addresses(tenant_id, city);
CREATE INDEX idx_addresses_postal_code ON addresses(tenant_id, postal_code);
CREATE INDEX idx_addresses_coordinates ON addresses(latitude, longitude) WHERE latitude IS NOT NULL AND longitude IS NOT NULL;
CREATE INDEX idx_addresses_primary ON addresses(tenant_id, entity_type, entity_id, is_primary) WHERE is_primary = TRUE;

-- Contact indexes
CREATE INDEX idx_contacts_tenant_id ON contacts(tenant_id);
CREATE INDEX idx_contacts_entity ON contacts(tenant_id, entity_type, entity_id);
CREATE INDEX idx_contacts_type ON contacts(tenant_id, contact_type);
CREATE INDEX idx_contacts_email ON contacts(tenant_id, email) WHERE email IS NOT NULL;
CREATE INDEX idx_contacts_name ON contacts(tenant_id, first_name, last_name);
CREATE INDEX idx_contacts_primary ON contacts(tenant_id, entity_type, entity_id, is_primary) WHERE is_primary = TRUE;

-- Performance metrics indexes
CREATE INDEX idx_customer_metrics_tenant_customer ON customer_performance_metrics(tenant_id, customer_id);
CREATE INDEX idx_customer_metrics_last_calculated ON customer_performance_metrics(last_calculated);
CREATE INDEX idx_customer_metrics_total_revenue ON customer_performance_metrics(tenant_id, total_revenue) WHERE total_revenue IS NOT NULL;
CREATE INDEX idx_customer_metrics_last_order ON customer_performance_metrics(tenant_id, last_order_date) WHERE last_order_date IS NOT NULL;

-- Behavioral data indexes
CREATE INDEX idx_customer_behavioral_tenant_customer ON customer_behavioral_data(tenant_id, customer_id);
CREATE INDEX idx_customer_behavioral_last_updated ON customer_behavioral_data(last_updated);
CREATE INDEX idx_customer_behavioral_propensity ON customer_behavioral_data(tenant_id, propensity_to_buy) WHERE propensity_to_buy IS NOT NULL;

-- ==============================================================================
-- TRIGGERS FOR AUTOMATIC TIMESTAMP UPDATES
-- ==============================================================================

-- Update modified_at timestamp for customers
CREATE TRIGGER update_customers_modified_at
    BEFORE UPDATE ON customers
    FOR EACH ROW
    EXECUTE FUNCTION public.update_updated_at_column();

-- Update modified_at timestamp for addresses
CREATE TRIGGER update_addresses_modified_at
    BEFORE UPDATE ON addresses
    FOR EACH ROW
    EXECUTE FUNCTION public.update_updated_at_column();

-- Update modified_at timestamp for contacts
CREATE TRIGGER update_contacts_modified_at
    BEFORE UPDATE ON contacts
    FOR EACH ROW
    EXECUTE FUNCTION public.update_updated_at_column();

-- Update modified_at timestamp for customer performance metrics
CREATE TRIGGER update_customer_metrics_modified_at
    BEFORE UPDATE ON customer_performance_metrics
    FOR EACH ROW
    EXECUTE FUNCTION public.update_updated_at_column();

-- Update modified_at timestamp for customer behavioral data
CREATE TRIGGER update_customer_behavioral_modified_at
    BEFORE UPDATE ON customer_behavioral_data
    FOR EACH ROW
    EXECUTE FUNCTION public.update_updated_at_column();

-- ==============================================================================
-- ROW LEVEL SECURITY (RLS)
-- ==============================================================================

-- Enable RLS on all tables
ALTER TABLE customers ENABLE ROW LEVEL SECURITY;
ALTER TABLE addresses ENABLE ROW LEVEL SECURITY;
ALTER TABLE contacts ENABLE ROW LEVEL SECURITY;
ALTER TABLE customer_performance_metrics ENABLE ROW LEVEL SECURITY;
ALTER TABLE customer_behavioral_data ENABLE ROW LEVEL SECURITY;

-- Create RLS policies for tenant isolation
CREATE POLICY customers_tenant_isolation ON customers
    USING (tenant_id = current_setting('app.current_tenant_id')::uuid);

CREATE POLICY addresses_tenant_isolation ON addresses
    USING (tenant_id = current_setting('app.current_tenant_id')::uuid);

CREATE POLICY contacts_tenant_isolation ON contacts
    USING (tenant_id = current_setting('app.current_tenant_id')::uuid);

CREATE POLICY customer_metrics_tenant_isolation ON customer_performance_metrics
    USING (tenant_id = current_setting('app.current_tenant_id')::uuid);

CREATE POLICY customer_behavioral_tenant_isolation ON customer_behavioral_data
    USING (tenant_id = current_setting('app.current_tenant_id')::uuid);

-- ==============================================================================
-- FOREIGN KEY CONSTRAINTS (Added after table creation for proper dependency order)
-- ==============================================================================

-- Add foreign key constraints for customer address references
ALTER TABLE customers
    ADD CONSTRAINT fk_customers_primary_address
    FOREIGN KEY (primary_address_id) REFERENCES addresses(id);

ALTER TABLE customers
    ADD CONSTRAINT fk_customers_billing_address
    FOREIGN KEY (billing_address_id) REFERENCES addresses(id);

-- Add foreign key constraint for customer contact reference
ALTER TABLE customers
    ADD CONSTRAINT fk_customers_primary_contact
    FOREIGN KEY (primary_contact_id) REFERENCES contacts(id);

-- ==============================================================================
-- HELPFUL VIEWS FOR COMMON QUERIES
-- ==============================================================================

-- View for customers with their primary address and contact
CREATE VIEW customer_summary AS
SELECT
    c.id,
    c.tenant_id,
    c.customer_number,
    c.legal_name,
    c.customer_type,
    c.lifecycle_stage,
    c.status,
    c.credit_status,
    -- Primary address
    pa.street_line_1 as primary_street,
    pa.city as primary_city,
    pa.state_province as primary_state,
    pa.country_code as primary_country,
    -- Primary contact
    pc.first_name as contact_first_name,
    pc.last_name as contact_last_name,
    pc.email as contact_email,
    pc.phone as contact_phone,
    -- Metrics
    pm.total_revenue,
    pm.total_orders,
    pm.last_order_date,
    -- Audit
    c.created_at,
    c.modified_at
FROM customers c
LEFT JOIN addresses pa ON c.primary_address_id = pa.id
LEFT JOIN contacts pc ON c.primary_contact_id = pc.id
LEFT JOIN customer_performance_metrics pm ON c.id = pm.customer_id;

-- Grant appropriate permissions on the view
GRANT SELECT ON customer_summary TO authenticated;

-- ==============================================================================
-- COMMENTS FOR DOCUMENTATION
-- ==============================================================================

COMMENT ON TABLE customers IS 'Master data for customer entities with comprehensive business intelligence features';
COMMENT ON TABLE addresses IS 'Polymorphic address table supporting customers, suppliers, locations, etc.';
COMMENT ON TABLE contacts IS 'Polymorphic contact information table supporting various entity types';
COMMENT ON TABLE customer_performance_metrics IS 'Calculated performance metrics for customer analytics';
COMMENT ON TABLE customer_behavioral_data IS 'AI/ML-driven behavioral analytics for customers';

COMMENT ON COLUMN customers.external_ids IS 'JSONB field storing external system identifiers for integration';
COMMENT ON COLUMN customers.industry_classification IS 'JSONB field storing multiple industry classification standards (NAICS, SIC, NACE, ISIC)';
COMMENT ON COLUMN customers.tax_jurisdictions IS 'JSONB array of tax jurisdiction information';
COMMENT ON COLUMN customers.customer_segments IS 'JSONB array of customer segmentation data';
COMMENT ON COLUMN customers.performance_metrics IS 'JSONB field for storing calculated performance metrics';
COMMENT ON COLUMN customers.behavioral_data IS 'JSONB field for storing AI/ML behavioral analysis results';