#!/bin/bash
###################################################################
# STACKVO PROJECT GENERATOR MODULE
# Generating project containers
###################################################################

##
# Sanitize project name for Traefik labels
# Traefik uses dots as separators, so we need to replace them
#
# Args:
#   $1 - Project name
#
# Returns:
#   Sanitized project name (dots replaced with dashes)
##
sanitize_project_name_for_traefik() {
    local project_name=$1
    # Replace dots with dashes for Traefik compatibility
    echo "$project_name" | tr '.' '-'
}

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
            # We're on the host, use HOST_UID and HOST_GID from .env
            if [ -n "${HOST_UID}" ] && [ -n "${HOST_GID}" ]; then
                sudo chown -R "${HOST_UID}:${HOST_GID}" "$GENERATED_DIR" 2>/dev/null || true
                chmod -R 755 "$GENERATED_DIR" 2>/dev/null || true
                log_info "Set generated directory ownership to ${HOST_UID}:${HOST_GID}"
            else
                log_warn "HOST_UID or HOST_GID not set, using chmod 777"
                chmod -R 777 "$GENERATED_DIR" 2>/dev/null || true
            fi
        fi
    fi
    
    local output="$GENERATED_DIR/docker-compose.projects.yml"
    local projects_dir="$ROOT_DIR/projects"
    
    # Start with project name
    echo "name: stackvo" > "$output"
    echo "" >> "$output"
    
    # Check if projects directory exists
    if [ ! -d "$projects_dir" ]; then
        log_info "No projects directory found"
        # Create valid empty services mapping (required by Docker Compose)
        echo "services: {}" >> "$output"
        # Still add network definition even if no projects
        echo "" >> "$output"
        echo "networks:" >> "$output"
        echo "  ${DOCKER_DEFAULT_NETWORK:-$CONST_DEFAULT_NETWORK}:" >> "$output"
        echo "    external: true" >> "$output"
        return
    fi
    
    # Projects directory exists, start services block
    echo "services:" >> "$output"
    echo "" >> "$output"
    
    # Track if we processed any valid projects
    local project_count=0
    
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
        
        # Generate Dockerfile for single container (web server + PHP)
        local project_dockerfile_dir="$GENERATED_DIR/projects/${project_name}"
        generate_single_dockerfile "$project_name" "$web_server" "$php_version" "$extensions" "$project_dockerfile_dir" "$document_root"
        
        # Generate single container (web server + PHP combined)
        generate_single_container "$project_name" "$project_path" "$web_server" "$php_version" "$project_domain" "$document_root" "$host_project_path" "$host_logs_path" "$host_generated_configs_dir" "$host_generated_projects_dir" >> "$output"
        
        # Increment project counter
        ((project_count++))
    done
    
    # If no valid projects were processed, recreate file with empty services mapping
    if [ $project_count -eq 0 ]; then
        log_info "No valid projects found (projects directory exists but no stackvo.json files)"
        # Recreate file with empty services mapping
        echo "name: stackvo" > "$output"
        echo "" >> "$output"
        echo "services: {}" >> "$output"
        echo "" >> "$output"
        echo "networks:" >> "$output"
        echo "  ${DOCKER_DEFAULT_NETWORK:-$CONST_DEFAULT_NETWORK}:" >> "$output"
        echo "    external: true" >> "$output"
        return
    fi
    
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
# Generate Single Container Dockerfile (Web Server + PHP)
#
# Args:
#   $1 - Project name
#   $2 - Web server (nginx/apache/caddy)
#   $3 - PHP version
#   $4 - Extensions (space-separated list)
#   $5 - Project directory path in generated/projects/
#   $6 - Document root
##
generate_single_dockerfile() {
    local project_name=$1
    local web_server=$2
    local php_version=$3
    local extensions=$4
    local project_dir=$5
    local document_root=$6
    
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
            configure_commands="$configure_commands\\nRUN $configure"
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
    
    # Generate Dockerfile based on web server
    case "$web_server" in
        apache)
            generate_apache_dockerfile "$dockerfile" "$php_version" "$apt_packages" "$configure_commands" "$docker_ext_install" "$pecl_install" "$project_name"
            ;;
        nginx)
            generate_nginx_dockerfile "$dockerfile" "$php_version" "$apt_packages" "$configure_commands" "$docker_ext_install" "$pecl_install" "$project_name" "$project_dir" "$document_root"
            ;;
        caddy)
            generate_caddy_dockerfile "$dockerfile" "$php_version" "$apt_packages" "$configure_commands" "$docker_ext_install" "$pecl_install" "$project_name" "$project_dir" "$document_root"
            ;;
        *)
            log_warn "Bilinmeyen web server: $web_server, nginx kullanılıyor"
            generate_nginx_dockerfile "$dockerfile" "$php_version" "$apt_packages" "$configure_commands" "$docker_ext_install" "$pecl_install" "$project_name" "$project_dir" "$document_root"
            ;;
    esac
    
    log_info "$project_name için Dockerfile oluşturuldu: $dockerfile"
}

##
# Generate Apache Dockerfile (Apache + mod_php)
##
generate_apache_dockerfile() {
    local dockerfile=$1
    local php_version=$2
    local apt_packages=$3
    local configure_commands=$4
    local docker_ext_install=$5
    local pecl_install=$6
    local project_name=$7
    
    # Get default tools
    local default_tools=${PHP_DEFAULT_TOOLS:-""}
    local composer_version=${PHP_TOOL_COMPOSER_VERSION:-latest}
    local nodejs_version=${PHP_TOOL_NODEJS_VERSION:-20}
    
    cat > "$dockerfile" <<EOF
# Auto-generated Dockerfile for $project_name
# Web Server: Apache + mod_php
# PHP Version: $php_version
FROM php:${php_version}-apache

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
    
    # Install development tools
    if [ -n "$default_tools" ]; then
        cat >> "$dockerfile" <<EOF

# Install Development Tools
EOF
        for tool in $(echo "$default_tools" | tr ',' ' '); do
            local install_cmd=$(get_tool_install_commands "$tool" "$composer_version" "$nodejs_version")
            if [ -n "$install_cmd" ]; then
                echo "$install_cmd" >> "$dockerfile"
                echo "" >> "$dockerfile"
            fi
        done
    fi
    
    # Enable Apache modules
    cat >> "$dockerfile" <<EOF

# Enable Apache modules
RUN a2enmod rewrite

WORKDIR /var/www/html
EOF
}

##
# Generate Nginx Dockerfile (Nginx + PHP-FPM + Supervisord)
##
generate_nginx_dockerfile() {
    local dockerfile=$1
    local php_version=$2
    local apt_packages=$3
    local configure_commands=$4
    local docker_ext_install=$5
    local pecl_install=$6
    local project_name=$7
    local project_dir=$8
    local document_root=$9
    
    # Get default tools
    local default_tools=${PHP_DEFAULT_TOOLS:-""}
    local composer_version=${PHP_TOOL_COMPOSER_VERSION:-latest}
    local nodejs_version=${PHP_TOOL_NODEJS_VERSION:-20}
    
    cat > "$dockerfile" <<EOF
# Auto-generated Dockerfile for $project_name
# Web Server: Nginx + PHP-FPM
# PHP Version: $php_version
FROM php:${php_version}-fpm

# Install Nginx and Supervisord
RUN apt-get update && apt-get install -y \\
    nginx \\
    supervisor \\
    && rm -rf /var/lib/apt/lists/*

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
    
    # Install development tools
    if [ -n "$default_tools" ]; then
        cat >> "$dockerfile" <<EOF

# Install Development Tools
EOF
        for tool in $(echo "$default_tools" | tr ',' ' '); do
            local install_cmd=$(get_tool_install_commands "$tool" "$composer_version" "$nodejs_version")
            if [ -n "$install_cmd" ]; then
                echo "$install_cmd" >> "$dockerfile"
                echo "" >> "$dockerfile"
            fi
        done
    fi
    
    # Generate Nginx config dynamically
    cat > "$project_dir/nginx.conf" <<'NGINXCONF'
server {
    listen 80;
    server_name _;
    
    # Explicit log paths
    access_log /var/log/nginx/access.log;
    error_log /var/log/nginx/error.log;
    
    root /var/www/html/DOCUMENT_ROOT_PLACEHOLDER;
    index index.php index.html;

    location / {
        try_files $uri $uri/ /index.php?$query_string;
    }

    location ~ \.php$ {
        fastcgi_pass 127.0.0.1:9000;
        fastcgi_index index.php;
        fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;
        include fastcgi_params;
    }

    location ~ /\.ht {
        deny all;
    }
}
NGINXCONF

    # Replace placeholder with actual document root (cross-platform compatible)
    sed "s|DOCUMENT_ROOT_PLACEHOLDER|$document_root|g" "$project_dir/nginx.conf" > "$project_dir/nginx.conf.tmp" && mv "$project_dir/nginx.conf.tmp" "$project_dir/nginx.conf"
    
    # Generate Supervisord config dynamically
    cat > "$project_dir/supervisord.conf" <<'SUPERVISORCONF'
[supervisord]
nodaemon=true
user=root
logfile=/var/log/supervisord.log
pidfile=/var/run/supervisord.pid

[program:php-fpm]
command=/usr/local/sbin/php-fpm -F
autostart=true
autorestart=true
stdout_logfile=/dev/stdout
stdout_logfile_maxbytes=0
stderr_logfile=/dev/stderr
stderr_logfile_maxbytes=0

[program:nginx]
command=/usr/sbin/nginx -g 'daemon off;'
autostart=true
autorestart=true
stdout_logfile=/dev/stdout
stdout_logfile_maxbytes=0
stderr_logfile=/dev/stderr
stderr_logfile_maxbytes=0
SUPERVISORCONF
    
    cat >> "$dockerfile" <<'DOCKEREOF'

# Configure PHP-FPM to listen on TCP port 127.0.0.1:9000
RUN sed -i 's|listen = .*|listen = 127.0.0.1:9000|' /usr/local/etc/php-fpm.d/www.conf

# Remove 'main' log format reference from Nginx config
RUN sed -i 's/ main;/;/' /etc/nginx/nginx.conf

# Disable default Nginx site (it conflicts with our config)
RUN rm -f /etc/nginx/sites-enabled/default

# Copy Nginx configuration
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Copy Supervisord configuration
COPY supervisord.conf /etc/supervisor/conf.d/supervisord.conf

# Create entrypoint script to ensure log directories exist at runtime
RUN echo '#!/bin/bash' > /entrypoint.sh && \
    echo 'mkdir -p /var/log/nginx /var/log/php-fpm' >> /entrypoint.sh && \
    echo 'touch /var/log/nginx/access.log /var/log/nginx/error.log' >> /entrypoint.sh && \
    echo 'chmod 666 /var/log/nginx/*.log' >> /entrypoint.sh && \
    echo 'exec "$@"' >> /entrypoint.sh && \
    chmod +x /entrypoint.sh

WORKDIR /var/www/html

# Use entrypoint to create log directories before starting supervisord
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/usr/bin/supervisord", "-c", "/etc/supervisor/conf.d/supervisord.conf"]
DOCKEREOF
}

##
# Generate Caddy Dockerfile (Caddy + PHP-FPM + Supervisord)
##
generate_caddy_dockerfile() {
    local dockerfile=$1
    local php_version=$2
    local apt_packages=$3
    local configure_commands=$4
    local docker_ext_install=$5
    local pecl_install=$6
    local project_name=$7
    local project_dir=$8
    local document_root=$9
    
    # Get default tools
    local default_tools=${PHP_DEFAULT_TOOLS:-""}
    local composer_version=${PHP_TOOL_COMPOSER_VERSION:-latest}
    local nodejs_version=${PHP_TOOL_NODEJS_VERSION:-20}
    
    cat > "$dockerfile" <<EOF
# Auto-generated Dockerfile for $project_name
# Web Server: Caddy + PHP-FPM
# PHP Version: $php_version
FROM php:${php_version}-fpm

# Install Supervisord and dependencies
RUN apt-get update && apt-get install -y \\
    curl \\
    ca-certificates \\
    supervisor \\
    && rm -rf /var/lib/apt/lists/*

# Install Caddy from official binary (cross-platform compatible, no GPG issues)
ARG CADDY_VERSION=2.8.4
RUN curl -o /tmp/caddy.tar.gz -L "https://github.com/caddyserver/caddy/releases/download/v\${CADDY_VERSION}/caddy_\${CADDY_VERSION}_linux_amd64.tar.gz" \\
    && tar -xzf /tmp/caddy.tar.gz -C /usr/bin caddy \\
    && chmod +x /usr/bin/caddy \\
    && rm /tmp/caddy.tar.gz \\
    && caddy version


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
    
    # Install development tools
    if [ -n "$default_tools" ]; then
        cat >> "$dockerfile" <<EOF

# Install Development Tools
EOF
        for tool in $(echo "$default_tools" | tr ',' ' '); do
            local install_cmd=$(get_tool_install_commands "$tool" "$composer_version" "$nodejs_version")
            if [ -n "$install_cmd" ]; then
                echo "$install_cmd" >> "$dockerfile"
                echo "" >> "$dockerfile"
            fi
        done
    fi
    
    # Generate Caddyfile dynamically
    cat > "$project_dir/Caddyfile" <<'CADDYCONF'
:80 {
    root * /var/www/html/DOCUMENT_ROOT_PLACEHOLDER
    
    # Enable PHP-FPM (localhost - same container)
    php_fastcgi 127.0.0.1:9000
    
    # Enable file server
    file_server
    
    # Logging
    log {
        output stdout
        format console
    }
}
CADDYCONF

    # Replace placeholder with actual document root (cross-platform compatible)
    sed "s|DOCUMENT_ROOT_PLACEHOLDER|$document_root|g" "$project_dir/Caddyfile" > "$project_dir/Caddyfile.tmp" && mv "$project_dir/Caddyfile.tmp" "$project_dir/Caddyfile"
    
    # Generate Supervisord config dynamically
    cat > "$project_dir/supervisord.conf" <<'SUPERVISORCONF'
[supervisord]
nodaemon=true
user=root
logfile=/var/log/supervisord.log
pidfile=/var/run/supervisord.pid

[program:php-fpm]
command=/usr/local/sbin/php-fpm -F
autostart=true
autorestart=true
stdout_logfile=/dev/stdout
stdout_logfile_maxbytes=0
stderr_logfile=/dev/stderr
stderr_logfile_maxbytes=0

[program:caddy]
command=/usr/bin/caddy run --config /etc/caddy/Caddyfile
autostart=true
autorestart=true
stdout_logfile=/dev/stdout
stdout_logfile_maxbytes=0
stderr_logfile=/dev/stderr
stderr_logfile_maxbytes=0
SUPERVISORCONF
    
    cat >> "$dockerfile" <<'DOCKEREOF'

# Configure PHP-FPM to listen on TCP port 127.0.0.1:9000
RUN sed -i 's|listen = .*|listen = 127.0.0.1:9000|' /usr/local/etc/php-fpm.d/www.conf

# Copy Caddyfile
COPY Caddyfile /etc/caddy/Caddyfile

# Copy Supervisord configuration
COPY supervisord.conf /etc/supervisor/conf.d/supervisord.conf

# Create entrypoint script to ensure log directories exist at runtime
RUN echo '#!/bin/bash' > /entrypoint.sh && \
    echo 'mkdir -p /var/log/caddy /var/log/php-fpm' >> /entrypoint.sh && \
    echo 'touch /var/log/caddy/access.log /var/log/caddy/error.log' >> /entrypoint.sh && \
    echo 'chmod 666 /var/log/caddy/*.log' >> /entrypoint.sh && \
    echo 'exec "$@"' >> /entrypoint.sh && \
    chmod +x /entrypoint.sh

WORKDIR /var/www/html

# Use entrypoint to create log directories before starting supervisord
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/usr/bin/supervisord", "-c", "/etc/supervisor/conf.d/supervisord.conf"]
DOCKEREOF
}

##
# Generate Single Container (Web Server + PHP combined)
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
#   $10 - Host generated projects dir (for volume mounts)
##
generate_single_container() {
    local project_name=$1
    local project_path=$2
    local web_server=$3
    local php_version=$4
    local project_domain=$5
    local document_root=$6
    local host_project_path=$7
    local host_logs_path=$8
    local host_generated_configs_dir=$9
    local host_generated_projects_dir=${10}
    
    case "$web_server" in
        apache)
            generate_apache_single_container "$project_name" "$project_path" "$php_version" "$project_domain" "$host_project_path" "$host_logs_path" "$host_generated_projects_dir"
            ;;
        nginx)
            generate_nginx_single_container "$project_name" "$project_path" "$project_domain" "$host_project_path" "$host_logs_path" "$host_generated_configs_dir" "$host_generated_projects_dir"
            ;;
        caddy)
            generate_caddy_single_container "$project_name" "$project_path" "$project_domain" "$document_root" "$host_project_path" "$host_logs_path" "$host_generated_configs_dir" "$host_generated_projects_dir"
            ;;
        *)
            log_warn "Bilinmeyen web server: $web_server, nginx kullanılıyor"
            generate_nginx_single_container "$project_name" "$project_path" "$project_domain" "$host_project_path" "$host_logs_path" "$host_generated_configs_dir" "$host_generated_projects_dir"
            ;;
    esac
}

##
# Generate Apache Single Container
##
generate_apache_single_container() {
    local project_name=$1
    local project_path=$2
    local php_version=$3
    local project_domain=$4
    local host_project_path=$5
    local host_logs_path=$6
    local host_generated_projects_dir=$7
    
    # Sanitize project name for Traefik (replace dots with dashes)
    local traefik_safe_name=$(sanitize_project_name_for_traefik "$project_name")
    
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
  ${project_name}:
    profiles: ["projects", "project-${project_name}"]  # --projects ile tümü, --profile project-{name} ile sadece bu proje
    build:
      context: ./projects/${project_name}
      dockerfile: Dockerfile
    image: stackvo-${project_name}:${php_version}
    container_name: "stackvo-${project_name}"
    restart: unless-stopped
    
    volumes:
      - ${host_project_path}:/var/www/html
      - ${host_logs_path}:/var/log/apache2
$apache_config_mount
    
    networks:
      - ${DOCKER_DEFAULT_NETWORK:-stackvo-net}
    
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.${traefik_safe_name}.rule=Host(\`${project_domain}\`)"
      - "traefik.http.routers.${traefik_safe_name}.entrypoints=websecure"
      - "traefik.http.routers.${traefik_safe_name}.tls=true"
      - "traefik.http.services.${traefik_safe_name}.loadbalancer.server.port=80"

EOF
}

##
# Generate Nginx Single Container
##
generate_nginx_single_container() {
    local project_name=$1
    local project_path=$2
    local project_domain=$3
    local host_project_path=$4
    local host_logs_path=$5
    local host_generated_configs_dir=$6
    local host_generated_projects_dir=$7
    
    # Sanitize project name for Traefik (replace dots with dashes)
    local traefik_safe_name=$(sanitize_project_name_for_traefik "$project_name")
    
    # Nginx config is already generated in generate_nginx_dockerfile()
    # and copied into the container via Dockerfile COPY command
    # No need to mount it separately
    local nginx_config_mount=""
    if [ -f "$project_path/$CONST_STACKVO_CONFIG_DIR/$CONST_CONFIG_NGINX" ]; then
        # User has custom nginx config in .stackvo/
        nginx_config_mount="      - ${host_project_path}/$CONST_STACKVO_CONFIG_DIR/$CONST_CONFIG_NGINX:/etc/nginx/conf.d/default.conf:ro"
    elif [ -f "$project_path/$CONST_CONFIG_NGINX" ]; then
        # User has custom nginx config in project root
        nginx_config_mount="      - ${host_project_path}/$CONST_CONFIG_NGINX:/etc/nginx/conf.d/default.conf:ro"
    fi
    # If no custom config, the dynamically generated one from Dockerfile will be used
    
    
    cat <<EOF
  ${project_name}:
    profiles: ["projects", "project-${project_name}"]  # --projects ile tümü, --profile project-{name} ile sadece bu proje
    build:
      context: ./projects/${project_name}
      dockerfile: Dockerfile
    image: stackvo-${project_name}:latest
    container_name: "stackvo-${project_name}"
    restart: unless-stopped
    
    volumes:
      - ${host_project_path}:/var/www/html
      - ${host_logs_path}:/var/log
$nginx_config_mount
    
    networks:
      - ${DOCKER_DEFAULT_NETWORK:-stackvo-net}
    
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.${traefik_safe_name}.rule=Host(\`${project_domain}\`)"
      - "traefik.http.routers.${traefik_safe_name}.entrypoints=websecure"
      - "traefik.http.routers.${traefik_safe_name}.tls=true"
      - "traefik.http.services.${traefik_safe_name}.loadbalancer.server.port=80"

EOF
}

##
# Generate Caddy Single Container
##
generate_caddy_single_container() {
    local project_name=$1
    local project_path=$2
    local project_domain=$3
    local document_root=$4
    local host_project_path=$5
    local host_logs_path=$6
    local host_generated_configs_dir=$7
    local host_generated_projects_dir=$8
    
    # Sanitize project name for Traefik (replace dots with dashes)
    local traefik_safe_name=$(sanitize_project_name_for_traefik "$project_name")
    
    # Caddyfile is already generated in generate_caddy_dockerfile()
    # and copied into the container via Dockerfile COPY command
    # No need to mount it separately
    local caddy_config_mount=""
    if [ -f "$project_path/.stackvo/Caddyfile" ]; then
        # User has custom Caddyfile in .stackvo/
        caddy_config_mount="      - ${host_project_path}/.stackvo/Caddyfile:/etc/caddy/Caddyfile:ro"
    elif [ -f "$project_path/Caddyfile" ]; then
        # User has custom Caddyfile in project root
        caddy_config_mount="      - ${host_project_path}/Caddyfile:/etc/caddy/Caddyfile:ro"
    fi
    # If no custom config, the dynamically generated one from Dockerfile will be used
    
    
    cat <<EOF
  ${project_name}:
    profiles: ["projects", "project-${project_name}"]  # --projects ile tümü, --profile project-{name} ile sadece bu proje
    build:
      context: ./projects/${project_name}
      dockerfile: Dockerfile
    image: stackvo-${project_name}:latest
    container_name: "stackvo-${project_name}"
    restart: unless-stopped
    
    volumes:
      - ${host_project_path}:/var/www/html
      - ${host_logs_path}:/var/log
$caddy_config_mount
    
    networks:
      - ${DOCKER_DEFAULT_NETWORK:-stackvo-net}
    
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.${traefik_safe_name}.rule=Host(\`${project_domain}\`)"
      - "traefik.http.routers.${traefik_safe_name}.entrypoints=websecure"
      - "traefik.http.routers.${traefik_safe_name}.tls=true"
      - "traefik.http.services.${traefik_safe_name}.loadbalancer.server.port=80"

EOF
}

