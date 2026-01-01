#!/bin/bash
###################################################################
# STACKVO TRAEFIK GENERATOR MODULE
# Traefik configuration generation
###################################################################

generate_traefik_routes() {
    log_info "Generating traefik routes..."
    
    mkdir -p "$GENERATED_TRAEFIK_DYNAMIC_DIR"
    local output="$GENERATED_TRAEFIK_DYNAMIC_DIR/routes.yml"
    mkdir -p "$GENERATED_TRAEFIK_DIR"
    
    # Start with routers section
    cat > "$output" <<EOF
http:
  routers:
    traefik:
      rule: "Host(\`traefik.${DEFAULT_TLD_SUFFIX}\`)"
      entryPoints:
        - websecure
      service: api@internal
      tls: {}
EOF
    
    # Add all routers
    add_router_if_enabled "SERVICE_RABBITMQ_ENABLE" "rabbitmq" "SERVICE_RABBITMQ_URL" >> "$output"
    add_router_if_enabled "SERVICE_MAILHOG_ENABLE" "mailhog" "SERVICE_MAILHOG_URL" >> "$output"
    add_router_if_enabled "SERVICE_KIBANA_ENABLE" "kibana" "SERVICE_KIBANA_URL" >> "$output"
    add_router_if_enabled "SERVICE_GRAFANA_ENABLE" "grafana" "SERVICE_GRAFANA_URL" >> "$output"
    
    # Tools container admin tools (subdomain-based routing - no path rewriting needed)
    if [ "${STACKVO_UI_TOOLS_CONTAINER_ENABLE}" = "true" ]; then
        add_router_if_enabled "STACKVO_UI_TOOLS_CONTAINER_ENABLE" "phpmyadmin" "TOOLS_PHPMYADMIN_URL" >> "$output"
        add_router_if_enabled "STACKVO_UI_TOOLS_CONTAINER_ENABLE" "adminer" "TOOLS_ADMINER_URL" >> "$output"
        add_router_if_enabled "STACKVO_UI_TOOLS_CONTAINER_ENABLE" "phppgadmin" "TOOLS_PHPPGADMIN_URL" >> "$output"
        add_router_if_enabled "STACKVO_UI_TOOLS_CONTAINER_ENABLE" "phpmemcachedadmin" "PHPMEMCACHEDADMIN_URL" >> "$output"
        add_router_if_enabled "STACKVO_UI_TOOLS_CONTAINER_ENABLE" "phpmongo" "TOOLS_PHPMONGO_URL" >> "$output"
        add_router_if_enabled "STACKVO_UI_TOOLS_CONTAINER_ENABLE" "opcache" "TOOLS_OPCACHE_URL" >> "$output"
        add_router_if_enabled "STACKVO_UI_TOOLS_CONTAINER_ENABLE" "kafbat" "TOOLS_KAFBAT_URL" >> "$output"
    fi
    
    # Add services section
    cat >> "$output" <<EOF

  services:
EOF
    
    # Add all services
    add_service_if_enabled "SERVICE_RABBITMQ_ENABLE" "rabbitmq" "15672" >> "$output"
    add_service_if_enabled "SERVICE_MAILHOG_ENABLE" "mailhog" "8025" >> "$output"
    add_service_if_enabled "SERVICE_KIBANA_ENABLE" "kibana" "5601" >> "$output"
    add_service_if_enabled "SERVICE_GRAFANA_ENABLE" "grafana" "3000" >> "$output"
    
    # Tools container services (subdomain-based, no path rewriting)
    if [ "${STACKVO_UI_TOOLS_CONTAINER_ENABLE}" = "true" ]; then
        for tool in phpmyadmin adminer phppgadmin phpmemcachedadmin phpmongo opcache kafbat; do
            cat >> "$output" <<EOF
    ${tool}:
      loadBalancer:
        servers:
          - url: "http://stackvo-tools:80"
EOF
        done
    fi
    
    # Add TLS configuration - Force use of core/certs certificates
    if [ "${SSL_ENABLE:-false}" = "true" ]; then
        cat >> "$output" <<EOF

# TLS Configuration - Force use of core/certs certificates
tls:
  stores:
    default:
      defaultCertificate:
        certFile: /certs/stackvo-wildcard.crt
        keyFile: /certs/stackvo-wildcard.key
  certificates:
    - certFile: /certs/stackvo-wildcard.crt
      keyFile: /certs/stackvo-wildcard.key
  options:
    default:
      minVersion: VersionTLS12
      sniStrict: false
EOF
    fi
    
    log_success "Generated traefik routes"
}

# Add Traefik router if service enabled
add_router_if_enabled() {
    local enable_flag=$1
    local service_name=$2
    local url_var=$3
    
    eval "local enabled=\${${enable_flag}:-false}"
    eval "local url=\${${url_var}:-$service_name}"
    
    if [ "$enabled" = "true" ]; then
        cat <<EOF
    ${service_name}:
      rule: "Host(\`${url}.${DEFAULT_TLD_SUFFIX}\`)"
      entryPoints:
        - websecure
      service: ${service_name}
      tls: {}
EOF
    fi
}



# Add Traefik service if enabled
add_service_if_enabled() {
    local enable_flag=$1
    local service_name=$2
    local port=$3
    
    eval "local enabled=\${${enable_flag}:-false}"
    
    if [ "$enabled" = "true" ]; then
        cat <<EOF
    ${service_name}:
      loadBalancer:
        servers:
          - url: "http://stackvo-${service_name}:${port}"
EOF
    fi
}

# Generate Traefik config
generate_traefik_config() {
    log_info "Generating traefik config..."
    
    local ssl_enabled="${SSL_ENABLE:-false}"
    local redirect_https="${REDIRECT_TO_HTTPS:-false}"
    local output="$GENERATED_TRAEFIK_DIR/traefik.yml"
    
    mkdir -p "$GENERATED_TRAEFIK_DIR"
    
    cat > "$output" <<EOF
api:
  dashboard: true
  insecure: false

providers:
  docker:
    endpoint: "unix:///var/run/docker.sock"
    exposedByDefault: false
    network: ${DOCKER_DEFAULT_NETWORK:-stackvo-net}
  file:
    directory: "/etc/traefik/dynamic"
    watch: true

entryPoints:
  web:
    address: ":80"
EOF
    
    if [ "$ssl_enabled" = "true" ]; then
        if [ "$redirect_https" = "true" ]; then
            cat >> "$output" <<EOF
    http:
      redirections:
        entryPoint:
          to: websecure
          scheme: https
EOF
        fi
        
        cat >> "$output" <<EOF
  websecure:
    address: ":443"
EOF
    fi
    
    log_success "Generated traefik config"
}
