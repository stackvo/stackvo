#!/bin/bash
###################################################################
# STACKVO APACHE COMPOSE GENERATOR MODULE
# Apache compose service generation
###################################################################

##
# Generate Apache single container compose service
#
# Parameters:
#   $1 - Project name
#   $2 - Project path (container path)
#   $3 - PHP version
#   $4 - Project domain
#   $5 - Host project path
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
    
    # Sanitized project name for Traefik
    local traefik_safe_name=$(sanitize_project_name_for_traefik "$project_name")
    
    # Determine Apache config mount path
    local apache_config_mount=$(get_apache_config_mount "$project_path" "$host_project_path")
    
    # Create compose service
    cat <<EOF
  ${project_name}:
    profiles: ["projects", "project-${project_name}"]  # --projects for all, --profile project-{name} for this project only
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
# Generate Apache volume mounts
#
# Parameters:
#   $1 - Host project path
#   $2 - Host logs path
#   $3 - Apache config mount (optional)
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
    
    # Add custom config mount if exists
    if [ -n "$apache_config_mount" ]; then
        echo "$apache_config_mount"
    fi
}

##
# Determine Apache config mount path
#
# Parameters:
#   $1 - Project path (container path)
#   $2 - Host project path
#
# Output:
#   Config mount line or empty string
##
get_apache_config_mount() {
    local project_path=$1
    local host_project_path=$2
    
    # Does .stackvo/apache.conf exist?
    if [ -f "$project_path/.stackvo/apache.conf" ]; then
        echo "      - ${host_project_path}/.stackvo/apache.conf:/etc/apache2/sites-available/000-default.conf:ro"
        return 0
    fi
    
    # Does apache.conf exist in project root?
    if [ -f "$project_path/apache.conf" ]; then
        echo "      - ${host_project_path}/apache.conf:/etc/apache2/sites-available/000-default.conf:ro"
        return 0
    fi
    
    # No custom config - generated config will be used
    # Create generated config
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
