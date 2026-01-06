#!/bin/bash
###################################################################
# STACKVO APACHE COMPOSE GENERATOR MODULE
# Apache compose service oluşturma
###################################################################

##
# Apache Single Container compose service oluştur
#
# Parametreler:
#   $1 - Proje adı
#   $2 - Proje path (container path)
#   $3 - PHP version
#   $4 - Proje domain
#   $5 - Host proje path
#   $6 - Host logs path
#   $7 - Host generated projects dir
##
generate_apache_single_container() {
    local project_name=$1
    local project_path=$2
    local php_version=$3
    local project_domain=$4
    local host_project_path=$5
    local host_logs_path=$6
    local host_generated_projects_dir=$7
    
    # Traefik için sanitize edilmiş proje adı
    local traefik_safe_name=$(sanitize_project_name_for_traefik "$project_name")
    
    # Apache config mount path belirle
    local apache_config_mount=$(get_apache_config_mount "$project_path" "$host_project_path")
    
    # Compose service oluştur
    cat <<EOF
  ${project_name}:
    profiles: ["projects", "project-${project_name}"]  # --projects ile tümü, --profile project-{name} ile sadece bu proje
    build:
      context: ./projects/${project_name}
      dockerfile: Dockerfile
    image: stackvo-${project_name}:${php_version}
    container_name: "stackvo-${project_name}"
    restart: unless-stopped
    
EOF
    
    # Volumes
    generate_apache_volumes "$host_project_path" "$host_logs_path" "$apache_config_mount"
    
    # Network
    cat <<EOF
    
    networks:
      - ${DOCKER_DEFAULT_NETWORK:-stackvo-net}
    
EOF
    
    # Traefik labels
    generate_traefik_labels "$traefik_safe_name" "$project_domain"
    
    echo ""
}

##
# Apache volume mounts oluştur
#
# Parametreler:
#   $1 - Host proje path
#   $2 - Host logs path
#   $3 - Apache config mount (opsiyonel)
##
generate_apache_volumes() {
    local host_project_path=$1
    local host_logs_path=$2
    local apache_config_mount=$3
    
    cat <<EOF
    volumes:
      - ${host_project_path}:/var/www/html
      - ${host_logs_path}:/var/log/apache2
EOF
    
    # Custom config mount varsa ekle
    if [ -n "$apache_config_mount" ]; then
        echo "$apache_config_mount"
    fi
}

##
# Apache config mount path belirle
#
# Parametreler:
#   $1 - Proje path (container path)
#   $2 - Host proje path
#
# Çıktı:
#   Config mount satırı veya boş string
##
get_apache_config_mount() {
    local project_path=$1
    local host_project_path=$2
    
    # .stackvo/apache.conf var mı?
    if [ -f "$project_path/.stackvo/apache.conf" ]; then
        echo "      - ${host_project_path}/.stackvo/apache.conf:/etc/apache2/sites-available/000-default.conf:ro"
        return 0
    fi
    
    # Proje root'unda apache.conf var mı?
    if [ -f "$project_path/apache.conf" ]; then
        echo "      - ${host_project_path}/apache.conf:/etc/apache2/sites-available/000-default.conf:ro"
        return 0
    fi
    
    # Custom config yok - generated config kullanılacak
    # Generated config'i oluştur
    mkdir -p "$GENERATED_CONFIGS_DIR"
    local project_name=$(basename "$project_path")
    local template_file="$ROOT_DIR/core/templates/servers/apache/default.conf"
    local generated_config="$GENERATED_CONFIGS_DIR/${project_name}-apache.conf"
    
    if [ -f "$template_file" ]; then
        cp "$template_file" "$generated_config"
        echo "      - ${generated_config}:/etc/apache2/sites-available/000-default.conf:ro"
    else
        echo ""
    fi
}
