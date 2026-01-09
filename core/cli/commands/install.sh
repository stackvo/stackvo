#!/usr/bin/env bash

###################################################################
# STACKVO INSTALLER
# Installs Stackvo CLI and sets up the environment
###################################################################

# Load common library and logger
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib/common.sh"
source "$SCRIPT_DIR/../lib/logger.sh"

##
# Tüm CLI bash scriptlerine execute izni verir
##
fix_cli_permissions() {
    log_info "Fixing CLI script permissions..."
    
    # Find all .sh files in CLI directory and make them executable
    find "$CLI_DIR" -type f -name "*.sh" -exec chmod +x {} \; 2>/dev/null || true
    
    log_success "CLI script permissions fixed"
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

# Fix CLI script permissions first
fix_cli_permissions

# Check Docker Compose version
validate_docker_compose_version

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
            # Read HOST_UID and HOST_GID from .env
            if [ -f "$STACKVO_ROOT/.env" ]; then
                HOST_UID=$(grep "^HOST_UID=" "$STACKVO_ROOT/.env" | cut -d= -f2)
                HOST_GID=$(grep "^HOST_GID=" "$STACKVO_ROOT/.env" | cut -d= -f2)
                
                if [ -n "$HOST_UID" ] && [ -n "$HOST_GID" ]; then
                    chown -R "$HOST_UID:$HOST_GID" "$STACKVO_ROOT/generated"
                    log_info "Fixed ownership of generated directory"
                else
                    # Fallback to SUDO_USER:staff for macOS compatibility
                    chown -R "$SUDO_USER:staff" "$STACKVO_ROOT/generated"
                    log_info "Fixed ownership of generated directory"
                fi
            else
                # Fallback to SUDO_USER:staff for macOS compatibility
                chown -R "$SUDO_USER:staff" "$STACKVO_ROOT/generated"
                log_info "Fixed ownership of generated directory"
            fi
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

# Create projects directory if it doesn't exist
log_info "Creating projects directory..."
if [ ! -d "$STACKVO_ROOT/projects" ]; then
    mkdir -p "$STACKVO_ROOT/projects"
    
    # Set ownership to HOST_UID:HOST_GID from .env
    if [ -f "$STACKVO_ROOT/.env" ]; then
        HOST_UID=$(grep "^HOST_UID=" "$STACKVO_ROOT/.env" | cut -d= -f2)
        HOST_GID=$(grep "^HOST_GID=" "$STACKVO_ROOT/.env" | cut -d= -f2)
        
        if [ -n "$HOST_UID" ] && [ -n "$HOST_GID" ]; then
            chown -R "$HOST_UID:$HOST_GID" "$STACKVO_ROOT/projects"
            log_info "Set ownership to $HOST_UID:$HOST_GID"
        fi
    fi
    
    log_success "Projects directory created"
else
    log_success "Projects directory already exists"
fi

# Create logs directory if it doesn't exist
log_info "Creating logs directory..."
if [ ! -d "$STACKVO_ROOT/logs" ]; then
    mkdir -p "$STACKVO_ROOT/logs/services"
    mkdir -p "$STACKVO_ROOT/logs/projects"
    
    # Set ownership to HOST_UID:HOST_GID from .env
    if [ -f "$STACKVO_ROOT/.env" ]; then
        HOST_UID=$(grep "^HOST_UID=" "$STACKVO_ROOT/.env" | cut -d= -f2)
        HOST_GID=$(grep "^HOST_GID=" "$STACKVO_ROOT/.env" | cut -d= -f2)
        
        if [ -n "$HOST_UID" ] && [ -n "$HOST_GID" ]; then
            chown -R "$HOST_UID:$HOST_GID" "$STACKVO_ROOT/logs"
            log_info "Set ownership to $HOST_UID:$HOST_GID"
        fi
    fi
    
    log_success "Logs directory created"
else
    log_success "Logs directory already exists"
fi

echo ""
log_success "Installation completed. Available commands:"
echo ""
echo "📋 Available Commands:"
echo ""
echo "  ./core/cli/stackvo.sh generate          → Generate all configuration files"
echo "  ./core/cli/stackvo.sh up                → Start all Stackvo services"
echo "  ./core/cli/stackvo.sh down              → Stop all Stackvo services"
echo "  ./core/cli/stackvo.sh restart           → Restart all Stackvo services"
echo "  ./core/cli/stackvo.sh ps                → List running containers"
echo "  ./core/cli/stackvo.sh logs [service]    → View service logs"
echo "  ./core/cli/stackvo.sh pull              → Pull latest Docker images"
echo "  ./core/cli/stackvo.sh --help            → Show all available commands"
echo ""
echo "🚀 Quick Start:"
echo ""
echo "  1. Generate configurations:"
echo "     ./core/cli/stackvo.sh generate"
echo ""
echo "  2. Start services:"
echo "     ./core/cli/stackvo.sh up"
echo ""
echo "  3. StackVo dashboard:"
echo "     https://stackvo.loc"
echo ""
echo "  4. Traefik dashboard:"
echo "     https://traefik.stackvo.loc"
echo ""
echo "📖 For more information, run: ./core/cli/stackvo.sh --help"
echo ""
