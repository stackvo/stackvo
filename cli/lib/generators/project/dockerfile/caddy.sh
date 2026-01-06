#!/bin/bash
###################################################################
# STACKVO CADDY DOCKERFILE GENERATOR MODULE
# Caddy + PHP-FPM + Supervisord Dockerfile oluşturma
###################################################################

##
# Caddy Dockerfile oluştur
#
# Parametreler:
#   $1 - Dockerfile path
#   $2 - PHP version
#   $3 - APT packages (boşlukla ayrılmış)
#   $4 - Configure commands
#   $5 - docker-php-ext-install extension'lar
#   $6 - PECL extension'lar
#   $7 - Proje adı
#   $8 - Proje dizini (config dosyaları için)
#   $9 - Document root
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
    
    # Default tools
    local default_tools=${PHP_DEFAULT_TOOLS:-""}
    local composer_version=${PHP_TOOL_COMPOSER_VERSION:-latest}
    local nodejs_version=${PHP_TOOL_NODEJS_VERSION:-20}
    
    # Dockerfile header
    generate_dockerfile_header "$project_name" "Caddy + PHP-FPM" "$php_version" "fpm" > "$dockerfile"
    
    # Install Supervisord and Caddy dependencies
    cat >> "$dockerfile" <<'EOF'
# Install Supervisord and dependencies
RUN apt-get update && apt-get install -y \
    curl \
    ca-certificates \
    supervisor \
    && rm -rf /var/lib/apt/lists/*

# Install Caddy from official binary (cross-platform compatible, no GPG issues)
ARG CADDY_VERSION=2.8.4
RUN curl -o /tmp/caddy.tar.gz -L "https://github.com/caddyserver/caddy/releases/download/v\${CADDY_VERSION}/caddy_\${CADDY_VERSION}_linux_amd64.tar.gz" \
    && tar -xzf /tmp/caddy.tar.gz -C /usr/bin caddy \
    && chmod +x /usr/bin/caddy \
    && rm /tmp/caddy.tar.gz \
    && caddy version


EOF
    
    # System dependencies
    generate_system_dependencies_install "$apt_packages" >> "$dockerfile"
    
    # Configure commands
    if [ -n "$configure_commands" ]; then
        echo -e "$configure_commands" >> "$dockerfile"
        echo "" >> "$dockerfile"
    fi
    
    # PHP extensions
    generate_php_extension_install "$docker_ext_install" "$pecl_install" >> "$dockerfile"
    
    # Development tools
    generate_development_tools_install "$default_tools" "$composer_version" "$nodejs_version" >> "$dockerfile"
    
    # Generate Caddyfile
    generate_caddyfile "$project_dir" "$document_root"
    
    # Generate Supervisord config
    generate_supervisord_config "$project_dir" "caddy" "/usr/bin/caddy run --config /etc/caddy/Caddyfile"
    
    # Caddy ve PHP-FPM konfigürasyonu
    cat >> "$dockerfile" <<'DOCKEREOF'

DOCKEREOF
    
    # PHP-FPM TCP config
    generate_php_fpm_tcp_config >> "$dockerfile"
    
    cat >> "$dockerfile" <<'DOCKEREOF'

# Copy Caddyfile
COPY Caddyfile /etc/caddy/Caddyfile

# Copy Supervisord configuration
COPY supervisord.conf /etc/supervisor/conf.d/supervisord.conf

DOCKEREOF
    
    # Entrypoint script
    generate_entrypoint_script "caddy" >> "$dockerfile"
    
    # Workdir ve CMD
    cat >> "$dockerfile" <<'DOCKEREOF'

WORKDIR /var/www/html

# Use entrypoint to create log directories before starting supervisord
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/usr/bin/supervisord", "-c", "/etc/supervisor/conf.d/supervisord.conf"]
DOCKEREOF
}

##
# Caddyfile oluştur
#
# Parametreler:
#   $1 - Proje dizini
#   $2 - Document root
##
generate_caddyfile() {
    local project_dir=$1
    local document_root=$2
    
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
}
