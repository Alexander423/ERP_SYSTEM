-- Migration: Create verification_tokens table
-- This table stores verification tokens for email verification, password reset, user invites, and email change workflows
-- Date: 2024-09-17
-- Note: This will be created in each tenant schema, not public schema

-- Create verification_tokens table (to be created in tenant schemas)
CREATE TABLE IF NOT EXISTS verification_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token VARCHAR(255) NOT NULL UNIQUE,
    purpose VARCHAR(50) NOT NULL CHECK (purpose IN ('email_verification', 'password_reset', 'invite_user', 'change_email')),
    user_id UUID,  -- Can be NULL for invite_user tokens
    tenant_id UUID NOT NULL,
    email VARCHAR(255),
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    used BOOLEAN DEFAULT FALSE NOT NULL,
    used_at TIMESTAMPTZ,
    created_ip INET,
    used_ip INET,

    -- Constraints
    CONSTRAINT verification_tokens_expires_after_created CHECK (expires_at > created_at),
    CONSTRAINT verification_tokens_used_after_created CHECK (used_at IS NULL OR used_at >= created_at),
    CONSTRAINT verification_tokens_user_id_required CHECK (
        (purpose IN ('email_verification', 'password_reset', 'change_email') AND user_id IS NOT NULL) OR
        (purpose = 'invite_user')
    )
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_verification_tokens_token ON verification_tokens(token);
CREATE INDEX IF NOT EXISTS idx_verification_tokens_user_id ON verification_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_verification_tokens_purpose ON verification_tokens(purpose);
CREATE INDEX IF NOT EXISTS idx_verification_tokens_expires_at ON verification_tokens(expires_at);
CREATE INDEX IF NOT EXISTS idx_verification_tokens_tenant_id ON verification_tokens(tenant_id);

-- Index for cleanup of expired tokens
CREATE INDEX IF NOT EXISTS idx_verification_tokens_cleanup ON verification_tokens(expires_at, used) WHERE NOT used;

-- Comments for documentation
COMMENT ON TABLE verification_tokens IS 'Stores verification tokens for email verification, password reset, and other verification workflows';
COMMENT ON COLUMN verification_tokens.token IS 'Cryptographically secure verification token';
COMMENT ON COLUMN verification_tokens.purpose IS 'Type of verification: email_verification, password_reset, 2fa_backup';
COMMENT ON COLUMN verification_tokens.metadata IS 'Additional metadata for the verification process';
COMMENT ON COLUMN verification_tokens.used IS 'Whether the token has been used';
COMMENT ON COLUMN verification_tokens.expires_at IS 'When the token expires';