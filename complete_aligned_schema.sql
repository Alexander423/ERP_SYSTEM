-- Complete aligned ERP database schema that matches all SQL queries in the codebase
-- This schema includes all tables referenced in repository code

-- Drop all existing tables and types in dependency order
DROP TABLE IF EXISTS customer_events CASCADE;
DROP TABLE IF EXISTS customer_snapshots CASCADE;
DROP TABLE IF EXISTS customer_performance_metrics CASCADE;
DROP TABLE IF EXISTS customer_behavioral_data CASCADE;
DROP TABLE IF EXISTS customer_addresses CASCADE;
DROP TABLE IF EXISTS customer_contacts CASCADE;
DROP TABLE IF EXISTS supplier_contacts CASCADE;
DROP TABLE IF EXISTS supplier_addresses CASCADE;
DROP TABLE IF EXISTS supplier_performance CASCADE;
DROP TABLE IF EXISTS sales_transactions CASCADE;
DROP TABLE IF EXISTS inventory_valuations CASCADE;
DROP TABLE IF EXISTS inventory_alerts CASCADE;
DROP TABLE IF EXISTS inventory_forecasts CASCADE;
DROP TABLE IF EXISTS inventory_movements CASCADE;
DROP TABLE IF EXISTS location_inventory CASCADE;
DROP TABLE IF EXISTS locations CASCADE;
DROP TABLE IF EXISTS suppliers CASCADE;
DROP TABLE IF EXISTS customers CASCADE;
DROP TABLE IF EXISTS products CASCADE;
DROP TABLE IF EXISTS security_audit_log CASCADE;
DROP TABLE IF EXISTS data_masking_policies CASCADE;
DROP TABLE IF EXISTS user_roles CASCADE;
DROP TABLE IF EXISTS role_permissions CASCADE;
DROP TABLE IF EXISTS roles CASCADE;
DROP TABLE IF EXISTS access_attempts CASCADE;
DROP TABLE IF EXISTS role_hierarchy CASCADE;
DROP TABLE IF EXISTS remediation_actions CASCADE;
DROP TABLE IF EXISTS addresses CASCADE;
DROP TABLE IF EXISTS contacts CASCADE;
DROP TABLE IF EXISTS sales_data CASCADE;
DROP TABLE IF EXISTS inventory_analytics CASCADE;
DROP TABLE IF EXISTS market_analysis CASCADE;

-- Drop all enum types
DROP TYPE IF EXISTS customer_type CASCADE;
DROP TYPE IF EXISTS supplier_type CASCADE;
DROP TYPE IF EXISTS product_type CASCADE;
DROP TYPE IF EXISTS location_type CASCADE;
DROP TYPE IF EXISTS alert_severity CASCADE;
DROP TYPE IF EXISTS alert_type CASCADE;
DROP TYPE IF EXISTS alert_status CASCADE;
DROP TYPE IF EXISTS order_status CASCADE;
DROP TYPE IF EXISTS order_priority CASCADE;
DROP TYPE IF EXISTS line_status CASCADE;
DROP TYPE IF EXISTS valuation_method CASCADE;
DROP TYPE IF EXISTS reservation_type CASCADE;
DROP TYPE IF EXISTS reservation_priority CASCADE;
DROP TYPE IF EXISTS reservation_status CASCADE;
DROP TYPE IF EXISTS forecast_method CASCADE;
DROP TYPE IF EXISTS abc_classification CASCADE;
DROP TYPE IF EXISTS movement_velocity CASCADE;
DROP TYPE IF EXISTS movement_type CASCADE;
DROP TYPE IF EXISTS transfer_status CASCADE;
DROP TYPE IF EXISTS transfer_priority CASCADE;
DROP TYPE IF EXISTS count_status CASCADE;
DROP TYPE IF EXISTS aging_category CASCADE;
DROP TYPE IF EXISTS supplier_status CASCADE;
DROP TYPE IF EXISTS supplier_category CASCADE;
DROP TYPE IF EXISTS payment_terms CASCADE;
DROP TYPE IF EXISTS customer_lifecycle_stage CASCADE;
DROP TYPE IF EXISTS credit_status CASCADE;
DROP TYPE IF EXISTS compliance_status CASCADE;
DROP TYPE IF EXISTS kyc_status CASCADE;
DROP TYPE IF EXISTS acquisition_channel CASCADE;
DROP TYPE IF EXISTS entity_status CASCADE;

-- Create all enum types that match Rust code
CREATE TYPE customer_type AS ENUM ('b2b', 'b2c', 'b2g', 'business', 'individual', 'government', 'internal', 'reseller', 'distributor', 'end_user', 'prospect');
CREATE TYPE customer_lifecycle_stage AS ENUM ('lead', 'prospect', 'prospect_customer', 'new_customer', 'active', 'active_customer', 'vip_customer', 'at_risk_customer', 'inactive_customer', 'churned', 'won_back_customer', 'former_customer');
CREATE TYPE credit_status AS ENUM ('excellent', 'good', 'fair', 'poor', 'on_hold', 'blocked', 'cash_only', 'requires_prepayment');
CREATE TYPE compliance_status AS ENUM ('compliant', 'non_compliant', 'under_review', 'pending_documents', 'exempt', 'unknown');
CREATE TYPE kyc_status AS ENUM ('not_started', 'in_progress', 'completed', 'requires_update', 'failed', 'exempted');
CREATE TYPE acquisition_channel AS ENUM ('direct_sales', 'website_inquiry', 'social_media', 'email_marketing', 'search_engine', 'referral', 'partner_channel', 'trade_show', 'cold_call', 'advertisement', 'other');
CREATE TYPE entity_status AS ENUM ('active', 'inactive', 'pending', 'suspended', 'terminated');

CREATE TYPE supplier_status AS ENUM ('active', 'inactive', 'pending', 'suspended', 'terminated');
CREATE TYPE supplier_category AS ENUM ('raw_materials', 'manufacturing', 'technology', 'services', 'logistics', 'office_supplies', 'marketing', 'utilities', 'other');
CREATE TYPE payment_terms AS ENUM ('net15', 'net30', 'net45', 'net60', 'net90', 'two_ten_net30', 'cod', 'prepaid');

CREATE TYPE product_type AS ENUM ('physical', 'digital', 'service', 'subscription');
CREATE TYPE location_type AS ENUM ('warehouse', 'store', 'distribution_center', 'manufacturing_plant', 'office', 'customer_site');
CREATE TYPE alert_severity AS ENUM ('info', 'low', 'medium', 'high', 'warning', 'critical', 'emergency');
CREATE TYPE alert_type AS ENUM ('low_stock', 'stockout', 'excess_stock', 'slow_moving', 'expiring', 'expired', 'quality_issue', 'variance_detected', 'supplier_delay', 'demand_spike', 'seasonal_alert');
CREATE TYPE alert_status AS ENUM ('new', 'acknowledged', 'in_progress', 'resolved', 'dismissed');
CREATE TYPE order_status AS ENUM ('draft', 'pending', 'approved', 'sent', 'acknowledged', 'partially_received', 'received', 'invoiced', 'paid', 'cancelled', 'rejected');
CREATE TYPE order_priority AS ENUM ('low', 'normal', 'high', 'rush', 'emergency');
CREATE TYPE line_status AS ENUM ('pending', 'partially_received', 'received', 'cancelled', 'rejected');
CREATE TYPE valuation_method AS ENUM ('fifo', 'lifo', 'weighted_average', 'standard_cost', 'specific_cost', 'retail_method');
CREATE TYPE reservation_type AS ENUM ('sales_order', 'production_order', 'transfer', 'quality', 'damage', 'special', 'promotional');
CREATE TYPE reservation_priority AS ENUM ('low', 'normal', 'high', 'critical');
CREATE TYPE reservation_status AS ENUM ('active', 'fulfilled', 'expired', 'cancelled', 'partially_fulfilled');
CREATE TYPE forecast_method AS ENUM ('moving_average', 'exponential_smoothing', 'linear_regression', 'seasonal_decomposition', 'arima', 'machine_learning', 'hybrid_model');
CREATE TYPE abc_classification AS ENUM ('a', 'b', 'c');
CREATE TYPE movement_velocity AS ENUM ('fast', 'medium', 'slow', 'dead');
CREATE TYPE movement_type AS ENUM ('receipt', 'issue', 'adjustment', 'transfer', 'return', 'sale', 'production', 'scrap');
CREATE TYPE transfer_status AS ENUM ('pending', 'in_transit', 'completed', 'cancelled');
CREATE TYPE transfer_priority AS ENUM ('low', 'normal', 'high', 'urgent');
CREATE TYPE count_status AS ENUM ('planned', 'in_progress', 'completed', 'cancelled');
CREATE TYPE aging_category AS ENUM ('current', 'slow_moving', 'dead_stock', 'obsolete');

-- Generic addresses table for all entity types
CREATE TABLE addresses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type VARCHAR(50) NOT NULL,
    entity_id UUID NOT NULL,
    address_type VARCHAR(50) NOT NULL DEFAULT 'primary',
    street_line_1 VARCHAR(255) NOT NULL,
    street_line_2 VARCHAR(255),
    city VARCHAR(100) NOT NULL,
    state_province VARCHAR(100),
    postal_code VARCHAR(20) NOT NULL,
    country_code VARCHAR(3) NOT NULL,
    coordinates JSONB,
    is_primary BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    tenant_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL,
    updated_by UUID NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    deleted_at TIMESTAMPTZ,
    deleted_by UUID
);

-- Generic contacts table for all entity types
CREATE TABLE contacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type VARCHAR(50) NOT NULL,
    entity_id UUID NOT NULL,
    contact_type VARCHAR(50) NOT NULL DEFAULT 'primary',
    first_name VARCHAR(100) NOT NULL,
    last_name VARCHAR(100) NOT NULL,
    title VARCHAR(100),
    department VARCHAR(100),
    email VARCHAR(255),
    phone VARCHAR(50),
    mobile VARCHAR(50),
    is_primary BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    tenant_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL,
    updated_by UUID NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    deleted_at TIMESTAMPTZ,
    deleted_by UUID
);

-- Comprehensive customers table
CREATE TABLE customers (
    -- Core Identity
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_number VARCHAR(50) NOT NULL,
    external_ids JSONB DEFAULT '{}',

    -- Business Information
    legal_name VARCHAR(255) NOT NULL,
    trade_names JSONB DEFAULT '[]',
    customer_type customer_type NOT NULL DEFAULT 'individual',

    -- Hierarchy & Grouping
    parent_customer_id UUID REFERENCES customers(id),
    corporate_group_id UUID,
    customer_hierarchy_level INTEGER DEFAULT 0,
    consolidation_group VARCHAR(100),

    -- Status & Lifecycle
    lifecycle_stage customer_lifecycle_stage NOT NULL DEFAULT 'lead',
    status entity_status NOT NULL DEFAULT 'active',
    credit_status credit_status NOT NULL DEFAULT 'good',

    -- Geographic & Contact Information
    primary_address_id UUID,
    billing_address_id UUID,
    shipping_address_ids JSONB DEFAULT '[]',
    primary_contact_id UUID,

    -- Tax & Legal
    tax_jurisdictions JSONB DEFAULT '[]',
    tax_numbers JSONB DEFAULT '{}',
    regulatory_classifications JSONB DEFAULT '[]',
    compliance_status compliance_status NOT NULL DEFAULT 'unknown',
    kyc_status kyc_status NOT NULL DEFAULT 'not_started',

    -- Commercial & Financial
    financial_info JSONB DEFAULT '{}',
    price_group_id UUID,
    discount_group_id UUID,

    -- Sales & Marketing
    sales_representative_id UUID,
    account_manager_id UUID,
    customer_segments JSONB DEFAULT '[]',
    acquisition_channel acquisition_channel,
    customer_lifetime_value DECIMAL(15,2),
    churn_probability DECIMAL(5,4),

    -- Analytics & Intelligence
    performance_metrics JSONB DEFAULT '{}',
    behavioral_data JSONB DEFAULT '{}',

    -- Integration & Sync
    sync_info JSONB DEFAULT '{}',

    -- Custom & Extended Data
    custom_fields JSONB DEFAULT '{}',
    contract_ids JSONB DEFAULT '[]',

    -- Multi-tenancy
    tenant_id UUID NOT NULL,

    -- Audit Trail
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL,
    updated_by UUID NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    deleted_at TIMESTAMPTZ,
    deleted_by UUID,

    UNIQUE(tenant_id, customer_number)
);

-- Customer performance metrics table (referenced in queries)
CREATE TABLE customer_performance_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id UUID NOT NULL REFERENCES customers(id),
    tenant_id UUID NOT NULL,
    reporting_period_start DATE NOT NULL,
    reporting_period_end DATE NOT NULL,
    total_revenue DECIMAL(15,2) DEFAULT 0,
    total_orders INTEGER DEFAULT 0,
    average_order_value DECIMAL(12,2) DEFAULT 0,
    customer_lifetime_value DECIMAL(15,2) DEFAULT 0,
    churn_risk_score DECIMAL(5,4) DEFAULT 0,
    satisfaction_score DECIMAL(3,2) DEFAULT 0,
    engagement_score DECIMAL(5,4) DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Customer behavioral data table (referenced in queries)
CREATE TABLE customer_behavioral_data (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id UUID NOT NULL REFERENCES customers(id),
    tenant_id UUID NOT NULL,
    last_login_at TIMESTAMPTZ,
    login_frequency INTEGER DEFAULT 0,
    page_views INTEGER DEFAULT 0,
    session_duration_avg INTEGER DEFAULT 0, -- in seconds
    preferred_communication_channel VARCHAR(50),
    interaction_patterns JSONB DEFAULT '{}',
    purchase_patterns JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Customer-specific addresses view table (for backward compatibility)
CREATE TABLE customer_addresses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id UUID NOT NULL REFERENCES customers(id),
    address_id UUID NOT NULL REFERENCES addresses(id),
    tenant_id UUID NOT NULL,
    UNIQUE(customer_id, address_id)
);

-- Customer-specific contacts view table (for backward compatibility)
CREATE TABLE customer_contacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id UUID NOT NULL REFERENCES customers(id),
    contact_id UUID NOT NULL REFERENCES contacts(id),
    tenant_id UUID NOT NULL,
    UNIQUE(customer_id, contact_id)
);

-- Customer event store
CREATE TABLE customer_events (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    aggregate_id UUID NOT NULL,
    tenant_id UUID NOT NULL,
    sequence_number BIGINT NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    event_data JSONB NOT NULL,
    metadata JSONB DEFAULT '{}',
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    recorded_by UUID NOT NULL,
    UNIQUE(aggregate_id, sequence_number)
);

-- Customer snapshots
CREATE TABLE customer_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    aggregate_id UUID NOT NULL,
    tenant_id UUID NOT NULL,
    version BIGINT NOT NULL,
    snapshot_data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(aggregate_id, tenant_id)
);

-- Comprehensive suppliers table
CREATE TABLE suppliers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,

    -- Basic Information
    supplier_code VARCHAR(50) NOT NULL,
    company_name VARCHAR(255) NOT NULL,
    legal_name VARCHAR(255),
    tax_id VARCHAR(100),
    registration_number VARCHAR(100),

    -- Classification
    category supplier_category NOT NULL DEFAULT 'other',
    status supplier_status NOT NULL DEFAULT 'pending',
    tags JSONB DEFAULT '[]',

    -- Contact Information
    website VARCHAR(500),
    phone VARCHAR(50),
    email VARCHAR(255),

    -- Business Terms
    payment_terms payment_terms NOT NULL DEFAULT 'net30',
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    credit_limit BIGINT, -- in cents
    lead_time_days INTEGER,

    -- Performance Metrics
    rating DECIMAL(3,2),
    on_time_delivery_rate DECIMAL(5,4),
    quality_rating DECIMAL(3,2),

    -- Metadata
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL,
    updated_by UUID NOT NULL,

    UNIQUE(tenant_id, supplier_code)
);

-- Supplier contacts (referenced in queries)
CREATE TABLE supplier_contacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    supplier_id UUID NOT NULL REFERENCES suppliers(id),
    contact_id UUID NOT NULL REFERENCES contacts(id),
    tenant_id UUID NOT NULL,
    UNIQUE(supplier_id, contact_id)
);

-- Supplier addresses (referenced in queries)
CREATE TABLE supplier_addresses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    supplier_id UUID NOT NULL REFERENCES suppliers(id),
    address_id UUID NOT NULL REFERENCES addresses(id),
    tenant_id UUID NOT NULL,
    UNIQUE(supplier_id, address_id)
);

-- Supplier performance tracking
CREATE TABLE supplier_performance (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    supplier_id UUID NOT NULL REFERENCES suppliers(id),
    tenant_id UUID NOT NULL,
    reporting_period_start DATE NOT NULL,
    reporting_period_end DATE NOT NULL,
    on_time_delivery_rate DECIMAL(5,4) DEFAULT 0,
    quality_score DECIMAL(3,2) DEFAULT 0,
    cost_competitiveness DECIMAL(5,4) DEFAULT 0,
    responsiveness_score DECIMAL(3,2) DEFAULT 0,
    compliance_score DECIMAL(3,2) DEFAULT 0,
    total_orders INTEGER DEFAULT 0,
    total_value DECIMAL(15,2) DEFAULT 0,
    average_lead_time INTEGER DEFAULT 0,
    defect_rate DECIMAL(5,4) DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Products table
CREATE TABLE products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    product_number VARCHAR(50) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    product_type product_type NOT NULL DEFAULT 'physical',
    category VARCHAR(100),
    unit_of_measure VARCHAR(20),
    weight DECIMAL(10,2),
    dimensions JSONB,
    barcode VARCHAR(100),
    sku VARCHAR(100),
    manufacturer VARCHAR(255),
    brand VARCHAR(255),
    list_price DECIMAL(12,2),
    cost_price DECIMAL(12,2),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(tenant_id, product_number)
);

-- Locations table
CREATE TABLE locations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    location_code VARCHAR(50) NOT NULL,
    name VARCHAR(255) NOT NULL,
    location_type location_type NOT NULL DEFAULT 'warehouse',
    address JSONB,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(tenant_id, location_code)
);

-- Location inventory with all fields referenced in queries
CREATE TABLE location_inventory (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL REFERENCES products(id),
    location_id UUID NOT NULL REFERENCES locations(id),
    quantity_available INTEGER NOT NULL DEFAULT 0,
    quantity_reserved INTEGER NOT NULL DEFAULT 0,
    quantity_on_order INTEGER NOT NULL DEFAULT 0,
    quantity_in_transit INTEGER NOT NULL DEFAULT 0,
    reorder_point INTEGER NOT NULL DEFAULT 0,
    max_stock_level INTEGER NOT NULL DEFAULT 0,
    min_stock_level INTEGER NOT NULL DEFAULT 0,
    safety_stock INTEGER NOT NULL DEFAULT 0,
    economic_order_quantity INTEGER NOT NULL DEFAULT 0,
    lead_time_days INTEGER NOT NULL DEFAULT 0,
    storage_cost_per_unit DECIMAL(10,4) NOT NULL DEFAULT 0,
    handling_cost_per_unit DECIMAL(10,4) NOT NULL DEFAULT 0,
    last_counted_at TIMESTAMPTZ,
    cycle_count_frequency_days INTEGER,
    abc_classification abc_classification DEFAULT 'c',
    movement_velocity movement_velocity DEFAULT 'medium',
    seasonal_factors JSONB DEFAULT '{}',
    storage_requirements JSONB,
    current_stock INTEGER NOT NULL DEFAULT 0, -- For optimization queries
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(product_id, location_id)
);

-- Inventory movements
CREATE TABLE inventory_movements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL REFERENCES products(id),
    location_id UUID NOT NULL REFERENCES locations(id),
    movement_type movement_type NOT NULL,
    quantity INTEGER NOT NULL,
    unit_cost DECIMAL(12,4),
    total_cost DECIMAL(12,2),
    reference_id UUID,
    reference_type VARCHAR(50),
    movement_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Inventory forecasts
CREATE TABLE inventory_forecasts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL REFERENCES products(id),
    location_id UUID NOT NULL REFERENCES locations(id),
    forecast_date TIMESTAMPTZ NOT NULL,
    forecast_horizon_days INTEGER NOT NULL,
    predicted_demand DECIMAL(12,4) NOT NULL,
    predicted_supply DECIMAL(12,4) NOT NULL DEFAULT 0,
    predicted_stock_level DECIMAL(12,4) NOT NULL DEFAULT 0,
    confidence_level DECIMAL(5,4) NOT NULL DEFAULT 0.9,
    confidence_lower DECIMAL(12,4) NOT NULL DEFAULT 0,
    confidence_upper DECIMAL(12,4) NOT NULL DEFAULT 0,
    forecast_method forecast_method NOT NULL DEFAULT 'moving_average',
    seasonal_index DECIMAL(8,4) NOT NULL DEFAULT 1.0,
    seasonal_component DECIMAL(12,4) NOT NULL DEFAULT 0,
    trend_factor DECIMAL(8,4) NOT NULL DEFAULT 1.0,
    trend_component DECIMAL(12,4) NOT NULL DEFAULT 0,
    external_factors JSONB,
    accuracy_score DECIMAL(5,4) NOT NULL DEFAULT 0.0,
    model_version VARCHAR(50) NOT NULL DEFAULT 'v1.0',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Inventory alerts
CREATE TABLE inventory_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL REFERENCES products(id),
    location_id UUID NOT NULL REFERENCES locations(id),
    alert_type alert_type NOT NULL,
    severity alert_severity NOT NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    current_quantity INTEGER NOT NULL,
    threshold_value DECIMAL(12,4) NOT NULL,
    recommended_action TEXT,
    alert_status alert_status NOT NULL DEFAULT 'new',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    acknowledged_at TIMESTAMPTZ,
    acknowledged_by UUID,
    resolved_at TIMESTAMPTZ,
    resolved_by UUID,
    resolution_notes TEXT
);

-- Inventory valuations
CREATE TABLE inventory_valuations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL REFERENCES products(id),
    location_id UUID NOT NULL REFERENCES locations(id),
    valuation_date TIMESTAMPTZ NOT NULL,
    valuation_method valuation_method NOT NULL,
    quantity INTEGER NOT NULL,
    unit_cost DECIMAL(12,4) NOT NULL,
    total_value DECIMAL(15,2) NOT NULL,
    average_cost DECIMAL(12,4) NOT NULL DEFAULT 0,
    fifo_cost DECIMAL(12,4) NOT NULL DEFAULT 0,
    lifo_cost DECIMAL(12,4) NOT NULL DEFAULT 0,
    standard_cost DECIMAL(12,4) NOT NULL DEFAULT 0,
    market_value DECIMAL(12,4) NOT NULL DEFAULT 0,
    replacement_cost DECIMAL(12,4) NOT NULL DEFAULT 0,
    net_realizable_value DECIMAL(12,4) NOT NULL DEFAULT 0,
    obsolescence_reserve DECIMAL(12,4) NOT NULL DEFAULT 0,
    shrinkage_reserve DECIMAL(12,4) NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Sales transactions
CREATE TABLE sales_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL REFERENCES products(id),
    customer_id UUID REFERENCES customers(id),
    transaction_date TIMESTAMPTZ NOT NULL,
    quantity INTEGER NOT NULL,
    unit_price DECIMAL(12,4) NOT NULL,
    total_amount DECIMAL(15,2) NOT NULL,
    cost_of_goods DECIMAL(15,2),
    profit_margin DECIMAL(15,2),
    sales_channel VARCHAR(100),
    region VARCHAR(100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Analytics helper tables (referenced in product analytics)
CREATE TABLE sales_data (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL REFERENCES products(id),
    location_id UUID REFERENCES locations(id),
    sales_date DATE NOT NULL,
    quantity_sold INTEGER NOT NULL,
    revenue DECIMAL(15,2) NOT NULL,
    cost DECIMAL(15,2),
    customer_segment VARCHAR(100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE inventory_analytics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL REFERENCES products(id),
    location_id UUID REFERENCES locations(id),
    analysis_date DATE NOT NULL,
    turnover_rate DECIMAL(8,4),
    days_on_hand INTEGER,
    stockout_frequency INTEGER,
    excess_quantity INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE market_analysis (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL REFERENCES products(id),
    analysis_date DATE NOT NULL,
    market_demand DECIMAL(12,4),
    competitive_price DECIMAL(12,2),
    market_share DECIMAL(5,4),
    trend_direction VARCHAR(20),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Security and access control tables
CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    priority INTEGER DEFAULT 0,
    is_system_role BOOLEAN DEFAULT false,
    tenant_id UUID NOT NULL,
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE role_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    role_id UUID NOT NULL REFERENCES roles(id),
    permission_id UUID NOT NULL,
    resource_type VARCHAR(100) NOT NULL,
    action VARCHAR(50) NOT NULL,
    scope VARCHAR(100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE user_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    role_id UUID NOT NULL REFERENCES roles(id),
    tenant_id UUID NOT NULL,
    assigned_by UUID NOT NULL,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, role_id)
);

CREATE TABLE role_hierarchy (
    parent_role_id UUID NOT NULL REFERENCES roles(id),
    child_role_id UUID NOT NULL REFERENCES roles(id),
    PRIMARY KEY(parent_role_id, child_role_id)
);

CREATE TABLE access_attempts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID,
    resource_type VARCHAR(100) NOT NULL,
    action VARCHAR(50) NOT NULL,
    resource_id UUID,
    success BOOLEAN NOT NULL,
    ip_address INET,
    user_agent TEXT,
    attempted_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE security_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    user_id UUID,
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(100),
    resource_id UUID,
    old_values JSONB,
    new_values JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE data_masking_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    table_name VARCHAR(100) NOT NULL,
    column_name VARCHAR(100) NOT NULL,
    masking_rule VARCHAR(100) NOT NULL,
    exemptions JSONB DEFAULT '[]',
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE remediation_actions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    action_type VARCHAR(100) NOT NULL,
    description TEXT NOT NULL,
    priority VARCHAR(20) DEFAULT 'medium',
    status VARCHAR(20) DEFAULT 'pending',
    assigned_to UUID,
    due_date TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

-- Create essential indexes
CREATE INDEX IF NOT EXISTS idx_products_tenant_active ON products(tenant_id, is_active);
CREATE INDEX IF NOT EXISTS idx_customers_tenant_active ON customers(tenant_id, status);
CREATE INDEX IF NOT EXISTS idx_customers_number ON customers(tenant_id, customer_number);
CREATE INDEX IF NOT EXISTS idx_suppliers_tenant_active ON suppliers(tenant_id, status);
CREATE INDEX IF NOT EXISTS idx_locations_tenant_active ON locations(tenant_id, is_active);
CREATE INDEX IF NOT EXISTS idx_location_inventory_product_location ON location_inventory(product_id, location_id);
CREATE INDEX IF NOT EXISTS idx_inventory_movements_product_date ON inventory_movements(product_id, movement_date);
CREATE INDEX IF NOT EXISTS idx_inventory_alerts_severity_status ON inventory_alerts(severity, alert_status);
CREATE INDEX IF NOT EXISTS idx_inventory_forecasts_product_date ON inventory_forecasts(product_id, forecast_date);
CREATE INDEX IF NOT EXISTS idx_sales_transactions_product_date ON sales_transactions(product_id, transaction_date);
CREATE INDEX IF NOT EXISTS idx_customer_events_aggregate ON customer_events(aggregate_id, sequence_number);
CREATE INDEX IF NOT EXISTS idx_customer_events_type ON customer_events(event_type);
CREATE INDEX IF NOT EXISTS idx_addresses_entity ON addresses(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_contacts_entity ON contacts(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_user_roles_user ON user_roles(user_id);
CREATE INDEX IF NOT EXISTS idx_access_attempts_user_time ON access_attempts(user_id, attempted_at);

-- Insert minimal sample data for testing
INSERT INTO products (tenant_id, product_number, name, product_type)
VALUES (gen_random_uuid(), 'TEST001', 'Test Product', 'physical')
ON CONFLICT DO NOTHING;

INSERT INTO customers (tenant_id, customer_number, legal_name, customer_type, created_by, updated_by)
VALUES (gen_random_uuid(), 'CUST001', 'Test Customer', 'business', gen_random_uuid(), gen_random_uuid())
ON CONFLICT DO NOTHING;

INSERT INTO suppliers (tenant_id, supplier_code, company_name, category, created_by, updated_by)
VALUES (gen_random_uuid(), 'SUPP001', 'Test Supplier', 'manufacturing', gen_random_uuid(), gen_random_uuid())
ON CONFLICT DO NOTHING;

INSERT INTO locations (tenant_id, location_code, name, location_type)
VALUES (gen_random_uuid(), 'LOC001', 'Main Warehouse', 'warehouse')
ON CONFLICT DO NOTHING;