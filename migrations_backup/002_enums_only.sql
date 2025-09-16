-- Master Data Enums Only
-- This migration creates just the custom types first

-- Create custom types for master data
CREATE TYPE customer_type AS ENUM (
    'b2b', 'b2c', 'b2g', 'internal', 'reseller',
    'distributor', 'end_user', 'prospect'
);

CREATE TYPE customer_lifecycle_stage AS ENUM (
    'lead', 'prospect', 'new_customer', 'active_customer',
    'vip_customer', 'at_risk_customer', 'inactive_customer',
    'won_back_customer', 'former_customer'
);

CREATE TYPE credit_status AS ENUM (
    'excellent', 'good', 'fair', 'poor', 'on_hold',
    'blocked', 'cash_only', 'requires_prepayment'
);

CREATE TYPE compliance_status AS ENUM (
    'compliant', 'non_compliant', 'under_review',
    'pending_documents', 'exempt', 'unknown'
);

CREATE TYPE kyc_status AS ENUM (
    'not_started', 'in_progress', 'completed',
    'requires_update', 'failed', 'exempted'
);

CREATE TYPE entity_status AS ENUM (
    'active', 'inactive', 'pending', 'suspended',
    'blocked', 'archived', 'deleted'
);

CREATE TYPE risk_rating AS ENUM (
    'low', 'medium', 'high', 'critical'
);

CREATE TYPE business_size AS ENUM (
    'micro', 'small', 'medium', 'large', 'enterprise'
);

CREATE TYPE acquisition_channel AS ENUM (
    'direct_sales', 'website_inquiry', 'social_media',
    'email_marketing', 'search_engine', 'referral',
    'partner_channel', 'trade_show', 'cold_call',
    'advertisement', 'other'
);

CREATE TYPE address_type AS ENUM (
    'billing', 'shipping', 'mailing', 'physical',
    'headquarters', 'branch', 'warehouse', 'other'
);

CREATE TYPE contact_type AS ENUM (
    'primary', 'billing', 'technical', 'sales',
    'purchasing', 'support', 'executive', 'decision_maker',
    'influencer', 'user', 'other'
);

CREATE TYPE payment_method AS ENUM (
    'cash', 'check', 'bank_transfer', 'credit_card',
    'debit_card', 'digital_wallet', 'cryptocurrency',
    'trade_credit', 'letter_of_credit', 'other'
);

CREATE TYPE data_source AS ENUM (
    'manual', 'import', 'api', 'integration', 'migration', 'synchronization'
);

CREATE TYPE sync_status AS ENUM (
    'pending', 'in_progress', 'success', 'failed', 'conflict', 'skipped'
);