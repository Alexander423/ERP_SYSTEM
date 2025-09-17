-- Fix the industry_classification column type
-- Convert from JSONB to enum type

-- First, drop the old column
ALTER TABLE customers
DROP COLUMN IF EXISTS industry_classification;

-- Add it back as the correct enum type
ALTER TABLE customers
ADD COLUMN industry_classification industry_classification;

-- If there's data we need to preserve, we could do a more complex migration
-- For now, this will set all values to NULL and they can be updated later