-- Updated suppliers table schema to match Rust structs

-- Drop and recreate supplier-related types
DROP TYPE IF EXISTS supplier_type CASCADE;
DROP TYPE IF EXISTS supplier_status CASCADE;
DROP TYPE IF EXISTS supplier_category CASCADE;
DROP TYPE IF EXISTS payment_terms CASCADE;

-- Create enum types that match Rust code
CREATE TYPE supplier_status AS ENUM ('active', 'inactive', 'pending', 'suspended', 'terminated');
CREATE TYPE supplier_category AS ENUM ('raw_materials', 'manufacturing', 'technology', 'services', 'logistics', 'office_supplies', 'marketing', 'utilities', 'other');
CREATE TYPE payment_terms AS ENUM ('net15', 'net30', 'net45', 'net60', 'net90', 'two_ten_net30', 'cod', 'prepaid');

-- Drop and recreate suppliers table
DROP TABLE IF EXISTS suppliers CASCADE;

CREATE TABLE suppliers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,

    -- Basic Information
    supplier_code VARCHAR(50) NOT NULL,
    company_name VARCHAR(255) NOT NULL,
    legal_name VARCHAR(255),
    tax_id VARCHAR(100),
    registration_number VARCHAR(100),

    -- Classification
    category supplier_category NOT NULL DEFAULT 'other',
    status supplier_status NOT NULL DEFAULT 'pending',
    tags JSONB,

    -- Contact Information
    website VARCHAR(500),
    phone VARCHAR(50),
    email VARCHAR(255),

    -- Business Terms
    payment_terms payment_terms NOT NULL DEFAULT 'net30',
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    credit_limit BIGINT, -- in cents
    lead_time_days INTEGER,

    -- Performance Metrics
    rating DECIMAL(3,2),
    on_time_delivery_rate DECIMAL(5,4),
    quality_rating DECIMAL(3,2),

    -- Metadata
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL,
    updated_by UUID NOT NULL,

    UNIQUE(tenant_id, supplier_code)
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_suppliers_tenant_active ON suppliers(tenant_id, status);
CREATE INDEX IF NOT EXISTS idx_suppliers_category ON suppliers(category);
CREATE INDEX IF NOT EXISTS idx_suppliers_rating ON suppliers(rating);