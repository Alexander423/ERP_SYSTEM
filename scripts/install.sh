#!/bin/bash

# ERP System - Production Installation Script
# Automatically sets up the complete ERP system on a fresh server
# Usage: curl -sSL https://install.erp-system.com | bash
# Or: ./scripts/install.sh [environment]

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
ENVIRONMENT=${1:-production}
INSTALL_DIR="/opt/erp-system"
SERVICE_USER="erp"
LOG_FILE="/var/log/erp-install.log"

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1" | tee -a "$LOG_FILE"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1" | tee -a "$LOG_FILE"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1" | tee -a "$LOG_FILE"
}

print_header() {
    echo -e "${BLUE}===========================================${NC}"
    echo -e "${BLUE}    ERP System - Production Installer${NC}"
    echo -e "${BLUE}    Environment: $ENVIRONMENT${NC}"
    echo -e "${BLUE}===========================================${NC}"
}

# Function to check if running as root
check_root() {
    if [[ $EUID -ne 0 ]]; then
        print_error "This script must be run as root (use sudo)"
        exit 1
    fi
}

# Function to detect OS
detect_os() {
    if [[ -f /etc/redhat-release ]]; then
        OS="rhel"
        PACKAGE_MANAGER="yum"
    elif [[ -f /etc/debian_version ]]; then
        OS="debian"
        PACKAGE_MANAGER="apt"
    elif [[ -f /etc/arch-release ]]; then
        OS="arch"
        PACKAGE_MANAGER="pacman"
    else
        print_error "Unsupported operating system"
        exit 1
    fi
    print_status "Detected OS: $OS"
}

# Function to install system dependencies
install_dependencies() {
    print_status "Installing system dependencies..."

    case $OS in
        "debian")
            apt update
            apt install -y curl wget gnupg lsb-release ca-certificates \
                postgresql-client redis-tools docker.io docker-compose \
                nginx certbot python3-certbot-nginx jq
            ;;
        "rhel")
            yum update -y
            yum install -y curl wget gnupg ca-certificates \
                postgresql docker docker-compose \
                nginx certbot python3-certbot-nginx jq
            ;;
        "arch")
            pacman -Syu --noconfirm
            pacman -S --noconfirm curl wget gnupg ca-certificates \
                postgresql-libs docker docker-compose \
                nginx certbot certbot-nginx jq
            ;;
    esac

    # Start and enable Docker
    systemctl start docker
    systemctl enable docker

    print_status "System dependencies installed successfully"
}

# Function to create service user
create_service_user() {
    print_status "Creating service user: $SERVICE_USER"

    if ! id "$SERVICE_USER" &>/dev/null; then
        useradd -r -m -s /bin/bash "$SERVICE_USER"
        usermod -aG docker "$SERVICE_USER"
        print_status "Service user created successfully"
    else
        print_warning "Service user $SERVICE_USER already exists"
    fi
}

# Function to setup directories
setup_directories() {
    print_status "Setting up directories..."

    mkdir -p "$INSTALL_DIR"/{config,data,logs,backups,ssl}
    mkdir -p /var/log/erp-system
    mkdir -p /etc/erp-system

    chown -R "$SERVICE_USER:$SERVICE_USER" "$INSTALL_DIR"
    chown -R "$SERVICE_USER:$SERVICE_USER" /var/log/erp-system
    chown -R "$SERVICE_USER:$SERVICE_USER" /etc/erp-system

    print_status "Directories created successfully"
}

# Function to download and install ERP system
install_erp_system() {
    print_status "Installing ERP system..."

    cd "$INSTALL_DIR"

    # Download latest release (or clone repository)
    if [[ -n "${ERP_DOWNLOAD_URL:-}" ]]; then
        wget -O erp-system.tar.gz "$ERP_DOWNLOAD_URL"
        tar -xzf erp-system.tar.gz --strip-components=1
        rm erp-system.tar.gz
    else
        # Fallback: clone repository
        if command -v git &> /dev/null; then
            git clone https://github.com/your-org/erp-system.git .
        else
            print_error "Neither ERP_DOWNLOAD_URL is set nor git is available"
            exit 1
        fi
    fi

    chown -R "$SERVICE_USER:$SERVICE_USER" "$INSTALL_DIR"
    print_status "ERP system installed successfully"
}

# Function to generate secure configuration
generate_config() {
    print_status "Generating secure configuration..."

    # Generate secure secrets
    JWT_SECRET=$(openssl rand -base64 64)
    ENCRYPTION_KEY=$(openssl rand -base64 32)
    DB_PASSWORD=$(openssl rand -base64 32 | tr -d "=+/" | cut -c1-25)
    REDIS_PASSWORD=$(openssl rand -base64 32 | tr -d "=+/" | cut -c1-25)

    # Determine external IP
    EXTERNAL_IP=$(curl -s ifconfig.me || curl -s ipinfo.io/ip || echo "localhost")

    cat > /etc/erp-system/config.toml <<EOF
# ERP System Configuration - Generated $(date)
# Environment: $ENVIRONMENT

[server]
host = "0.0.0.0"
port = 8080
workers = 4

[database]
url = "postgresql://erp_admin:${DB_PASSWORD}@localhost:5432/erp_main"
max_connections = 50
min_connections = 5

[redis]
url = "redis://:${REDIS_PASSWORD}@localhost:6379"
max_connections = 20

[jwt]
secret = "${JWT_SECRET}"
access_token_expiry = 3600  # 1 hour
refresh_token_expiry = 604800  # 7 days

[security]
aes_encryption_key = "${ENCRYPTION_KEY}"
bcrypt_cost = 12
session_timeout = 1800  # 30 minutes

[cors]
allowed_origins = ["https://${EXTERNAL_IP}", "https://app.erp-system.com"]
allowed_methods = ["GET", "POST", "PUT", "DELETE", "OPTIONS"]
allowed_headers = ["Content-Type", "Authorization", "X-Requested-With"]
expose_headers = ["X-Request-ID"]
allow_credentials = true
max_age = 3600

[email]
provider = "smtp"
smtp_host = "localhost"
smtp_port = 587
smtp_username = "noreply@erp-system.com"
smtp_password = "change_me_in_production"
from_address = "noreply@erp-system.com"
from_name = "ERP System"

[logging]
level = "info"
format = "json"
file = "/var/log/erp-system/app.log"
max_file_size = "100MB"
max_files = 10

[features]
registration_enabled = false
email_verification_required = true
two_factor_auth_enabled = true
audit_logging = true
rate_limiting = true

[monitoring]
metrics_enabled = true
health_check_interval = 30
performance_monitoring = true
EOF

    # Store passwords securely
    cat > /etc/erp-system/secrets.env <<EOF
# Generated secrets - Keep secure!
DB_PASSWORD=${DB_PASSWORD}
REDIS_PASSWORD=${REDIS_PASSWORD}
JWT_SECRET=${JWT_SECRET}
ENCRYPTION_KEY=${ENCRYPTION_KEY}
EOF

    chmod 600 /etc/erp-system/secrets.env
    chown "$SERVICE_USER:$SERVICE_USER" /etc/erp-system/config.toml
    chown "$SERVICE_USER:$SERVICE_USER" /etc/erp-system/secrets.env

    print_status "Configuration generated successfully"
}

# Function to setup PostgreSQL
setup_postgresql() {
    print_status "Setting up PostgreSQL..."

    # Source the secrets
    source /etc/erp-system/secrets.env

    # Start PostgreSQL with Docker
    docker run -d \
        --name erp-postgres \
        --restart unless-stopped \
        -e POSTGRES_USER=erp_admin \
        -e POSTGRES_PASSWORD="$DB_PASSWORD" \
        -e POSTGRES_DB=erp_main \
        -v "$INSTALL_DIR/data/postgres:/var/lib/postgresql/data" \
        -v "$INSTALL_DIR/migrations:/docker-entrypoint-initdb.d" \
        -p 5432:5432 \
        postgres:16-alpine

    # Wait for PostgreSQL to be ready
    print_status "Waiting for PostgreSQL to be ready..."
    for i in {1..30}; do
        if docker exec erp-postgres pg_isready -U erp_admin -d erp_main; then
            print_status "PostgreSQL is ready"
            break
        fi
        sleep 2
    done

    print_status "PostgreSQL setup completed"
}

# Function to setup Redis
setup_redis() {
    print_status "Setting up Redis..."

    source /etc/erp-system/secrets.env

    docker run -d \
        --name erp-redis \
        --restart unless-stopped \
        -e REDIS_PASSWORD="$REDIS_PASSWORD" \
        -v "$INSTALL_DIR/data/redis:/data" \
        -p 6379:6379 \
        redis:7-alpine \
        redis-server --appendonly yes --requirepass "$REDIS_PASSWORD"

    print_status "Redis setup completed"
}

# Function to run database migrations
run_migrations() {
    print_status "Running database migrations..."

    cd "$INSTALL_DIR"

    # Install Rust if not present
    if ! command -v cargo &> /dev/null; then
        print_status "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source ~/.cargo/env
    fi

    # Install sqlx-cli
    if ! command -v sqlx &> /dev/null; then
        cargo install sqlx-cli --no-default-features --features postgres
    fi

    # Source database URL
    source /etc/erp-system/secrets.env
    export DATABASE_URL="postgresql://erp_admin:${DB_PASSWORD}@localhost:5432/erp_main"

    # Run migrations
    sqlx migrate run --source migrations

    print_status "Database migrations completed"
}

# Function to build and install the application
build_application() {
    print_status "Building ERP application..."

    cd "$INSTALL_DIR"

    # Build in release mode
    cargo build --release --bin erp-server

    # Install binary
    cp target/release/erp-server /usr/local/bin/
    chmod +x /usr/local/bin/erp-server

    print_status "Application built and installed successfully"
}

# Function to create systemd service
create_systemd_service() {
    print_status "Creating systemd service..."

    cat > /etc/systemd/system/erp-system.service <<EOF
[Unit]
Description=ERP System API Server
After=network.target postgresql.service redis.service
Wants=postgresql.service redis.service
Documentation=https://docs.erp-system.com

[Service]
Type=simple
User=$SERVICE_USER
Group=$SERVICE_USER
WorkingDirectory=$INSTALL_DIR
Environment=RUST_LOG=info
EnvironmentFile=/etc/erp-system/secrets.env
ExecStart=/usr/local/bin/erp-server --config /etc/erp-system/config.toml
ExecReload=/bin/kill -HUP \$MAINPID
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal
SyslogIdentifier=erp-system

# Security settings
NoNewPrivileges=yes
PrivateTmp=yes
ProtectSystem=strict
ProtectHome=yes
ReadWritePaths=$INSTALL_DIR /var/log/erp-system /tmp

# Resource limits
LimitNOFILE=1048576
LimitNPROC=1048576

[Install]
WantedBy=multi-user.target
EOF

    systemctl daemon-reload
    systemctl enable erp-system

    print_status "Systemd service created successfully"
}

# Function to setup nginx reverse proxy
setup_nginx() {
    print_status "Setting up Nginx reverse proxy..."

    cat > /etc/nginx/sites-available/erp-system <<EOF
# ERP System - Nginx Configuration
upstream erp_backend {
    server 127.0.0.1:8080;
    keepalive 32;
}

# Rate limiting
limit_req_zone \$binary_remote_addr zone=api:10m rate=10r/s;
limit_req_zone \$binary_remote_addr zone=auth:10m rate=5r/s;

server {
    listen 80;
    server_name _;

    # Security headers
    add_header X-Frame-Options DENY always;
    add_header X-Content-Type-Options nosniff always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;
    add_header Content-Security-Policy "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'" always;

    # Health check (no rate limiting)
    location /health {
        proxy_pass http://erp_backend;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }

    # Auth endpoints (stricter rate limiting)
    location /api/v1/auth {
        limit_req zone=auth burst=10 nodelay;
        proxy_pass http://erp_backend;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        proxy_set_header Connection "";
        proxy_http_version 1.1;
    }

    # API endpoints (general rate limiting)
    location /api/ {
        limit_req zone=api burst=20 nodelay;
        proxy_pass http://erp_backend;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        proxy_set_header Connection "";
        proxy_http_version 1.1;
    }

    # Swagger UI
    location /swagger-ui {
        proxy_pass http://erp_backend;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }

    # Static files (if any)
    location /static/ {
        root $INSTALL_DIR/public;
        expires 1y;
        add_header Cache-Control "public, immutable";
    }

    # Redirect all other traffic
    location / {
        return 301 https://docs.erp-system.com;
    }
}
EOF

    # Enable site
    ln -sf /etc/nginx/sites-available/erp-system /etc/nginx/sites-enabled/
    rm -f /etc/nginx/sites-enabled/default

    # Test configuration
    nginx -t
    systemctl enable nginx
    systemctl restart nginx

    print_status "Nginx configured successfully"
}

# Function to setup SSL with Let's Encrypt
setup_ssl() {
    if [[ "$ENVIRONMENT" == "production" ]]; then
        print_status "Setting up SSL certificates..."

        read -p "Enter your domain name (e.g., api.yourdomain.com): " DOMAIN_NAME
        read -p "Enter your email for Let's Encrypt: " EMAIL

        if [[ -n "$DOMAIN_NAME" && -n "$EMAIL" ]]; then
            certbot --nginx -d "$DOMAIN_NAME" --email "$EMAIL" --agree-tos --non-interactive
            print_status "SSL certificate installed successfully"
        else
            print_warning "Skipping SSL setup - domain name or email not provided"
        fi
    else
        print_warning "Skipping SSL setup for non-production environment"
    fi
}

# Function to setup monitoring and logging
setup_monitoring() {
    print_status "Setting up monitoring and logging..."

    # Setup log rotation
    cat > /etc/logrotate.d/erp-system <<EOF
/var/log/erp-system/*.log {
    daily
    missingok
    rotate 52
    compress
    delaycompress
    notifempty
    create 0644 $SERVICE_USER $SERVICE_USER
    postrotate
        systemctl reload erp-system
    endscript
}
EOF

    # Create monitoring script
    cat > "$INSTALL_DIR/scripts/health-check.sh" <<'EOF'
#!/bin/bash
# ERP System Health Check

HEALTH_URL="http://localhost:8080/health"
LOG_FILE="/var/log/erp-system/health-check.log"

timestamp() {
    date '+%Y-%m-%d %H:%M:%S'
}

# Check API health
if curl -f -s "$HEALTH_URL" > /dev/null; then
    echo "$(timestamp) - Health check: OK" >> "$LOG_FILE"
else
    echo "$(timestamp) - Health check: FAILED" >> "$LOG_FILE"
    # Restart service if unhealthy
    systemctl restart erp-system
    echo "$(timestamp) - Service restarted" >> "$LOG_FILE"
fi
EOF

    chmod +x "$INSTALL_DIR/scripts/health-check.sh"
    chown "$SERVICE_USER:$SERVICE_USER" "$INSTALL_DIR/scripts/health-check.sh"

    # Add cron job for health checks
    echo "*/5 * * * * $SERVICE_USER $INSTALL_DIR/scripts/health-check.sh" >> /etc/crontab

    print_status "Monitoring and logging configured"
}

# Function to setup backups
setup_backups() {
    print_status "Setting up automated backups..."

    cat > "$INSTALL_DIR/scripts/backup.sh" <<'EOF'
#!/bin/bash
# ERP System Backup Script

source /etc/erp-system/secrets.env
BACKUP_DIR="/opt/erp-system/backups"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RETENTION_DAYS=30

# Create backup directory
mkdir -p "$BACKUP_DIR"

# Database backup
echo "Starting database backup..."
docker exec erp-postgres pg_dump -U erp_admin erp_main | gzip > "$BACKUP_DIR/db_backup_$TIMESTAMP.sql.gz"

# Configuration backup
tar -czf "$BACKUP_DIR/config_backup_$TIMESTAMP.tar.gz" /etc/erp-system/

# Remove old backups
find "$BACKUP_DIR" -name "*.gz" -mtime +$RETENTION_DAYS -delete

echo "Backup completed: $TIMESTAMP"
EOF

    chmod +x "$INSTALL_DIR/scripts/backup.sh"
    chown "$SERVICE_USER:$SERVICE_USER" "$INSTALL_DIR/scripts/backup.sh"

    # Add daily backup cron job
    echo "0 2 * * * $SERVICE_USER $INSTALL_DIR/scripts/backup.sh" >> /etc/crontab

    print_status "Backup system configured"
}

# Function to start services
start_services() {
    print_status "Starting ERP system services..."

    systemctl start erp-system
    systemctl start nginx

    # Wait for service to be ready
    sleep 5

    # Check if service is running
    if systemctl is-active --quiet erp-system; then
        print_status "✅ ERP System is running successfully!"
    else
        print_error "❌ Failed to start ERP System"
        print_error "Check logs: journalctl -u erp-system -f"
        exit 1
    fi
}

# Function to print installation summary
print_summary() {
    local external_ip=$(curl -s ifconfig.me || echo "YOUR_SERVER_IP")

    echo ""
    echo -e "${GREEN}===========================================${NC}"
    echo -e "${GREEN}    ERP System Installation Complete!${NC}"
    echo -e "${GREEN}===========================================${NC}"
    echo ""
    echo -e "${BLUE}📊 System Information:${NC}"
    echo "   • Environment: $ENVIRONMENT"
    echo "   • Installation Directory: $INSTALL_DIR"
    echo "   • Service User: $SERVICE_USER"
    echo "   • Configuration: /etc/erp-system/"
    echo ""
    echo -e "${BLUE}🌐 Access URLs:${NC}"
    echo "   • API Health: http://$external_ip/health"
    echo "   • API Documentation: http://$external_ip/swagger-ui"
    echo "   • API Base URL: http://$external_ip/api/v1/"
    echo ""
    echo -e "${BLUE}🔧 Management Commands:${NC}"
    echo "   • Start service: systemctl start erp-system"
    echo "   • Stop service: systemctl stop erp-system"
    echo "   • View logs: journalctl -u erp-system -f"
    echo "   • Check status: systemctl status erp-system"
    echo ""
    echo -e "${BLUE}📁 Important Files:${NC}"
    echo "   • Configuration: /etc/erp-system/config.toml"
    echo "   • Secrets: /etc/erp-system/secrets.env"
    echo "   • Application logs: /var/log/erp-system/"
    echo "   • Backups: $INSTALL_DIR/backups/"
    echo ""
    echo -e "${YELLOW}⚠️  Security Notes:${NC}"
    echo "   • Generated passwords are stored in /etc/erp-system/secrets.env"
    echo "   • Configure SSL certificates for production use"
    echo "   • Review and update CORS settings in config.toml"
    echo "   • Change default email settings in config.toml"
    echo ""
    echo -e "${BLUE}📚 Next Steps:${NC}"
    echo "   1. Test the API: curl http://$external_ip/health"
    echo "   2. Review configuration files"
    echo "   3. Setup SSL certificates for production"
    echo "   4. Configure external email provider"
    echo "   5. Setup monitoring and alerting"
    echo ""
}

# Main installation flow
main() {
    print_header

    # Pre-installation checks
    check_root
    detect_os

    # Create log file
    mkdir -p "$(dirname "$LOG_FILE")"
    touch "$LOG_FILE"

    print_status "Starting ERP System installation..."
    print_status "Logs will be written to: $LOG_FILE"

    # Installation steps
    install_dependencies
    create_service_user
    setup_directories
    install_erp_system
    generate_config
    setup_postgresql
    setup_redis
    run_migrations
    build_application
    create_systemd_service
    setup_nginx
    setup_ssl
    setup_monitoring
    setup_backups
    start_services

    print_summary

    print_status "🎉 Installation completed successfully!"
}

# Handle script interruption
trap 'print_error "Installation interrupted!"; exit 1' INT TERM

# Run main function
main "$@"