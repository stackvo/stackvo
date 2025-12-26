#!/bin/bash
###################################################################
# STACKVO PERMISSION FIX SCRIPT
# Fixes ownership of generated and logs directories
###################################################################

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# Load environment and logger
source "$ROOT_DIR/cli/lib/env-loader.sh"
source "$ROOT_DIR/cli/lib/logger.sh"

log_info "Fixing directory permissions..."

# Load UID/GID from .env
HOST_UID="${HOST_UID:-1000}"
HOST_GID="${HOST_GID:-1000}"

log_info "Using UID: $HOST_UID, GID: $HOST_GID"

# Fix generated directory
if [ -d "$ROOT_DIR/generated" ]; then
    log_info "Fixing generated directory..."
    sudo chown -R "${HOST_UID}:${HOST_GID}" "$ROOT_DIR/generated"
    chmod -R 755 "$ROOT_DIR/generated"
    log_success "Fixed generated directory"
fi

# Fix logs directory
if [ -d "$ROOT_DIR/logs" ]; then
    log_info "Fixing logs directory..."
    sudo chown -R "${HOST_UID}:${HOST_GID}" "$ROOT_DIR/logs"
    chmod -R 755 "$ROOT_DIR/logs"
    log_success "Fixed logs directory"
fi

# Fix projects directory
if [ -d "$ROOT_DIR/projects" ]; then
    log_info "Fixing projects directory..."
    sudo chown -R "${HOST_UID}:${HOST_GID}" "$ROOT_DIR/projects"
    chmod -R 755 "$ROOT_DIR/projects"
    log_success "Fixed projects directory"
fi

log_success "All permissions fixed!"
