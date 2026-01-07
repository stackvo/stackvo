#!/bin/bash
###################################################################
# STACKVO CONFIG GENERATOR MODULE
# Generating configuration files from templates
###################################################################

##
# Generates module configuration files
#
# Finds template files (.tpl) and renders them
# to generate configuration files (e.g., redis.conf, mysql.cnf)
#
# Returns:
#   0 - Success
##
generate_module_configs() {
    log_info "Generating service configuration files..."
    
    local configs_dir="$GENERATED_CONFIGS_DIR"
    local services_dir="$ROOT_DIR/core/templates/services"
    
    # Create configs directory
    mkdir -p "$configs_dir"
    
    # Config file mappings: template_file -> output_file
    # Format: "module/file.tpl:output_name"
    local config_mappings=(
        "redis/redis.conf.tpl:redis.conf"
        "mysql/my.cnf.tpl:mysql.cnf"
        "mongo/mongo.conf.tpl:mongo.conf"
        "postgres/postgres.conf.tpl:postgres.conf"
        "elasticsearch/elasticsearch.yml.tpl:elasticsearch.yml"
    )
    
    # Static config files (copy as-is, no template processing)
    local static_configs=(
        "mariadb/my.cnf:mariadb.cnf"
        "percona/my.cnf:percona.cnf"
    )
    
    # Process template files
    for mapping in "${config_mappings[@]}"; do
        local template_path="${mapping%%:*}"
        local output_name="${mapping##*:}"
        local full_template_path="$services_dir/$template_path"
        local output_path="$configs_dir/$output_name"
        
        if [ -f "$full_template_path" ]; then
            log_info "Generating: $output_name"
            
            if ! render_template "$full_template_path" > "$output_path" 2>/dev/null; then
                log_warn "Failed to generate $output_name from template"
                continue
            fi
            
            log_success "Generated: $output_name"
        else
            log_warn "Template not found: $template_path"
        fi
    done
    
    # Copy static config files
    for mapping in "${static_configs[@]}"; do
        local source_path="${mapping%%:*}"
        local output_name="${mapping##*:}"
        local full_source_path="$services_dir/$source_path"
        local output_path="$configs_dir/$output_name"
        
        if [ -f "$full_source_path" ]; then
            log_info "Copying: $output_name"
            
            if ! cp "$full_source_path" "$output_path" 2>/dev/null; then
                log_warn "Failed to copy $output_name"
                continue
            fi
            
            log_success "Copied: $output_name"
        else
            log_warn "Config file not found: $source_path"
        fi
    done
    
    log_success "Module configuration files generated"
}
