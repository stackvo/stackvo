#!/bin/bash
###################################################################
# STACKVO PROJECT GENERATOR MODULE
# Main orchestrator - Creates project containers
###################################################################

# Load dependencies
# Bu dosya cli/lib/generators/project.sh konumunda
# Modules are in the same directory
MODULE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$MODULE_DIR/project/config-parser.sh"
source "$MODULE_DIR/project/path-resolver.sh"
source "$MODULE_DIR/common/webserver-utils.sh"
source "$MODULE_DIR/project/dockerfile/apache.sh"
source "$MODULE_DIR/project/dockerfile/nginx.sh"
source "$MODULE_DIR/project/dockerfile/caddy.sh"
source "$MODULE_DIR/project/compose/apache.sh"
source "$MODULE_DIR/project/compose/nginx.sh"
source "$MODULE_DIR/project/compose/caddy.sh"

##
# Ana proje generator fonksiyonu
# Tüm projeleri tarayıp Dockerfile ve compose service'leri oluşturur
##
generate_projects() {
    log_info "Proje containerları oluşturuluyor..."
    
    # Resolve host paths
    local paths=$(resolve_host_paths)
    local HOST_ROOT_DIR=$(echo "$paths" | grep "^HOST_ROOT_DIR=" | cut -d= -f2)
    
    # Create generated directory
    mkdir -p "$GENERATED_DIR"
    mkdir -p "$GENERATED_CONFIGS_DIR"
    
    # Fix permissions
    fix_generated_permissions
    
    # Initialize compose file
    initialize_compose_file
    
    # Process projects
    local project_count=$(process_all_projects "$HOST_ROOT_DIR")
    
    # If no projects, finish with empty services mapping
    if [ $project_count -eq 0 ]; then
        finalize_compose_file_empty
    else
        finalize_compose_file
    fi
    
    log_success "docker-compose.projects.yml oluşturuldu"
}

##
# Compose dosyasını başlat
##
initialize_compose_file() {
    local output="$GENERATED_DIR/docker-compose.projects.yml"
    
    echo "name: stackvo" > "$output"
    echo "" >> "$output"
    echo "services:" >> "$output"
    echo "" >> "$output"
}

##
# Tüm projeleri işle
#
# Parametreler:
#   $1 - Host root directory
#
# Çıktı:
#   İşlenen proje sayısı
##
process_all_projects() {
    local HOST_ROOT_DIR=$1
    local projects_dir="$ROOT_DIR/projects"
    local output="$GENERATED_DIR/docker-compose.projects.yml"
    local project_count=0
    
    # Exit if projects directory doesn't exist
    if [ ! -d "$projects_dir" ]; then
        log_info "Projects dizini bulunamadı"
        return 0
    fi
    
    # Process each project
    for project_path in "$projects_dir"/*; do
        [ ! -d "$project_path" ] && continue
        
        local project_name=$(basename "$project_path")
        local project_json="$project_path/stackvo.json"
        
        # stackvo.json yoksa atla
        if [ ! -f "$project_json" ]; then
            log_warn "$project_name atlanıyor: stackvo.json bulunamadı"
            continue
        fi
        
        # Process project
        if process_single_project "$project_name" "$project_path" "$project_json" "$HOST_ROOT_DIR" >> "$output"; then
            ((project_count++))
        fi
    done
    
    echo $project_count
}

##
# Tek bir projeyi işle
#
# Parametreler:
#   $1 - Proje adı
#   $2 - Proje path
#   $3 - stackvo.json path
#   $4 - Host root directory
##
process_single_project() {
    local project_name=$1
    local project_path=$2
    local project_json=$3
    local HOST_ROOT_DIR=$4
    
    log_info "Proje işleniyor: $project_name"
    
    # Config parse et
    local config=$(parse_project_config "$project_json" "$project_name")
    if [ $? -ne 0 ]; then
        log_error "$project_name için config parse edilemedi"
        return 1
    fi
    
    # Extract config values (assigns to global variables)
    extract_config_values "$config"
    
    # Validate et
    if ! validate_project_config "$project_name" "$php_version" "$web_server" "$project_domain"; then
        log_error "$project_name için config geçersiz"
        return 1
    fi
    
    # Host path'lerini hesapla
    local host_paths=$(calculate_project_host_paths "$project_name" "$HOST_ROOT_DIR")
    local host_project_path=$(echo "$host_paths" | grep "^HOST_PROJECT_PATH=" | cut -d= -f2)
    local host_logs_path=$(echo "$host_paths" | grep "^HOST_LOGS_PATH=" | cut -d= -f2)
    local host_generated_configs_dir=$(echo "$host_paths" | grep "^HOST_GENERATED_CONFIGS_DIR=" | cut -d= -f2)
    local host_generated_projects_dir=$(echo "$host_paths" | grep "^HOST_GENERATED_PROJECTS_DIR=" | cut -d= -f2)
    
    # Create Dockerfile
    local project_dockerfile_dir="$GENERATED_DIR/projects/${project_name}"
    generate_single_dockerfile "$project_name" "$web_server" "$php_version" "$extensions" "$project_dockerfile_dir" "$document_root"
    
    # Create compose service
    generate_single_container "$project_name" "$project_path" "$web_server" "$php_version" "$project_domain" "$document_root" "$host_project_path" "$host_logs_path" "$host_generated_configs_dir" "$host_generated_projects_dir"
    
    return 0
}

##
# Dockerfile oluştur (dispatcher)
# Web server tipine göre ilgili modülü çağırır
#
# Parametreler:
#   $1 - Proje adı
#   $2 - Web server (nginx/apache/caddy)
#   $3 - PHP version
#   $4 - Extensions (boşlukla ayrılmış)
#   $5 - Proje dockerfile dizini
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
    
    # Collect extension dependencies
    local apt_packages=""
    local configure_commands=""
    local docker_ext_install=""
    local pecl_install=""
    
    for ext in $extensions; do
        # Sistem paketlerini al
        local packages=$(get_extension_packages "$ext")
        if [ -n "$packages" ]; then
            apt_packages="$apt_packages $packages"
        fi
        
        # Configure komutunu al
        local configure=$(get_extension_configure "$ext")
        if [ -n "$configure" ]; then
            configure_commands="$configure_commands\\nRUN $configure"
        fi
        
        # PECL mi yoksa docker-php-ext-install mi?
        if is_pecl_extension "$ext"; then
            pecl_install="$pecl_install $ext"
        else
            docker_ext_install="$docker_ext_install $ext"
        fi
    done
    
    # Remove duplicate packages
    apt_packages=$(echo "$apt_packages" | tr ' ' '\n' | sort -u | tr '\n' ' ')
    
    # Create Dockerfile based on web server type
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
# Compose service oluştur (dispatcher)
# Web server tipine göre ilgili modülü çağırır
#
# Parametreler:
#   $1 - Proje adı
#   $2 - Proje path (container path)
#   $3 - Web server
#   $4 - PHP version
#   $5 - Proje domain
#   $6 - Document root
#   $7 - Host proje path
#   $8 - Host logs path
#   $9 - Host generated configs dir
#   $10 - Host generated projects dir
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
# Compose dosyasını sonlandır (network ekle)
##
finalize_compose_file() {
    local output="$GENERATED_DIR/docker-compose.projects.yml"
    
    cat >> "$output" <<EOF

networks:
  ${DOCKER_DEFAULT_NETWORK:-stackvo-net}:
    external: true
EOF
}

##
# Compose dosyasını boş services ile sonlandır
##
finalize_compose_file_empty() {
    local output="$GENERATED_DIR/docker-compose.projects.yml"
    
    # Recreate file
    echo "name: stackvo" > "$output"
    echo "" >> "$output"
    echo "services: {}" >> "$output"
    echo "" >> "$output"
    echo "networks:" >> "$output"
    echo "  ${DOCKER_DEFAULT_NETWORK:-stackvo-net}:" >> "$output"
    echo "    external: true" >> "$output"
}
