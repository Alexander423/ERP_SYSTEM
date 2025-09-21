-- Tenant Schema Template
-- This script creates the complete schema structure for a new tenant
-- Variables: {TENANT_SCHEMA} will be replaced with actual tenant schema name

-- Create tenant-specific schema
CREATE SCHEMA IF NOT EXISTS {TENANT_SCHEMA};

-- Set search path to tenant schema
SET search_path TO {TENANT_SCHEMA}, public;

-- Grant permissions to erp_admin
GRANT ALL PRIVILEGES ON SCHEMA {TENANT_SCHEMA} TO erp_admin;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA {TENANT_SCHEMA} TO erp_admin;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA {TENANT_SCHEMA} TO erp_admin;
GRANT ALL PRIVILEGES ON ALL FUNCTIONS IN SCHEMA {TENANT_SCHEMA} TO erp_admin;

-- Default privileges for future objects
ALTER DEFAULT PRIVILEGES IN SCHEMA {TENANT_SCHEMA} GRANT ALL ON TABLES TO erp_admin;
ALTER DEFAULT PRIVILEGES IN SCHEMA {TENANT_SCHEMA} GRANT ALL ON SEQUENCES TO erp_admin;
ALTER DEFAULT PRIVILEGES IN SCHEMA {TENANT_SCHEMA} GRANT ALL ON FUNCTIONS TO erp_admin;

-- Enable required extensions (schema-specific where possible)
CREATE EXTENSION IF NOT EXISTS "uuid-ossp" SCHEMA {TENANT_SCHEMA};

-- Customer Management Tables
CREATE TABLE {TENANT_SCHEMA}.customers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    customer_number VARCHAR(50) UNIQUE NOT NULL,
    legal_name VARCHAR(255) NOT NULL,
    trade_names TEXT[],
    customer_type VARCHAR(50) NOT NULL CHECK (customer_type IN ('Individual', 'Business', 'Government', 'NonProfit')),
    industry_classification VARCHAR(100),
    business_size VARCHAR(50),
    parent_customer_id UUID REFERENCES {TENANT_SCHEMA}.customers(id),
    corporate_group_id UUID,
    lifecycle_stage VARCHAR(50) CHECK (lifecycle_stage IN ('Lead', 'Prospect', 'Active', 'Inactive', 'Churned')),
    status VARCHAR(50) DEFAULT 'Active' CHECK (status IN ('Active', 'Inactive', 'Suspended', 'Deleted')),
    credit_status VARCHAR(50) CHECK (credit_status IN ('Excellent', 'Good', 'Fair', 'Poor', 'NoCredit')),
    customer_hierarchy_level INTEGER DEFAULT 1,
    consolidation_group VARCHAR(100),

    -- Audit fields
    version INTEGER DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by UUID NOT NULL,
    modified_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    modified_by UUID NOT NULL,
    deleted_at TIMESTAMPTZ,
    deleted_by UUID,

    -- Performance fields
    last_order_date TIMESTAMPTZ,
    total_orders_count INTEGER DEFAULT 0,
    total_revenue DECIMAL(15,2) DEFAULT 0.00,
    average_order_value DECIMAL(15,2) DEFAULT 0.00,
    customer_lifetime_value DECIMAL(15,2) DEFAULT 0.00,
    risk_score INTEGER DEFAULT 0 CHECK (risk_score >= 0 AND risk_score <= 100),
    satisfaction_score INTEGER CHECK (satisfaction_score >= 1 AND satisfaction_score <= 10),
    engagement_level VARCHAR(20) CHECK (engagement_level IN ('Low', 'Medium', 'High')),
    preferred_communication_channel VARCHAR(50)
);

-- Customer Addresses
CREATE TABLE {TENANT_SCHEMA}.customer_addresses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id UUID NOT NULL REFERENCES {TENANT_SCHEMA}.customers(id) ON DELETE CASCADE,
    address_type VARCHAR(50) NOT NULL CHECK (address_type IN ('Billing', 'Shipping', 'Office', 'Warehouse', 'Home')),
    is_primary BOOLEAN DEFAULT FALSE,
    company_name VARCHAR(255),
    attention_to VARCHAR(255),
    street_address_1 VARCHAR(255) NOT NULL,
    street_address_2 VARCHAR(255),
    city VARCHAR(100) NOT NULL,
    state_province VARCHAR(100),
    postal_code VARCHAR(20),
    country_code VARCHAR(3) NOT NULL,
    latitude DECIMAL(10, 8),
    longitude DECIMAL(11, 8),
    is_verified BOOLEAN DEFAULT FALSE,
    verification_date TIMESTAMPTZ,

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by UUID NOT NULL,
    modified_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    modified_by UUID NOT NULL,

    UNIQUE(customer_id, address_type, is_primary) DEFERRABLE INITIALLY DEFERRED
);

-- Customer Contacts
CREATE TABLE {TENANT_SCHEMA}.customer_contacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id UUID NOT NULL REFERENCES {TENANT_SCHEMA}.customers(id) ON DELETE CASCADE,
    contact_type VARCHAR(50) NOT NULL CHECK (contact_type IN ('Primary', 'Billing', 'Technical', 'Sales', 'Support')),
    is_primary BOOLEAN DEFAULT FALSE,
    title VARCHAR(50),
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    full_name VARCHAR(255) GENERATED ALWAYS AS (
        CASE
            WHEN first_name IS NOT NULL AND last_name IS NOT NULL
            THEN TRIM(CONCAT(first_name, ' ', last_name))
            WHEN first_name IS NOT NULL
            THEN first_name
            WHEN last_name IS NOT NULL
            THEN last_name
            ELSE NULL
        END
    ) STORED,
    position_title VARCHAR(255),
    department VARCHAR(100),
    email VARCHAR(255),
    phone VARCHAR(50),
    mobile VARCHAR(50),
    fax VARCHAR(50),
    preferred_contact_method VARCHAR(20) CHECK (preferred_contact_method IN ('Email', 'Phone', 'Mobile', 'Fax')),
    language_preference VARCHAR(10) DEFAULT 'en',
    timezone VARCHAR(50),

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by UUID NOT NULL,
    modified_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    modified_by UUID NOT NULL,

    UNIQUE(customer_id, contact_type, is_primary) DEFERRABLE INITIALLY DEFERRED
);

-- Customer Tax Information
CREATE TABLE {TENANT_SCHEMA}.customer_tax_info (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id UUID NOT NULL REFERENCES {TENANT_SCHEMA}.customers(id) ON DELETE CASCADE,
    tax_jurisdiction VARCHAR(100) NOT NULL,
    tax_number VARCHAR(100) NOT NULL,
    tax_type VARCHAR(50) NOT NULL CHECK (tax_type IN ('VAT', 'GST', 'Sales', 'Income', 'Corporate')),
    is_exempt BOOLEAN DEFAULT FALSE,
    exemption_certificate VARCHAR(255),
    exemption_expiry_date DATE,

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by UUID NOT NULL,
    modified_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    modified_by UUID NOT NULL,

    UNIQUE(customer_id, tax_jurisdiction, tax_type)
);

-- Customer Financial Information
CREATE TABLE {TENANT_SCHEMA}.customer_financial_info (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id UUID NOT NULL REFERENCES {TENANT_SCHEMA}.customers(id) ON DELETE CASCADE,
    credit_limit DECIMAL(15,2),
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    payment_terms VARCHAR(50),
    payment_method VARCHAR(50),
    bank_account_number VARCHAR(100),
    bank_routing_number VARCHAR(50),
    bank_name VARCHAR(255),
    annual_revenue DECIMAL(18,2),
    employee_count INTEGER,
    duns_number VARCHAR(20),

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by UUID NOT NULL,
    modified_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    modified_by UUID NOT NULL,

    UNIQUE(customer_id)
);

-- Customer External IDs (for integrations)
CREATE TABLE {TENANT_SCHEMA}.customer_external_ids (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id UUID NOT NULL REFERENCES {TENANT_SCHEMA}.customers(id) ON DELETE CASCADE,
    external_system VARCHAR(100) NOT NULL,
    external_id VARCHAR(255) NOT NULL,
    sync_status VARCHAR(50) DEFAULT 'Active' CHECK (sync_status IN ('Active', 'Inactive', 'Error')),
    last_sync_at TIMESTAMPTZ,
    sync_error_message TEXT,

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by UUID NOT NULL,

    UNIQUE(external_system, external_id),
    UNIQUE(customer_id, external_system)
);

-- Orders (Basic structure for customer relationship)
CREATE TABLE {TENANT_SCHEMA}.orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_number VARCHAR(50) UNIQUE NOT NULL,
    customer_id UUID NOT NULL REFERENCES {TENANT_SCHEMA}.customers(id),
    order_date DATE NOT NULL DEFAULT CURRENT_DATE,
    status VARCHAR(50) NOT NULL DEFAULT 'Draft' CHECK (status IN ('Draft', 'Pending', 'Confirmed', 'Shipped', 'Delivered', 'Cancelled', 'Returned')),
    total_amount DECIMAL(15,2) NOT NULL DEFAULT 0.00,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by UUID NOT NULL,
    modified_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    modified_by UUID NOT NULL
);

-- User Management (Tenant-specific users)
CREATE TABLE {TENANT_SCHEMA}.users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    username VARCHAR(100) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    full_name VARCHAR(255) GENERATED ALWAYS AS (
        CASE
            WHEN first_name IS NOT NULL AND last_name IS NOT NULL
            THEN TRIM(CONCAT(first_name, ' ', last_name))
            WHEN first_name IS NOT NULL
            THEN first_name
            WHEN last_name IS NOT NULL
            THEN last_name
            ELSE username
        END
    ) STORED,
    status VARCHAR(50) DEFAULT 'Active' CHECK (status IN ('Active', 'Inactive', 'Suspended', 'Deleted')),
    email_verified BOOLEAN DEFAULT FALSE,
    email_verification_token VARCHAR(255),
    email_verification_expires_at TIMESTAMPTZ,
    password_reset_token VARCHAR(255),
    password_reset_expires_at TIMESTAMPTZ,
    last_login_at TIMESTAMPTZ,
    login_attempts INTEGER DEFAULT 0,
    locked_until TIMESTAMPTZ,
    two_factor_enabled BOOLEAN DEFAULT FALSE,
    two_factor_secret VARCHAR(255),
    language_preference VARCHAR(10) DEFAULT 'en',
    timezone VARCHAR(50) DEFAULT 'UTC',

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by UUID,
    modified_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    modified_by UUID,
    deleted_at TIMESTAMPTZ,
    deleted_by UUID
);

-- Role-Based Access Control
CREATE TABLE {TENANT_SCHEMA}.roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) UNIQUE NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    description TEXT,
    is_system_role BOOLEAN DEFAULT FALSE,
    permissions JSONB DEFAULT '[]'::jsonb,

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by UUID NOT NULL,
    modified_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    modified_by UUID NOT NULL
);

-- User Role Assignments
CREATE TABLE {TENANT_SCHEMA}.user_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES {TENANT_SCHEMA}.users(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES {TENANT_SCHEMA}.roles(id) ON DELETE CASCADE,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    granted_by UUID NOT NULL,
    expires_at TIMESTAMPTZ,

    UNIQUE(user_id, role_id)
);

-- Audit Log (Tenant-specific)
CREATE TABLE {TENANT_SCHEMA}.audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    table_name VARCHAR(100) NOT NULL,
    record_id UUID NOT NULL,
    action VARCHAR(20) NOT NULL CHECK (action IN ('INSERT', 'UPDATE', 'DELETE')),
    old_values JSONB,
    new_values JSONB,
    changed_fields TEXT[],
    user_id UUID,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Settings (Tenant-specific configuration)
CREATE TABLE {TENANT_SCHEMA}.settings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    category VARCHAR(100) NOT NULL,
    key VARCHAR(255) NOT NULL,
    value JSONB NOT NULL,
    description TEXT,
    is_encrypted BOOLEAN DEFAULT FALSE,

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by UUID NOT NULL,
    modified_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    modified_by UUID NOT NULL,

    UNIQUE(category, key)
);

-- ================================
-- INDEXES for Performance
-- ================================

-- Customer Indexes
CREATE INDEX idx_customers_tenant_id ON {TENANT_SCHEMA}.customers(tenant_id);
CREATE INDEX idx_customers_customer_number ON {TENANT_SCHEMA}.customers(customer_number);
CREATE INDEX idx_customers_legal_name ON {TENANT_SCHEMA}.customers(legal_name);
CREATE INDEX idx_customers_customer_type ON {TENANT_SCHEMA}.customers(customer_type);
CREATE INDEX idx_customers_status ON {TENANT_SCHEMA}.customers(status);
CREATE INDEX idx_customers_lifecycle_stage ON {TENANT_SCHEMA}.customers(lifecycle_stage);
CREATE INDEX idx_customers_created_at ON {TENANT_SCHEMA}.customers(created_at);
CREATE INDEX idx_customers_parent_id ON {TENANT_SCHEMA}.customers(parent_customer_id);
CREATE INDEX idx_customers_deleted_at ON {TENANT_SCHEMA}.customers(deleted_at) WHERE deleted_at IS NULL;

-- Search indexes for customer names
CREATE INDEX idx_customers_legal_name_trgm ON {TENANT_SCHEMA}.customers USING gin(legal_name gin_trgm_ops);
CREATE INDEX idx_customers_trade_names_gin ON {TENANT_SCHEMA}.customers USING gin(trade_names);

-- Customer Address Indexes
CREATE INDEX idx_customer_addresses_customer_id ON {TENANT_SCHEMA}.customer_addresses(customer_id);
CREATE INDEX idx_customer_addresses_type ON {TENANT_SCHEMA}.customer_addresses(address_type);
CREATE INDEX idx_customer_addresses_country ON {TENANT_SCHEMA}.customer_addresses(country_code);
CREATE INDEX idx_customer_addresses_primary ON {TENANT_SCHEMA}.customer_addresses(customer_id, is_primary) WHERE is_primary = true;

-- Customer Contact Indexes
CREATE INDEX idx_customer_contacts_customer_id ON {TENANT_SCHEMA}.customer_contacts(customer_id);
CREATE INDEX idx_customer_contacts_email ON {TENANT_SCHEMA}.customer_contacts(email);
CREATE INDEX idx_customer_contacts_phone ON {TENANT_SCHEMA}.customer_contacts(phone);
CREATE INDEX idx_customer_contacts_primary ON {TENANT_SCHEMA}.customer_contacts(customer_id, is_primary) WHERE is_primary = true;

-- Order Indexes
CREATE INDEX idx_orders_customer_id ON {TENANT_SCHEMA}.orders(customer_id);
CREATE INDEX idx_orders_order_date ON {TENANT_SCHEMA}.orders(order_date);
CREATE INDEX idx_orders_status ON {TENANT_SCHEMA}.orders(status);
CREATE INDEX idx_orders_order_number ON {TENANT_SCHEMA}.orders(order_number);

-- User Indexes
CREATE INDEX idx_users_tenant_id ON {TENANT_SCHEMA}.users(tenant_id);
CREATE INDEX idx_users_username ON {TENANT_SCHEMA}.users(username);
CREATE INDEX idx_users_email ON {TENANT_SCHEMA}.users(email);
CREATE INDEX idx_users_status ON {TENANT_SCHEMA}.users(status);
CREATE INDEX idx_users_deleted_at ON {TENANT_SCHEMA}.users(deleted_at) WHERE deleted_at IS NULL;

-- Audit Log Indexes
CREATE INDEX idx_audit_log_table_record ON {TENANT_SCHEMA}.audit_log(table_name, record_id);
CREATE INDEX idx_audit_log_user_id ON {TENANT_SCHEMA}.audit_log(user_id);
CREATE INDEX idx_audit_log_created_at ON {TENANT_SCHEMA}.audit_log(created_at);

-- ================================
-- TRIGGERS for Audit and Updates
-- ================================

-- Updated timestamp triggers
CREATE OR REPLACE FUNCTION {TENANT_SCHEMA}.update_modified_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.modified_at = now();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply update triggers to all relevant tables
CREATE TRIGGER update_customers_modified_at
    BEFORE UPDATE ON {TENANT_SCHEMA}.customers
    FOR EACH ROW EXECUTE FUNCTION {TENANT_SCHEMA}.update_modified_at();

CREATE TRIGGER update_customer_addresses_modified_at
    BEFORE UPDATE ON {TENANT_SCHEMA}.customer_addresses
    FOR EACH ROW EXECUTE FUNCTION {TENANT_SCHEMA}.update_modified_at();

CREATE TRIGGER update_customer_contacts_modified_at
    BEFORE UPDATE ON {TENANT_SCHEMA}.customer_contacts
    FOR EACH ROW EXECUTE FUNCTION {TENANT_SCHEMA}.update_modified_at();

CREATE TRIGGER update_customer_tax_info_modified_at
    BEFORE UPDATE ON {TENANT_SCHEMA}.customer_tax_info
    FOR EACH ROW EXECUTE FUNCTION {TENANT_SCHEMA}.update_modified_at();

CREATE TRIGGER update_customer_financial_info_modified_at
    BEFORE UPDATE ON {TENANT_SCHEMA}.customer_financial_info
    FOR EACH ROW EXECUTE FUNCTION {TENANT_SCHEMA}.update_modified_at();

CREATE TRIGGER update_orders_modified_at
    BEFORE UPDATE ON {TENANT_SCHEMA}.orders
    FOR EACH ROW EXECUTE FUNCTION {TENANT_SCHEMA}.update_modified_at();

CREATE TRIGGER update_users_modified_at
    BEFORE UPDATE ON {TENANT_SCHEMA}.users
    FOR EACH ROW EXECUTE FUNCTION {TENANT_SCHEMA}.update_modified_at();

CREATE TRIGGER update_roles_modified_at
    BEFORE UPDATE ON {TENANT_SCHEMA}.roles
    FOR EACH ROW EXECUTE FUNCTION {TENANT_SCHEMA}.update_modified_at();

CREATE TRIGGER update_settings_modified_at
    BEFORE UPDATE ON {TENANT_SCHEMA}.settings
    FOR EACH ROW EXECUTE FUNCTION {TENANT_SCHEMA}.update_modified_at();

-- ================================
-- CONSTRAINTS and Rules
-- ================================

-- Ensure only one primary address/contact per customer per type
CREATE UNIQUE INDEX idx_customers_primary_address_unique
    ON {TENANT_SCHEMA}.customer_addresses(customer_id, address_type)
    WHERE is_primary = true;

CREATE UNIQUE INDEX idx_customers_primary_contact_unique
    ON {TENANT_SCHEMA}.customer_contacts(customer_id, contact_type)
    WHERE is_primary = true;

-- Reset search path
SET search_path TO DEFAULT;