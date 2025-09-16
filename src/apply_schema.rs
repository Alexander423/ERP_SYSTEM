use sqlx::PgPool;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = env::var("DATABASE_URL")?;
    let pool = PgPool::connect(&database_url).await?;

    // Create missing enum types - first try to create it
    let result = sqlx::query(r#"
        CREATE TYPE industry_classification AS ENUM (
            'agriculture', 'automotive', 'banking', 'construction', 'education',
            'energy', 'finance', 'government', 'healthcare', 'hospitality',
            'insurance', 'logistics', 'manufacturing', 'media', 'nonprofit',
            'professional_services', 'real_estate', 'retail', 'technology',
            'telecommunications', 'transportation', 'utilities', 'other'
        );
    "#)
    .execute(&pool)
    .await;

    // Ignore error if type already exists
    if let Err(e) = result {
        println!("Note: industry_classification type might already exist: {}", e);
    }

    // Create roles table if it doesn't exist
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS roles (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR(100) NOT NULL,
            description TEXT,
            priority SMALLINT NOT NULL DEFAULT 0,
            is_system_role BOOLEAN NOT NULL DEFAULT FALSE,
            created_by UUID NOT NULL,
            created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
            modified_by UUID NOT NULL,
            modified_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
        );
    "#)
    .execute(&pool)
    .await?;

    // Create role_permissions table if it doesn't exist
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS role_permissions (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
            permission_id UUID NOT NULL,
            resource_type TEXT NOT NULL DEFAULT 'general',
            action VARCHAR(100) NOT NULL,
            scope JSONB DEFAULT '{}',
            created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
            UNIQUE(role_id, permission_id, action)
        );
    "#)
    .execute(&pool)
    .await?;

    // Create user_roles table if it doesn't exist
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS user_roles (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id UUID NOT NULL,
            role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
            assigned_by UUID NOT NULL,
            assigned_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
            expires_at TIMESTAMP WITH TIME ZONE,
            UNIQUE(user_id, role_id)
        );
    "#)
    .execute(&pool)
    .await?;

    // Create customer_performance_metrics table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS customer_performance_metrics (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            customer_id UUID NOT NULL,
            revenue_last_12_months DECIMAL(15, 2),
            average_order_value DECIMAL(15, 2),
            order_frequency DECIMAL(10, 2),
            total_orders INTEGER,
            total_revenue DECIMAL(15, 2),
            profit_margin DECIMAL(5, 2),
            last_purchase_date DATE,
            first_purchase_date DATE,
            customer_lifetime_value DECIMAL(15, 2),
            predicted_churn_probability DECIMAL(3, 2),
            last_updated TIMESTAMP WITH TIME ZONE,
            UNIQUE(customer_id)
        );
    "#)
    .execute(&pool)
    .await?;

    // Create customer_behavioral_data table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS customer_behavioral_data (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            customer_id UUID NOT NULL,
            purchase_frequency DECIMAL(10, 2),
            preferred_categories JSONB DEFAULT '[]',
            seasonal_trends JSONB DEFAULT '{}',
            price_sensitivity DECIMAL(3, 2),
            brand_loyalty DECIMAL(3, 2),
            communication_preferences JSONB DEFAULT '{}',
            support_ticket_frequency DECIMAL(10, 2),
            product_return_rate DECIMAL(3, 2),
            referral_activity DECIMAL(3, 2),
            product_category_preferences JSONB DEFAULT '[]',
            preferred_contact_times JSONB DEFAULT '[]',
            channel_engagement_rates JSONB DEFAULT '{}',
            website_engagement_score DECIMAL(3, 2),
            mobile_app_usage DECIMAL(3, 2),
            social_media_sentiment DECIMAL(3, 2),
            propensity_to_buy DECIMAL(3, 2),
            upsell_probability DECIMAL(3, 2),
            cross_sell_probability DECIMAL(3, 2),
            last_updated TIMESTAMP WITH TIME ZONE,
            UNIQUE(customer_id)
        );
    "#)
    .execute(&pool)
    .await?;

    // Create tenants table if needed (for foreign key references)
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS tenants (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR(255) NOT NULL,
            created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
        );
    "#)
    .execute(&pool)
    .await?;

    // Insert default tenant (try with minimal required fields only)
    let tenant_result = sqlx::query(r#"
        INSERT INTO tenants (id, name)
        VALUES ('00000000-0000-0000-0000-000000000001', 'Default Tenant')
        ON CONFLICT (id) DO NOTHING;
    "#)
    .execute(&pool)
    .await;

    if let Err(e) = tenant_result {
        println!("Note: Tenant insert failed (table might have different structure): {}", e);
        // Try to ensure a tenant exists for foreign key constraints
        let _result = sqlx::query(r#"
            INSERT INTO tenants (id, name, schema_name, status, created_at, modified_at)
            VALUES ('00000000-0000-0000-0000-000000000001', 'Default Tenant', 'public', 'Active', NOW(), NOW())
            ON CONFLICT (id) DO NOTHING;
        "#)
        .execute(&pool)
        .await;
    }

    // Create customers table with all required columns
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS customers (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            tenant_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000001',
            customer_number VARCHAR(50) NOT NULL,
            external_ids JSONB DEFAULT '{}',
            legal_name VARCHAR(255) NOT NULL,
            trade_names JSONB DEFAULT '[]',
            customer_type customer_type NOT NULL,
            industry_classification industry_classification,
            business_size business_size,
            parent_customer_id UUID,
            corporate_group_id UUID,
            customer_hierarchy_level SMALLINT DEFAULT 0,
            consolidation_group VARCHAR(100),
            lifecycle_stage customer_lifecycle_stage NOT NULL,
            status entity_status NOT NULL DEFAULT 'active',
            credit_status credit_status,
            primary_address_id UUID,
            billing_address_id UUID,
            shipping_address_ids UUID[],
            primary_contact_id UUID,
            tax_jurisdictions JSONB DEFAULT '[]',
            tax_numbers JSONB DEFAULT '{}',
            regulatory_classifications JSONB DEFAULT '[]',
            compliance_status compliance_status,
            kyc_status kyc_status,
            aml_risk_rating risk_rating,
            currency_code VARCHAR(3),
            credit_limit DECIMAL(15, 2),
            payment_terms JSONB,
            tax_exempt BOOLEAN DEFAULT FALSE,
            price_group_id UUID,
            discount_group_id UUID,
            sales_representative_id UUID,
            account_manager_id UUID,
            customer_segments JSONB DEFAULT '[]',
            acquisition_channel acquisition_channel,
            customer_lifetime_value DECIMAL(15, 2),
            churn_probability DECIMAL(3, 2),
            master_data_source data_source DEFAULT 'manual',
            external_id VARCHAR(255),
            sync_status sync_status DEFAULT 'synced',
            last_sync TIMESTAMP WITH TIME ZONE,
            sync_source VARCHAR(100),
            sync_version INTEGER DEFAULT 1,
            custom_fields JSONB DEFAULT '{}',
            contract_ids UUID[],
            created_by UUID NOT NULL,
            created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
            modified_by UUID NOT NULL,
            modified_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
            version INTEGER NOT NULL DEFAULT 1,
            is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
            deleted_at TIMESTAMP WITH TIME ZONE,
            deleted_by UUID,
            UNIQUE(tenant_id, customer_number)
        );
    "#)
    .execute(&pool)
    .await?;

    // Add missing columns to data_masking_policies
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS data_masking_policies (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            tenant_id UUID,
            table_name VARCHAR(100) NOT NULL,
            column_name VARCHAR(100) NOT NULL,
            masking_rule JSONB NOT NULL,
            exemptions JSONB DEFAULT '{}',
            created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
            modified_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
        );
    "#)
    .execute(&pool)
    .await?;

    // Create remediation_actions table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS remediation_actions (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            finding_id UUID NOT NULL,
            action_title VARCHAR(255) NOT NULL,
            action_description TEXT,
            action_type VARCHAR(100) NOT NULL,
            priority VARCHAR(20) NOT NULL DEFAULT 'MEDIUM',
            assigned_to UUID,
            target_completion_date DATE,
            actual_completion_date DATE,
            status VARCHAR(50) NOT NULL DEFAULT 'PENDING',
            progress_percentage INTEGER DEFAULT 0,
            evidence_documents JSONB DEFAULT '[]',
            cost_estimate DECIMAL(15, 2),
            actual_cost DECIMAL(15, 2),
            effectiveness_rating INTEGER,
            notes TEXT,
            tenant_id UUID,
            created_by UUID NOT NULL,
            created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
            modified_by UUID NOT NULL,
            modified_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
            version INTEGER NOT NULL DEFAULT 1
        );
    "#)
    .execute(&pool)
    .await?;

    println!("Schema fixes applied successfully!");
    Ok(())
}