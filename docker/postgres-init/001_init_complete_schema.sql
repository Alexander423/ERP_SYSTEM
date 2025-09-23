-- =====================================================
-- Complete ERP Schema Initialization for Docker
-- =====================================================
-- Consolidated script that applies all structured migrations in order
-- This file is automatically executed when PostgreSQL container starts

\echo 'Starting ERP Schema Initialization...'

-- =====================================================
-- 001_FOUNDATION: Extensions, Enums, Functions
-- =====================================================

\echo '[1/5] Applying foundation layer...'

-- Extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp" WITH SCHEMA public;
CREATE EXTENSION IF NOT EXISTS "pgcrypto" WITH SCHEMA public;

-- Enums
CREATE TYPE product_status AS ENUM (
    'active', 'inactive', 'development', 'discontinued', 'planned'
);

CREATE TYPE product_type AS ENUM (
    'physical', 'digital', 'service', 'bundle', 'subscription'
);

CREATE TYPE customer_type AS ENUM (
    'individual', 'corporate', 'government', 'non_profit'
);

CREATE TYPE entity_status AS ENUM (
    'active', 'inactive', 'suspended', 'pending', 'archived'
);

CREATE TYPE address_type AS ENUM (
    'billing', 'shipping', 'mailing', 'headquarters', 'branch', 'warehouse'
);

CREATE TYPE contact_type AS ENUM (
    'primary', 'billing', 'technical', 'sales', 'support', 'emergency'
);

CREATE TYPE unit_of_measure AS ENUM (
    'piece', 'kilogram', 'gram', 'liter', 'meter', 'centimeter', 'square_meter',
    'cubic_meter', 'hour', 'day', 'month', 'year', 'set', 'package', 'box', 'pallet'
);

CREATE TYPE movement_type AS ENUM (
    'inbound', 'outbound', 'transfer', 'adjustment', 'return', 'loss', 'found'
);

CREATE TYPE adjustment_type AS ENUM (
    'physical_count', 'damage', 'theft', 'expiry', 'system_error', 'other'
);

CREATE TYPE transfer_status AS ENUM (
    'requested', 'approved', 'picked', 'shipped', 'received', 'cancelled'
);

-- Sales transaction status
CREATE TYPE sales_status AS ENUM (
    'draft', 'pending', 'processing', 'completed', 'cancelled', 'refunded'
);

CREATE TYPE alert_type AS ENUM (
    'low_stock', 'stockout', 'overstock', 'slow_moving', 'expiry_warning', 'reorder_point'
);

CREATE TYPE alert_severity AS ENUM (
    'low', 'medium', 'high', 'critical'
);

CREATE TYPE abc_classification AS ENUM ('A', 'B', 'C');

CREATE TYPE xyz_classification AS ENUM ('X', 'Y', 'Z');

CREATE TYPE movement_velocity AS ENUM (
    'fast', 'medium', 'slow', 'non_moving'
);

CREATE TYPE turnover_classification AS ENUM (
    'high_turnover', 'medium_turnover', 'low_turnover', 'excess_inventory'
);

-- Utility Functions
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION validate_sku(sku_value TEXT)
RETURNS BOOLEAN AS $$
BEGIN
    RETURN sku_value ~ '^[A-Z0-9][A-Z0-9\-_]*[A-Z0-9]$' AND LENGTH(sku_value) >= 3;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION generate_customer_number(tenant_uuid UUID)
RETURNS TEXT AS $$
DECLARE
    next_number INTEGER;
    customer_number TEXT;
BEGIN
    SELECT COALESCE(MAX(CAST(SUBSTRING(customer_number FROM 6) AS INTEGER)), 0) + 1
    INTO next_number
    FROM customers
    WHERE tenant_id = tenant_uuid AND customer_number ~ '^CUST-[0-9]+$';

    customer_number := 'CUST-' || LPAD(next_number::TEXT, 6, '0');
    RETURN customer_number;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION generate_supplier_number(tenant_uuid UUID)
RETURNS TEXT AS $$
DECLARE
    next_number INTEGER;
    supplier_number TEXT;
BEGIN
    SELECT COALESCE(MAX(CAST(SUBSTRING(supplier_number FROM 6) AS INTEGER)), 0) + 1
    INTO next_number
    FROM suppliers
    WHERE tenant_id = tenant_uuid AND supplier_number ~ '^SUPP-[0-9]+$';

    supplier_number := 'SUPP-' || LPAD(next_number::TEXT, 6, '0');
    RETURN supplier_number;
END;
$$ LANGUAGE plpgsql;

\echo '✓ Foundation layer completed'

-- =====================================================
-- TENANTS TABLE (Required by all other tables)
-- =====================================================

CREATE TABLE tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(100) NOT NULL UNIQUE,
    schema_name VARCHAR(100) UNIQUE,
    subscription_tier VARCHAR(50) NOT NULL DEFAULT 'basic',
    status VARCHAR(20) DEFAULT 'active',
    is_active BOOLEAN NOT NULL DEFAULT true,
    settings JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL,
    updated_by UUID NOT NULL,
    CONSTRAINT check_tenant_status
        CHECK (status IN ('active', 'suspended', 'deleted'))
);

-- =====================================================
-- 002_CORE_TABLES: Products, Customers, Suppliers, Addresses
-- =====================================================

\echo '[2/5] Applying core tables layer...'

-- Product Categories
CREATE TABLE product_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    parent_id UUID,
    level INTEGER NOT NULL DEFAULT 0,
    path TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL,
    updated_by UUID NOT NULL,
    CONSTRAINT fk_product_categories_parent
        FOREIGN KEY (parent_id) REFERENCES product_categories(id),
    CONSTRAINT unique_tenant_category_name
        UNIQUE (tenant_id, name, parent_id)
);

-- Products
CREATE TABLE products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    sku VARCHAR(100) NOT NULL,
    name VARCHAR(500) NOT NULL,
    description TEXT,
    short_description TEXT,
    category_id UUID,
    product_type product_type NOT NULL DEFAULT 'physical',
    status product_status NOT NULL DEFAULT 'development',
    tags TEXT[],
    unit_of_measure unit_of_measure NOT NULL DEFAULT 'piece',
    weight DECIMAL(10,3),
    dimensions_length DECIMAL(10,2),
    dimensions_width DECIMAL(10,2),
    dimensions_height DECIMAL(10,2),
    base_price BIGINT NOT NULL DEFAULT 0,
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    cost_price BIGINT,
    list_price BIGINT,
    is_tracked BOOLEAN NOT NULL DEFAULT true,
    current_stock INTEGER,
    min_stock_level INTEGER,
    max_stock_level INTEGER,
    reorder_point INTEGER,
    primary_supplier_id UUID,
    lead_time_days INTEGER,
    barcode VARCHAR(100),
    brand VARCHAR(100),
    manufacturer VARCHAR(100),
    model_number VARCHAR(100),
    warranty_months INTEGER,
    slug VARCHAR(200),
    meta_title VARCHAR(200),
    meta_description TEXT,
    is_featured BOOLEAN NOT NULL DEFAULT false,
    is_digital_download BOOLEAN NOT NULL DEFAULT false,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL,
    updated_by UUID NOT NULL,
    CONSTRAINT fk_products_category
        FOREIGN KEY (category_id) REFERENCES product_categories(id),
    CONSTRAINT unique_tenant_sku
        UNIQUE (tenant_id, sku),
    CONSTRAINT check_sku_format
        CHECK (validate_sku(sku)),
    CONSTRAINT check_positive_prices
        CHECK (base_price >= 0 AND (cost_price IS NULL OR cost_price >= 0) AND (list_price IS NULL OR list_price >= 0))
);

-- Product Variants
CREATE TABLE product_variants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL,
    variant_sku VARCHAR(100) NOT NULL,
    variant_name VARCHAR(255) NOT NULL,
    attributes JSONB,
    price_adjustment BIGINT DEFAULT 0,
    cost_price BIGINT,
    current_stock INTEGER DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL,
    updated_by UUID NOT NULL,
    CONSTRAINT fk_product_variants_product
        FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE,
    CONSTRAINT unique_variant_sku
        UNIQUE (variant_sku)
);

-- Customers
CREATE TABLE customers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    customer_number VARCHAR(50) NOT NULL,
    legal_name VARCHAR(255) NOT NULL,
    customer_type customer_type NOT NULL DEFAULT 'individual',
    status entity_status NOT NULL DEFAULT 'active',
    parent_customer_id UUID,
    primary_contact_id UUID,
    primary_address_id UUID,
    billing_address_id UUID,
    credit_limit DECIMAL(15,2),
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    payment_terms_days INTEGER DEFAULT 30,
    total_orders INTEGER DEFAULT 0,
    total_spent DECIMAL(15,2) DEFAULT 0,
    last_order_date TIMESTAMPTZ,
    preferred_communication VARCHAR(20) DEFAULT 'email',
    marketing_consent BOOLEAN DEFAULT false,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL,
    updated_by UUID NOT NULL,
    CONSTRAINT fk_customers_parent
        FOREIGN KEY (parent_customer_id) REFERENCES customers(id),
    CONSTRAINT unique_tenant_customer_number
        UNIQUE (tenant_id, customer_number),
    CONSTRAINT check_positive_credit_limit
        CHECK (credit_limit IS NULL OR credit_limit >= 0),
    CONSTRAINT check_payment_terms
        CHECK (payment_terms_days > 0)
);

-- Customer Groups
CREATE TABLE customer_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    group_name VARCHAR(100) NOT NULL,
    description TEXT,
    discount_percentage DECIMAL(5,2) DEFAULT 0,
    payment_terms_days INTEGER,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL,
    updated_by UUID NOT NULL,
    CONSTRAINT unique_tenant_group_name
        UNIQUE (tenant_id, group_name),
    CONSTRAINT check_discount_percentage
        CHECK (discount_percentage >= 0 AND discount_percentage <= 100)
);

-- Customer Group Memberships
CREATE TABLE customer_group_memberships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id UUID NOT NULL,
    group_id UUID NOT NULL,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL,
    CONSTRAINT fk_customer_group_memberships_customer
        FOREIGN KEY (customer_id) REFERENCES customers(id) ON DELETE CASCADE,
    CONSTRAINT fk_customer_group_memberships_group
        FOREIGN KEY (group_id) REFERENCES customer_groups(id) ON DELETE CASCADE,
    CONSTRAINT unique_customer_group
        UNIQUE (customer_id, group_id)
);

-- Customer Credit History
CREATE TABLE customer_credit_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id UUID NOT NULL,
    previous_limit DECIMAL(15,2),
    new_limit DECIMAL(15,2) NOT NULL,
    change_reason TEXT,
    approved_by UUID NOT NULL,
    approved_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL,
    CONSTRAINT fk_customer_credit_history_customer
        FOREIGN KEY (customer_id) REFERENCES customers(id) ON DELETE CASCADE,
    CONSTRAINT check_credit_limits_positive
        CHECK (
            (previous_limit IS NULL OR previous_limit >= 0) AND
            new_limit >= 0
        )
);

-- Suppliers
CREATE TABLE suppliers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    supplier_number VARCHAR(50) NOT NULL,
    legal_name VARCHAR(255) NOT NULL,
    trade_name VARCHAR(255),
    status entity_status NOT NULL DEFAULT 'active',
    primary_contact_id UUID,
    primary_address_id UUID,
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    payment_terms_days INTEGER DEFAULT 30,
    quality_rating DECIMAL(3,2),
    delivery_rating DECIMAL(3,2),
    overall_rating DECIMAL(3,2),
    total_orders INTEGER DEFAULT 0,
    on_time_deliveries INTEGER DEFAULT 0,
    lead_time_days INTEGER,
    minimum_order_amount DECIMAL(15,2),
    maximum_order_amount DECIMAL(15,2),
    certifications TEXT[],
    compliance_status JSONB,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL,
    updated_by UUID NOT NULL,
    CONSTRAINT unique_tenant_supplier_number
        UNIQUE (tenant_id, supplier_number),
    CONSTRAINT check_positive_payment_terms
        CHECK (payment_terms_days > 0),
    CONSTRAINT check_rating_ranges
        CHECK (
            (quality_rating IS NULL OR (quality_rating >= 0 AND quality_rating <= 5)) AND
            (delivery_rating IS NULL OR (delivery_rating >= 0 AND delivery_rating <= 5)) AND
            (overall_rating IS NULL OR (overall_rating >= 0 AND overall_rating <= 5))
        ),
    CONSTRAINT check_order_amounts
        CHECK (
            (minimum_order_amount IS NULL OR minimum_order_amount >= 0) AND
            (maximum_order_amount IS NULL OR maximum_order_amount >= 0) AND
            (minimum_order_amount IS NULL OR maximum_order_amount IS NULL OR minimum_order_amount <= maximum_order_amount)
        )
);

-- Supplier Products
CREATE TABLE supplier_products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    supplier_id UUID NOT NULL,
    product_id UUID NOT NULL,
    supplier_sku VARCHAR(100),
    supplier_name VARCHAR(255),
    unit_cost DECIMAL(15,4) NOT NULL,
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    minimum_quantity INTEGER DEFAULT 1,
    lead_time_days INTEGER DEFAULT 7,
    is_active BOOLEAN NOT NULL DEFAULT true,
    is_preferred BOOLEAN NOT NULL DEFAULT false,
    quality_rating DECIMAL(3,2),
    last_delivery_rating DECIMAL(3,2),
    valid_from TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    valid_until TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL,
    updated_by UUID NOT NULL,
    CONSTRAINT fk_supplier_products_supplier
        FOREIGN KEY (supplier_id) REFERENCES suppliers(id) ON DELETE CASCADE,
    CONSTRAINT fk_supplier_products_product
        FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE,
    CONSTRAINT unique_supplier_product
        UNIQUE (supplier_id, product_id),
    CONSTRAINT check_positive_cost
        CHECK (unit_cost >= 0),
    CONSTRAINT check_positive_minimum_quantity
        CHECK (minimum_quantity > 0),
    CONSTRAINT check_validity_period
        CHECK (valid_until IS NULL OR valid_until > valid_from)
);

-- Supplier Performance History
CREATE TABLE supplier_performance_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    supplier_id UUID NOT NULL,
    evaluation_date DATE NOT NULL,
    evaluation_period_start DATE NOT NULL,
    evaluation_period_end DATE NOT NULL,
    orders_count INTEGER NOT NULL DEFAULT 0,
    on_time_deliveries INTEGER NOT NULL DEFAULT 0,
    quality_issues INTEGER NOT NULL DEFAULT 0,
    total_value DECIMAL(15,2) NOT NULL DEFAULT 0,
    delivery_performance DECIMAL(5,2),
    quality_rating DECIMAL(3,2),
    overall_rating DECIMAL(3,2),
    evaluation_notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL,
    CONSTRAINT fk_supplier_performance_history_supplier
        FOREIGN KEY (supplier_id) REFERENCES suppliers(id) ON DELETE CASCADE,
    CONSTRAINT check_evaluation_period
        CHECK (evaluation_period_end >= evaluation_period_start),
    CONSTRAINT check_performance_metrics
        CHECK (
            orders_count >= 0 AND
            on_time_deliveries >= 0 AND
            on_time_deliveries <= orders_count AND
            quality_issues >= 0 AND
            total_value >= 0
        )
);

-- Addresses
CREATE TABLE addresses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type VARCHAR(50) NOT NULL,
    entity_id UUID NOT NULL,
    address_type address_type NOT NULL,
    street_line_1 VARCHAR(255) NOT NULL,
    street_line_2 VARCHAR(255),
    city VARCHAR(100) NOT NULL,
    state_province VARCHAR(100),
    postal_code VARCHAR(20) NOT NULL,
    country_code CHAR(3) NOT NULL,
    latitude DECIMAL(10,8),
    longitude DECIMAL(11,8),
    formatted_address TEXT,
    validation_status VARCHAR(20) DEFAULT 'pending',
    validation_date TIMESTAMPTZ,
    is_primary BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    delivery_instructions TEXT,
    access_code VARCHAR(50),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL,
    updated_by UUID NOT NULL,
    CONSTRAINT check_entity_type
        CHECK (entity_type IN ('customer', 'supplier', 'location', 'warehouse')),
    CONSTRAINT check_coordinates
        CHECK (
            (latitude IS NULL AND longitude IS NULL) OR
            (latitude IS NOT NULL AND longitude IS NOT NULL AND
             latitude >= -90 AND latitude <= 90 AND
             longitude >= -180 AND longitude <= 180)
        )
);

-- Contact Information
CREATE TABLE contact_info (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type VARCHAR(50) NOT NULL,
    entity_id UUID NOT NULL,
    contact_type contact_type NOT NULL,
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    email VARCHAR(255),
    phone VARCHAR(50),
    mobile VARCHAR(50),
    title VARCHAR(100),
    department VARCHAR(100),
    is_primary BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    website VARCHAR(500),
    social_media_accounts JSONB,
    timezone VARCHAR(50),
    preferred_contact_method VARCHAR(20) DEFAULT 'email',
    notes TEXT,
    tags TEXT[],
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL,
    updated_by UUID NOT NULL,
    CONSTRAINT check_entity_type_contact
        CHECK (entity_type IN ('customer', 'supplier', 'location', 'warehouse')),
    CONSTRAINT check_email_format
        CHECK (email IS NULL OR email ~ '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$'),
    CONSTRAINT check_contact_method
        CHECK (preferred_contact_method IN ('email', 'phone', 'mobile'))
);

-- Address History
CREATE TABLE address_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    address_id UUID NOT NULL,
    old_address JSONB NOT NULL,
    change_reason TEXT,
    changed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    changed_by UUID NOT NULL,
    CONSTRAINT fk_address_history_address
        FOREIGN KEY (address_id) REFERENCES addresses(id) ON DELETE CASCADE
);

-- Locations (Warehouses/Stores)
CREATE TABLE locations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    location_type VARCHAR(50) NOT NULL DEFAULT 'warehouse',
    code VARCHAR(50) UNIQUE,
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    is_default BOOLEAN NOT NULL DEFAULT false,
    parent_location_id UUID,
    address_id UUID,
    capacity JSONB,
    operating_hours JSONB,
    contact_info JSONB,
    gps_coordinates POINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL,
    updated_by UUID NOT NULL,
    tenant_id UUID,
    CONSTRAINT fk_locations_parent
        FOREIGN KEY (parent_location_id) REFERENCES locations(id),
    CONSTRAINT fk_locations_address
        FOREIGN KEY (address_id) REFERENCES addresses(id),
    CONSTRAINT fk_locations_tenant
        FOREIGN KEY (tenant_id) REFERENCES tenants(id),
    CONSTRAINT check_location_type
        CHECK (location_type IN ('warehouse', 'store', 'distribution_center', 'manufacturing', 'office')),
    CONSTRAINT check_only_one_default
        UNIQUE (tenant_id, is_default) DEFERRABLE INITIALLY DEFERRED
);

-- Users
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    role_id UUID,
    is_active BOOLEAN NOT NULL DEFAULT true,
    email_verified BOOLEAN NOT NULL DEFAULT false,
    last_login_at TIMESTAMPTZ,
    password_changed_at TIMESTAMPTZ DEFAULT NOW(),
    failed_login_attempts INTEGER DEFAULT 0,
    locked_until TIMESTAMPTZ,
    preferences JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID,
    updated_by UUID,
    CONSTRAINT fk_users_tenant
        FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
    CONSTRAINT check_email_format
        CHECK (email ~ '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$')
);

-- Roles
CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    permissions JSONB,
    is_system_role BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID,
    updated_by UUID
);

-- User Permissions
CREATE TABLE user_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    permission VARCHAR(255) NOT NULL,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    granted_by UUID NOT NULL,
    CONSTRAINT fk_user_permissions_user
        FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT fk_user_permissions_granted_by
        FOREIGN KEY (granted_by) REFERENCES users(id),
    UNIQUE (user_id, permission)
);

\echo '✓ Core tables layer completed'