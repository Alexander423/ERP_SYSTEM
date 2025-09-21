#!/bin/bash

# ERP System - Production Security Hardening Script
# Applies security best practices for production deployments

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_status() {
    echo -e "${GREEN}[SECURITY]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_header() {
    echo -e "${BLUE}================================================${NC}"
    echo -e "${BLUE}    ERP System - Production Security Setup${NC}"
    echo -e "${BLUE}================================================${NC}"
}

# Function to check if running as root
check_root() {
    if [[ $EUID -ne 0 ]]; then
        print_error "This script must be run as root (use sudo)"
        exit 1
    fi
}

# Function to setup firewall
setup_firewall() {
    print_status "Configuring firewall (UFW)..."

    # Install UFW if not present
    if ! command -v ufw &> /dev/null; then
        apt update && apt install -y ufw
    fi

    # Reset to defaults
    ufw --force reset

    # Default policies
    ufw default deny incoming
    ufw default allow outgoing

    # Allow SSH (be careful!)
    ufw allow ssh

    # Allow HTTP/HTTPS
    ufw allow 80/tcp
    ufw allow 443/tcp

    # Allow only specific database access (local only)
    ufw allow from 127.0.0.1 to any port 5432
    ufw allow from 127.0.0.1 to any port 6379

    # Enable firewall
    ufw --force enable

    print_status "Firewall configured successfully"
}

# Function to secure SSH
secure_ssh() {
    print_status "Securing SSH configuration..."

    # Backup original config
    cp /etc/ssh/sshd_config /etc/ssh/sshd_config.backup

    # Apply security settings
    cat >> /etc/ssh/sshd_config.d/99-erp-security.conf <<EOF
# ERP System SSH Security Configuration

# Disable root login
PermitRootLogin no

# Use protocol 2 only
Protocol 2

# Disable password authentication (use key-based only)
PasswordAuthentication no
PermitEmptyPasswords no
ChallengeResponseAuthentication no

# Disable X11 forwarding
X11Forwarding no

# Disable unused features
AllowAgentForwarding no
AllowTcpForwarding no
GatewayPorts no

# Set login grace time
LoginGraceTime 30

# Limit max authentication attempts
MaxAuthTries 3

# Limit concurrent sessions
MaxSessions 3

# Only allow specific users
AllowUsers erp

# Use strong ciphers only
Ciphers aes256-gcm@openssh.com,aes128-gcm@openssh.com,aes256-ctr,aes192-ctr,aes128-ctr
MACs hmac-sha2-256-etm@openssh.com,hmac-sha2-512-etm@openssh.com,hmac-sha2-256,hmac-sha2-512
KexAlgorithms diffie-hellman-group16-sha512,diffie-hellman-group18-sha512,ecdh-sha2-nistp521,ecdh-sha2-nistp384,ecdh-sha2-nistp256
EOF

    # Restart SSH service
    systemctl restart sshd

    print_status "SSH secured successfully"
}

# Function to setup fail2ban
setup_fail2ban() {
    print_status "Installing and configuring Fail2Ban..."

    # Install fail2ban
    apt install -y fail2ban

    # Configure fail2ban for ERP system
    cat > /etc/fail2ban/jail.local <<EOF
[DEFAULT]
# Ban duration (24 hours)
bantime = 86400

# Find time window (10 minutes)
findtime = 600

# Max retries before ban
maxretry = 3

# Email notifications
destemail = admin@yourdomain.com
sender = fail2ban@yourdomain.com
mta = sendmail

# Action to take
action = %(action_mwl)s

[sshd]
enabled = true
port = ssh
filter = sshd
logpath = /var/log/auth.log
maxretry = 3

[nginx-http-auth]
enabled = true
filter = nginx-http-auth
port = http,https
logpath = /var/log/nginx/error.log
maxretry = 5

[nginx-limit-req]
enabled = true
filter = nginx-limit-req
port = http,https
logpath = /var/log/nginx/error.log
maxretry = 10

[erp-api]
enabled = true
filter = erp-api
port = http,https
logpath = /var/log/erp-system/app.log
maxretry = 10
findtime = 300
bantime = 3600
EOF

    # Create custom filter for ERP API
    cat > /etc/fail2ban/filter.d/erp-api.conf <<EOF
[Definition]
failregex = ^.*\[ERROR\].*Authentication failed.*client_ip="<HOST>".*$
            ^.*\[WARN\].*Rate limit exceeded.*client_ip="<HOST>".*$
            ^.*\[ERROR\].*Invalid token.*client_ip="<HOST>".*$

ignoreregex =
EOF

    # Create nginx filters
    cat > /etc/fail2ban/filter.d/nginx-http-auth.conf <<EOF
[Definition]
failregex = ^ \[error\] \d+#\d+: \*\d+ user "\S+":? password mismatch, client: <HOST>, server: \S+, request: "\S+ \S+ HTTP/\d+\.\d+", host: "\S+"\s*$
            ^ \[error\] \d+#\d+: \*\d+ no user/password was provided for basic authentication, client: <HOST>, server: \S+, request: "\S+ \S+ HTTP/\d+\.\d+", host: "\S+"\s*$

ignoreregex =
EOF

    cat > /etc/fail2ban/filter.d/nginx-limit-req.conf <<EOF
[Definition]
failregex = ^\s*\[error\] \d+#\d+: \*\d+ limiting requests, excess: \S+ by zone "\S+", client: <HOST>,.*$

ignoreregex =
EOF

    # Start and enable fail2ban
    systemctl enable fail2ban
    systemctl start fail2ban

    print_status "Fail2Ban configured successfully"
}

# Function to secure file permissions
secure_file_permissions() {
    print_status "Securing file permissions..."

    # Secure configuration files
    chmod 600 /etc/erp-system/secrets.env
    chmod 644 /etc/erp-system/config.toml
    chown erp:erp /etc/erp-system/*

    # Secure log directories
    chmod 750 /var/log/erp-system
    chown erp:erp /var/log/erp-system

    # Secure installation directory
    chmod 750 /opt/erp-system
    chown -R erp:erp /opt/erp-system

    # Secure backup directory
    chmod 700 /opt/erp-system/backups
    chown erp:erp /opt/erp-system/backups

    # Remove world-readable permissions from sensitive files
    find /opt/erp-system -type f -name "*.toml" -exec chmod 640 {} \;
    find /opt/erp-system -type f -name "*.env" -exec chmod 600 {} \;

    print_status "File permissions secured successfully"
}

# Function to setup system limits
setup_system_limits() {
    print_status "Configuring system limits..."

    cat >> /etc/security/limits.conf <<EOF
# ERP System Limits
erp soft nofile 65536
erp hard nofile 65536
erp soft nproc 32768
erp hard nproc 32768
EOF

    # Set kernel parameters for better performance and security
    cat >> /etc/sysctl.conf <<EOF
# ERP System Kernel Parameters

# Network Security
net.ipv4.ip_forward = 0
net.ipv4.conf.all.send_redirects = 0
net.ipv4.conf.default.send_redirects = 0
net.ipv4.conf.all.accept_redirects = 0
net.ipv4.conf.default.accept_redirects = 0
net.ipv4.conf.all.accept_source_route = 0
net.ipv4.conf.default.accept_source_route = 0
net.ipv4.conf.all.log_martians = 1
net.ipv4.conf.default.log_martians = 1
net.ipv4.icmp_echo_ignore_broadcasts = 1
net.ipv4.icmp_ignore_bogus_error_responses = 1
net.ipv4.tcp_syncookies = 1

# Performance
net.core.rmem_max = 16777216
net.core.wmem_max = 16777216
net.ipv4.tcp_rmem = 4096 87380 16777216
net.ipv4.tcp_wmem = 4096 65536 16777216
net.core.netdev_max_backlog = 5000
net.ipv4.tcp_max_syn_backlog = 8192

# File handles
fs.file-max = 2097152
EOF

    # Apply sysctl changes
    sysctl -p

    print_status "System limits configured successfully"
}

# Function to setup log monitoring
setup_log_monitoring() {
    print_status "Setting up log monitoring..."

    # Install logwatch
    apt install -y logwatch

    # Configure logwatch for ERP system
    cat > /etc/logwatch/conf/services/erp-system.conf <<EOF
Title = "ERP System"
LogFile = /var/log/erp-system/app.log
*OnlyService = erp-system
*RemoveHeaders
EOF

    # Create logwatch script
    cat > /etc/logwatch/scripts/services/erp-system <<'EOF'
#!/usr/bin/perl

use strict;
use warnings;

my $Debug = $ENV{'LOGWATCH_DEBUG'} || 0;
my %errors = ();
my %warnings = ();
my $authentications = 0;
my $requests = 0;

while (defined(my $ThisLine = <STDIN>)) {
    chomp($ThisLine);

    if ($ThisLine =~ /\[ERROR\]/) {
        $errors{$ThisLine}++;
    }
    elsif ($ThisLine =~ /\[WARN\]/) {
        $warnings{$ThisLine}++;
    }
    elsif ($ThisLine =~ /Authentication successful/) {
        $authentications++;
    }
    elsif ($ThisLine =~ /HTTP request/) {
        $requests++;
    }
}

if (keys %errors) {
    print "\nERRORS:\n";
    foreach my $error (sort keys %errors) {
        print "   $error: $errors{$error} time(s)\n";
    }
}

if (keys %warnings) {
    print "\nWARNINGS:\n";
    foreach my $warning (sort keys %warnings) {
        print "   $warning: $warnings{$warning} time(s)\n";
    }
}

if ($authentications > 0) {
    print "\nAuthentications: $authentications\n";
}

if ($requests > 0) {
    print "\nHTTP Requests: $requests\n";
}
EOF

    chmod +x /etc/logwatch/scripts/services/erp-system

    print_status "Log monitoring configured successfully"
}

# Function to setup automatic security updates
setup_auto_updates() {
    print_status "Setting up automatic security updates..."

    # Install unattended-upgrades
    apt install -y unattended-upgrades apt-listchanges

    # Configure automatic updates
    cat > /etc/apt/apt.conf.d/50unattended-upgrades <<EOF
Unattended-Upgrade::Allowed-Origins {
    "\${distro_id}:\${distro_codename}-security";
    "\${distro_id} ESMApps:\${distro_codename}-apps-security";
    "\${distro_id} ESM:\${distro_codename}-infra-security";
};

Unattended-Upgrade::DevRelease "false";
Unattended-Upgrade::Remove-Unused-Dependencies "true";
Unattended-Upgrade::Remove-New-Unused-Dependencies "true";
Unattended-Upgrade::Automatic-Reboot "false";
Unattended-Upgrade::Automatic-Reboot-Time "02:00";

Unattended-Upgrade::Mail "admin@yourdomain.com";
Unattended-Upgrade::MailOnlyOnError "true";
EOF

    # Enable automatic updates
    cat > /etc/apt/apt.conf.d/20auto-upgrades <<EOF
APT::Periodic::Update-Package-Lists "1";
APT::Periodic::Download-Upgradeable-Packages "1";
APT::Periodic::AutocleanInterval "7";
APT::Periodic::Unattended-Upgrade "1";
EOF

    print_status "Automatic security updates configured successfully"
}

# Function to setup intrusion detection
setup_intrusion_detection() {
    print_status "Setting up intrusion detection (AIDE)..."

    # Install AIDE
    apt install -y aide aide-common

    # Initialize AIDE database
    aideinit

    # Copy database
    cp /var/lib/aide/aide.db.new /var/lib/aide/aide.db

    # Create check script
    cat > /opt/erp-system/scripts/aide-check.sh <<'EOF'
#!/bin/bash

AIDE_LOG="/var/log/aide/aide.log"
mkdir -p /var/log/aide

# Run AIDE check
aide --check > "$AIDE_LOG" 2>&1

# Send report if changes detected
if [ $? -ne 0 ]; then
    echo "AIDE detected file system changes on $(hostname) at $(date)" | \
        mail -s "AIDE Alert: File System Changes Detected" admin@yourdomain.com \
        -A "$AIDE_LOG"
fi
EOF

    chmod +x /opt/erp-system/scripts/aide-check.sh
    chown erp:erp /opt/erp-system/scripts/aide-check.sh

    # Add daily AIDE check
    echo "0 3 * * * root /opt/erp-system/scripts/aide-check.sh" >> /etc/crontab

    print_status "Intrusion detection configured successfully"
}

# Function to setup Docker security
secure_docker() {
    print_status "Securing Docker configuration..."

    # Create Docker daemon configuration
    mkdir -p /etc/docker
    cat > /etc/docker/daemon.json <<EOF
{
    "live-restore": true,
    "userland-proxy": false,
    "no-new-privileges": true,
    "seccomp-profile": "/etc/docker/seccomp.json",
    "log-driver": "json-file",
    "log-opts": {
        "max-size": "10m",
        "max-file": "3"
    },
    "default-ulimits": {
        "nofile": {
            "Hard": 64000,
            "Name": "nofile",
            "Soft": 64000
        }
    }
}
EOF

    # Restart Docker with new configuration
    systemctl restart docker

    # Ensure Docker containers run with limited capabilities
    print_status "Docker security configured successfully"
}

# Function to create security monitoring script
create_security_monitor() {
    print_status "Creating security monitoring script..."

    cat > /opt/erp-system/scripts/security-monitor.sh <<'EOF'
#!/bin/bash

LOG_FILE="/var/log/erp-system/security-monitor.log"
ALERT_EMAIL="admin@yourdomain.com"

timestamp() {
    date '+%Y-%m-%d %H:%M:%S'
}

log_alert() {
    echo "$(timestamp) - SECURITY ALERT: $1" | tee -a "$LOG_FILE"
    echo "$1" | mail -s "ERP Security Alert - $(hostname)" "$ALERT_EMAIL"
}

# Check for failed authentication attempts
FAILED_AUTH=$(grep "Authentication failed" /var/log/erp-system/app.log | grep "$(date '+%Y-%m-%d')" | wc -l)
if [ "$FAILED_AUTH" -gt 50 ]; then
    log_alert "High number of failed authentication attempts: $FAILED_AUTH"
fi

# Check for rate limiting hits
RATE_LIMIT=$(grep "Rate limit exceeded" /var/log/erp-system/app.log | grep "$(date '+%Y-%m-%d')" | wc -l)
if [ "$RATE_LIMIT" -gt 100 ]; then
    log_alert "High number of rate limit violations: $RATE_LIMIT"
fi

# Check disk usage
DISK_USAGE=$(df /opt/erp-system | awk 'NR==2 {print $5}' | sed 's/%//')
if [ "$DISK_USAGE" -gt 90 ]; then
    log_alert "High disk usage: ${DISK_USAGE}%"
fi

# Check memory usage
MEM_USAGE=$(free | awk '/Mem:/ {printf("%.2f", $3/$2 * 100.0)}')
if [ "${MEM_USAGE%.*}" -gt 90 ]; then
    log_alert "High memory usage: ${MEM_USAGE}%"
fi

# Check for unusual network connections
UNUSUAL_CONNECTIONS=$(netstat -an | grep :8080 | grep ESTABLISHED | wc -l)
if [ "$UNUSUAL_CONNECTIONS" -gt 100 ]; then
    log_alert "High number of connections to ERP service: $UNUSUAL_CONNECTIONS"
fi

# Check service status
if ! systemctl is-active --quiet erp-system; then
    log_alert "ERP System service is not running"
fi

if ! systemctl is-active --quiet nginx; then
    log_alert "Nginx service is not running"
fi

# Check for SSH brute force attempts
SSH_ATTEMPTS=$(grep "Failed password" /var/log/auth.log | grep "$(date '+%b %d')" | wc -l)
if [ "$SSH_ATTEMPTS" -gt 20 ]; then
    log_alert "High number of SSH brute force attempts: $SSH_ATTEMPTS"
fi
EOF

    chmod +x /opt/erp-system/scripts/security-monitor.sh
    chown erp:erp /opt/erp-system/scripts/security-monitor.sh

    # Add to crontab for every 15 minutes
    echo "*/15 * * * * erp /opt/erp-system/scripts/security-monitor.sh" >> /etc/crontab

    print_status "Security monitoring script created successfully"
}

# Function to print security summary
print_security_summary() {
    echo ""
    echo -e "${GREEN}================================================${NC}"
    echo -e "${GREEN}    Production Security Setup Complete!${NC}"
    echo -e "${GREEN}================================================${NC}"
    echo ""
    echo -e "${BLUE}🔒 Security Measures Implemented:${NC}"
    echo "   • Firewall (UFW) configured with minimal open ports"
    echo "   • SSH hardened with key-based authentication only"
    echo "   • Fail2Ban configured for intrusion prevention"
    echo "   • File permissions secured (600/640/750)"
    echo "   • System limits optimized for performance"
    echo "   • Log monitoring with Logwatch configured"
    echo "   • Automatic security updates enabled"
    echo "   • Intrusion detection (AIDE) installed"
    echo "   • Docker security hardened"
    echo "   • Security monitoring script deployed"
    echo ""
    echo -e "${BLUE}📊 Monitoring:${NC}"
    echo "   • Security alerts: /var/log/erp-system/security-monitor.log"
    echo "   • AIDE reports: /var/log/aide/"
    echo "   • Fail2Ban status: fail2ban-client status"
    echo "   • Service logs: journalctl -u erp-system -f"
    echo ""
    echo -e "${YELLOW}⚠️  Important Notes:${NC}"
    echo "   • Update admin@yourdomain.com email addresses in configs"
    echo "   • Setup SSH key-based authentication before disabling passwords"
    echo "   • Review and test firewall rules carefully"
    echo "   • Monitor security logs regularly"
    echo "   • Keep system and dependencies updated"
    echo ""
    echo -e "${BLUE}🔧 Security Commands:${NC}"
    echo "   • Check firewall: ufw status verbose"
    echo "   • View fail2ban status: fail2ban-client status"
    echo "   • Check intrusions: aide --check"
    echo "   • Security scan: /opt/erp-system/scripts/security-monitor.sh"
    echo ""
}

# Main security setup flow
main() {
    print_header

    check_root

    print_status "Starting production security hardening..."

    setup_firewall
    secure_ssh
    setup_fail2ban
    secure_file_permissions
    setup_system_limits
    setup_log_monitoring
    setup_auto_updates
    setup_intrusion_detection
    secure_docker
    create_security_monitor

    print_security_summary

    print_status "🔒 Production security setup completed successfully!"
}

# Handle interruption
trap 'print_error "Security setup interrupted!"; exit 1' INT TERM

# Run main function
main "$@"