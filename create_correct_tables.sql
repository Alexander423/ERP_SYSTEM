-- Create all tables with correct names from error messages

-- Location inventory table (referenced as location_inventory in queries)
CREATE TABLE IF NOT EXISTS location_inventory (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL,
    location_id UUID NOT NULL,
    quantity_available INTEGER NOT NULL DEFAULT 0,
    quantity_reserved INTEGER NOT NULL DEFAULT 0,
    quantity_on_order INTEGER NOT NULL DEFAULT 0,
    reorder_point INTEGER NOT NULL DEFAULT 0,
    max_stock_level INTEGER NOT NULL DEFAULT 0,
    min_stock_level INTEGER NOT NULL DEFAULT 0,
    safety_stock INTEGER NOT NULL DEFAULT 0,
    lead_time_days INTEGER NOT NULL DEFAULT 0,
    cost_per_unit DECIMAL(12,4) NOT NULL DEFAULT 0,
    last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Sales transactions table for analytics
CREATE TABLE IF NOT EXISTS sales_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL,
    customer_id UUID,
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

-- Update existing inventory_movements if needed
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'inventory_movements') THEN
        CREATE TABLE inventory_movements (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            product_id UUID NOT NULL,
            location_id UUID NOT NULL,
            movement_type VARCHAR(50) NOT NULL,
            quantity INTEGER NOT NULL,
            unit_cost DECIMAL(12,4),
            reference_id UUID,
            reference_type VARCHAR(50),
            movement_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            created_by UUID,
            notes TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
    END IF;
END $$;

-- Add missing columns to existing InventoryValuation table
DO $$
BEGIN
    -- Add missing columns if they don't exist
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'inventory_valuation' AND column_name = 'standard_cost') THEN
        ALTER TABLE inventory_valuation ADD COLUMN standard_cost DECIMAL(12,4) DEFAULT 0;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'inventory_valuation' AND column_name = 'market_value') THEN
        ALTER TABLE inventory_valuation ADD COLUMN market_value DECIMAL(12,4) DEFAULT 0;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'inventory_valuation' AND column_name = 'replacement_cost') THEN
        ALTER TABLE inventory_valuation ADD COLUMN replacement_cost DECIMAL(12,4) DEFAULT 0;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'inventory_valuation' AND column_name = 'net_realizable_value') THEN
        ALTER TABLE inventory_valuation ADD COLUMN net_realizable_value DECIMAL(12,4) DEFAULT 0;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'inventory_valuation' AND column_name = 'obsolescence_reserve') THEN
        ALTER TABLE inventory_valuation ADD COLUMN obsolescence_reserve DECIMAL(12,4) DEFAULT 0;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'inventory_valuation' AND column_name = 'shrinkage_reserve') THEN
        ALTER TABLE inventory_valuation ADD COLUMN shrinkage_reserve DECIMAL(12,4) DEFAULT 0;
    END IF;
END $$;

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_location_inventory_product_location ON location_inventory(product_id, location_id);
CREATE INDEX IF NOT EXISTS idx_sales_transactions_product_date ON sales_transactions(product_id, transaction_date);
CREATE INDEX IF NOT EXISTS idx_inventory_movements_product_location ON inventory_movements(product_id, location_id);

-- Insert some sample data for testing
INSERT INTO location_inventory (product_id, location_id, quantity_available)
VALUES (gen_random_uuid(), gen_random_uuid(), 100)
ON CONFLICT DO NOTHING;

INSERT INTO sales_transactions (product_id, transaction_date, quantity, unit_price, total_amount)
VALUES (gen_random_uuid(), NOW(), 5, 10.50, 52.50)
ON CONFLICT DO NOTHING;