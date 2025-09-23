-- Reference data for tenant setup
-- Categories, units, currencies, etc.

-- Default product categories
INSERT INTO product_categories (id, name, description, parent_id, is_active, created_at, updated_at) VALUES
    (gen_random_uuid(), 'Electronics', 'Electronic products and components', NULL, true, NOW(), NOW()),
    (gen_random_uuid(), 'Software', 'Software products and licenses', NULL, true, NOW(), NOW()),
    (gen_random_uuid(), 'Office Supplies', 'Office equipment and supplies', NULL, true, NOW(), NOW()),
    (gen_random_uuid(), 'Industrial', 'Industrial equipment and materials', NULL, true, NOW(), NOW()),
    (gen_random_uuid(), 'Services', 'Service-based products', NULL, true, NOW(), NOW());

-- Default units of measure
INSERT INTO units_of_measure (id, name, symbol, type, base_unit, conversion_factor, is_active, created_at, updated_at) VALUES
    (gen_random_uuid(), 'Piece', 'pcs', 'quantity', NULL, 1.0, true, NOW(), NOW()),
    (gen_random_uuid(), 'Kilogram', 'kg', 'weight', NULL, 1.0, true, NOW(), NOW()),
    (gen_random_uuid(), 'Gram', 'g', 'weight', 'kg', 0.001, true, NOW(), NOW()),
    (gen_random_uuid(), 'Meter', 'm', 'length', NULL, 1.0, true, NOW(), NOW()),
    (gen_random_uuid(), 'Centimeter', 'cm', 'length', 'm', 0.01, true, NOW(), NOW()),
    (gen_random_uuid(), 'Liter', 'l', 'volume', NULL, 1.0, true, NOW(), NOW()),
    (gen_random_uuid(), 'Milliliter', 'ml', 'volume', 'l', 0.001, true, NOW(), NOW());

-- Default currencies
INSERT INTO currencies (id, code, name, symbol, exchange_rate, is_base, is_active, created_at, updated_at) VALUES
    (gen_random_uuid(), 'EUR', 'Euro', '€', 1.0, true, true, NOW(), NOW()),
    (gen_random_uuid(), 'USD', 'US Dollar', '$', 1.1, false, true, NOW(), NOW()),
    (gen_random_uuid(), 'GBP', 'British Pound', '£', 0.86, false, true, NOW(), NOW());

-- Default customer groups
INSERT INTO customer_groups (id, name, description, discount_percentage, pricing_tier, is_active, created_at, updated_at) VALUES
    (gen_random_uuid(), 'Standard', 'Standard customers', 0.0, 'standard', true, NOW(), NOW()),
    (gen_random_uuid(), 'Premium', 'Premium customers with discounts', 5.0, 'premium', true, NOW(), NOW()),
    (gen_random_uuid(), 'VIP', 'VIP customers with special pricing', 10.0, 'vip', true, NOW(), NOW()),
    (gen_random_uuid(), 'Wholesale', 'Wholesale customers', 15.0, 'wholesale', true, NOW(), NOW());

-- Default location setup
INSERT INTO locations (id, name, type, address, capacity, is_active, created_at, updated_at) VALUES
    (gen_random_uuid(), 'Main Warehouse', 'warehouse', 'Main facility', '{"volume": 10000, "weight": 50000}', true, NOW(), NOW()),
    (gen_random_uuid(), 'Retail Store', 'store', 'Customer-facing location', '{"volume": 500, "weight": 2000}', true, NOW(), NOW());