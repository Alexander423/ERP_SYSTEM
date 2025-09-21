-- Drop and recreate all enum types to fix conflicts
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

-- Create all required enum types
CREATE TYPE customer_type AS ENUM (
    'individual',
    'business',
    'government',
    'non_profit'
);

CREATE TYPE supplier_type AS ENUM (
    'manufacturer',
    'distributor',
    'service_provider',
    'consultant'
);

CREATE TYPE product_type AS ENUM (
    'physical',
    'digital',
    'service',
    'subscription'
);

CREATE TYPE location_type AS ENUM (
    'warehouse',
    'store',
    'distribution_center',
    'manufacturing_plant',
    'office',
    'customer_site'
);

CREATE TYPE alert_severity AS ENUM (
    'info',
    'low',
    'medium',
    'high',
    'warning',
    'critical',
    'emergency'
);

CREATE TYPE alert_type AS ENUM (
    'low_stock',
    'stockout',
    'excess_stock',
    'slow_moving',
    'expiring',
    'expired',
    'quality_issue',
    'variance_detected',
    'supplier_delay',
    'demand_spike',
    'seasonal_alert'
);

CREATE TYPE alert_status AS ENUM (
    'new',
    'acknowledged',
    'in_progress',
    'resolved',
    'dismissed'
);

CREATE TYPE order_status AS ENUM (
    'draft',
    'pending',
    'approved',
    'sent',
    'acknowledged',
    'partially_received',
    'received',
    'invoiced',
    'paid',
    'cancelled',
    'rejected'
);

CREATE TYPE order_priority AS ENUM (
    'low',
    'normal',
    'high',
    'rush',
    'emergency'
);

CREATE TYPE line_status AS ENUM (
    'pending',
    'partially_received',
    'received',
    'cancelled',
    'rejected'
);

CREATE TYPE valuation_method AS ENUM (
    'fifo',
    'lifo',
    'weighted_average',
    'standard_cost',
    'specific_cost',
    'retail_method'
);

CREATE TYPE reservation_type AS ENUM (
    'sales_order',
    'production_order',
    'transfer',
    'quality',
    'damage',
    'special',
    'promotional'
);

CREATE TYPE reservation_priority AS ENUM (
    'low',
    'normal',
    'high',
    'critical'
);

CREATE TYPE reservation_status AS ENUM (
    'active',
    'fulfilled',
    'expired',
    'cancelled',
    'partially_fulfilled'
);

CREATE TYPE forecast_method AS ENUM (
    'moving_average',
    'exponential_smoothing',
    'linear_regression',
    'seasonal_decomposition',
    'arima',
    'machine_learning',
    'hybrid_model'
);

-- Create products table if not exists
CREATE TABLE IF NOT EXISTS products (
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

-- Create customers table if not exists
CREATE TABLE IF NOT EXISTS customers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    customer_number VARCHAR(50) NOT NULL,
    name VARCHAR(255) NOT NULL,
    customer_type customer_type NOT NULL DEFAULT 'individual',
    email VARCHAR(255),
    phone VARCHAR(50),
    address JSONB,
    credit_limit DECIMAL(12,2),
    payment_terms VARCHAR(50),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(tenant_id, customer_number)
);

-- Create locations table if not exists
CREATE TABLE IF NOT EXISTS locations (
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

-- Refresh migration tracking
DELETE FROM _sqlx_migrations WHERE version > 1;