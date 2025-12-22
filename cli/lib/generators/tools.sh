#!/bin/bash
###################################################################
# STACKVO TOOLS GENERATOR MODULE
# Generates Dockerfile, nginx.conf and supervisord.conf files
###################################################################

##
# Creates Dockerfile, nginx.conf and supervisord.conf for tools
#
# Returns:
#   0 - Success
#   1 - Error
##
generate_tools_configs() {
    log_info "Generating tools configurations..."
    
    local tools_dir="$ROOT_DIR/core/templates/ui/tools"
    local tpl_dir="$tools_dir/tpl"
    
    # Check if tools container is enabled
    if [ "${STACKVO_UI_TOOLS_CONTAINER_ENABLE}" != "true" ]; then
        log_warn "Tools container is disabled, skipping..."
        return 0
    fi
    
    # Backup existing files (first time only)
    create_tools_backup "$tools_dir"
    
    # Generate Dockerfile
    log_info "Generating Dockerfile from template..."
    if ! render_tools_template "$tpl_dir/Dockerfile.tpl" > "$tools_dir/Dockerfile"; then
        log_error "Failed to generate Dockerfile"
        return 1
    fi
    
    # Generate nginx.conf
    log_info "Generating nginx.conf from template..."
    if ! render_tools_template "$tpl_dir/nginx.conf.tpl" > "$tools_dir/nginx.conf"; then
        log_error "Failed to generate nginx.conf"
        return 1
    fi
    
    # Generate supervisord.conf
    log_info "Generating supervisord.conf from template..."
    if ! render_tools_template "$tpl_dir/supervisord.conf.tpl" > "$tools_dir/supervisord.conf"; then
        log_error "Failed to generate supervisord.conf"
        return 1
    fi
    
    log_success "Tools configurations generated successfully!"
    log_info "  - Dockerfile: $(wc -l < "$tools_dir/Dockerfile") lines"
    log_info "  - nginx.conf: $(wc -l < "$tools_dir/nginx.conf") lines"
    log_info "  - supervisord.conf: $(wc -l < "$tools_dir/supervisord.conf") lines"
}

##
# Backup existing tools configuration files
#
# Args:
#   $1 - Tools directory path
##
create_tools_backup() {
    local tools_dir=$1
    
    # Backup Dockerfile
    if [ -f "$tools_dir/Dockerfile" ] && [ ! -f "$tools_dir/Dockerfile.backup" ]; then
        log_info "Backing up existing Dockerfile..."
        cp "$tools_dir/Dockerfile" "$tools_dir/Dockerfile.backup"
    fi
    
    # Backup nginx.conf
    if [ -f "$tools_dir/nginx.conf" ] && [ ! -f "$tools_dir/nginx.conf.backup" ]; then
        log_info "Backing up existing nginx.conf..."
        cp "$tools_dir/nginx.conf" "$tools_dir/nginx.conf.backup"
    fi
    
    # Backup supervisord.conf
    if [ -f "$tools_dir/supervisord.conf" ] && [ ! -f "$tools_dir/supervisord.conf.backup" ]; then
        log_info "Backing up existing supervisord.conf..."
        cp "$tools_dir/supervisord.conf" "$tools_dir/supervisord.conf.backup"
    fi
}

##
# Renders tools templates (with conditional blocks)
#
# Args:
#   $1 - Template file path
#
# Returns:
#   Rendered content
##
render_tools_template() {
    local template_file=$1
    
    if [ ! -f "$template_file" ]; then
        log_error "Template file not found: $template_file"
        return 1
    fi
    
    # Read template content
    local content
    content=$(cat "$template_file")
    
    # Process conditional blocks: {{#VAR}} ... {{/VAR}}
    # If VAR=true, keep content; if VAR=false, remove block
    local tools=(
        "TOOLS_ADMINER_ENABLE"
        "TOOLS_PHPMYADMIN_ENABLE"
        "TOOLS_PHPMEMCACHEDADMIN_ENABLE"
        "TOOLS_OPCACHE_ENABLE"
        "TOOLS_PHPMONGO_ENABLE"
        "TOOLS_PHPPGADMIN_ENABLE"
        "TOOLS_KAFBAT_ENABLE"
    )
    
    for tool_var in "${tools[@]}"; do
        eval "local tool_enabled=\${${tool_var}:-false}"
        
        if [ "$tool_enabled" = "true" ]; then
            # Keep the block, remove markers
            content=$(echo "$content" | sed -e "/{{#${tool_var}}}/d" -e "/{{\\/${tool_var}}}/d")
        else
            # Remove entire block (including markers)
            # Use perl for multi-line matching (more reliable than sed)
            content=$(echo "$content" | perl -0pe "s/\{\{#${tool_var}\}\}.*?\{\{\/${tool_var}\}\}//gs")
        fi
    done
    
    # Replace {{ VAR }} with actual values
    content=$(echo "$content" | sed \
        -e "s/{{ TOOLS_ADMINER_VERSION }}/${TOOLS_ADMINER_VERSION}/g" \
        -e "s/{{ TOOLS_PHPMYADMIN_VERSION }}/${TOOLS_PHPMYADMIN_VERSION}/g" \
        -e "s/{{ TOOLS_PHPMEMCACHEDADMIN_VERSION }}/${TOOLS_PHPMEMCACHEDADMIN_VERSION}/g" \
        -e "s/{{ TOOLS_OPCACHE_VERSION }}/${TOOLS_OPCACHE_VERSION}/g" \
        -e "s/{{ TOOLS_PHPMONGO_VERSION }}/${TOOLS_PHPMONGO_VERSION}/g" \
        -e "s/{{ TOOLS_PHPPGADMIN_VERSION }}/${TOOLS_PHPPGADMIN_VERSION}/g" \
        -e "s/{{ TOOLS_KAFBAT_VERSION }}/${TOOLS_KAFBAT_VERSION}/g")
    
    echo "$content"
}
