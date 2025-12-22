#!/usr/bin/env bash

###################################################################
# STACKVO INSTALLER
# Installs Stackvo CLI and sets up the environment
###################################################################

# Load common library and logger
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib/common.sh"
source "$SCRIPT_DIR/../lib/logger.sh"

# Update /etc/hosts with stackvo domains
update_hosts_file() {
    log_info "Checking /etc/hosts configuration..."
    
    # Check if stackvo domains already exist
    if grep -q "stackvo.loc" /etc/hosts 2>/dev/null; then
        log_success "/etc/hosts already configured"
        return
    fi
    
    log_info "Adding stackvo domains to /etc/hosts..."
    
    # Create hosts entries
    cat > /tmp/stackvo-hosts <<'EOF'

# Stackvo Domains
127.0.0.1 stackvo.loc
127.0.0.1 traefik.stackvo.loc
127.0.0.1 activemq.stackvo.loc
127.0.0.1 elasticsearch.stackvo.loc
127.0.0.1 grafana.stackvo.loc
127.0.0.1 kafbat.stackvo.loc
127.0.0.1 kibana.stackvo.loc
127.0.0.1 kong.stackvo.loc
127.0.0.1 mailhog.stackvo.loc
127.0.0.1 mariadb.stackvo.loc
127.0.0.1 meilisearch.stackvo.loc
127.0.0.1 mongo.stackvo.loc
127.0.0.1 mysql.stackvo.loc
127.0.0.1 netdata.stackvo.loc
127.0.0.1 postgres.stackvo.loc
127.0.0.1 rabbitmq.stackvo.loc
127.0.0.1 redis.stackvo.loc
127.0.0.1 sentry.stackvo.loc
127.0.0.1 sonarqube.stackvo.loc
127.0.0.1 tomcat.stackvo.loc
127.0.0.1 tools.stackvo.loc
127.0.0.1 adminer.stackvo.loc
127.0.0.1 phpmyadmin.stackvo.loc
127.0.0.1 phppgadmin.stackvo.loc
127.0.0.1 phpmemcachedadmin.stackvo.loc
127.0.0.1 phpmongo.stackvo.loc
127.0.0.1 opcache.stackvo.loc
EOF
    
    # Append to /etc/hosts with sudo
    if sudo bash -c 'cat /tmp/stackvo-hosts >> /etc/hosts'; then
        rm /tmp/stackvo-hosts
        log_success "Added stackvo domains to /etc/hosts"
    else
        log_warn "Failed to update /etc/hosts. You may need to add domains manually."
        rm /tmp/stackvo-hosts
    fi
}

# Check Docker Compose version
validate_docker_compose_version() {
    log_info "Checking Docker Compose version..."
    
    # Get current version
    local current_version=$(docker compose version 2>/dev/null | sed -E 's/.*v([0-9]+\.[0-9]+\.[0-9]+).*/\1/' | head -1)
    
    if [ -z "$current_version" ]; then
        log_error "Docker Compose not found! Please install Docker Compose first."
        log_info "Visit: https://docs.docker.com/compose/install/"
        exit 1
    fi
    
    log_info "Current Docker Compose version: v$current_version"
    
    # Extract major and minor version
    local current_major=$(echo "$current_version" | cut -d. -f1)
    local current_minor=$(echo "$current_version" | cut -d. -f2)
    
    # Minimum recommended version is 2.0.0 (Docker Compose v2 series)
    local min_major=2
    local min_minor=0
    
    if [ "$current_major" -lt "$min_major" ] || ([ "$current_major" -eq "$min_major" ] && [ "$current_minor" -lt "$min_minor" ]); then
        log_error "⚠️  Your Docker Compose version (v$current_version) is too old!"
        log_error "   Minimum required version: v${min_major}.${min_minor}.0"
        echo ""
        echo "   Please update Docker Compose manually:"
        echo "   - macOS: Update Docker Desktop from https://www.docker.com/products/docker-desktop"
        echo "   - Linux: Visit https://docs.docker.com/compose/install/"
        echo ""
        exit 1
    else
        log_success "Docker Compose version is sufficient (v$current_version >= v${min_major}.${min_minor}.0)"
    fi
}

echo "🔧 Installing Stackvo CLI..."
echo ""

# Check if running as root
if [ "$EUID" -eq 0 ]; then
    echo "⚠️  WARNING: Running as root (sudo)"
    echo "   This may cause issues with Homebrew on macOS."
    echo "   Recommended: Run without sudo, it will ask for password when needed."
    echo ""
fi

# Check Docker Compose version first
validate_docker_compose_version

# Update /etc/hosts
update_hosts_file

# Make scripts executable
chmod +x "$CLI_DIR/stackvo.sh"
chmod +x "$CLI_DIR/commands/generate.sh"
chmod +x "$CLI_DIR/utils/generate-ssl-certs.sh"
chmod +x "$CLI_DIR/commands/uninstall.sh"

# Create symlink (requires sudo)
if [ "$EUID" -eq 0 ]; then
    # Already running as root
    ln -sf "$CLI_DIR/stackvo.sh" /usr/local/bin/stackvo
else
    # Request sudo for this specific command
    log_info "Creating system command (requires sudo)..."
    sudo ln -sf "$CLI_DIR/stackvo.sh" /usr/local/bin/stackvo
fi

# Generate SSL certificates automatically
log_info "Generating SSL certificates..."
if bash "$CLI_DIR/utils/generate-ssl-certs.sh" 2>&1 | grep -E '^\[|^🔐|^✅|^📁|^📌'; then
    log_success "SSL certificates generated successfully"
    
    # Fix ownership of generated directory (created as root during SSL cert generation)
    # This ensures normal user can create subdirectories during 'generate' command
    if [ -d "$STACKVO_ROOT/generated" ]; then
        if [ "$EUID" -eq 0 ] && [ -n "$SUDO_USER" ]; then
            # Running as sudo, fix ownership to actual user (macOS uses 'staff' group)
            chown -R "$SUDO_USER:staff" "$STACKVO_ROOT/generated"
            log_info "Fixed ownership of generated directory"
        fi
    fi
else
    log_warn "SSL certificate generation failed. You can generate them later with: ./cli/utils/generate-ssl-certs.sh"
fi

# Create Docker network if it doesn't exist
log_info "Creating Docker network..."
if docker network inspect stackvo-net >/dev/null 2>&1; then
    log_success "Docker network 'stackvo-net' already exists"
else
    if docker network create stackvo-net >/dev/null 2>&1; then
        log_success "Docker network 'stackvo-net' created"
    else
        log_warn "Failed to create Docker network. You can create it manually with: docker network create stackvo-net"
    fi
fi

echo ""
log_success "Installation completed. Available commands:"
echo ""
echo "📋 Available Commands:"
echo ""
echo "  stackvo generate          → Generate all configuration files"
echo "  stackvo up                → Start all Stackvo services"
echo "  stackvo down              → Stop all Stackvo services"
echo "  stackvo restart           → Restart all Stackvo services"
echo "  stackvo ps                → List running containers"
echo "  stackvo logs [service]    → View service logs"
echo "  stackvo pull              → Pull latest Docker images"
echo "  stackvo --help            → Show all available commands"
echo ""
echo "🚀 Quick Start:"
echo ""
echo "  1. Generate configurations:"
echo "     stackvo generate"
echo ""
echo "  2. Start services:"
echo "     stackvo up"
echo ""
echo "  3. Access Traefik dashboard:"
echo "     https://traefik.stackvo.loc"
echo ""
echo "📖 For more information, run: stackvo --help"
echo ""
