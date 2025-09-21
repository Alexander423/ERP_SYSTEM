#!/bin/bash

# ERP System - Tenant Creation Script
# Creates a new tenant with complete schema and default data

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_status() {
    echo -e "${GREEN}[TENANT]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_header() {
    echo -e "${BLUE}============================================${NC}"
    echo -e "${BLUE}    ERP System - Tenant Creation${NC}"
    echo -e "${BLUE}============================================${NC}"
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -n, --name TENANT_NAME      Tenant display name"
    echo "  -e, --email ADMIN_EMAIL     Admin user email"
    echo "  -d, --domain DOMAIN         Tenant domain (optional)"
    echo "  -p, --password PASSWORD     Admin password (optional, will prompt if not provided)"
    echo "  -s, --schema SCHEMA_NAME    Database schema name (optional, auto-generated)"
    echo "  -h, --help                  Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 --name \"Acme Corp\" --email admin@acme.com"
    echo "  $0 -n \"Tech Startup\" -e admin@startup.com -d startup.com"
    echo ""
}

# Function to validate email
validate_email() {
    local email="$1"
    if [[ ! "$email" =~ ^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$ ]]; then
        print_error "Invalid email format: $email"
        return 1
    fi
}

# Function to validate tenant name
validate_tenant_name() {
    local name="$1"
    if [[ ${#name} -lt 2 || ${#name} -gt 100 ]]; then
        print_error "Tenant name must be between 2 and 100 characters"
        return 1
    fi
}

# Function to generate schema name from tenant name
generate_schema_name() {
    local name="$1"
    # Convert to lowercase, replace spaces and special chars with underscores
    echo "$name" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9]/_/g' | sed 's/_\+/_/g' | sed 's/^_\|_$//g'
}

# Function to generate secure password
generate_password() {
    openssl rand -base64 16 | tr -d "=+/" | cut -c1-12
}

# Function to hash password
hash_password() {
    local password="$1"
    local cost="${2:-12}"

    # Use Python to generate bcrypt hash if available
    if command -v python3 &> /dev/null; then
        python3 -c "
import bcrypt
import sys
password = sys.argv[1].encode('utf-8')
hashed = bcrypt.hashpw(password, bcrypt.gensalt(rounds=$cost))
print(hashed.decode('utf-8'))
" "$password"
    else
        print_error "Python3 with bcrypt is required for password hashing"
        print_error "Install with: pip3 install bcrypt"
        exit 1
    fi
}

# Function to check if schema exists
schema_exists() {
    local schema_name="$1"
    local result

    result=$(psql "$DATABASE_URL" -t -c "SELECT COUNT(*) FROM information_schema.schemata WHERE schema_name = '$schema_name';" 2>/dev/null | xargs)

    if [[ "$result" == "1" ]]; then
        return 0
    else
        return 1
    fi
}

# Function to check if tenant exists
tenant_exists() {
    local tenant_name="$1"
    local result

    result=$(psql "$DATABASE_URL" -t -c "SELECT COUNT(*) FROM public.tenants WHERE name = '$tenant_name';" 2>/dev/null | xargs)

    if [[ "$result" -gt "0" ]]; then
        return 0
    else
        return 1
    fi
}

# Function to create tenant record in public schema
create_tenant_record() {
    local tenant_id="$1"
    local tenant_name="$2"
    local schema_name="$3"

    print_status "Creating tenant record in public.tenants..."

    psql "$DATABASE_URL" -c "
        INSERT INTO public.tenants (id, name, schema_name, status, created_at, updated_at)
        VALUES (
            '$tenant_id'::uuid,
            '$tenant_name',
            '$schema_name',
            'active',
            NOW(),
            NOW()
        );
    "
}

# Function to create tenant schema
create_tenant_schema() {
    local schema_name="$1"
    local tenant_id="$2"
    local tenant_name="$3"
    local tenant_domain="$4"
    local admin_email="$5"
    local admin_password_hash="$6"
    local admin_user_id="$7"

    print_status "Creating tenant schema: $schema_name"

    # Read and process schema template
    local template_file="migrations/002_tenant_schema_template.sql"
    if [[ ! -f "$template_file" ]]; then
        print_error "Schema template not found: $template_file"
        exit 1
    fi

    # Replace variables in template
    local processed_sql
    processed_sql=$(sed "s/{TENANT_SCHEMA}/$schema_name/g" "$template_file")

    # Execute schema creation
    echo "$processed_sql" | psql "$DATABASE_URL"

    print_status "Tenant schema created successfully"
}

# Function to seed default data
seed_tenant_data() {
    local schema_name="$1"
    local tenant_id="$2"
    local tenant_name="$3"
    local tenant_domain="$4"
    local admin_email="$5"
    local admin_password_hash="$6"
    local admin_user_id="$7"

    print_status "Seeding default data for tenant..."

    # Process and execute roles seed
    local roles_file="migrations/seeds/001_default_roles.sql"
    if [[ -f "$roles_file" ]]; then
        local processed_sql
        processed_sql=$(sed "s/{TENANT_SCHEMA}/$schema_name/g" "$roles_file")
        echo "$processed_sql" | psql "$DATABASE_URL"
        print_status "Default roles created"
    fi

    # Process and execute reference data seed
    local ref_data_file="migrations/seeds/002_reference_data.sql"
    if [[ -f "$ref_data_file" ]]; then
        local processed_sql
        processed_sql=$(sed "s/{TENANT_SCHEMA}/$schema_name/g; s/{TENANT_NAME}/$tenant_name/g; s/{TENANT_DOMAIN}/$tenant_domain/g" "$ref_data_file")
        echo "$processed_sql" | psql "$DATABASE_URL"
        print_status "Reference data created"
    fi

    # Process and execute admin user seed
    local admin_file="migrations/seeds/003_admin_user.sql"
    if [[ -f "$admin_file" ]]; then
        local processed_sql
        processed_sql=$(sed "s/{TENANT_SCHEMA}/$schema_name/g; s/{TENANT_ID}/$tenant_id/g; s/{ADMIN_EMAIL}/$admin_email/g; s/{ADMIN_PASSWORD_HASH}/$admin_password_hash/g; s/{ADMIN_USER_ID}/$admin_user_id/g" "$admin_file")
        echo "$processed_sql" | psql "$DATABASE_URL"
        print_status "Admin user created"
    fi

    print_status "Default data seeded successfully"
}

# Function to verify tenant creation
verify_tenant() {
    local schema_name="$1"
    local tenant_id="$2"

    print_status "Verifying tenant creation..."

    # Check if schema exists
    if ! schema_exists "$schema_name"; then
        print_error "Schema verification failed: $schema_name not found"
        return 1
    fi

    # Check if tenant record exists
    local tenant_count
    tenant_count=$(psql "$DATABASE_URL" -t -c "SELECT COUNT(*) FROM public.tenants WHERE id = '$tenant_id';" | xargs)

    if [[ "$tenant_count" != "1" ]]; then
        print_error "Tenant record verification failed"
        return 1
    fi

    # Check if roles were created
    local roles_count
    roles_count=$(psql "$DATABASE_URL" -t -c "SELECT COUNT(*) FROM $schema_name.roles;" | xargs)

    if [[ "$roles_count" -lt "5" ]]; then
        print_error "Roles verification failed: expected at least 5 roles, found $roles_count"
        return 1
    fi

    # Check if admin user was created
    local admin_count
    admin_count=$(psql "$DATABASE_URL" -t -c "SELECT COUNT(*) FROM $schema_name.users WHERE username = 'admin';" | xargs)

    if [[ "$admin_count" != "1" ]]; then
        print_error "Admin user verification failed"
        return 1
    fi

    print_status "✅ Tenant verification completed successfully"
    return 0
}

# Function to print tenant summary
print_tenant_summary() {
    local tenant_id="$1"
    local tenant_name="$2"
    local schema_name="$3"
    local admin_email="$4"
    local admin_password="$5"
    local tenant_domain="$6"

    echo ""
    echo -e "${GREEN}============================================${NC}"
    echo -e "${GREEN}    Tenant Created Successfully!${NC}"
    echo -e "${GREEN}============================================${NC}"
    echo ""
    echo -e "${BLUE}📊 Tenant Information:${NC}"
    echo "   • Tenant ID: $tenant_id"
    echo "   • Tenant Name: $tenant_name"
    echo "   • Database Schema: $schema_name"
    echo "   • Domain: ${tenant_domain:-"Not specified"}"
    echo ""
    echo -e "${BLUE}👤 Admin User:${NC}"
    echo "   • Email: $admin_email"
    echo "   • Username: admin"
    echo "   • Password: $admin_password"
    echo "   • Role: Super Administrator"
    echo ""
    echo -e "${BLUE}🔧 Database Access:${NC}"
    echo "   • Schema: $schema_name"
    echo "   • Connect: psql \"$DATABASE_URL\" -c \"SET search_path TO $schema_name, public;\""
    echo ""
    echo -e "${BLUE}📁 Created Resources:${NC}"
    echo "   • Database schema with complete table structure"
    echo "   • Default roles and permissions"
    echo "   • System settings and configuration"
    echo "   • Admin user account"
    echo ""
    echo -e "${YELLOW}⚠️  Important Notes:${NC}"
    echo "   • Change the admin password immediately after first login"
    echo "   • Review and customize system settings as needed"
    echo "   • Configure email and notification settings"
    echo "   • Setup additional users and roles as required"
    echo ""
    echo -e "${BLUE}🌐 Next Steps:${NC}"
    echo "   1. Test admin login via API"
    echo "   2. Configure tenant-specific settings"
    echo "   3. Create additional users"
    echo "   4. Setup integrations if needed"
    echo ""
}

# Parse command line arguments
TENANT_NAME=""
ADMIN_EMAIL=""
ADMIN_PASSWORD=""
TENANT_DOMAIN=""
SCHEMA_NAME=""

while [[ $# -gt 0 ]]; do
    case $1 in
        -n|--name)
            TENANT_NAME="$2"
            shift 2
            ;;
        -e|--email)
            ADMIN_EMAIL="$2"
            shift 2
            ;;
        -p|--password)
            ADMIN_PASSWORD="$2"
            shift 2
            ;;
        -d|--domain)
            TENANT_DOMAIN="$2"
            shift 2
            ;;
        -s|--schema)
            SCHEMA_NAME="$2"
            shift 2
            ;;
        -h|--help)
            show_usage
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Main execution
main() {
    print_header

    # Validate required parameters
    if [[ -z "$TENANT_NAME" ]]; then
        print_error "Tenant name is required (use -n or --name)"
        show_usage
        exit 1
    fi

    if [[ -z "$ADMIN_EMAIL" ]]; then
        print_error "Admin email is required (use -e or --email)"
        show_usage
        exit 1
    fi

    # Validate inputs
    validate_tenant_name "$TENANT_NAME"
    validate_email "$ADMIN_EMAIL"

    # Check database connection
    if [[ -z "${DATABASE_URL:-}" ]]; then
        print_error "DATABASE_URL environment variable is required"
        exit 1
    fi

    if ! psql "$DATABASE_URL" -c "SELECT 1;" &>/dev/null; then
        print_error "Cannot connect to database: $DATABASE_URL"
        exit 1
    fi

    # Generate missing values
    if [[ -z "$SCHEMA_NAME" ]]; then
        SCHEMA_NAME=$(generate_schema_name "$TENANT_NAME")
    fi

    if [[ -z "$TENANT_DOMAIN" ]]; then
        TENANT_DOMAIN="${SCHEMA_NAME}.erp-system.com"
    fi

    if [[ -z "$ADMIN_PASSWORD" ]]; then
        print_status "Generating secure admin password..."
        ADMIN_PASSWORD=$(generate_password)
    fi

    # Generate IDs
    TENANT_ID=$(uuidgen | tr '[:upper:]' '[:lower:]')
    ADMIN_USER_ID=$(uuidgen | tr '[:upper:]' '[:lower:]')

    print_status "Tenant Details:"
    print_status "  Name: $TENANT_NAME"
    print_status "  Schema: $SCHEMA_NAME"
    print_status "  Admin Email: $ADMIN_EMAIL"
    print_status "  Domain: $TENANT_DOMAIN"

    # Check if tenant already exists
    if tenant_exists "$TENANT_NAME"; then
        print_error "Tenant with name '$TENANT_NAME' already exists"
        exit 1
    fi

    if schema_exists "$SCHEMA_NAME"; then
        print_error "Schema '$SCHEMA_NAME' already exists"
        exit 1
    fi

    # Hash the admin password
    print_status "Hashing admin password..."
    ADMIN_PASSWORD_HASH=$(hash_password "$ADMIN_PASSWORD")

    # Create tenant
    print_status "Creating tenant: $TENANT_NAME"

    # Step 1: Create tenant record
    create_tenant_record "$TENANT_ID" "$TENANT_NAME" "$SCHEMA_NAME"

    # Step 2: Create tenant schema
    create_tenant_schema "$SCHEMA_NAME" "$TENANT_ID" "$TENANT_NAME" "$TENANT_DOMAIN" "$ADMIN_EMAIL" "$ADMIN_PASSWORD_HASH" "$ADMIN_USER_ID"

    # Step 3: Seed default data
    seed_tenant_data "$SCHEMA_NAME" "$TENANT_ID" "$TENANT_NAME" "$TENANT_DOMAIN" "$ADMIN_EMAIL" "$ADMIN_PASSWORD_HASH" "$ADMIN_USER_ID"

    # Step 4: Verify creation
    if verify_tenant "$SCHEMA_NAME" "$TENANT_ID"; then
        print_tenant_summary "$TENANT_ID" "$TENANT_NAME" "$SCHEMA_NAME" "$ADMIN_EMAIL" "$ADMIN_PASSWORD" "$TENANT_DOMAIN"
        print_status "🎉 Tenant creation completed successfully!"
    else
        print_error "Tenant verification failed"
        exit 1
    fi
}

# Handle script interruption
trap 'print_error "Tenant creation interrupted!"; exit 1' INT TERM

# Check for required dependencies
if ! command -v psql &> /dev/null; then
    print_error "PostgreSQL client (psql) is required"
    exit 1
fi

if ! command -v uuidgen &> /dev/null; then
    print_error "uuidgen is required"
    exit 1
fi

if ! command -v python3 &> /dev/null; then
    print_error "Python3 is required for password hashing"
    exit 1
fi

if ! python3 -c "import bcrypt" &> /dev/null; then
    print_error "Python bcrypt library is required. Install with: pip3 install bcrypt"
    exit 1
fi

# Run main function
main "$@"