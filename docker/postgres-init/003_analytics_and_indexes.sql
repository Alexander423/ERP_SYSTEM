-- =====================================================
-- ANALYTICS AND INDEXES
-- =====================================================

\echo '[4/5] Applying analytics and indexes layer...'

-- Inventory Analytics Tables
CREATE TABLE inventory_turnover_analysis (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    analysis_date DATE NOT NULL,
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    product_id UUID NOT NULL,
    location_id UUID NOT NULL,
    average_inventory DECIMAL(15,2) NOT NULL,
    cost_of_goods_sold DECIMAL(15,2) NOT NULL,
    inventory_turns DECIMAL(10,4) NOT NULL,
    days_inventory_outstanding INTEGER NOT NULL,
    turnover_classification turnover_classification NOT NULL,
    performance_trend VARCHAR(20),
    recommended_stock_level INTEGER,
    recommended_reorder_point INTEGER,
    optimization_notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL,
    CONSTRAINT fk_inventory_turnover_analysis_product
        FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE,
    CONSTRAINT unique_analysis_product_location_date
        UNIQUE (analysis_date, product_id, location_id),
    CONSTRAINT check_positive_values
        CHECK (
            average_inventory >= 0 AND
            cost_of_goods_sold >= 0 AND
            inventory_turns >= 0 AND
            days_inventory_outstanding >= 0
        )
);

CREATE TABLE demand_forecasts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    forecast_date DATE NOT NULL,
    forecast_period_start DATE NOT NULL,
    forecast_period_end DATE NOT NULL,
    forecast_model VARCHAR(50) NOT NULL,
    product_id UUID NOT NULL,
    location_id UUID NOT NULL,
    historical_demand INTEGER NOT NULL,
    forecasted_demand INTEGER NOT NULL,
    forecast_accuracy DECIMAL(5,2),
    confidence_level DECIMAL(5,2) NOT NULL,
    seasonal_index DECIMAL(6,4) DEFAULT 1.0000,
    trend_factor DECIMAL(8,6) DEFAULT 1.000000,
    standard_deviation DECIMAL(10,4),
    mean_absolute_deviation DECIMAL(10,4),
    recommended_safety_stock INTEGER,
    service_level_target DECIMAL(5,2) DEFAULT 95.00,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL,
    CONSTRAINT fk_demand_forecasts_product
        FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE,
    CONSTRAINT check_forecast_values
        CHECK (
            historical_demand >= 0 AND
            forecasted_demand >= 0 AND
            (forecast_accuracy IS NULL OR (forecast_accuracy >= 0 AND forecast_accuracy <= 100)) AND
            confidence_level >= 0 AND confidence_level <= 100 AND
            service_level_target >= 0 AND service_level_target <= 100
        )
);

CREATE TABLE abc_analysis_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    analysis_date DATE NOT NULL,
    analysis_period_start DATE NOT NULL,
    analysis_period_end DATE NOT NULL,
    product_id UUID NOT NULL,
    annual_usage_value DECIMAL(15,2) NOT NULL,
    annual_usage_quantity INTEGER NOT NULL,
    cumulative_usage_value DECIMAL(15,2) NOT NULL,
    cumulative_percentage DECIMAL(5,2) NOT NULL,
    abc_class abc_classification NOT NULL,
    xyz_class xyz_classification,
    velocity_class movement_velocity NOT NULL,
    recommended_count_frequency INTEGER,
    recommended_safety_stock_days INTEGER,
    recommended_review_frequency INTEGER,
    control_priority INTEGER NOT NULL,
    management_attention_level VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL,
    CONSTRAINT fk_abc_analysis_results_product
        FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE,
    CONSTRAINT unique_analysis_product_date
        UNIQUE (analysis_date, product_id),
    CONSTRAINT check_positive_metrics
        CHECK (
            annual_usage_value >= 0 AND
            annual_usage_quantity >= 0 AND
            cumulative_usage_value >= 0 AND
            cumulative_percentage >= 0 AND cumulative_percentage <= 100
        ),
    CONSTRAINT check_control_priority
        CHECK (control_priority >= 1 AND control_priority <= 5),
    CONSTRAINT check_management_attention
        CHECK (management_attention_level IN ('high', 'medium', 'low'))
);

-- Performance Indexes
CREATE INDEX CONCURRENTLY idx_products_sku_tenant ON products(tenant_id, sku);
CREATE INDEX CONCURRENTLY idx_products_name_search ON products USING gin(to_tsvector('english', name));
CREATE INDEX CONCURRENTLY idx_products_category_status ON products(category_id, status) WHERE status = 'active';
CREATE INDEX CONCURRENTLY idx_products_price_range ON products(base_price) WHERE base_price > 0;

CREATE INDEX CONCURRENTLY idx_customers_tenant_number ON customers(tenant_id, customer_number);
CREATE INDEX CONCURRENTLY idx_customers_type_status ON customers(customer_type, status);
CREATE INDEX CONCURRENTLY idx_customers_last_order ON customers(last_order_date DESC) WHERE last_order_date IS NOT NULL;

CREATE INDEX CONCURRENTLY idx_suppliers_tenant_number ON suppliers(tenant_id, supplier_number);
CREATE INDEX CONCURRENTLY idx_suppliers_status_rating ON suppliers(status, overall_rating DESC) WHERE status = 'active';

CREATE INDEX CONCURRENTLY idx_addresses_entity_primary ON addresses(entity_type, entity_id, is_primary) WHERE is_primary = true;
CREATE INDEX CONCURRENTLY idx_addresses_country_state ON addresses(country_code, state_province);
CREATE INDEX CONCURRENTLY idx_addresses_postal_code ON addresses(postal_code);

CREATE INDEX CONCURRENTLY idx_contact_info_entity_primary ON contact_info(entity_type, entity_id, is_primary) WHERE is_primary = true;
CREATE INDEX CONCURRENTLY idx_contact_info_email_active ON contact_info(email) WHERE email IS NOT NULL AND is_active = true;

CREATE INDEX CONCURRENTLY idx_location_items_product_location ON location_items(product_id, location_id);
CREATE INDEX CONCURRENTLY idx_location_items_reorder_needed ON location_items(product_id) WHERE quantity_available <= reorder_point;
CREATE INDEX CONCURRENTLY idx_location_items_abc_velocity ON location_items(abc_classification, movement_velocity);

CREATE INDEX CONCURRENTLY idx_inventory_transactions_product_date ON inventory_transactions(product_id, transaction_date DESC);
CREATE INDEX CONCURRENTLY idx_inventory_transactions_location_date ON inventory_transactions(location_id, transaction_date DESC);
CREATE INDEX CONCURRENTLY idx_inventory_transactions_type_date ON inventory_transactions(transaction_type, transaction_date DESC);

CREATE INDEX CONCURRENTLY idx_stock_alerts_active ON stock_alerts(product_id, status) WHERE status = 'active';
CREATE INDEX CONCURRENTLY idx_stock_alerts_severity_date ON stock_alerts(severity, triggered_at DESC);

-- Full-text search indexes
CREATE INDEX CONCURRENTLY idx_products_fulltext_search ON products USING gin(
    to_tsvector('english',
        COALESCE(name, '') || ' ' ||
        COALESCE(description, '') || ' ' ||
        COALESCE(sku, '') || ' ' ||
        COALESCE(brand, '') || ' ' ||
        COALESCE(manufacturer, '')
    )
);

CREATE INDEX CONCURRENTLY idx_customers_fulltext_search ON customers USING gin(
    to_tsvector('english',
        COALESCE(legal_name, '') || ' ' ||
        COALESCE(customer_number, '')
    )
);

CREATE INDEX CONCURRENTLY idx_suppliers_fulltext_search ON suppliers USING gin(
    to_tsvector('english',
        COALESCE(legal_name, '') || ' ' ||
        COALESCE(trade_name, '') || ' ' ||
        COALESCE(supplier_number, '')
    )
);

-- Triggers
CREATE TRIGGER update_product_categories_updated_at
    BEFORE UPDATE ON product_categories
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_products_updated_at
    BEFORE UPDATE ON products
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_product_variants_updated_at
    BEFORE UPDATE ON product_variants
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_customers_updated_at
    BEFORE UPDATE ON customers
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_customer_groups_updated_at
    BEFORE UPDATE ON customer_groups
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_suppliers_updated_at
    BEFORE UPDATE ON suppliers
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_supplier_products_updated_at
    BEFORE UPDATE ON supplier_products
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_addresses_updated_at
    BEFORE UPDATE ON addresses
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_contact_info_updated_at
    BEFORE UPDATE ON contact_info
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_location_items_updated_at
    BEFORE UPDATE ON location_items
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_location_capacity_updated_at
    BEFORE UPDATE ON location_capacity
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_inventory_transfers_updated_at
    BEFORE UPDATE ON inventory_transfers
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Address history trigger
CREATE OR REPLACE FUNCTION record_address_change()
RETURNS TRIGGER AS $$
BEGIN
    IF OLD IS DISTINCT FROM NEW THEN
        INSERT INTO address_history (address_id, old_address, changed_by)
        VALUES (
            OLD.id,
            row_to_json(OLD),
            NEW.updated_by
        );
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER record_address_changes
    AFTER UPDATE ON addresses
    FOR EACH ROW EXECUTE FUNCTION record_address_change();

-- Sales Transactions (Analytics requirement)
CREATE TABLE sales_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    transaction_id VARCHAR(50) UNIQUE NOT NULL,
    product_id UUID NOT NULL REFERENCES products(id),
    customer_id UUID REFERENCES customers(id),
    location_id UUID REFERENCES locations(id),
    quantity DECIMAL(10,3) NOT NULL,
    unit_price DECIMAL(10,2) NOT NULL,
    total_amount DECIMAL(12,2) NOT NULL,
    discount_amount DECIMAL(10,2) DEFAULT 0,
    tax_amount DECIMAL(10,2) DEFAULT 0,
    transaction_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status sales_status DEFAULT 'completed',
    payment_method VARCHAR(50),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id),
    updated_by UUID NOT NULL REFERENCES users(id),
    tenant_id UUID REFERENCES tenants(id),
    CONSTRAINT check_sales_positive_amounts
        CHECK (quantity > 0 AND unit_price >= 0 AND total_amount >= 0)
);

-- Customer Feedback (Analytics requirement)
CREATE TABLE customer_feedback (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id UUID NOT NULL REFERENCES customers(id),
    product_id UUID REFERENCES products(id),
    order_id UUID,
    transaction_id VARCHAR(50),
    rating INTEGER CHECK (rating >= 1 AND rating <= 5),
    feedback_type VARCHAR(50) NOT NULL DEFAULT 'general',
    subject VARCHAR(255),
    message TEXT,
    sentiment VARCHAR(20) DEFAULT 'neutral',
    is_verified BOOLEAN DEFAULT false,
    response TEXT,
    responded_at TIMESTAMPTZ,
    responded_by UUID REFERENCES users(id),
    status VARCHAR(20) DEFAULT 'open',
    priority VARCHAR(10) DEFAULT 'medium',
    customer_satisfaction DECIMAL(3,2) CHECK (customer_satisfaction >= 0 AND customer_satisfaction <= 5),
    return_reason TEXT,
    tags TEXT[],
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id),
    updated_by UUID NOT NULL REFERENCES users(id),
    tenant_id UUID REFERENCES tenants(id),
    CONSTRAINT check_feedback_type
        CHECK (feedback_type IN ('general', 'product', 'service', 'complaint', 'suggestion')),
    CONSTRAINT check_sentiment
        CHECK (sentiment IN ('positive', 'neutral', 'negative')),
    CONSTRAINT check_status
        CHECK (status IN ('open', 'in_progress', 'resolved', 'closed')),
    CONSTRAINT check_priority
        CHECK (priority IN ('low', 'medium', 'high', 'urgent'))
);

-- Inventory Analytics (Product Performance requirement)
CREATE TABLE inventory_analytics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL REFERENCES products(id),
    location_id UUID REFERENCES locations(id),
    analysis_date DATE NOT NULL DEFAULT CURRENT_DATE,
    total_sold DECIMAL(10,3) DEFAULT 0,
    revenue DECIMAL(12,2) DEFAULT 0,
    profit_margin DECIMAL(5,2) DEFAULT 0,
    return_rate DECIMAL(5,2) DEFAULT 0,
    inventory_turns DECIMAL(8,2) DEFAULT 0,
    turnover_rate DECIMAL(8,2) DEFAULT 0,
    days_on_hand INTEGER DEFAULT 0,
    stockout_days INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID REFERENCES users(id),
    updated_by UUID REFERENCES users(id),
    tenant_id UUID REFERENCES tenants(id),
    UNIQUE (product_id, location_id, analysis_date)
);

-- Compatibility Views for Rust Code
-- inventory_movements as alias for inventory_transactions
CREATE OR REPLACE VIEW inventory_movements AS
SELECT
    id,
    product_id,
    location_id,
    transaction_type as movement_type,
    quantity_change as quantity,
    unit_cost,
    total_cost,
    notes,
    reason_code as reason,
    reference_number,
    batch_number,
    lot_number,
    created_by as operator_id,
    'System' as operator_name,
    transaction_date as effective_date,
    transaction_date as movement_date,
    ARRAY[]::TEXT[] as serial_numbers,
    expiry_date,
    '{}' as audit_trail,
    transaction_date,
    reference_document,
    created_at,
    created_at as updated_at,  -- Compatibility mapping
    created_by,
    created_by as updated_by,  -- Compatibility mapping
    (SELECT tenant_id FROM products WHERE id = inventory_transactions.product_id LIMIT 1) as tenant_id
FROM inventory_transactions;

-- location_inventory view with enhanced data
CREATE OR REPLACE VIEW location_inventory AS
SELECT
    li.id,
    li.product_id,
    li.location_id,
    li.location_name,
    li.location_type,
    li.quantity_available as quantity_on_hand,
    li.quantity_available as current_stock,
    li.quantity_reserved,
    li.quantity_on_order,
    li.quantity_in_transit,
    li.quantity_available - li.quantity_reserved as quantity_available,
    li.reorder_point,
    li.min_stock_level,
    li.max_stock_level,
    li.safety_stock,
    li.economic_order_quantity,
    li.abc_classification,
    li.movement_velocity,
    li.last_counted_at as last_count_date,
    li.cycle_count_frequency_days,
    li.created_at,
    li.updated_at,
    NULL::UUID as created_by,
    NULL::UUID as updated_by,
    (SELECT tenant_id FROM products WHERE id = li.product_id LIMIT 1) as tenant_id
FROM location_items li;

-- Performance indexes for new tables and views
CREATE INDEX IF NOT EXISTS idx_sales_transactions_product_date ON sales_transactions(product_id, transaction_date);
CREATE INDEX IF NOT EXISTS idx_sales_transactions_customer ON sales_transactions(customer_id);
CREATE INDEX IF NOT EXISTS idx_sales_transactions_status ON sales_transactions(status);
CREATE INDEX IF NOT EXISTS idx_inventory_transactions_reference_doc ON inventory_transactions(reference_document);

\echo '✓ Analytics and indexes layer completed'