-- =====================================================
-- SEED DATA: Demo and Development Data (Fixed UUIDs)
-- =====================================================

\echo '[5/5] Applying seed data layer...'

-- Create demo tenant
INSERT INTO tenants (id, name, slug, schema_name, subscription_tier, status, is_active, created_by, updated_by)
VALUES (
    '12345678-1234-5678-9abc-def123456789'::uuid,
    'ACME Corporation',
    'acme-corp',
    'acme_corp',
    'enterprise',
    'active',
    true,
    '87654321-4321-8765-cba9-fed987654321'::uuid,
    '87654321-4321-8765-cba9-fed987654321'::uuid
);

-- Product Categories
INSERT INTO product_categories (id, tenant_id, name, description, parent_id, level, path, created_by, updated_by) VALUES
('11111111-1111-1111-1111-111111111111'::uuid, '12345678-1234-5678-9abc-def123456789'::uuid, 'Electronics', 'Electronic products and components', NULL, 0, '/electronics', '87654321-4321-8765-cba9-fed987654321'::uuid, '87654321-4321-8765-cba9-fed987654321'::uuid),
('22222222-2222-2222-2222-222222222222'::uuid, '12345678-1234-5678-9abc-def123456789'::uuid, 'Furniture', 'Office and home furniture', NULL, 0, '/furniture', '87654321-4321-8765-cba9-fed987654321'::uuid, '87654321-4321-8765-cba9-fed987654321'::uuid),
('33333333-3333-3333-3333-333333333333'::uuid, '12345678-1234-5678-9abc-def123456789'::uuid, 'Office Supplies', 'General office supplies and materials', NULL, 0, '/office-supplies', '87654321-4321-8765-cba9-fed987654321'::uuid, '87654321-4321-8765-cba9-fed987654321'::uuid);

-- Sub-categories
INSERT INTO product_categories (id, tenant_id, name, description, parent_id, level, path, created_by, updated_by) VALUES
('11111111-1111-1111-2222-111111111111'::uuid, '12345678-1234-5678-9abc-def123456789'::uuid, 'Computers', 'Desktop and laptop computers', '11111111-1111-1111-1111-111111111111'::uuid, 1, '/electronics/computers', '87654321-4321-8765-cba9-fed987654321'::uuid, '87654321-4321-8765-cba9-fed987654321'::uuid),
('11111111-1111-1111-3333-111111111111'::uuid, '12345678-1234-5678-9abc-def123456789'::uuid, 'Monitors', 'Computer monitors and displays', '11111111-1111-1111-1111-111111111111'::uuid, 1, '/electronics/monitors', '87654321-4321-8765-cba9-fed987654321'::uuid, '87654321-4321-8765-cba9-fed987654321'::uuid),
('22222222-2222-2222-3333-222222222222'::uuid, '12345678-1234-5678-9abc-def123456789'::uuid, 'Desks', 'Office desks and workstations', '22222222-2222-2222-2222-222222222222'::uuid, 1, '/furniture/desks', '87654321-4321-8765-cba9-fed987654321'::uuid, '87654321-4321-8765-cba9-fed987654321'::uuid),
('22222222-2222-2222-4444-222222222222'::uuid, '12345678-1234-5678-9abc-def123456789'::uuid, 'Chairs', 'Office chairs and seating', '22222222-2222-2222-2222-222222222222'::uuid, 1, '/furniture/chairs', '87654321-4321-8765-cba9-fed987654321'::uuid, '87654321-4321-8765-cba9-fed987654321'::uuid);

-- Products
INSERT INTO products (id, tenant_id, sku, name, description, short_description, category_id, product_type, status, tags, unit_of_measure, weight, dimensions_length, dimensions_width, dimensions_height, base_price, currency, cost_price, list_price, is_tracked, current_stock, min_stock_level, max_stock_level, reorder_point, lead_time_days, brand, manufacturer, model_number, warranty_months, is_featured, created_by, updated_by) VALUES

-- Electronics
('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'::uuid, '12345678-1234-5678-9abc-def123456789'::uuid, 'LAPTOP-001', 'ThinkPad X1 Carbon Gen 11', 'High-performance business laptop with 14-inch display, Intel Core i7, 16GB RAM, 512GB SSD', 'Premium business laptop', '11111111-1111-1111-2222-111111111111'::uuid, 'physical', 'active', ARRAY['laptop', 'business', 'premium', 'intel'], 'piece', 1.120, 31.50, 22.70, 1.49, 159999, 'USD', 120000, 179999, true, 25, 5, 50, 10, 7, 'Lenovo', 'Lenovo', 'X1-CARBON-G11', 36, true, '87654321-4321-8765-cba9-fed987654321'::uuid, '87654321-4321-8765-cba9-fed987654321'::uuid),

('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb'::uuid, '12345678-1234-5678-9abc-def123456789'::uuid, 'MONITOR-001', 'Dell UltraSharp 27" 4K Monitor', '27-inch 4K UHD monitor with USB-C connectivity and height adjustment', '27" 4K USB-C monitor', '11111111-1111-1111-3333-111111111111'::uuid, 'physical', 'active', ARRAY['monitor', '4k', 'usb-c', 'adjustable'], 'piece', 6.200, 61.00, 40.50, 18.30, 59999, 'USD', 45000, 69999, true, 15, 3, 30, 6, 5, 'Dell', 'Dell Technologies', 'S2722DC', 36, true, '87654321-4321-8765-cba9-fed987654321'::uuid, '87654321-4321-8765-cba9-fed987654321'::uuid),

-- Furniture
('cccccccc-cccc-cccc-cccc-cccccccccccc'::uuid, '12345678-1234-5678-9abc-def123456789'::uuid, 'DESK-001', 'UPLIFT V2 Standing Desk', 'Height-adjustable standing desk with electric motor, 60x30 inch bamboo top', 'Electric standing desk', '22222222-2222-2222-3333-222222222222'::uuid, 'physical', 'active', ARRAY['desk', 'standing', 'adjustable', 'bamboo'], 'piece', 45.400, 152.40, 76.20, 5.08, 79999, 'USD', 60000, 89999, true, 8, 2, 15, 4, 14, 'UPLIFT', 'UPLIFT Desk', 'V2-BAMBOO-60X30', 72, true, '87654321-4321-8765-cba9-fed987654321'::uuid, '87654321-4321-8765-cba9-fed987654321'::uuid),

('dddddddd-dddd-dddd-dddd-dddddddddddd'::uuid, '12345678-1234-5678-9abc-def123456789'::uuid, 'CHAIR-001', 'Herman Miller Aeron Chair', 'Ergonomic office chair with PostureFit SL support and 8Z Pellicle suspension', 'Premium ergonomic office chair', '22222222-2222-2222-4444-222222222222'::uuid, 'physical', 'active', ARRAY['chair', 'ergonomic', 'premium', 'adjustable'], 'piece', 19.050, 68.58, 68.58, 104.14, 139999, 'USD', 105000, 159999, true, 12, 2, 20, 5, 21, 'Herman Miller', 'Herman Miller Inc.', 'AERON-REMASTERED', 144, true, '87654321-4321-8765-cba9-fed987654321'::uuid, '87654321-4321-8765-cba9-fed987654321'::uuid),

-- Office Supplies
('eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee'::uuid, '12345678-1234-5678-9abc-def123456789'::uuid, 'NOTEBOOK-001', 'Moleskine Classic Notebook', 'Hard cover notebook with dotted pages, 240 pages, large size', 'Premium dotted notebook', '33333333-3333-3333-3333-333333333333'::uuid, 'physical', 'active', ARRAY['notebook', 'moleskine', 'dotted', 'hardcover'], 'piece', 0.350, 21.00, 13.00, 1.70, 2299, 'USD', 1500, 2799, true, 150, 20, 300, 50, 3, 'Moleskine', 'Moleskine S.p.A.', 'CLASSIC-LARGE-DOT', 0, false, '87654321-4321-8765-cba9-fed987654321'::uuid, '87654321-4321-8765-cba9-fed987654321'::uuid),

('ffffffff-ffff-ffff-ffff-ffffffffffff'::uuid, '12345678-1234-5678-9abc-def123456789'::uuid, 'PEN-001', 'Pilot G2 Gel Pen', 'Retractable gel pen with fine point, black ink, comfortable grip', 'Fine point gel pen', '33333333-3333-3333-3333-333333333333'::uuid, 'physical', 'active', ARRAY['pen', 'gel', 'retractable', 'fine-point'], 'piece', 0.012, 14.20, 1.20, 1.20, 299, 'USD', 180, 399, true, 500, 50, 1000, 150, 2, 'Pilot', 'Pilot Corporation', 'G2-FINE-BLACK', 0, false, '87654321-4321-8765-cba9-fed987654321'::uuid, '87654321-4321-8765-cba9-fed987654321'::uuid);

-- Customers
INSERT INTO customers (id, tenant_id, customer_number, legal_name, customer_type, status, credit_limit, currency, payment_terms_days, total_orders, total_spent, marketing_consent, created_by, updated_by) VALUES
('ccc11111-1111-1111-1111-111111111111'::uuid, '12345678-1234-5678-9abc-def123456789'::uuid, 'CUST-001', 'TechCorp Solutions Ltd.', 'corporate', 'active', 50000.00, 'USD', 30, 15, 87500.50, true, '87654321-4321-8765-cba9-fed987654321'::uuid, '87654321-4321-8765-cba9-fed987654321'::uuid),
('ccc22222-2222-2222-2222-222222222222'::uuid, '12345678-1234-5678-9abc-def123456789'::uuid, 'CUST-002', 'InnovateNow Inc.', 'corporate', 'active', 25000.00, 'USD', 15, 8, 34200.00, true, '87654321-4321-8765-cba9-fed987654321'::uuid, '87654321-4321-8765-cba9-fed987654321'::uuid),
('ccc33333-3333-3333-3333-333333333333'::uuid, '12345678-1234-5678-9abc-def123456789'::uuid, 'CUST-003', 'Sarah Johnson', 'individual', 'active', 5000.00, 'USD', 15, 3, 4800.00, false, '87654321-4321-8765-cba9-fed987654321'::uuid, '87654321-4321-8765-cba9-fed987654321'::uuid);

-- Suppliers
INSERT INTO suppliers (id, tenant_id, supplier_number, legal_name, trade_name, status, currency, payment_terms_days, quality_rating, delivery_rating, overall_rating, total_orders, on_time_deliveries, lead_time_days, minimum_order_amount, certifications, created_by, updated_by) VALUES
('22222222-2222-2222-2222-222222222222'::uuid, '12345678-1234-5678-9abc-def123456789'::uuid, 'SUPP-001', 'TechWare Distribution LLC', 'TechWare', 'active', 'USD', 30, 4.50, 4.20, 4.35, 85, 78, 7, 1000.00, ARRAY['ISO-9001', 'ISO-14001'], '87654321-4321-8765-cba9-fed987654321'::uuid, '87654321-4321-8765-cba9-fed987654321'::uuid),
('33333333-3333-3333-3333-333333333333'::uuid, '12345678-1234-5678-9abc-def123456789'::uuid, 'SUPP-002', 'Premium Office Solutions Inc.', 'POS Inc.', 'active', 'USD', 45, 4.80, 4.60, 4.70, 42, 40, 14, 2500.00, ARRAY['GREENGUARD', 'FSC'], '87654321-4321-8765-cba9-fed987654321'::uuid, '87654321-4321-8765-cba9-fed987654321'::uuid),
('44444444-4444-4444-4444-444444444444'::uuid, '12345678-1234-5678-9abc-def123456789'::uuid, 'SUPP-003', 'Office Essentials Corp.', 'OE Corp', 'active', 'USD', 15, 4.20, 4.80, 4.50, 156, 149, 3, 100.00, ARRAY['ISO-9001'], '87654321-4321-8765-cba9-fed987654321'::uuid, '87654321-4321-8765-cba9-fed987654321'::uuid);

-- Location Items (Inventory)
INSERT INTO location_items (id, product_id, location_id, location_name, location_type, quantity_available, quantity_reserved, reorder_point, min_stock_level, max_stock_level, abc_classification, movement_velocity) VALUES
('lll11111-1111-1111-1111-111111111111'::uuid, 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'::uuid, 'wh-main-1111-1111-1111-111111111111'::uuid, 'Main Warehouse', 'warehouse', 25, 5, 10, 5, 50, 'A', 'fast'),
('lll22222-2222-2222-2222-222222222222'::uuid, 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb'::uuid, 'wh-main-1111-1111-1111-111111111111'::uuid, 'Main Warehouse', 'warehouse', 15, 2, 6, 3, 30, 'A', 'medium'),
('lll33333-3333-3333-3333-333333333333'::uuid, 'cccccccc-cccc-cccc-cccc-cccccccccccc'::uuid, 'wh-main-1111-1111-1111-111111111111'::uuid, 'Main Warehouse', 'warehouse', 8, 1, 4, 2, 15, 'B', 'slow'),
('lll44444-4444-4444-4444-444444444444'::uuid, 'dddddddd-dddd-dddd-dddd-dddddddddddd'::uuid, 'wh-main-1111-1111-1111-111111111111'::uuid, 'Main Warehouse', 'warehouse', 12, 3, 5, 2, 20, 'A', 'medium'),
('lll55555-5555-5555-5555-555555555555'::uuid, 'eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee'::uuid, 'wh-main-1111-1111-1111-111111111111'::uuid, 'Main Warehouse', 'warehouse', 150, 25, 50, 20, 300, 'C', 'fast'),
('lll66666-6666-6666-6666-666666666666'::uuid, 'ffffffff-ffff-ffff-ffff-ffffffffffff'::uuid, 'wh-main-1111-1111-1111-111111111111'::uuid, 'Main Warehouse', 'warehouse', 500, 75, 150, 50, 1000, 'C', 'fast');

-- Sample Inventory Transactions
INSERT INTO inventory_transactions (id, transaction_number, transaction_type, product_id, location_id, quantity_change, unit_cost, total_cost, reference_type, notes, created_by) VALUES
('ttt11111-1111-1111-1111-111111111111'::uuid, 'TXN-001', 'inbound', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'::uuid, 'wh-main-1111-1111-1111-111111111111'::uuid, 10, 1200.00, 12000.00, 'purchase_order', 'Initial stock receipt', '87654321-4321-8765-cba9-fed987654321'::uuid),
('ttt22222-2222-2222-2222-222222222222'::uuid, 'TXN-002', 'inbound', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb'::uuid, 'wh-main-1111-1111-1111-111111111111'::uuid, 20, 450.00, 9000.00, 'purchase_order', 'Initial stock receipt', '87654321-4321-8765-cba9-fed987654321'::uuid),
('ttt33333-3333-3333-3333-333333333333'::uuid, 'TXN-003', 'outbound', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'::uuid, 'wh-main-1111-1111-1111-111111111111'::uuid, -2, 1200.00, -2400.00, 'sales_order', 'Sale to TechCorp', '87654321-4321-8765-cba9-fed987654321'::uuid);

\echo '✓ Seed data layer completed'
\echo ''
\echo '=====================================================';
\echo 'ERP Schema Initialization Complete!';
\echo '=====================================================';
\echo 'Database structure created with:';
\echo '- Foundation: Extensions, enums, functions';
\echo '- Core Tables: Products, customers, suppliers, addresses';
\echo '- Inventory: Locations, movements, analytics';
\echo '- Indexes: Performance optimizations';
\echo '- Seed Data: Demo data for testing';
\echo '';
\echo 'The ERP system is ready for development!';
\echo '=====================================================';

-- Final verification
SELECT
    'Tables Created' as status,
    count(*) as count
FROM information_schema.tables
WHERE table_schema = 'public'
  AND table_type = 'BASE TABLE'
  AND table_name != '_sqlx_migrations';

SELECT
    'Sample Products' as status,
    count(*) as count
FROM products;

SELECT
    'Sample Customers' as status,
    count(*) as count
FROM customers;

SELECT
    'Sample Suppliers' as status,
    count(*) as count
FROM suppliers;

-- Sample Sales Transactions for Analytics
INSERT INTO sales_transactions (
    transaction_id, product_id, customer_id, location_id,
    quantity, unit_price, total_amount, transaction_date,
    created_by, updated_by, tenant_id
)
SELECT
    'TXN-' || LPAD((ROW_NUMBER() OVER ())::text, 6, '0'),
    p.id,
    c.id,
    l.id,
    (random() * 10 + 1)::DECIMAL(10,3),
    p.price,
    p.price * (random() * 10 + 1)::DECIMAL(10,3),
    NOW() - (random() * interval '90 days'),
    '87654321-4321-8765-cba9-fed987654321'::uuid,
    '87654321-4321-8765-cba9-fed987654321'::uuid,
    '12345678-1234-5678-9abc-def123456789'::uuid
FROM products p
CROSS JOIN customers c
CROSS JOIN locations l
LIMIT 50;

-- Sample Inventory Transactions
INSERT INTO inventory_transactions (
    transaction_number, transaction_type, product_id, location_id,
    quantity_change, unit_cost, total_cost, reference_document,
    created_by
)
SELECT
    'IT-' || LPAD((ROW_NUMBER() OVER ())::text, 6, '0'),
    'adjustment',
    p.id,
    l.id,
    (random() * 100 + 10)::INTEGER,
    p.cost,
    p.cost * (random() * 100 + 10)::INTEGER,
    'Initial Stock Load',
    '87654321-4321-8765-cba9-fed987654321'::uuid
FROM products p
CROSS JOIN locations l
LIMIT 20;

SELECT
    'Sample Sales Transactions' as status,
    count(*) as count
FROM sales_transactions;

SELECT
    'Sample Inventory Transactions' as status,
    count(*) as count
FROM inventory_transactions;

-- Sample Customer Feedback
INSERT INTO customer_feedback (
    customer_id, product_id, rating, feedback_type, subject, message,
    sentiment, status, created_by, updated_by, tenant_id
)
SELECT
    c.id,
    p.id,
    (random() * 4 + 1)::INTEGER,
    CASE
        WHEN random() < 0.3 THEN 'product'
        WHEN random() < 0.6 THEN 'service'
        ELSE 'general'
    END,
    'Sample feedback for ' || p.name,
    'This is a sample feedback message for testing purposes.',
    CASE
        WHEN random() < 0.7 THEN 'positive'
        WHEN random() < 0.9 THEN 'neutral'
        ELSE 'negative'
    END,
    'open',
    '87654321-4321-8765-cba9-fed987654321'::uuid,
    '87654321-4321-8765-cba9-fed987654321'::uuid,
    '12345678-1234-5678-9abc-def123456789'::uuid
FROM customers c
CROSS JOIN products p
LIMIT 10;

SELECT
    'Sample Customer Feedback' as status,
    count(*) as count
FROM customer_feedback;