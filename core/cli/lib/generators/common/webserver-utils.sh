#!/bin/bash
###################################################################
# STACKVO WEBSERVER UTILITIES MODULE
# Common utility functions for web servers
###################################################################

##
# Generate PHP extension install block
# Used commonly for Apache, Nginx, Caddy
#
# Parameters:
#   $1 - Extensions to install with docker-php-ext-install (space-separated)
#   $2 - Extensions to install with PECL (space-separated)
#
# Output:
#   Dockerfile RUN commands
##
generate_php_extension_install() {
    local docker_ext_install=$1
    local pecl_install=$2
    
    local output=""
    
    # docker-php-ext-install extensions
    if [ -n "$docker_ext_install" ]; then
        output+="# Install PHP extensions\n"
        output+="RUN docker-php-ext-install$docker_ext_install\n"
        output+="\n"
    fi
    
    # PECL extensions
    if [ -n "$pecl_install" ]; then
        output+="# Install PECL extensions\n"
        output+="RUN pecl install$pecl_install \\\\\n"
        output+="    && docker-php-ext-enable$pecl_install\n"
        output+="\n"
    fi
    
    echo -e "$output"
}

##
# Generate system dependencies install block
#
# Parameters:
#   $1 - APT packages (space-separated)
#
# Output:
#   Dockerfile RUN command
##
generate_system_dependencies_install() {
    local apt_packages=$1
    
    if [ -z "$apt_packages" ]; then
        return 0
    fi
    
    cat <<EOF
# Install system dependencies
RUN apt-get update && apt-get install -y \\
    $apt_packages \\
    && rm -rf /var/lib/apt/lists/*

EOF
}

##
# Generate development tools install block
# Tools like Composer, Node.js, npm, yarn
#
# Parameters:
#   $1 - Tool list (comma-separated, e.g., "composer,nodejs")
#   $2 - Composer version (default: latest)
#   $3 - Node.js version (default: 20)
#
# Output:
#   Dockerfile RUN commands
##
generate_development_tools_install() {
    local default_tools=$1
    local composer_version=${2:-latest}
    local nodejs_version=${3:-20}
    
    if [ -z "$default_tools" ]; then
        return 0
    fi
    
    echo ""
    echo "# Install Development Tools"
    
    for tool in $(echo "$default_tools" | tr ',' ' '); do
        local install_cmd=$(get_tool_install_commands "$tool" "$composer_version" "$nodejs_version")
        if [ -n "$install_cmd" ]; then
            echo "$install_cmd"
            echo ""
        fi
    done
}

##
# Generate Supervisord PHP-FPM configuration
# Used commonly for Nginx and Caddy
#
# Parameters:
#   $1 - Project directory (where config file will be written)
#   $2 - Web server name (nginx or caddy)
#   $3 - Web server command
#
# Output:
#   supervisord.conf file is created
##
generate_supervisord_config() {
    local project_dir=$1
    local webserver_name=$2
    local webserver_command=$3
    
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

SUPERVISORCONF

    # Add web server program
    cat >> "$project_dir/supervisord.conf" <<EOF
[program:${webserver_name}]
command=${webserver_command}
autostart=true
autorestart=true
stdout_logfile=/dev/stdout
stdout_logfile_maxbytes=0
stderr_logfile=/dev/stderr
stderr_logfile_maxbytes=0
EOF
}

##
# Generate Traefik labels block
# Common for all web servers
#
# Parameters:
#   $1 - Traefik-safe project name (sanitized)
#   $2 - Project domain
#
# Output:
#   Docker Compose labels block
##
generate_traefik_labels() {
    local traefik_safe_name=$1
    local project_domain=$2
    
    cat <<EOF
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.${traefik_safe_name}.rule=Host(\`${project_domain}\`)"
      - "traefik.http.routers.${traefik_safe_name}.entrypoints=websecure"
      - "traefik.http.routers.${traefik_safe_name}.tls=true"
      - "traefik.http.services.${traefik_safe_name}.loadbalancer.server.port=80"
EOF
}

##
# Generate common volume mounts block
#
# Parameters:
#   $1 - Host project path
#   $2 - Host logs path
#   $3 - Custom config mount (optional, can be empty string)
#
# Output:
#   Docker Compose volumes block
##
generate_common_volumes() {
    local host_project_path=$1
    local host_logs_path=$2
    local custom_config_mount=$3
    
    cat <<EOF
    volumes:
      - ${host_project_path}:/var/www/html
      - ${host_logs_path}:/var/log
EOF
    
    # Add custom config mount if exists
    if [ -n "$custom_config_mount" ]; then
        echo "$custom_config_mount"
    fi
}

##
# Configure PHP-FPM to listen on TCP port
# Common for Nginx and Caddy
#
# Output:
#   Dockerfile RUN command
##
generate_php_fpm_tcp_config() {
    cat <<'EOF'
# Configure PHP-FPM to listen on TCP port 127.0.0.1:9000
RUN sed -i 's|listen = .*|listen = 127.0.0.1:9000|' /usr/local/etc/php-fpm.d/www.conf
EOF
}

##
# Generate entrypoint script
# Creates log directories at runtime
#
# Parameters:
#   $1 - Log directory name (nginx, caddy, apache)
#
# Output:
#   Dockerfile RUN command
##
generate_entrypoint_script() {
    local log_dir=$1
    
    cat <<EOF
# Create entrypoint script to ensure log directories exist at runtime
RUN echo '#!/bin/bash' > /entrypoint.sh && \\
    echo 'mkdir -p /var/log/${log_dir} /var/log/php-fpm' >> /entrypoint.sh && \\
    echo 'touch /var/log/${log_dir}/access.log /var/log/${log_dir}/error.log' >> /entrypoint.sh && \\
    echo 'chmod 666 /var/log/${log_dir}/*.log' >> /entrypoint.sh && \\
    echo 'exec "\$@"' >> /entrypoint.sh && \\
    chmod +x /entrypoint.sh
EOF
}

##
# Generate Dockerfile base image block
#
# Parameters:
#   $1 - Project name
#   $2 - Web server name (Apache, Nginx, Caddy)
#   $3 - PHP version
#   $4 - PHP image variant (apache, fpm)
#
# Output:
#   Dockerfile FROM and comment lines
##
generate_dockerfile_header() {
    local project_name=$1
    local web_server=$2
    local php_version=$3
    local php_variant=$4
    
    cat <<EOF
# Auto-generated Dockerfile for $project_name
# Web Server: $web_server
# PHP Version: $php_version
FROM php:${php_version}-${php_variant}

EOF
}
