#!/bin/bash
###################################################################
# STACKVO NGINX COMPOSE GENERATOR MODULE
# Nginx compose service generation
###################################################################

##
# Generate Nginx single container compose service
#
# Parameters:
#   $1 - Project name
#   $2 - Project path (container path)
#   $3 - Project domain
#   $4 - Host project path
#   $5 - Host logs path
#   $6 - Host generated configs dir
#   $7 - Host generated projects dir
##
generate_nginx_single_container() {
    local project_name=$1
    local project_path=$2
    local project_domain=$3
    local host_project_path=$4
    local host_logs_path=$5
    local host_generated_configs_dir=$6
    local host_generated_projects_dir=$7
    
    # Sanitized project name for Traefik
    local traefik_safe_name=$(sanitize_project_name_for_traefik "$project_name")
    
    # Determine Nginx config mount path
    # Since Nginx config is generated inside Dockerfile,
    # we only mount if custom config exists
    local nginx_config_mount=""
    if [ -f "$project_path/$CONST_STACKVO_CONFIG_DIR/$CONST_CONFIG_NGINX" ]; then
        # User has custom nginx config in .stackvo/
        nginx_config_mount="      - ${host_project_path}/$CONST_STACKVO_CONFIG_DIR/$CONST_CONFIG_NGINX:/etc/nginx/conf.d/default.conf:ro"
    elif [ -f "$project_path/$CONST_CONFIG_NGINX" ]; then
        # User has custom nginx config in project root
        nginx_config_mount="      - ${host_project_path}/$CONST_CONFIG_NGINX:/etc/nginx/conf.d/default.conf:ro"
    fi
    
    # Create compose service
    cat <<EOF
  ${project_name}:
    profiles: ["projects", "project-${project_name}"]  # --projects for all, --profile project-{name} for this project only
    build:
      context: ./projects/${project_name}
      dockerfile: Dockerfile
    image: stackvo-${project_name}:latest
    container_name: "stackvo-${project_name}"
    restart: unless-stopped
    
EOF
    
    # Volumes
    generate_common_volumes "$host_project_path" "$host_logs_path" "$nginx_config_mount"
    
    # Network
    cat <<EOF
    
    networks:
      - ${DOCKER_DEFAULT_NETWORK:-stackvo-net}
    
EOF
    
    # Traefik labels
    generate_traefik_labels "$traefik_safe_name" "$project_domain"
    
    echo ""
}
