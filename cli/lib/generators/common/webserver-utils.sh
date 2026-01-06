#!/bin/bash
###################################################################
# STACKVO WEBSERVER UTILITIES MODULE
# Web server'lar için ortak utility fonksiyonlar
###################################################################

##
# PHP extension install bloğunu oluştur
# Apache, Nginx, Caddy için ortak kullanılır
#
# Parametreler:
#   $1 - docker-php-ext-install ile kurulacak extension'lar (boşlukla ayrılmış)
#   $2 - PECL ile kurulacak extension'lar (boşlukla ayrılmış)
#
# Çıktı:
#   Dockerfile RUN komutları
##
generate_php_extension_install() {
    local docker_ext_install=$1
    local pecl_install=$2
    
    local output=""
    
    # docker-php-ext-install extension'ları
    if [ -n "$docker_ext_install" ]; then
        output+="# Install PHP extensions\n"
        output+="RUN docker-php-ext-install$docker_ext_install\n"
        output+="\n"
    fi
    
    # PECL extension'ları
    if [ -n "$pecl_install" ]; then
        output+="# Install PECL extensions\n"
        output+="RUN pecl install$pecl_install \\\\\n"
        output+="    && docker-php-ext-enable$pecl_install\n"
        output+="\n"
    fi
    
    echo -e "$output"
}

##
# Sistem bağımlılıklarını install bloğunu oluştur
#
# Parametreler:
#   $1 - APT paketleri (boşlukla ayrılmış)
#
# Çıktı:
#   Dockerfile RUN komutu
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
# Development tools install bloğunu oluştur
# Composer, Node.js, npm, yarn gibi araçlar
#
# Parametreler:
#   $1 - Tool listesi (virgülle ayrılmış, örn: "composer,nodejs")
#   $2 - Composer version (varsayılan: latest)
#   $3 - Node.js version (varsayılan: 20)
#
# Çıktı:
#   Dockerfile RUN komutları
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
# Supervisord PHP-FPM konfigürasyonunu oluştur
# Nginx ve Caddy için ortak kullanılır
#
# Parametreler:
#   $1 - Proje dizini (config dosyasının yazılacağı yer)
#   $2 - Web server adı (nginx veya caddy)
#   $3 - Web server command
#
# Çıktı:
#   supervisord.conf dosyası oluşturulur
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

    # Web server programını ekle
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
# Traefik labels bloğunu oluştur
# Tüm web server'lar için ortak
#
# Parametreler:
#   $1 - Traefik-safe proje adı (sanitize edilmiş)
#   $2 - Proje domain
#
# Çıktı:
#   Docker Compose labels bloğu
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
# Ortak volume mounts bloğunu oluştur
#
# Parametreler:
#   $1 - Host proje path
#   $2 - Host logs path
#   $3 - Custom config mount (opsiyonel, boş string olabilir)
#
# Çıktı:
#   Docker Compose volumes bloğu
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
    
    # Custom config mount varsa ekle
    if [ -n "$custom_config_mount" ]; then
        echo "$custom_config_mount"
    fi
}

##
# PHP-FPM'i TCP port'ta dinleyecek şekilde yapılandır
# Nginx ve Caddy için ortak
#
# Çıktı:
#   Dockerfile RUN komutu
##
generate_php_fpm_tcp_config() {
    cat <<'EOF'
# Configure PHP-FPM to listen on TCP port 127.0.0.1:9000
RUN sed -i 's|listen = .*|listen = 127.0.0.1:9000|' /usr/local/etc/php-fpm.d/www.conf
EOF
}

##
# Entrypoint script oluştur
# Log dizinlerini runtime'da oluşturur
#
# Parametreler:
#   $1 - Log dizini adı (nginx, caddy, apache)
#
# Çıktı:
#   Dockerfile RUN komutu
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
# Dockerfile base image bloğunu oluştur
#
# Parametreler:
#   $1 - Proje adı
#   $2 - Web server adı (Apache, Nginx, Caddy)
#   $3 - PHP version
#   $4 - PHP image variant (apache, fpm)
#
# Çıktı:
#   Dockerfile FROM ve comment satırları
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
