-- Add missing enum types for compilation
-- Date: 2024-12-16
-- Purpose: Fix SQLX compilation errors by adding missing enum types

-- Create industry_classification enum
CREATE TYPE industry_classification AS ENUM (
    'technology', 'manufacturing', 'healthcare', 'finance',
    'retail', 'education', 'government', 'energy',
    'transportation', 'real_estate', 'agriculture', 'construction',
    'entertainment', 'telecommunications', 'other'
);

-- Add any other missing enum types that may be needed
-- Note: Most other types already exist in 002_enums_only.sql