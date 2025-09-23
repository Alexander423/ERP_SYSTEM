-- =====================================================
-- 003_INVENTORY: Inventory Management System
-- =====================================================

\echo '[3/5] Applying inventory system layer...'

-- Location Items
CREATE TABLE location_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL,
    location_id UUID NOT NULL,
    location_name VARCHAR(255) NOT NULL,
    location_type VARCHAR(50) NOT NULL DEFAULT 'warehouse',
    quantity_available INTEGER NOT NULL DEFAULT 0,
    quantity_reserved INTEGER NOT NULL DEFAULT 0,
    quantity_on_order INTEGER NOT NULL DEFAULT 0,
    quantity_in_transit INTEGER NOT NULL DEFAULT 0,
    reorder_point INTEGER NOT NULL DEFAULT 0,
    max_stock_level INTEGER NOT NULL DEFAULT 0,
    min_stock_level INTEGER NOT NULL DEFAULT 0,
    safety_stock INTEGER NOT NULL DEFAULT 0,
    economic_order_quantity INTEGER NOT NULL DEFAULT 0,
    turnover_rate DECIMAL(8,2) NOT NULL DEFAULT 0.0,
    lead_time_days INTEGER NOT NULL DEFAULT 0,
    storage_cost_per_unit DECIMAL(10,4) NOT NULL DEFAULT 0,
    handling_cost_per_unit DECIMAL(10,4) NOT NULL DEFAULT 0,
    last_counted_at TIMESTAMPTZ,
    cycle_count_frequency_days INTEGER DEFAULT 30,
    abc_classification abc_classification NOT NULL DEFAULT 'A',
    movement_velocity movement_velocity NOT NULL DEFAULT 'medium',
    seasonal_factors JSONB,
    storage_requirements JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_location_items_product
        FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE,
    CONSTRAINT unique_product_location
        UNIQUE (product_id, location_id),
    CONSTRAINT check_quantities_non_negative
        CHECK (
            quantity_available >= 0 AND
            quantity_reserved >= 0 AND
            quantity_on_order >= 0 AND
            quantity_in_transit >= 0
        ),
    CONSTRAINT check_stock_levels_non_negative
        CHECK (
            reorder_point >= 0 AND
            max_stock_level >= 0 AND
            min_stock_level >= 0 AND
            safety_stock >= 0 AND
            economic_order_quantity >= 0
        ),
    CONSTRAINT check_costs_non_negative
        CHECK (
            storage_cost_per_unit >= 0 AND
            handling_cost_per_unit >= 0
        ),
    CONSTRAINT check_logical_stock_levels
        CHECK (min_stock_level <= max_stock_level)
);

-- Stock Reservations
CREATE TABLE stock_reservations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    location_item_id UUID NOT NULL,
    reserved_quantity INTEGER NOT NULL,
    reservation_type VARCHAR(50) NOT NULL,
    reference_id UUID,
    reference_number VARCHAR(100),
    reserved_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    released_at TIMESTAMPTZ,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL,
    CONSTRAINT fk_stock_reservations_location_item
        FOREIGN KEY (location_item_id) REFERENCES location_items(id) ON DELETE CASCADE,
    CONSTRAINT check_positive_quantity
        CHECK (reserved_quantity > 0),
    CONSTRAINT check_status_values
        CHECK (status IN ('active', 'fulfilled', 'cancelled', 'expired'))
);

-- Cycle Count Schedules
CREATE TABLE cycle_count_schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    location_item_id UUID NOT NULL,
    scheduled_date DATE NOT NULL,
    priority INTEGER NOT NULL DEFAULT 1,
    count_type VARCHAR(20) NOT NULL DEFAULT 'regular',
    assigned_to UUID,
    assigned_at TIMESTAMPTZ,
    status VARCHAR(20) NOT NULL DEFAULT 'scheduled',
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    expected_quantity INTEGER,
    actual_quantity INTEGER,
    variance INTEGER,
    variance_percentage DECIMAL(5,2),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL,
    CONSTRAINT fk_cycle_count_schedules_location_item
        FOREIGN KEY (location_item_id) REFERENCES location_items(id) ON DELETE CASCADE,
    CONSTRAINT check_priority_range
        CHECK (priority >= 1 AND priority <= 3),
    CONSTRAINT check_count_status
        CHECK (status IN ('scheduled', 'in_progress', 'completed', 'cancelled')),
    CONSTRAINT check_count_type
        CHECK (count_type IN ('regular', 'spot', 'full'))
);

-- Location Capacity
CREATE TABLE location_capacity (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    location_id UUID NOT NULL,
    total_volume DECIMAL(12,3),
    available_volume DECIMAL(12,3),
    total_weight DECIMAL(12,3),
    available_weight DECIMAL(12,3),
    total_positions INTEGER,
    available_positions INTEGER,
    capacity_type VARCHAR(50),
    temperature_min DECIMAL(5,2),
    temperature_max DECIMAL(5,2),
    humidity_min DECIMAL(5,2),
    humidity_max DECIMAL(5,2),
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL,
    updated_by UUID NOT NULL,
    CONSTRAINT unique_location_capacity_type
        UNIQUE (location_id, capacity_type),
    CONSTRAINT check_capacity_values
        CHECK (
            (total_volume IS NULL OR total_volume >= 0) AND
            (available_volume IS NULL OR (available_volume >= 0 AND available_volume <= total_volume)) AND
            (total_weight IS NULL OR total_weight >= 0) AND
            (available_weight IS NULL OR (available_weight >= 0 AND available_weight <= total_weight)) AND
            (total_positions IS NULL OR total_positions >= 0) AND
            (available_positions IS NULL OR (available_positions >= 0 AND available_positions <= total_positions))
        )
);

-- Inventory Transactions
CREATE TABLE inventory_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    transaction_number VARCHAR(50) NOT NULL,
    transaction_type movement_type NOT NULL,
    transaction_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    product_id UUID NOT NULL,
    location_id UUID NOT NULL,
    quantity_change INTEGER NOT NULL,
    unit_cost DECIMAL(15,4),
    total_cost DECIMAL(15,2),
    reference_type VARCHAR(50),
    reference_id UUID,
    reference_number VARCHAR(100),
    batch_number VARCHAR(100),
    lot_number VARCHAR(100),
    expiry_date DATE,
    reason_code VARCHAR(50),
    reference_document VARCHAR(255),
    notes TEXT,
    status VARCHAR(20) NOT NULL DEFAULT 'completed',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL,
    CONSTRAINT fk_inventory_transactions_product
        FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE RESTRICT,
    CONSTRAINT check_quantity_not_zero
        CHECK (quantity_change != 0),
    CONSTRAINT check_positive_costs
        CHECK (
            (unit_cost IS NULL OR unit_cost >= 0) AND
            (total_cost IS NULL OR total_cost >= 0)
        ),
    CONSTRAINT check_transaction_status
        CHECK (status IN ('pending', 'completed', 'cancelled'))
);

-- Inventory Adjustments
CREATE TABLE inventory_adjustments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    adjustment_number VARCHAR(50) NOT NULL,
    adjustment_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    adjustment_type adjustment_type NOT NULL,
    product_id UUID NOT NULL,
    location_id UUID NOT NULL,
    expected_quantity INTEGER NOT NULL,
    actual_quantity INTEGER NOT NULL,
    adjustment_quantity INTEGER NOT NULL,
    unit_cost DECIMAL(15,4),
    cost_impact DECIMAL(15,2),
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    rejection_reason TEXT,
    reason_code VARCHAR(50) NOT NULL,
    reason_description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL,
    CONSTRAINT fk_inventory_adjustments_product
        FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE RESTRICT,
    CONSTRAINT check_adjustment_status
        CHECK (status IN ('pending', 'approved', 'rejected')),
    CONSTRAINT check_consistent_adjustment
        CHECK (adjustment_quantity = actual_quantity - expected_quantity)
);

-- Inventory Transfers
CREATE TABLE inventory_transfers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    transfer_number VARCHAR(50) NOT NULL,
    transfer_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    product_id UUID NOT NULL,
    from_location_id UUID NOT NULL,
    to_location_id UUID NOT NULL,
    quantity_requested INTEGER NOT NULL,
    quantity_shipped INTEGER,
    quantity_received INTEGER,
    unit_cost DECIMAL(15,4),
    transfer_cost DECIMAL(15,2),
    status transfer_status NOT NULL DEFAULT 'requested',
    requested_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    shipped_date TIMESTAMPTZ,
    received_date TIMESTAMPTZ,
    requested_by UUID NOT NULL,
    shipped_by UUID,
    received_by UUID,
    priority INTEGER DEFAULT 2,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_inventory_transfers_product
        FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE RESTRICT,
    CONSTRAINT check_different_locations
        CHECK (from_location_id != to_location_id),
    CONSTRAINT check_positive_quantities
        CHECK (
            quantity_requested > 0 AND
            (quantity_shipped IS NULL OR quantity_shipped >= 0) AND
            (quantity_received IS NULL OR quantity_received >= 0)
        ),
    CONSTRAINT check_transfer_priority
        CHECK (priority >= 1 AND priority <= 3)
);

-- Stock Alerts
CREATE TABLE stock_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alert_type alert_type NOT NULL,
    severity alert_severity NOT NULL DEFAULT 'medium',
    product_id UUID NOT NULL,
    location_id UUID,
    current_stock INTEGER,
    threshold_value INTEGER,
    message TEXT NOT NULL,
    triggered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    acknowledged_at TIMESTAMPTZ,
    resolved_at TIMESTAMPTZ,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    acknowledged_by UUID,
    resolved_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_stock_alerts_product
        FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE,
    CONSTRAINT check_alert_status
        CHECK (status IN ('active', 'acknowledged', 'resolved', 'dismissed'))
);

-- Inventory Snapshots
CREATE TABLE inventory_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    snapshot_date DATE NOT NULL,
    snapshot_type VARCHAR(20) NOT NULL DEFAULT 'daily',
    product_id UUID NOT NULL,
    location_id UUID NOT NULL,
    quantity_available INTEGER NOT NULL DEFAULT 0,
    quantity_reserved INTEGER NOT NULL DEFAULT 0,
    quantity_on_order INTEGER NOT NULL DEFAULT 0,
    quantity_in_transit INTEGER NOT NULL DEFAULT 0,
    unit_cost DECIMAL(15,4),
    total_value DECIMAL(15,2),
    turns_ytd DECIMAL(10,4),
    days_on_hand INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_inventory_snapshots_product
        FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE,
    CONSTRAINT unique_snapshot_product_location_date
        UNIQUE (snapshot_date, product_id, location_id),
    CONSTRAINT check_snapshot_quantities
        CHECK (
            quantity_available >= 0 AND
            quantity_reserved >= 0 AND
            quantity_on_order >= 0 AND
            quantity_in_transit >= 0
        )
);

\echo '✓ Inventory system layer completed'