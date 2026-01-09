#!/bin/bash
###################################################################
# STACKVO COMMON LIBRARY
# Shared variables and paths used across all scripts
###################################################################

# Resolve script directory and root paths
# This works from any script location (root CLI or lib/)
if [ -n "${BASH_SOURCE[0]}" ]; then
    # Get the directory of the script that sourced this file
    CURRENT_SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    
    # Check if we're in lib/ directory
    if [[ "$CURRENT_SCRIPT_DIR" == */lib ]]; then
        readonly CLI_DIR="$(cd "$CURRENT_SCRIPT_DIR/.." && pwd)"
    else
        readonly CLI_DIR="$CURRENT_SCRIPT_DIR"
    fi
    
    # CLI is now in core/cli/, so go up two levels to reach stackvo root
    readonly STACKVO_ROOT="$(cd "$CLI_DIR/../.." && pwd)"
else
    # Fallback if BASH_SOURCE is not available
    readonly STACKVO_ROOT="$(pwd)"
    readonly CLI_DIR="$STACKVO_ROOT/core/cli"
fi

# Docker Compose file paths
readonly COMPOSE_FILES=(
    -f "$STACKVO_ROOT/generated/stackvo.yml"
    -f "$STACKVO_ROOT/generated/docker-compose.dynamic.yml"
    -f "$STACKVO_ROOT/generated/docker-compose.projects.yml"
)

# Generated files paths
readonly GENERATED_DIR="$STACKVO_ROOT/generated"
readonly GENERATED_TRAEFIK_DIR="$GENERATED_DIR/traefik"
readonly GENERATED_TRAEFIK_DYNAMIC_DIR="$GENERATED_TRAEFIK_DIR/dynamic"
readonly GENERATED_CONFIGS_DIR="$GENERATED_DIR/configs"
readonly GENERATED_CERTS_DIR="$GENERATED_DIR/certs"

# Export for use in subshells and external scripts
export STACKVO_ROOT
export CLI_DIR
export GENERATED_DIR
export GENERATED_TRAEFIK_DIR
export GENERATED_TRAEFIK_DYNAMIC_DIR
export GENERATED_CONFIGS_DIR
export GENERATED_CERTS_DIR

