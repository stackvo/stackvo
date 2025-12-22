#!/bin/bash
###################################################################
# STACKVO TOOLS UNINSTALLER MODULE
# Restores original static configuration files
###################################################################

##
# Uninstall dynamic tools configuration and restore static files
#
# Returns:
#   0 - Success
#   1 - Error
##
uninstall_tools_configs() {
    log_info "Uninstalling dynamic tools configuration..."
    
    local tools_dir="$ROOT_DIR/core/templates/ui/tools"
    
    # Check if backup files exist
    if [ ! -f "$tools_dir/Dockerfile.backup" ]; then
        log_error "Backup files not found! Cannot uninstall."
        log_error "Please restore manually or reinstall from scratch."
        return 1
    fi
    
    # Restore from backup
    log_info "Restoring original static files from backup..."
    
    if [ -f "$tools_dir/Dockerfile.backup" ]; then
        cp "$tools_dir/Dockerfile.backup" "$tools_dir/Dockerfile"
        log_info "  ✓ Restored Dockerfile"
    fi
    
    if [ -f "$tools_dir/nginx.conf.backup" ]; then
        cp "$tools_dir/nginx.conf.backup" "$tools_dir/nginx.conf"
        log_info "  ✓ Restored nginx.conf"
    fi
    
    if [ -f "$tools_dir/supervisord.conf.backup" ]; then
        cp "$tools_dir/supervisord.conf.backup" "$tools_dir/supervisord.conf"
        log_info "  ✓ Restored supervisord.conf"
    fi
    
    # Remove backup files
    log_info "Cleaning up backup files..."
    rm -f "$tools_dir/Dockerfile.backup"
    rm -f "$tools_dir/nginx.conf.backup"
    rm -f "$tools_dir/supervisord.conf.backup"
    
    # Remove template directory (optional - ask user)
    if [ "$REMOVE_TEMPLATES" = "true" ]; then
        log_info "Removing template directory..."
        rm -rf "$tools_dir/tpl"
        log_info "  ✓ Removed tpl/ directory"
    else
        log_info "Template directory preserved at: $tools_dir/tpl"
        log_info "To remove it, run: rm -rf $tools_dir/tpl"
    fi
    
    log_success "Dynamic tools configuration uninstalled successfully!"
    log_info "Original static configuration restored."
    log_info ""
    log_info "Next steps:"
    log_info "  1. Rebuild tools container: docker compose up -d --build tools"
    log_info "  2. To reinstall dynamic config: ./stackvo generate"
}

##
# Show uninstall confirmation prompt
#
# Returns:
#   0 - User confirmed
#   1 - User cancelled
##
confirm_uninstall() {
    echo ""
    echo "⚠️  WARNING: This will restore original static configuration"
    echo ""
    echo "The following will happen:"
    echo "  - Dynamic configuration files will be replaced with static versions"
    echo "  - Backup files will be removed"
    echo "  - Template directory (tpl/) will be preserved (unless --remove-templates)"
    echo ""
    echo "You can reinstall dynamic config anytime by running: ./stackvo generate"
    echo ""
    
    # In non-interactive mode, skip confirmation
    if [ "$STACKVO_DRY_RUN" = "true" ] || [ "$FORCE_UNINSTALL" = "true" ]; then
        return 0
    fi
    
    read -p "Do you want to continue? [y/N] " -n 1 -r
    echo
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        return 0
    else
        log_info "Uninstall cancelled."
        return 1
    fi
}
