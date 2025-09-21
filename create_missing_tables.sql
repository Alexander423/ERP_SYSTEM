-- Create all missing tables based on the codebase structure

-- Inventory tables
CREATE TABLE IF NOT EXISTS inventory_locations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL,
    location_id UUID NOT NULL,
    location_name VARCHAR(255) NOT NULL,
    location_type location_type NOT NULL DEFAULT 'warehouse',
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
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS inventory_movements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL,
    location_id UUID NOT NULL,
    movement_type VARCHAR(50) NOT NULL,
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
    product_id UUID NOT NULL,
    location_id UUID NOT NULL,
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
    product_id UUID NOT NULL,
    location_id UUID NOT NULL,
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

CREATE TABLE IF NOT EXISTS stock_transfers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL,
    from_location_id UUID NOT NULL,
    to_location_id UUID NOT NULL,
    quantity INTEGER NOT NULL,
    unit_cost DECIMAL(12,4),
    total_cost DECIMAL(12,2),
    transfer_status VARCHAR(50) NOT NULL DEFAULT 'pending',
    requested_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    shipped_date TIMESTAMPTZ,
    received_date TIMESTAMPTZ,
    requested_by UUID,
    approved_by UUID,
    tracking_number VARCHAR(100),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
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

-- Product analytics tables
CREATE TABLE IF NOT EXISTS product_performance_analytics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL,
    analysis_period_start TIMESTAMPTZ NOT NULL,
    analysis_period_end TIMESTAMPTZ NOT NULL,
    total_sales_volume INTEGER NOT NULL DEFAULT 0,
    total_revenue DECIMAL(15,2) NOT NULL DEFAULT 0,
    average_selling_price DECIMAL(12,4) NOT NULL DEFAULT 0,
    profit_margin DECIMAL(5,4) NOT NULL DEFAULT 0,
    inventory_turnover DECIMAL(8,4) NOT NULL DEFAULT 0,
    days_inventory_outstanding DECIMAL(8,2) NOT NULL DEFAULT 0,
    stockout_events INTEGER NOT NULL DEFAULT 0,
    backorder_count INTEGER NOT NULL DEFAULT 0,
    customer_satisfaction DECIMAL(3,2) NOT NULL DEFAULT 0,
    return_rate DECIMAL(5,4) NOT NULL DEFAULT 0,
    quality_score DECIMAL(3,2) NOT NULL DEFAULT 0,
    trend_classification VARCHAR(50),
    seasonality_pattern JSONB,
    demand_variability DECIMAL(8,4) NOT NULL DEFAULT 0,
    forecast_accuracy DECIMAL(5,4) NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create essential indexes
CREATE INDEX IF NOT EXISTS idx_inventory_locations_product_location ON inventory_locations(product_id, location_id);
CREATE INDEX IF NOT EXISTS idx_inventory_movements_product_date ON inventory_movements(product_id, movement_date);
CREATE INDEX IF NOT EXISTS idx_inventory_alerts_severity_status ON inventory_alerts(severity, alert_status);
CREATE INDEX IF NOT EXISTS idx_inventory_forecasts_product_date ON inventory_forecasts(product_id, forecast_date);
CREATE INDEX IF NOT EXISTS idx_products_tenant_active ON products(tenant_id, is_active);
CREATE INDEX IF NOT EXISTS idx_customers_tenant_active ON customers(tenant_id, is_active);
CREATE INDEX IF NOT EXISTS idx_suppliers_tenant_active ON suppliers(tenant_id, is_active);