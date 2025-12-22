#!/bin/bash
set -eo pipefail

###################################################################
# STACKVO GENERATOR - PURE BASH IMPLEMENTATION
# Compatible with Bash 3.x+ (macOS default)
# No PHP dependency required!
#
# This file only serves as an orchestrator.
# All functions are separated into modules.
###################################################################

# Load common library
readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib/common.sh"

# Set ROOT_DIR for compatibility with existing code
readonly ROOT_DIR="$STACKVO_ROOT"

# Load libraries
source "$SCRIPT_DIR/../lib/logger.sh"
source "$SCRIPT_DIR/../lib/constants.sh"
source "$SCRIPT_DIR/../lib/env-loader.sh"
source "$SCRIPT_DIR/../lib/template-processor.sh"
source "$SCRIPT_DIR/../lib/generators/config.sh"
source "$SCRIPT_DIR/../lib/generators/compose.sh"
source "$SCRIPT_DIR/../lib/generators/traefik.sh"
source "$SCRIPT_DIR/../lib/generators/project.sh"
source "$SCRIPT_DIR/../lib/generators/tools.sh"
source "$SCRIPT_DIR/../lib/uninstallers/tools.sh"

##
# Show help message
##
show_help() {
    cat << EOF
Stackvo Generator - Dynamic Configuration Generator

Usage: ./stackvo generate [OPTIONS]

Options:
  --uninstall-tools       Uninstall dynamic tools configuration and restore static files
  --remove-templates      Remove template directory when uninstalling (use with --uninstall-tools)
  --force                 Skip confirmation prompts
  -h, --help             Show this help message

Examples:
  # Generate all configurations
  ./stackvo generate

  # Uninstall dynamic tools configuration
  ./stackvo generate --uninstall-tools

  # Uninstall and remove templates
  ./stackvo generate --uninstall-tools --remove-templates

  # Force uninstall without confirmation
  ./stackvo generate --uninstall-tools --force

EOF
}

##
# Main orchestrator function
# Runs all generator modules in sequence
##
main() {
    local MODE=$1
    
    log_info "Stackvo Generator (Bash - No PHP!)"
    cd "$ROOT_DIR"
    
    # Load environment
    load_env
    
    # Generate SSL certificates if not exist (only for full generate)
    if [ "$MODE" != "projects" ] && [ "$MODE" != "services" ]; then
        if [ ! -f "$GENERATED_CERTS_DIR/stackvo-wildcard.crt" ]; then
            log_info "SSL certificates not found. Generating..."
            if bash "$CLI_DIR/utils/generate-ssl-certs.sh"; then
                log_success "SSL certificates generated"
            else
                log_warn "SSL certificate generation failed. You can generate them later with: bash cli/utils/generate-ssl-certs.sh"
            fi
        fi
    fi
    
    case "$MODE" in
        projects)
            log_info "Generating projects only..."
            generate_projects
            log_success "Projects generation completed!"
            ;;
        services)
            log_info "Generating services only..."
            generate_tools_configs
            generate_module_configs
            generate_base_compose
            generate_traefik_config
            generate_traefik_routes
            generate_dynamic_compose
            log_success "Services generation completed!"
            ;;
        *)
            # Generate everything
            generate_tools_configs
            generate_module_configs
            generate_base_compose
            generate_traefik_config
            generate_traefik_routes
            generate_dynamic_compose
            generate_projects
            log_success "Generation completed!"
            ;;
    esac
}

##
# Parse command line arguments
##
UNINSTALL_TOOLS=false
REMOVE_TEMPLATES=false
FORCE_UNINSTALL=false
MODE=""

while [[ $# -gt 0 ]]; do
    case $1 in
        projects|services)
            MODE=$1
            shift
            ;;
        --uninstall-tools)
            UNINSTALL_TOOLS=true
            shift
            ;;
        --remove-templates)
            REMOVE_TEMPLATES=true
            shift
            ;;
        --force)
            FORCE_UNINSTALL=true
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Export for use in other modules
export REMOVE_TEMPLATES
export FORCE_UNINSTALL

# Execute based on mode
if [ "$UNINSTALL_TOOLS" = "true" ]; then
    # Load env for uninstall
    cd "$ROOT_DIR"
    load_env
    
    # Confirm and uninstall
    if confirm_uninstall; then
        uninstall_tools_configs
    fi
else
    # Normal generation
    main "$MODE"
fi
