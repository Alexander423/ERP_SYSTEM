-- Security and access control schema

-- User roles table
CREATE TABLE IF NOT EXISTS user_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    role_id UUID NOT NULL,
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    assigned_by UUID NOT NULL,
    assigned_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, role_id)
);

-- Roles table
CREATE TABLE IF NOT EXISTS roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    priority SMALLINT NOT NULL DEFAULT 0,
    is_system_role BOOLEAN NOT NULL DEFAULT FALSE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    metadata JSONB DEFAULT '{}',
    created_by UUID NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    modified_by UUID NOT NULL,
    modified_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    version INTEGER NOT NULL DEFAULT 1,
    UNIQUE(name, tenant_id)
);

-- Role permissions table
CREATE TABLE IF NOT EXISTS role_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL,
    resource_type TEXT NOT NULL,
    action TEXT NOT NULL,
    scope TEXT NOT NULL,
    field_restrictions JSONB,
    conditions JSONB,
    time_restrictions JSONB,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Role hierarchy table for inheritance
CREATE TABLE IF NOT EXISTS role_hierarchy (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    child_role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(parent_role_id, child_role_id),
    CHECK(parent_role_id != child_role_id)
);

-- Access attempts table for audit logging
CREATE TABLE IF NOT EXISTS access_attempts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    resource_id UUID NOT NULL,
    resource_type TEXT NOT NULL,
    action TEXT NOT NULL,
    permission_checked JSONB,
    context_data JSONB,
    granted BOOLEAN NOT NULL,
    policy_decision JSONB,
    ip_address INET,
    user_agent TEXT,
    session_id VARCHAR(255),
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE
);

-- Security policies table
CREATE TABLE IF NOT EXISTS security_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    policy_rules JSONB NOT NULL,
    priority SMALLINT NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    metadata JSONB DEFAULT '{}',
    created_by UUID NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    modified_by UUID NOT NULL,
    modified_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    version INTEGER NOT NULL DEFAULT 1
);

-- Encrypted fields table for field-level encryption metadata
CREATE TABLE IF NOT EXISTS encrypted_fields (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    table_name VARCHAR(255) NOT NULL,
    column_name VARCHAR(255) NOT NULL,
    record_id UUID NOT NULL,
    encrypted_data TEXT NOT NULL,
    nonce TEXT NOT NULL,
    algorithm VARCHAR(50) NOT NULL,
    key_id VARCHAR(255) NOT NULL,
    field_name VARCHAR(255) NOT NULL,
    encrypted_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    integrity_hash TEXT NOT NULL,
    context_hash TEXT NOT NULL,
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(table_name, column_name, record_id, tenant_id)
);

-- Audit log table for comprehensive security auditing
CREATE TABLE IF NOT EXISTS security_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type VARCHAR(100) NOT NULL,
    event_category VARCHAR(50) NOT NULL,
    user_id UUID,
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    resource_type VARCHAR(100),
    resource_id UUID,
    action VARCHAR(100) NOT NULL,
    outcome VARCHAR(20) NOT NULL, -- SUCCESS, FAILURE, DENIED
    risk_level VARCHAR(20) NOT NULL DEFAULT 'LOW', -- LOW, MEDIUM, HIGH, CRITICAL
    event_data JSONB DEFAULT '{}',
    ip_address INET,
    user_agent TEXT,
    session_id VARCHAR(255),
    correlation_id UUID,
    source_system VARCHAR(100) DEFAULT 'ERP_SYSTEM',
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    retention_until TIMESTAMP WITH TIME ZONE
);

-- Data masking policies table
CREATE TABLE IF NOT EXISTS data_masking_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    table_name VARCHAR(255) NOT NULL,
    column_name VARCHAR(255) NOT NULL,
    masking_type VARCHAR(50) NOT NULL, -- REDACTION, SUBSTITUTION, SHUFFLING, etc.
    masking_config JSONB NOT NULL,
    conditions JSONB, -- When to apply masking
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    created_by UUID NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    modified_by UUID NOT NULL,
    modified_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(table_name, column_name, tenant_id)
);

-- Compliance framework tracking
CREATE TABLE IF NOT EXISTS compliance_frameworks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    framework_name VARCHAR(100) NOT NULL, -- GDPR, SOX, HIPAA, etc.
    version VARCHAR(50) NOT NULL,
    requirements JSONB NOT NULL,
    controls JSONB NOT NULL,
    assessments JSONB DEFAULT '{}',
    compliance_status VARCHAR(20) NOT NULL DEFAULT 'UNKNOWN', -- COMPLIANT, NON_COMPLIANT, UNDER_REVIEW
    last_assessment_date TIMESTAMP WITH TIME ZONE,
    next_assessment_date TIMESTAMP WITH TIME ZONE,
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    created_by UUID NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    modified_by UUID NOT NULL,
    modified_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Data classification table
CREATE TABLE IF NOT EXISTS data_classifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    table_name VARCHAR(255) NOT NULL,
    column_name VARCHAR(255) NOT NULL,
    classification_level VARCHAR(50) NOT NULL, -- PUBLIC, INTERNAL, CONFIDENTIAL, RESTRICTED, TOP_SECRET
    sensitivity_tags TEXT[],
    regulatory_requirements TEXT[],
    retention_period_days INTEGER,
    encryption_required BOOLEAN NOT NULL DEFAULT FALSE,
    masking_required BOOLEAN NOT NULL DEFAULT FALSE,
    audit_required BOOLEAN NOT NULL DEFAULT FALSE,
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    created_by UUID NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    modified_by UUID NOT NULL,
    modified_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(table_name, column_name, tenant_id)
);

-- Security incidents table
CREATE TABLE IF NOT EXISTS security_incidents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    incident_type VARCHAR(100) NOT NULL,
    severity VARCHAR(20) NOT NULL, -- LOW, MEDIUM, HIGH, CRITICAL
    status VARCHAR(50) NOT NULL DEFAULT 'OPEN', -- OPEN, INVESTIGATING, RESOLVED, CLOSED
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    affected_systems TEXT[],
    affected_data_types TEXT[],
    user_id UUID,
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    detection_method VARCHAR(100),
    detection_timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    response_timestamp TIMESTAMP WITH TIME ZONE,
    resolution_timestamp TIMESTAMP WITH TIME ZONE,
    remediation_actions JSONB DEFAULT '{}',
    lessons_learned TEXT,
    created_by UUID NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    modified_by UUID NOT NULL,
    modified_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_user_roles_user_id ON user_roles(user_id);
CREATE INDEX IF NOT EXISTS idx_user_roles_role_id ON user_roles(role_id);
CREATE INDEX IF NOT EXISTS idx_user_roles_tenant_id ON user_roles(tenant_id);

CREATE INDEX IF NOT EXISTS idx_roles_tenant_id ON roles(tenant_id);
CREATE INDEX IF NOT EXISTS idx_roles_system_role ON roles(is_system_role);
CREATE INDEX IF NOT EXISTS idx_roles_active ON roles(is_active);

CREATE INDEX IF NOT EXISTS idx_role_permissions_role_id ON role_permissions(role_id);
CREATE INDEX IF NOT EXISTS idx_role_permissions_resource_action ON role_permissions(resource_type, action);

CREATE INDEX IF NOT EXISTS idx_access_attempts_user_id ON access_attempts(user_id);
CREATE INDEX IF NOT EXISTS idx_access_attempts_timestamp ON access_attempts(timestamp);
CREATE INDEX IF NOT EXISTS idx_access_attempts_granted ON access_attempts(granted);
CREATE INDEX IF NOT EXISTS idx_access_attempts_tenant_id ON access_attempts(tenant_id);

CREATE INDEX IF NOT EXISTS idx_encrypted_fields_table_record ON encrypted_fields(table_name, record_id);
CREATE INDEX IF NOT EXISTS idx_encrypted_fields_tenant_id ON encrypted_fields(tenant_id);

CREATE INDEX IF NOT EXISTS idx_security_audit_log_user_id ON security_audit_log(user_id);
CREATE INDEX IF NOT EXISTS idx_security_audit_log_timestamp ON security_audit_log(timestamp);
CREATE INDEX IF NOT EXISTS idx_security_audit_log_event_type ON security_audit_log(event_type);
CREATE INDEX IF NOT EXISTS idx_security_audit_log_risk_level ON security_audit_log(risk_level);
CREATE INDEX IF NOT EXISTS idx_security_audit_log_tenant_id ON security_audit_log(tenant_id);

CREATE INDEX IF NOT EXISTS idx_data_masking_policies_table_column ON data_masking_policies(table_name, column_name);
CREATE INDEX IF NOT EXISTS idx_data_masking_policies_tenant_id ON data_masking_policies(tenant_id);

CREATE INDEX IF NOT EXISTS idx_compliance_frameworks_tenant_id ON compliance_frameworks(tenant_id);
CREATE INDEX IF NOT EXISTS idx_compliance_frameworks_status ON compliance_frameworks(compliance_status);

CREATE INDEX IF NOT EXISTS idx_data_classifications_table_column ON data_classifications(table_name, column_name);
CREATE INDEX IF NOT EXISTS idx_data_classifications_level ON data_classifications(classification_level);
CREATE INDEX IF NOT EXISTS idx_data_classifications_tenant_id ON data_classifications(tenant_id);

CREATE INDEX IF NOT EXISTS idx_security_incidents_status ON security_incidents(status);
CREATE INDEX IF NOT EXISTS idx_security_incidents_severity ON security_incidents(severity);
CREATE INDEX IF NOT EXISTS idx_security_incidents_tenant_id ON security_incidents(tenant_id);
CREATE INDEX IF NOT EXISTS idx_security_incidents_detection_timestamp ON security_incidents(detection_timestamp);

-- Create function for automatic audit logging
CREATE OR REPLACE FUNCTION log_security_event()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO security_audit_log (
        event_type,
        event_category,
        user_id,
        tenant_id,
        resource_type,
        resource_id,
        action,
        outcome,
        event_data,
        timestamp
    ) VALUES (
        TG_TABLE_NAME || '_' || TG_OP,
        'DATA_ACCESS',
        COALESCE(NEW.modified_by, NEW.created_by),
        COALESCE(NEW.tenant_id, OLD.tenant_id),
        TG_TABLE_NAME,
        COALESCE(NEW.id, OLD.id),
        TG_OP,
        'SUCCESS',
        jsonb_build_object(
            'old_data', row_to_json(OLD),
            'new_data', row_to_json(NEW),
            'operation', TG_OP
        ),
        NOW()
    );
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

-- Apply audit triggers to sensitive tables
CREATE TRIGGER security_audit_trigger_customers
    AFTER INSERT OR UPDATE OR DELETE ON customers
    FOR EACH ROW EXECUTE FUNCTION log_security_event();

-- Data retention function for audit logs
CREATE OR REPLACE FUNCTION cleanup_old_audit_logs()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM security_audit_log
    WHERE retention_until IS NOT NULL
      AND retention_until < NOW();

    GET DIAGNOSTICS deleted_count = ROW_COUNT;

    DELETE FROM access_attempts
    WHERE timestamp < NOW() - INTERVAL '1 year';

    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Create periodic cleanup job (would be scheduled externally)
COMMENT ON FUNCTION cleanup_old_audit_logs() IS
'Cleans up old audit logs based on retention policy. Should be scheduled to run daily.';