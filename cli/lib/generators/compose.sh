#!/bin/bash
###################################################################
# STACKVO COMPOSE GENERATOR MODULE
# Docker Compose file generation
###################################################################

##
# Check if any tool is enabled
#
# Returns:
#   0 - At least one tool is enabled
#   1 - No tools are enabled
##
has_any_tool_enabled() {
    local tools=(
        "TOOLS_ADMINER_ENABLE"
        "TOOLS_PHPMYADMIN_ENABLE"
        "TOOLS_PHPPGADMIN_ENABLE"
        "TOOLS_PHPMONGO_ENABLE"
        "TOOLS_PHPMEMCACHEDADMIN_ENABLE"
        "TOOLS_OPCACHE_ENABLE"
        "TOOLS_KAFBAT_ENABLE"
    )
    
    for tool_var in "${tools[@]}"; do
        eval "local tool_enabled=\${${tool_var}:-false}"
        if [ "$tool_enabled" = "true" ]; then
            return 0  # At least one tool is enabled
        fi
    done
    
    return 1  # No tools are enabled
}

##
# Generates base stackvo.yml file (Traefik and network)
#
# Returns:
#   0 - Success
##
generate_base_compose() {
    log_info "Generating stackvo.yml (base compose)..."
    
    # Create generated directory
    mkdir -p "$GENERATED_DIR"
    
    # Create logs directory and set ownership
    mkdir -p "$ROOT_DIR/logs/projects" "$ROOT_DIR/logs/services"
    
    # Set ownership for logs directory
    if [ -n "${HOST_UID}" ] && [ -n "${HOST_GID}" ]; then
        sudo chown -R "${HOST_UID}:${HOST_GID}" "$ROOT_DIR/logs" 2>/dev/null || true
        chmod -R 755 "$ROOT_DIR/logs" 2>/dev/null || true
        log_info "Set logs directory ownership to ${HOST_UID}:${HOST_GID}"
    fi
    
    local output="$GENERATED_DIR/stackvo.yml"
    local template="$ROOT_DIR/core/compose/base.yml"
    
    if [ -f "$template" ]; then
        if ! render_template "$template" > "$output" 2>/dev/null; then
            log_error "Failed to generate stackvo.yml from template"
            return 1
        fi
    else
        # Fallback: create minimal base compose
        if ! cat > "$output" <<'EOF'
services:
  traefik:
    image: traefik:latest
    container_name: stackvo-traefik
    restart: unless-stopped
    command:
      - "--api.dashboard=true"
      - "--api.insecure=true"
      - "--providers.docker=true"
      - "--providers.docker.exposedByDefault=false"
      - "--providers.file.directory=/etc/traefik/dynamic"
      - "--providers.file.watch=true"
      - "--entrypoints.web.address=:80"
      - "--entrypoints.websecure.address=:443"
    ports:
      - "80:80"
      - "443:443"
      - "8080:8080"
    volumes:
      - "/var/run/docker.sock:/var/run/docker.sock:ro"
      - "./core/traefik/traefik.yml:/etc/traefik/traefik.yml:ro"
      - "./core/traefik/dynamic:/etc/traefik/dynamic:ro"
      - "./core/certs:/certs:ro"
    networks:
      - stackvo-net

networks:
  stackvo-net:
    name: stackvo-net
    driver: bridge
EOF
        then
            log_error "Failed to create stackvo.yml"
            return 1
        fi
    fi
    
    log_success "Generated stackvo.yml"
}

##
# Generates dynamic docker-compose.dynamic.yml file (services)
#
# Returns:
#   0 - Success
##
generate_dynamic_compose() {
    log_info "Generating docker-compose.dynamic.yml..."
    
    # Create generated directory
    mkdir -p "$GENERATED_DIR"
    
    local output="$GENERATED_DIR/docker-compose.dynamic.yml"
    
    if ! echo "services:" > "$output" 2>/dev/null; then
        log_error "Failed to create docker-compose.dynamic.yml"
        return 1
    fi
    
    echo "" >> "$output"
    
    # Service definitions: "ENABLE_VAR:template/path"
    local services=(
        # Databases
        "SERVICE_MYSQL_ENABLE:services/mysql/docker-compose.mysql.tpl"
        "SERVICE_MARIADB_ENABLE:services/mariadb/docker-compose.mariadb.tpl"
        "SERVICE_POSTGRES_ENABLE:services/postgres/docker-compose.postgres.tpl"
        "SERVICE_MONGO_ENABLE:services/mongo/docker-compose.mongo.tpl"
        "SERVICE_CASSANDRA_ENABLE:services/cassandra/docker-compose.cassandra.tpl"
        "SERVICE_PERCONA_ENABLE:services/percona/docker-compose.percona.tpl"
        "SERVICE_COUCHDB_ENABLE:services/couchdb/docker-compose.couchdb.tpl"
        "SERVICE_COUCHBASE_ENABLE:services/couchbase/docker-compose.couchbase.tpl"
        
        # Caching
        "SERVICE_REDIS_ENABLE:services/redis/docker-compose.redis.tpl"
        "SERVICE_MEMCACHED_ENABLE:services/memcached/docker-compose.memcached.tpl"
        
        # Message Queues
        "SERVICE_RABBITMQ_ENABLE:services/rabbitmq/docker-compose.rabbitmq.tpl"
        "SERVICE_NATS_ENABLE:services/nats/docker-compose.nats.tpl"
        "SERVICE_KAFKA_ENABLE:services/kafka/docker-compose.kafka.tpl"
        "SERVICE_ACTIVEMQ_ENABLE:services/activemq/docker-compose.activemq.tpl"
        
        # Search
        "SERVICE_ELASTICSEARCH_ENABLE:services/elasticsearch/docker-compose.elasticsearch.tpl"
        "SERVICE_MEILISEARCH_ENABLE:services/meilisearch/docker-compose.meilisearch.tpl"
        "SERVICE_SOLR_ENABLE:services/solr/docker-compose.solr.tpl"
        
        # Monitoring
        "SERVICE_KIBANA_ENABLE:services/kibana/docker-compose.kibana.tpl"
        "SERVICE_GRAFANA_ENABLE:services/grafana/docker-compose.grafana.tpl"
        "SERVICE_LOGSTASH_ENABLE:services/logstash/docker-compose.logstash.tpl"
        "SERVICE_NETDATA_ENABLE:services/netdata/docker-compose.netdata.tpl"
        
        # QA
        "SERVICE_SONARQUBE_ENABLE:services/sonarqube/docker-compose.sonarqube.tpl"
        "SERVICE_SENTRY_ENABLE:services/sentry/docker-compose.sentry.tpl"
        "SERVICE_BLACKFIRE_ENABLE:services/blackfire/docker-compose.blackfire.tpl"
        
        # App Servers
        "SERVICE_TOMCAT_ENABLE:services/tomcat/docker-compose.tomcat.tpl"
        "SERVICE_KONG_ENABLE:services/kong/docker-compose.kong.tpl"
        
        # Tools
        "SERVICE_MAILHOG_ENABLE:services/mailhog/docker-compose.mailhog.tpl"
        "SERVICE_NGROK_ENABLE:services/ngrok/docker-compose.ngrok.tpl"
        "SERVICE_SELENIUM_ENABLE:services/selenium/docker-compose.selenium.tpl"
        
        # Stackvo Web UI
        "STACKVO_UI_ENABLE:ui/stackvo-ui/docker-compose.stackvo-ui.tpl"
    )
    
    # Process all services
    for service_def in "${services[@]}"; do
        local enable_flag="${service_def%%:*}"
        local template_path="${service_def##*:}"
        include_module "$enable_flag" "$template_path" >> "$output"
    done
    
    # Tools container - only if enabled AND at least one tool is active
    if [ "${STACKVO_UI_TOOLS_CONTAINER_ENABLE}" = "true" ]; then
        if has_any_tool_enabled; then
            log_info "Including tools container (at least one tool is enabled)"
            include_module "STACKVO_UI_TOOLS_CONTAINER_ENABLE" "ui/tools/docker-compose.tools.tpl" >> "$output"
        else
            log_warn "Tools container is enabled but no tools are active, skipping..."
        fi
    fi
    
    # Add volumes section
    echo "" >> "$output"
    echo "volumes:" >> "$output"
    
    # Extract volume names to temp file (use /tmp for read-only root)
    local volumes_tmp="/tmp/.stackvo-volumes.tmp"
    > "$volumes_tmp"  # Clear temp file
    
    # Use find instead of glob for Bash 3.x compatibility
    find "$ROOT_DIR/core/templates" -name "*.tpl" -type f | while read -r tpl; do
        # Extract volume names
        awk '/^volumes:/,0 {
            if (/^  [a-z]/) {
                gsub(/:.*/, "")
                gsub(/^  /, "")
                if (length($0) > 0) print "  " $0 ": {}"
            }
        }' "$tpl" >> "$volumes_tmp" 2>/dev/null || true
    done
    
    # Sort and append unique volumes
    sort -u "$volumes_tmp" >> "$output"
    rm -f "$volumes_tmp"
    
    log_success "Generated docker-compose.dynamic.yml"
}
