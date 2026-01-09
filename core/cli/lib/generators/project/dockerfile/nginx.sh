#!/bin/bash
###################################################################
# STACKVO NGINX DOCKERFILE GENERATOR MODULE
# Nginx + PHP-FPM + Supervisord Dockerfile generation
###################################################################

##
# Generate Nginx Dockerfile
#
# Parameters:
#   $1 - Dockerfile path
#   $2 - PHP version
#   $3 - APT packages (space-separated)
#   $4 - Configure commands
#   $5 - docker-php-ext-install extensions
#   $6 - PECL extensions
#   $7 - Project name
#   $8 - Project directory (for config files)
#   $9 - Document root
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
    
    # Default tools
    local default_tools=${PHP_DEFAULT_TOOLS:-""}
    local composer_version=${PHP_TOOL_COMPOSER_VERSION:-latest}
    local nodejs_version=${PHP_TOOL_NODEJS_VERSION:-20}
    
    # Dockerfile header
    generate_dockerfile_header "$project_name" "Nginx + PHP-FPM" "$php_version" "fpm" > "$dockerfile"
    
    # Install Nginx and Supervisord
    cat >> "$dockerfile" <<'EOF'
# Install Nginx and Supervisord
RUN apt-get update && apt-get install -y \
    nginx \
    supervisor \
    && rm -rf /var/lib/apt/lists/*

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
    
    # Generate Nginx config
    generate_nginx_config "$project_dir" "$document_root"
    
    # Generate Supervisord config
    generate_supervisord_config "$project_dir" "nginx" "/usr/sbin/nginx -g 'daemon off;'"
    
    # Nginx and PHP-FPM configuration
    cat >> "$dockerfile" <<'DOCKEREOF'

DOCKEREOF
    
    # PHP-FPM TCP config
    generate_php_fpm_tcp_config >> "$dockerfile"
    
    cat >> "$dockerfile" <<'DOCKEREOF'

# Remove 'main' log format reference from Nginx config
RUN sed -i 's/ main;/;/' /etc/nginx/nginx.conf

# Disable default Nginx site (it conflicts with our config)
RUN rm -f /etc/nginx/sites-enabled/default

# Copy Nginx configuration
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Copy Supervisord configuration
COPY supervisord.conf /etc/supervisor/conf.d/supervisord.conf

DOCKEREOF
    
    # Entrypoint script
    generate_entrypoint_script "nginx" >> "$dockerfile"
    
    # Workdir and CMD
    cat >> "$dockerfile" <<'DOCKEREOF'

WORKDIR /var/www/html

# Use entrypoint to create log directories before starting supervisord
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/usr/bin/supervisord", "-c", "/etc/supervisor/conf.d/supervisord.conf"]
DOCKEREOF
}

##
# Generate Nginx configuration file
#
# Parameters:
#   $1 - Project directory
#   $2 - Document root
##
generate_nginx_config() {
    local project_dir=$1
    local document_root=$2
    
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
}
