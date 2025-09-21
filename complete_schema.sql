-- Complete ERP database schema with all required tables

-- Create all enum types first
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

CREATE TYPE customer_type AS ENUM ('individual', 'business', 'government', 'non_profit');
CREATE TYPE supplier_type AS ENUM ('manufacturer', 'distributor', 'service_provider', 'consultant');
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

-- Core master data tables
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

CREATE TABLE IF NOT EXISTS suppliers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    supplier_number VARCHAR(50) NOT NULL,
    name VARCHAR(255) NOT NULL,
    supplier_type supplier_type NOT NULL DEFAULT 'manufacturer',
    contact_person VARCHAR(255),
    email VARCHAR(255),
    phone VARCHAR(50),
    address JSONB,
    payment_terms VARCHAR(50),
    lead_time_days INTEGER DEFAULT 0,
    quality_rating DECIMAL(3,2) DEFAULT 0.0,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(tenant_id, supplier_number)
);

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

-- Inventory specific tables
CREATE TABLE IF NOT EXISTS location_inventory (
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
    storage_requirements JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(product_id, location_id)
);

CREATE TABLE IF NOT EXISTS inventory_movements (
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

CREATE TABLE IF NOT EXISTS inventory_forecasts (
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

CREATE TABLE IF NOT EXISTS inventory_alerts (
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

CREATE TABLE IF NOT EXISTS inventory_valuations (
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

CREATE TABLE IF NOT EXISTS sales_transactions (
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

-- Create essential indexes
CREATE INDEX IF NOT EXISTS idx_products_tenant_active ON products(tenant_id, is_active);
CREATE INDEX IF NOT EXISTS idx_customers_tenant_active ON customers(tenant_id, is_active);
CREATE INDEX IF NOT EXISTS idx_suppliers_tenant_active ON suppliers(tenant_id, is_active);
CREATE INDEX IF NOT EXISTS idx_locations_tenant_active ON locations(tenant_id, is_active);
CREATE INDEX IF NOT EXISTS idx_location_inventory_product_location ON location_inventory(product_id, location_id);
CREATE INDEX IF NOT EXISTS idx_inventory_movements_product_date ON inventory_movements(product_id, movement_date);
CREATE INDEX IF NOT EXISTS idx_inventory_alerts_severity_status ON inventory_alerts(severity, alert_status);
CREATE INDEX IF NOT EXISTS idx_inventory_forecasts_product_date ON inventory_forecasts(product_id, forecast_date);
CREATE INDEX IF NOT EXISTS idx_sales_transactions_product_date ON sales_transactions(product_id, transaction_date);

-- Insert sample data for testing
INSERT INTO products (tenant_id, product_number, name, product_type)
VALUES (gen_random_uuid(), 'TEST001', 'Test Product', 'physical')
ON CONFLICT DO NOTHING;

INSERT INTO customers (tenant_id, customer_number, name, customer_type)
VALUES (gen_random_uuid(), 'CUST001', 'Test Customer', 'business')
ON CONFLICT DO NOTHING;

INSERT INTO suppliers (tenant_id, supplier_number, name, supplier_type)
VALUES (gen_random_uuid(), 'SUPP001', 'Test Supplier', 'manufacturer')
ON CONFLICT DO NOTHING;

INSERT INTO locations (tenant_id, location_code, name, location_type)
VALUES (gen_random_uuid(), 'LOC001', 'Main Warehouse', 'warehouse')
ON CONFLICT DO NOTHING;