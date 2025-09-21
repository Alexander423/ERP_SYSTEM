-- Reference Data for New Tenants
-- This script inserts reference data and default settings for a new tenant
-- Variables: {TENANT_SCHEMA} will be replaced with actual tenant schema name

-- Set search path to tenant schema
SET search_path TO {TENANT_SCHEMA}, public;

-- Default System Settings
INSERT INTO {TENANT_SCHEMA}.settings (category, key, value, description, created_by, modified_by) VALUES
-- General System Settings
('system', 'tenant_name', '"{TENANT_NAME}"'::jsonb, 'Display name for this tenant', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),
('system', 'default_currency', '"USD"'::jsonb, 'Default currency for the tenant', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),
('system', 'default_language', '"en"'::jsonb, 'Default language for the tenant', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),
('system', 'default_timezone', '"UTC"'::jsonb, 'Default timezone for the tenant', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),
('system', 'date_format', '"YYYY-MM-DD"'::jsonb, 'Default date format', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),
('system', 'time_format', '"HH:mm:ss"'::jsonb, 'Default time format', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),

-- Customer Management Settings
('customer', 'auto_generate_customer_number', 'true'::jsonb, 'Automatically generate customer numbers', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),
('customer', 'customer_number_prefix', '"CUST"'::jsonb, 'Prefix for auto-generated customer numbers', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),
('customer', 'customer_number_length', '8'::jsonb, 'Length of auto-generated customer numbers', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),
('customer', 'require_customer_approval', 'false'::jsonb, 'Require approval for new customers', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),
('customer', 'default_lifecycle_stage', '"Lead"'::jsonb, 'Default lifecycle stage for new customers', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),
('customer', 'default_credit_status', '"Good"'::jsonb, 'Default credit status for new customers', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),

-- Order Management Settings
('order', 'auto_generate_order_number', 'true'::jsonb, 'Automatically generate order numbers', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),
('order', 'order_number_prefix', '"ORD"'::jsonb, 'Prefix for auto-generated order numbers', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),
('order', 'order_number_length', '10'::jsonb, 'Length of auto-generated order numbers', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),
('order', 'require_order_approval', 'false'::jsonb, 'Require approval for orders above threshold', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),
('order', 'approval_threshold', '10000.00'::jsonb, 'Order amount requiring approval', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),

-- Security Settings
('security', 'password_min_length', '8'::jsonb, 'Minimum password length', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),
('security', 'password_require_uppercase', 'true'::jsonb, 'Require uppercase letters in passwords', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),
('security', 'password_require_lowercase', 'true'::jsonb, 'Require lowercase letters in passwords', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),
('security', 'password_require_numbers', 'true'::jsonb, 'Require numbers in passwords', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),
('security', 'password_require_symbols', 'true'::jsonb, 'Require symbols in passwords', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),
('security', 'max_login_attempts', '5'::jsonb, 'Maximum failed login attempts before account lock', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),
('security', 'account_lockout_duration', '300'::jsonb, 'Account lockout duration in seconds', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),
('security', 'session_timeout', '1800'::jsonb, 'Session timeout in seconds', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),
('security', 'require_two_factor', 'false'::jsonb, 'Require two-factor authentication', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),

-- Email Settings
('email', 'from_name', '"{TENANT_NAME} ERP System"'::jsonb, 'Default from name for emails', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),
('email', 'from_address', '"noreply@{TENANT_DOMAIN}"'::jsonb, 'Default from address for emails', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),
('email', 'signature', '"Best regards,<br>{TENANT_NAME} Team"'::jsonb, 'Default email signature', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),

-- Notification Settings
('notifications', 'new_customer_notification', 'true'::jsonb, 'Send notifications for new customers', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),
('notifications', 'new_order_notification', 'true'::jsonb, 'Send notifications for new orders', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),
('notifications', 'order_status_change_notification', 'true'::jsonb, 'Send notifications for order status changes', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),
('notifications', 'payment_received_notification', 'true'::jsonb, 'Send notifications for payments received', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),

-- Audit and Compliance
('audit', 'enable_audit_logging', 'true'::jsonb, 'Enable comprehensive audit logging', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),
('audit', 'audit_retention_days', '2555'::jsonb, 'Audit log retention period in days (7 years)', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),
('audit', 'enable_data_export_logging', 'true'::jsonb, 'Log all data exports', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),

-- Integration Settings
('integration', 'api_rate_limit', '1000'::jsonb, 'API rate limit per hour per user', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),
('integration', 'webhook_timeout', '30'::jsonb, 'Webhook timeout in seconds', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),
('integration', 'enable_external_sync', 'false'::jsonb, 'Enable external system synchronization', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),

-- Backup and Recovery
('backup', 'auto_backup_enabled', 'true'::jsonb, 'Enable automatic backups', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),
('backup', 'backup_retention_days', '90'::jsonb, 'Backup retention period in days', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),
('backup', 'backup_schedule', '"0 2 * * *"'::jsonb, 'Backup schedule (cron format)', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),

-- Performance Settings
('performance', 'search_results_per_page', '25'::jsonb, 'Default number of search results per page', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),
('performance', 'max_export_records', '10000'::jsonb, 'Maximum records allowed in data export', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid),
('performance', 'cache_duration', '300'::jsonb, 'Cache duration in seconds', '00000000-0000-0000-0000-000000000000'::uuid, '00000000-0000-0000-0000-000000000000'::uuid);

-- Reset search path
SET search_path TO DEFAULT;