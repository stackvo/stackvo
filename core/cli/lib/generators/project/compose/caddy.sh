#!/bin/bash
###################################################################
# STACKVO CADDY COMPOSE GENERATOR MODULE
# Caddy compose service generation
###################################################################

##
# Generate Caddy single container compose service
#
# Parameters:
#   $1 - Project name
#   $2 - Project path (container path)
#   $3 - Project domain
#   $4 - Document root
#   $5 - Host project path
#   $6 - Host logs path
#   $7 - Host generated configs dir
#   $8 - Host generated projects dir
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
    
    # Sanitized project name for Traefik
    local traefik_safe_name=$(sanitize_project_name_for_traefik "$project_name")
    
    # Determine Caddyfile mount path
    # Since Caddyfile is generated inside Dockerfile,
    # we only mount if custom config exists
    local caddy_config_mount=""
    if [ -f "$project_path/.stackvo/Caddyfile" ]; then
        # User has custom Caddyfile in .stackvo/
        caddy_config_mount="      - ${host_project_path}/.stackvo/Caddyfile:/etc/caddy/Caddyfile:ro"
    elif [ -f "$project_path/Caddyfile" ]; then
        # User has custom Caddyfile in project root
        caddy_config_mount="      - ${host_project_path}/Caddyfile:/etc/caddy/Caddyfile:ro"
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
    generate_common_volumes "$host_project_path" "$host_logs_path" "$caddy_config_mount"
    
    # Network
    cat <<EOF
    
    networks:
      - ${DOCKER_DEFAULT_NETWORK:-stackvo-net}
    
EOF
    
    # Traefik labels
    generate_traefik_labels "$traefik_safe_name" "$project_domain"
    
    echo ""
}
