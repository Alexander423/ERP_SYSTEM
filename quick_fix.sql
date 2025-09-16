-- Quick fix for missing enums and tables
CREATE TYPE IF NOT EXISTS industry_classification AS ENUM (
    'agriculture', 'automotive', 'banking', 'construction', 'education',
    'energy', 'finance', 'government', 'healthcare', 'hospitality',
    'insurance', 'logistics', 'manufacturing', 'media', 'nonprofit',
    'professional_services', 'real_estate', 'retail', 'technology',
    'telecommunications', 'transportation', 'utilities', 'other'
);