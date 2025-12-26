#!/bin/bash
###################################################################
# STACKVO UI GENERATOR MODULE
# Generates Dockerfile and nginx.conf files for Stackvo UI
###################################################################

##
# Creates Dockerfile and nginx.conf for Stackvo UI
#
# Returns:
#   0 - Success
#   1 - Error
##
generate_stackvo_ui_configs() {
    log_info "Generating Stackvo UI configurations..."
    
    local template_dir="$ROOT_DIR/core/templates/ui/stackvo-ui"
    local output_dir="$GENERATED_DIR/ui"
    
    # Check if Stackvo UI is enabled
    if [ "${STACKVO_UI_ENABLE}" != "true" ]; then
        log_warn "Stackvo UI is disabled, skipping..."
        return 0
    fi
    
    # Create output directory
    mkdir -p "$output_dir"
    
    # Generate Dockerfile
    log_info "Generating Dockerfile from template..."
    if ! render_template "$template_dir/Dockerfile.tpl" > "$output_dir/Dockerfile"; then
        log_error "Failed to generate Dockerfile"
        return 1
    fi
    
    # Generate nginx.conf
    log_info "Generating nginx.conf from template..."
    if ! render_template "$template_dir/nginx.conf.tpl" > "$output_dir/nginx.conf"; then
        log_error "Failed to generate nginx.conf"
        return 1
    fi
    
    log_success "Stackvo UI configurations generated successfully!"
    log_info "  - Dockerfile: $(wc -l < "$output_dir/Dockerfile") lines"
    log_info "  - nginx.conf: $(wc -l < "$output_dir/nginx.conf") lines"
}
