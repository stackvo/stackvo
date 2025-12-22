#!/bin/bash
###################################################################
# STACKVO PROJECT GENERATOR MODULE
# Generating project containers
###################################################################

##
# Main function to generate all project containers
##
generate_projects() {
    log_info "Generating project containers..."
    
    # Detect if running in container and determine host path
    local HOST_ROOT_DIR="$ROOT_DIR"
    if [ -f "/.dockerenv" ] || [ -f "/run/.containerenv" ]; then
        # We're in container - use host path for volume mounts
        if [ -n "$HOST_STACKVO_ROOT" ]; then
            HOST_ROOT_DIR="$HOST_STACKVO_ROOT"
            log_info "Running in container, using host path: $HOST_ROOT_DIR"
        else
            log_warn "Running in container but HOST_STACKVO_ROOT not set, volume mounts may fail"
        fi
    fi
    
    # Create generated directory
    mkdir -p "$GENERATED_DIR"
    mkdir -p "$GENERATED_CONFIGS_DIR"
    
    # Fix permissions based on where we're running
    if [ -d "$GENERATED_DIR" ]; then
        # Check if we're inside a container (check for /.dockerenv or /run/.containerenv)
        if [ -f "/.dockerenv" ] || [ -f "/run/.containerenv" ]; then
            # We're inside a container, set ownership to nginx user
            if [ "$(id -u)" -eq 0 ]; then
                # Running as root in container
                chown -R 100:101 "$GENERATED_DIR" 2>/dev/null || true
                log_info "Set generated directory ownership to nginx user (100:101)"
            fi
        else
            # We're on the host, use chmod 777 for simplicity
            # This allows both host user and container nginx user to write
            chmod -R 777 "$GENERATED_DIR" 2>/dev/null || true
            log_info "Set generated directory permissions to 777 (writable by all)"
        fi
    fi
    
    local output="$GENERATED_DIR/docker-compose.projects.yml"
    local projects_dir="$ROOT_DIR/projects"
    
    # Start with empty services
    echo "services:" > "$output"
    echo "" >> "$output"
    
    # Check if projects directory exists
    if [ ! -d "$projects_dir" ]; then
        log_info "No projects directory found"
        # Still add network definition even if no projects
        echo "" >> "$output"
        echo "networks:" >> "$output"
        echo "  ${DOCKER_DEFAULT_NETWORK:-$CONST_DEFAULT_NETWORK}:" >> "$output"
        echo "    external: true" >> "$output"
        return
    fi
    
    # Process each project
    for project_path in "$projects_dir"/*; do
        [ ! -d "$project_path" ] && continue
        
        local project_name=$(basename "$project_path")
        local project_json="$project_path/stackvo.json"
        
        # Skip if no stackvo.json
        if [ ! -f "$project_json" ]; then
            log_warn "Skipping $project_name: stackvo.json not found"
            continue
        fi
        
        log_info "Processing project: $project_name"
        
        # Parse project configuration
        local config=$(parse_project_config "$project_json" "$project_name")
        if [ $? -ne 0 ]; then
            log_error "Failed to parse config for $project_name"
            continue
        fi
        
        # Extract config values
        local php_version=$(echo "$config" | grep "^PHP_VERSION=" | cut -d= -f2)
        local web_server=$(echo "$config" | grep "^WEB_SERVER=" | cut -d= -f2)
        local project_domain=$(echo "$config" | grep "^DOMAIN=" | cut -d= -f2)
        local document_root=$(echo "$config" | grep "^DOCUMENT_ROOT=" | cut -d= -f2)
        local extensions=$(echo "$config" | grep "^EXTENSIONS=" | cut -d= -f2-)
        
        # Calculate host paths for volume mounts
        local host_project_path="${HOST_ROOT_DIR}/projects/${project_name}"
        local host_logs_path="${HOST_ROOT_DIR}/logs/projects/${project_name}"
        local host_generated_configs_dir="${HOST_ROOT_DIR}/generated/configs"
        local host_generated_projects_dir="${HOST_ROOT_DIR}/generated/projects"
        
        # Generate Dockerfile for PHP container with extensions
        local project_dockerfile_dir="$GENERATED_DIR/projects/${project_name}"
        generate_php_dockerfile "$project_name" "$php_version" "$extensions" "$project_dockerfile_dir"
        
        # Generate PHP container
        generate_php_container "$project_name" "$project_path" "$php_version" "$host_project_path" "$host_logs_path" "$host_generated_projects_dir" >> "$output"
        
        # Generate web server container
        generate_web_container "$project_name" "$project_path" "$web_server" "$php_version" "$project_domain" "$document_root" "$host_project_path" "$host_logs_path" "$host_generated_configs_dir" >> "$output"
    done
    
    # Add network definition at the end
    echo "" >> "$output"
    echo "networks:" >> "$output"
    echo "  ${DOCKER_DEFAULT_NETWORK:-$CONST_DEFAULT_NETWORK}:" >> "$output"
    echo "    external: true" >> "$output"
    
    log_success "Generated docker-compose.projects.yml"
}

##
# Parse project JSON configuration
#
# Args:
#   $1 - Path to stackvo.json
#   $2 - Project name
#
# Returns:
#   Config values as KEY=VALUE pairs
##
parse_project_config() {
    local project_json=$1
    local project_name=$2
    
    # Parse JSON using sed (macOS BSD awk compatible)
    local php_version=$(sed -n 's/.*"version"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$project_json")
    local web_server=$(sed -n 's/.*"webserver"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$project_json")
    local project_domain=$(sed -n 's/.*"domain"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$project_json")
    local document_root=$(sed -n 's/.*"document_root"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$project_json")
    
    # Parse extensions array from JSON (extract values between brackets, remove quotes and commas)
    local extensions=$(sed -n 's/.*"extensions"[[:space:]]*:[[:space:]]*\[\([^]]*\)\].*/\1/p' "$project_json" | tr -d '",' | tr -s ' ')
    
    # Default document_root to "public" if not specified
    if [ -z "$document_root" ]; then
        document_root="public"
    fi
    
    # Default extensions if not specified
    if [ -z "$extensions" ]; then
        extensions="pdo pdo_mysql mysqli gd curl zip mbstring"
    fi
    
    # Validate and set defaults for missing values
    if [ -z "$php_version" ]; then
        log_warn "PHP version not found in $project_json, using default: ${DEFAULT_PHP_VERSION:-$CONST_DEFAULT_PHP_VERSION}"
        php_version="${DEFAULT_PHP_VERSION:-$CONST_DEFAULT_PHP_VERSION}"
    fi
    
    if [ -z "$web_server" ]; then
        log_warn "Webserver not found in $project_json, using default: ${DEFAULT_WEBSERVER:-$CONST_DEFAULT_WEBSERVER}"
        web_server="${DEFAULT_WEBSERVER:-$CONST_DEFAULT_WEBSERVER}"
    fi
    
    if [ -z "$project_domain" ]; then
        log_error "Domain not found in $project_json for project $project_name"
        return 1
    fi
    
    # Output config
    echo "PHP_VERSION=$php_version"
    echo "WEB_SERVER=$web_server"
    echo "DOMAIN=$project_domain"
    echo "DOCUMENT_ROOT=$document_root"
    echo "EXTENSIONS=$extensions"
}

##
# Generate PHP Dockerfile with extensions
#
# Args:
#   $1 - Project name
#   $2 - PHP version
#   $3 - Extensions (space-separated list)
#   $4 - Project directory path in generated/projects/
##
generate_php_dockerfile() {
    local project_name=$1
    local php_version=$2
    local extensions=$3
    local project_dir=$4
    
    # Load PHP extensions library
    source "$SCRIPT_DIR/../lib/php-extensions.sh"
    
    # Create project directory
    mkdir -p "$project_dir"
    local dockerfile="$project_dir/Dockerfile"
    
    # Collect dependencies and configure commands
    local apt_packages=""
    local configure_commands=""
    local docker_ext_install=""
    local pecl_install=""
    
    for ext in $extensions; do
        # Get system packages
        local packages=$(get_extension_packages "$ext")
        if [ -n "$packages" ]; then
            apt_packages="$apt_packages $packages"
        fi
        
        # Get configure command
        local configure=$(get_extension_configure "$ext")
        if [ -n "$configure" ]; then
            configure_commands="$configure_commands\nRUN $configure"
        fi
        
        # PECL or docker-php-ext-install?
        if is_pecl_extension "$ext"; then
            pecl_install="$pecl_install $ext"
        else
            docker_ext_install="$docker_ext_install $ext"
        fi
    done
    
    # Remove duplicate packages
    apt_packages=$(echo "$apt_packages" | tr ' ' '\n' | sort -u | tr '\n' ' ')
    
    # Generate Dockerfile
    cat > "$dockerfile" <<EOF
# Auto-generated Dockerfile for $project_name
# PHP Version: $php_version
# Extensions: $extensions
FROM php:${php_version}-fpm

EOF
    
    # Install system dependencies
    if [ -n "$apt_packages" ]; then
        cat >> "$dockerfile" <<EOF
# Install system dependencies
RUN apt-get update && apt-get install -y \\
    $apt_packages \\
    && rm -rf /var/lib/apt/lists/*

EOF
    fi
    
    # Add configure commands
    if [ -n "$configure_commands" ]; then
        echo -e "$configure_commands" >> "$dockerfile"
        echo "" >> "$dockerfile"
    fi
    
    # Install PHP extensions
    if [ -n "$docker_ext_install" ]; then
        cat >> "$dockerfile" <<EOF
# Install PHP extensions
RUN docker-php-ext-install$docker_ext_install

EOF
    fi
    
    # Install PECL extensions
    if [ -n "$pecl_install" ]; then
        cat >> "$dockerfile" <<EOF
# Install PECL extensions
RUN pecl install$pecl_install \\
    && docker-php-ext-enable$pecl_install

EOF
    fi
    
    # Install default development tools
    local default_tools=${PHP_DEFAULT_TOOLS:-""}
    if [ -n "$default_tools" ]; then
        cat >> "$dockerfile" <<EOF

# Install Development Tools
EOF
        
        local composer_version=${PHP_TOOL_COMPOSER_VERSION:-latest}
        local nodejs_version=${PHP_TOOL_NODEJS_VERSION:-20}
        
        for tool in $(echo "$default_tools" | tr ',' ' '); do
            local install_cmd=$(get_tool_install_commands "$tool" "$composer_version" "$nodejs_version")
            if [ -n "$install_cmd" ]; then
                echo "$install_cmd" >> "$dockerfile"
                echo "" >> "$dockerfile"
            fi
        done
    fi
    
    cat >> "$dockerfile" <<EOF
WORKDIR /var/www/html
EOF
    
    
    log_info "$project_name için Dockerfile oluşturuldu: $dockerfile"
}

##
# Generate PHP container configuration

#
# Args:
#   $1 - Project name
#   $2 - Project path (container path)
#   $3 - PHP version
#   $4 - Host project path (for volume mounts)
#   $5 - Host logs path (for volume mounts)
##
##
generate_php_container() {
    local project_name=$1
    local project_path=$2
    local php_version=$3
    local host_project_path=$4
    local host_logs_path=$5
    local host_generated_projects_dir=$6
    
    cat <<EOF
  ${project_name}-php:
    build:
      context: ${host_generated_projects_dir}/${project_name}
      dockerfile: Dockerfile
    image: stackvo-${project_name}-php:${php_version}
    container_name: "stackvo-${project_name}-php"
    restart: unless-stopped
    
    volumes:
      - ${host_project_path}:/var/www/html
      - ${host_logs_path}:/var/log/${project_name}
EOF
    
    # Add custom PHP config if exists
    if [ -f "$project_path/$CONST_STACKVO_CONFIG_DIR/$CONST_CONFIG_PHP_INI" ]; then
        echo "      - ${project_path}/$CONST_STACKVO_CONFIG_DIR/$CONST_CONFIG_PHP_INI:/usr/local/etc/php/conf.d/custom.ini:ro"
    fi
    
    # Add custom PHP-FPM config if exists
    if [ -f "$project_path/$CONST_STACKVO_CONFIG_DIR/$CONST_CONFIG_PHP_FPM" ]; then
        echo "      - ${project_path}/$CONST_STACKVO_CONFIG_DIR/$CONST_CONFIG_PHP_FPM:/usr/local/etc/php-fpm.d/zz-custom.conf:ro"
    fi
    
    cat <<EOF
    
    networks:
      - ${DOCKER_DEFAULT_NETWORK:-$CONST_DEFAULT_NETWORK}

EOF
}

##
# Generate web server container configuration
#
# Args:
#   $1 - Project name
#   $2 - Project path (container path)
#   $3 - Web server type
#   $4 - PHP version
#   $5 - Project domain
#   $6 - Document root
#   $7 - Host project path (for volume mounts)
#   $8 - Host logs path (for volume mounts)
#   $9 - Host generated configs dir (for volume mounts)
##
generate_web_container() {
    local project_name=$1
    local project_path=$2
    local web_server=$3
    local php_version=$4
    local project_domain=$5
    local document_root=$6
    local host_project_path=$7
    local host_logs_path=$8
    local host_generated_configs_dir=$9
    
    case "$web_server" in
        nginx|"$CONST_DEFAULT_WEBSERVER")
            generate_nginx_container "$project_name" "$project_path" "$project_domain" "$host_project_path" "$host_logs_path" "$host_generated_configs_dir"
            ;;
        apache)
            generate_apache_container "$project_name" "$project_path" "$php_version" "$project_domain" "$host_project_path" "$host_logs_path"
            ;;
        caddy)
            generate_caddy_container "$project_name" "$project_path" "$project_domain" "$document_root" "$host_project_path" "$host_logs_path" "$host_generated_configs_dir"
            ;;
        *)
            log_warn "Bilinmeyen web server: $web_server, nginx kullanılıyor"
            generate_nginx_container "$project_name" "$project_path" "$project_domain" "$host_project_path" "$host_logs_path" "$host_generated_configs_dir"
            ;;
    esac
}

##
# Generate Nginx container configuration
#
# Args:
#   $1 - Project name
#   $2 - Project path (container path)
#   $3 - Project domain
#   $4 - Host project path (for volume mounts)
#   $5 - Host logs path (for volume mounts)
#   $6 - Host generated configs dir (for volume mounts)
##
generate_nginx_container() {
    local project_name=$1
    local project_path=$2
    local project_domain=$3
    local host_project_path=$4
    local host_logs_path=$5
    local host_generated_configs_dir=$6
    
    # Determine nginx config path
    local nginx_config_mount=""
    if [ -f "$project_path/$CONST_STACKVO_CONFIG_DIR/$CONST_CONFIG_NGINX" ]; then
        nginx_config_mount="      - ${host_project_path}/$CONST_STACKVO_CONFIG_DIR/$CONST_CONFIG_NGINX:/etc/nginx/conf.d/default.conf:ro"
    elif [ -f "$project_path/$CONST_CONFIG_NGINX" ]; then
        nginx_config_mount="      - ${host_project_path}/$CONST_CONFIG_NGINX:/etc/nginx/conf.d/default.conf:ro"
    else
        # Use default template - generate in core/generated/configs/
        mkdir -p "$GENERATED_CONFIGS_DIR"
        local template_file="$ROOT_DIR/$CONST_PATH_TEMPLATES/servers/nginx/default.conf"
        local generated_config="$GENERATED_CONFIGS_DIR/${project_name}-nginx.conf"
        
        # Generate config from template
        sed "s/{{PROJECT_NAME}}/${project_name}/g" "$template_file" > "$generated_config"
        nginx_config_mount="      - ${host_generated_configs_dir}/${project_name}-nginx.conf:/etc/nginx/conf.d/default.conf:ro"
    fi
    
    cat <<EOF
  ${project_name}-web:
    image: "$CONST_IMAGE_NGINX"
    container_name: "stackvo-${project_name}-web"
    restart: unless-stopped
    
    volumes:
      - ${host_project_path}:/var/www/html
      - ${host_logs_path}:/var/log/nginx
$nginx_config_mount
    
    networks:
      - ${DOCKER_DEFAULT_NETWORK:-stackvo-net}
    
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.${project_name}.rule=Host(\`${project_domain}\`)"
      - "traefik.http.routers.${project_name}.entrypoints=websecure"
      - "traefik.http.routers.${project_name}.tls=true"
      - "traefik.http.services.${project_name}.loadbalancer.server.port=80"
    
    depends_on:
      - ${project_name}-php

EOF
}

##
# Generate Apache container configuration
#
# Args:
#   $1 - Project name
#   $2 - Project path (container path)
#   $3 - PHP version
#   $4 - Project domain
#   $5 - Host project path (for volume mounts)
#   $6 - Host logs path (for volume mounts)
##
generate_apache_container() {
    local project_name=$1
    local project_path=$2
    local php_version=$3
    local project_domain=$4
    local host_project_path=$5
    local host_logs_path=$6
    
    # Determine apache config path
    local apache_config_mount=""
    if [ -f "$project_path/.stackvo/apache.conf" ]; then
        apache_config_mount="      - ${host_project_path}/.stackvo/apache.conf:/etc/apache2/sites-available/000-default.conf:ro"
    elif [ -f "$project_path/apache.conf" ]; then
        apache_config_mount="      - ${host_project_path}/apache.conf:/etc/apache2/sites-available/000-default.conf:ro"
    else
        # Use default template
        mkdir -p "$GENERATED_CONFIGS_DIR"
        local template_file="$ROOT_DIR/core/templates/servers/apache/default.conf"
        local generated_config="$GENERATED_CONFIGS_DIR/${project_name}-apache.conf"
        cp "$template_file" "$generated_config"
        apache_config_mount="      - ${generated_config}:/etc/apache2/sites-available/000-default.conf:ro"
    fi
    
    cat <<EOF
  ${project_name}-web:
    image: "php:${php_version:-8.2}-apache"
    container_name: "stackvo-${project_name}-web"
    restart: unless-stopped
    
    volumes:
      - ${host_project_path}:/var/www/html
      - ${host_logs_path}:/var/log/apache2
$apache_config_mount
    
EOF
    
    # If no custom config, add command to set DocumentRoot
    if [ -z "$apache_config_mount" ]; then
        cat <<EOF
    command: >
      bash -c "sed -i 's|DocumentRoot /var/www/html|DocumentRoot /var/www/html/public|g' /etc/apache2/sites-available/000-default.conf &&
               sed -i '/\<VirtualHost/a\    \<Directory /var/www/html/public\>\n        Options Indexes FollowSymLinks\n        AllowOverride All\n        Require all granted\n    \</Directory\>' /etc/apache2/sites-available/000-default.conf &&
               apache2-foreground"
    
EOF
    fi
    
    cat <<EOF
    networks:
      - ${DOCKER_DEFAULT_NETWORK:-stackvo-net}
    
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.${project_name}.rule=Host(\`${project_domain}\`)"
      - "traefik.http.routers.${project_name}.entrypoints=websecure"
      - "traefik.http.routers.${project_name}.tls=true"
      - "traefik.http.services.${project_name}.loadbalancer.server.port=80"
    
    depends_on:
      - ${project_name}-php

EOF
}

##
# Generate Caddy container configuration
#
# Args:
#   $1 - Project name
#   $2 - Project path (container path)
#   $3 - Project domain
#   $4 - Document root
#   $5 - Host project path (for volume mounts)
#   $6 - Host logs path (for volume mounts)
#   $7 - Host generated configs dir (for volume mounts)
##
generate_caddy_container() {
    local project_name=$1
    local project_path=$2
    local project_domain=$3
    local document_root=$4
    local host_project_path=$5
    local host_logs_path=$6
    local host_generated_configs_dir=$7
    
    # Determine caddy config path
    local caddy_config_mount=""
    if [ -f "$project_path/.stackvo/Caddyfile" ]; then
        caddy_config_mount="      - ${host_project_path}/.stackvo/Caddyfile:/etc/caddy/Caddyfile:ro"
    elif [ -f "$project_path/Caddyfile" ]; then
        caddy_config_mount="      - ${host_project_path}/Caddyfile:/etc/caddy/Caddyfile:ro"
    else
        # Use default template
        mkdir -p "$GENERATED_CONFIGS_DIR"
        local template_file="$ROOT_DIR/core/templates/servers/caddy/Caddyfile"
        local generated_config="$GENERATED_CONFIGS_DIR/${project_name}-caddy.conf"
        
        sed -e "s/{{PROJECT_NAME}}/${project_name}/g" \
            -e "s|{{DOCUMENT_ROOT}}|${document_root}|g" \
            "$template_file" > "$generated_config"
        caddy_config_mount="      - ${host_generated_configs_dir}/${project_name}-caddy.conf:/etc/caddy/Caddyfile:ro"
    fi
    
    cat <<EOF
  ${project_name}-web:
    image: "caddy:latest"
    container_name: "stackvo-${project_name}-web"
    restart: unless-stopped
    
    volumes:
      - ${host_project_path}:/var/www/html
      - ${host_logs_path}:/var/log/caddy
$caddy_config_mount
    
    networks:
      - ${DOCKER_DEFAULT_NETWORK:-stackvo-net}
    
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.${project_name}.rule=Host(\`${project_domain}\`)"
      - "traefik.http.routers.${project_name}.entrypoints=websecure"
      - "traefik.http.routers.${project_name}.tls=true"
      - "traefik.http.services.${project_name}.loadbalancer.server.port=80"
    
    depends_on:
      - ${project_name}-php

EOF
}

