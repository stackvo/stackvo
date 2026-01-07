#!/bin/bash
###################################################################
# STACKVO ENV LOADER MODULE
# Loading environment variables
###################################################################

##
# Loads .env file and exports environment variables
#
# Returns:
#   0 - Success
#   1 - .env file not found
##
load_env() {
    log_info "Loading environment..."
    
    if [ ! -f "$ROOT_DIR/.env" ]; then
        log_error ".env not found"
        exit 1
    fi
    
    set -a
    source "$ROOT_DIR/.env"
    set +a
    
    # Auto-detect HOST_UID and HOST_GID if not set in .env
    export HOST_UID="${HOST_UID:-$(id -u)}"
    export HOST_GID="${HOST_GID:-$(id -g)}"
}
