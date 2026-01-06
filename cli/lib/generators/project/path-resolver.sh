#!/bin/bash
###################################################################
# STACKVO PROJECT PATH RESOLVER MODULE
# Host/container path resolution and permission management
###################################################################

##
# Container ortamını tespit et
#
# Çıktı:
#   0 = container içinde, 1 = host'ta
##
detect_container_environment() {
    if [ -f "/.dockerenv" ] || [ -f "/run/.containerenv" ]; then
        return 0  # We are in a container
    else
        return 1  # We are on host
    fi
}

##
# Host path'lerini çözümle
# Container içinde mi yoksa host'ta mı çalıştığımızı tespit edip
# Returns correct paths based on execution environment
#
# Çıktı:
#   HOST_ROOT_DIR=<path> formatında
##
resolve_host_paths() {
    local HOST_ROOT_DIR="$ROOT_DIR"
    
    if detect_container_environment; then
        # We are in container - use host path
        if [ -n "$HOST_STACKVO_ROOT" ]; then
            HOST_ROOT_DIR="$HOST_STACKVO_ROOT"
            log_info "Container içinde çalışıyor, host path kullanılıyor: $HOST_ROOT_DIR"
        else
            log_warn "Container içinde çalışıyor ama HOST_STACKVO_ROOT set edilmemiş, volume mount'lar başarısız olabilir"
        fi
    fi
    
    echo "HOST_ROOT_DIR=$HOST_ROOT_DIR"
}

##
# Generated dizini için permission'ları düzelt
# Container içinde mi yoksa host'ta mı çalıştığımıza göre
# Sets correct ownership based on execution environment
##
fix_generated_permissions() {
    if [ ! -d "$GENERATED_DIR" ]; then
        return 0
    fi
    
    if detect_container_environment; then
        # We are in container - use nginx user
        if [ "$(id -u)" -eq 0 ]; then
            # Running as root
            chown -R 100:101 "$GENERATED_DIR" 2>/dev/null || true
            log_info "Generated dizini ownership'i nginx user'a ayarlandı (100:101)"
        fi
    else
        # We are on host - use HOST_UID and HOST_GID
        if [ -n "${HOST_UID}" ] && [ -n "${HOST_GID}" ]; then
            sudo chown -R "${HOST_UID}:${HOST_GID}" "$GENERATED_DIR" 2>/dev/null || true
            chmod -R 755 "$GENERATED_DIR" 2>/dev/null || true
            log_info "Generated dizini ownership'i ${HOST_UID}:${HOST_GID} olarak ayarlandı"
        else
            log_warn "HOST_UID veya HOST_GID set edilmemiş, chmod 777 kullanılıyor"
            chmod -R 777 "$GENERATED_DIR" 2>/dev/null || true
        fi
    fi
}

##
# Proje için host path'lerini hesapla
#
# Parametreler:
#   $1 - Proje adı
#   $2 - Host root directory
#
# Çıktı:
#   HOST_PROJECT_PATH=<path>
#   HOST_LOGS_PATH=<path>
#   HOST_GENERATED_CONFIGS_DIR=<path>
#   HOST_GENERATED_PROJECTS_DIR=<path>
##
calculate_project_host_paths() {
    local project_name=$1
    local host_root_dir=$2
    
    echo "HOST_PROJECT_PATH=${host_root_dir}/projects/${project_name}"
    echo "HOST_LOGS_PATH=${host_root_dir}/logs/projects/${project_name}"
    echo "HOST_GENERATED_CONFIGS_DIR=${host_root_dir}/generated/configs"
    echo "HOST_GENERATED_PROJECTS_DIR=${host_root_dir}/generated/projects"
}

##
# Custom config dosyası mount path'ini bul
# Öncelik sırası: .stackvo/config > project_root/config > generated/config
#
# Parametreler:
#   $1 - Proje path (container path)
#   $2 - Host proje path
#   $3 - Config dosya adı (örn: nginx.conf, apache.conf, Caddyfile)
#   $4 - Generated config path (fallback)
#
# Çıktı:
#   Config mount satırı (boş string = mount yok, generated kullan)
##
get_config_mount_path() {
    local project_path=$1
    local host_project_path=$2
    local config_filename=$3
    local generated_config=$4
    
    # Is there a custom config in .stackvo/ directory?
    if [ -f "$project_path/.stackvo/$config_filename" ]; then
        echo "      - ${host_project_path}/.stackvo/${config_filename}:${generated_config}:ro"
        return 0
    fi
    
    # Is there a custom config in project root?
    if [ -f "$project_path/$config_filename" ]; then
        echo "      - ${host_project_path}/${config_filename}:${generated_config}:ro"
        return 0
    fi
    
    # No custom config - return empty string (generated config from Dockerfile will be used)
    echo ""
}
