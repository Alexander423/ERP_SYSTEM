-- Create indexes for advanced customer search capabilities

-- Full-text search indexes
CREATE INDEX IF NOT EXISTS idx_customers_fts_legal_name
ON customers USING gin(to_tsvector('english', legal_name));

CREATE INDEX IF NOT EXISTS idx_customers_fts_combined
ON customers USING gin(
    to_tsvector('english',
        COALESCE(legal_name, '') || ' ' ||
        COALESCE(customer_number, '') || ' ' ||
        COALESCE(notes, '')
    )
);

-- Search performance indexes
CREATE INDEX IF NOT EXISTS idx_customers_search_basic
ON customers (tenant_id, is_deleted, customer_type, lifecycle_stage);

CREATE INDEX IF NOT EXISTS idx_customers_search_industry
ON customers (tenant_id, is_deleted, industry_classification);

CREATE INDEX IF NOT EXISTS idx_customers_search_revenue
ON customers (tenant_id, is_deleted, customer_lifetime_value);

CREATE INDEX IF NOT EXISTS idx_customers_search_dates
ON customers (tenant_id, is_deleted, created_at, modified_at);

CREATE INDEX IF NOT EXISTS idx_customers_search_credit
ON customers (tenant_id, is_deleted, credit_status, credit_limit);

-- Geographic search support (for future use)
CREATE INDEX IF NOT EXISTS idx_customers_acquisition_channel
ON customers (tenant_id, acquisition_channel) WHERE acquisition_channel IS NOT NULL;

-- Composite indexes for common search patterns
CREATE INDEX IF NOT EXISTS idx_customers_type_stage_industry
ON customers (customer_type, lifecycle_stage, industry_classification)
WHERE NOT is_deleted;

CREATE INDEX IF NOT EXISTS idx_customers_revenue_range
ON customers (customer_lifetime_value, credit_limit)
WHERE NOT is_deleted AND customer_lifetime_value IS NOT NULL;

-- JSONB indexes for advanced filtering
CREATE INDEX IF NOT EXISTS idx_customers_external_ids_gin
ON customers USING gin(external_ids);

CREATE INDEX IF NOT EXISTS idx_customers_tags_gin
ON customers USING gin(tags);

CREATE INDEX IF NOT EXISTS idx_customers_custom_fields_gin
ON customers USING gin(custom_fields);

-- Performance optimization for similarity search
CREATE INDEX IF NOT EXISTS idx_customers_similarity_attrs
ON customers (customer_type, industry_classification, lifecycle_stage, customer_lifetime_value)
WHERE NOT is_deleted;

-- Index for search suggestions/autocomplete
CREATE INDEX IF NOT EXISTS idx_customers_legal_name_trgm
ON customers USING gin(legal_name gin_trgm_ops);

CREATE INDEX IF NOT EXISTS idx_customers_number_trgm
ON customers USING gin(customer_number gin_trgm_ops);

-- Enable pg_trgm extension for fuzzy search
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- Create materialized view for search performance (optional)
CREATE MATERIALIZED VIEW IF NOT EXISTS customer_search_cache AS
SELECT
    c.id,
    c.tenant_id,
    c.customer_number,
    c.legal_name,
    c.customer_type,
    c.lifecycle_stage,
    c.industry_classification,
    c.customer_lifetime_value,
    c.credit_limit,
    c.created_at,
    c.modified_at,
    to_tsvector('english',
        COALESCE(c.legal_name, '') || ' ' ||
        COALESCE(c.customer_number, '') || ' ' ||
        COALESCE(c.notes, '') || ' ' ||
        COALESCE(array_to_string(c.tags, ' '), '')
    ) as search_vector
FROM customers c
WHERE NOT c.is_deleted;

-- Index on the materialized view
CREATE INDEX IF NOT EXISTS idx_customer_search_cache_vector
ON customer_search_cache USING gin(search_vector);

CREATE INDEX IF NOT EXISTS idx_customer_search_cache_basic
ON customer_search_cache (tenant_id, customer_type, lifecycle_stage);

-- Function to refresh the search cache
CREATE OR REPLACE FUNCTION refresh_customer_search_cache()
RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY customer_search_cache;
END;
$$ LANGUAGE plpgsql;