---
title: Generator System
description: Examine this section to understand Stackvo generator system and operating principles.
---

# Generator System

Stackvo's generator system is written in Pure Bash and does not require PHP dependencies. This page detailedly explains how the Bash 3.x+ compatible generator works, its 5 main modules (compose.sh, project.sh, traefik.sh, tools.sh, config.sh), template processor, smart volume management, and dynamic route generation. The generator automatically creates Docker Compose files from the .env file.

---

## Generator Workflow

```bash
┌─────────────┐
│   .env      │  → Configuration source
└──────┬──────┘
       │
       ↓
┌──────────────────────────────────────────────────┐
│     cli/commands/generate.sh                     │
│     (Orchestrator)                               │
│                                                  │
│  1. load_env()           → Load .env            │
│  2. generate_tools_configs()                    │
│  3. generate_module_configs()                   │
│  4. generate_base_compose()                     │
│  5. generate_traefik_config()                   │
│  6. generate_traefik_routes()                   │
│  7. generate_dynamic_compose()                  │
│  8. generate_projects()                         │
└──────┬───────────────────────────────────────────┘
       │
       ├──→ generated/stackvo.yml
       │    • Traefik configuration
       │    • Network definition
       │
       ├──→ generated/docker-compose.dynamic.yml
       │    • Enabled services only
       │    • Auto-generated volumes
       │
       ├──→ generated/docker-compose.projects.yml
       │    • PHP-FPM containers
       │    • Webserver containers
       │
       ├──→ core/traefik/dynamic/routes.yml
       │    • Dynamic service routes
       │    • TLS configuration
       │
       └──→ core/generated/configs/
            • project1-nginx.conf
            • project2-apache.conf
            • ...
```

---

## Generator Modules

The generator system consists of 5 main modules:

### 1. compose.sh

**Location:** `cli/lib/generators/compose.sh`
**Size:** 6.2 KB

**Responsibilities:**
- Creating `generated/docker-compose.dynamic.yml`
- Reading active services from `.env` file
- Processing service templates
- Creating volume definitions automatically

**Main Functions:**
```bash
generate_dynamic_compose()    # Main function
generate_service()            # Compose entry for a single service
process_service_template()    # Template processing
create_volume_definitions()   # Volume definitions
```

**Example Output:**
```yaml
services:
  mysql:
    image: mysql:8.0
    container_name: stackvo-mysql
    # ... (configuration from template)

volumes:
  mysql-data:
  redis-data:
  # ... (volumes for active services only)
```

### 2. project.sh

**Location:** `cli/lib/generators/project.sh`
**Size:** 14.6 KB

**Responsibilities:**
- Creating `generated/docker-compose.projects.yml`
- Scanning all projects in `projects/` directory
- Reading `stackvo.json` for each project
- Creating PHP-FPM and webserver containers
- Detecting special configurations

**Main Functions:**
```bash
generate_projects()              # Main function
parse_project_config()           # stackvo.json parse
generate_php_container()         # PHP-FPM container
generate_web_container()         # Webserver container
generate_nginx_container()       # Nginx specific
generate_apache_container()      # Apache specific
generate_caddy_container()       # Caddy specific
generate_ferron_container()      # Ferron specific
```

**Configuration Priority:**
1. `projects/myproject/.stackvo/nginx.conf` (custom)
2. `projects/myproject/nginx.conf` (project root)
3. `core/generated/configs/myproject-nginx.conf` (auto-generated)

### 3. traefik.sh

**Location:** `cli/lib/generators/traefik.sh`
**Size:** 7 KB

**Responsibilities:**
- Creating `generated/stackvo.yml` (Traefik base)
- Creating `core/traefik/dynamic/routes.yml`
- Creating service and project routes automatically
- SSL/TLS configuration

**Main Functions:**
```bash
generate_base_compose()          # Traefik base compose
generate_traefik_config()        # Traefik static config
generate_traefik_routes()        # Dynamic routes
generate_service_route()         # Service route
generate_project_route()         # Project route
```

**Example Route:**
```yaml
http:
  routers:
    mysql:
      rule: "Host(`mysql.stackvo.loc`)"
      service: mysql
      entryPoints:
        - websecure
      tls: {}
  
  services:
    mysql:
      loadBalancer:
        servers:
          - url: "http://stackvo-mysql:3306"
```

### 4. tools.sh

**Location:** `cli/lib/generators/tools.sh`
**Size:** 4.8 KB

**Responsibilities:**
- Stackvo UI Tools container configuration
- Tools like Adminer, PhpMyAdmin, PhpPgAdmin, etc.
- Nginx configuration for Tools container

**Main Functions:**
```bash
generate_tools_configs()         # Main function
generate_tool_config()           # Config for a single tool
generate_tools_nginx_conf()      # Tools Nginx config
```

### 5. config.sh

**Location:** `cli/lib/generators/config.sh`
**Size:** 2.8 KB

**Responsibilities:**
- Service-specific configuration files
- Creating files in `core/generated/configs/` directory

**Main Functions:**
```bash
generate_module_configs()        # Main function
generate_service_config()        # Service config
```

---

## Template Processor

**Location:** `cli/lib/template-processor.sh`
**Size:** 4 KB

`envsubst` is used for template processing:

```bash
process_template() {
    local template_file=$1
    local output_file=$2
    
    # Apply environment variables to template
    envsubst < "$template_file" > "$output_file"
}
```

**Template Example:**
```yaml
# core/templates/services/mysql/docker-compose.yml
services:
  mysql:
    image: mysql:${SERVICE_MYSQL_VERSION}
    container_name: stackvo-mysql
    environment:
      MYSQL_ROOT_PASSWORD: ${SERVICE_MYSQL_ROOT_PASSWORD}
      MYSQL_DATABASE: ${SERVICE_MYSQL_DATABASE}
      MYSQL_USER: ${SERVICE_MYSQL_USER}
      MYSQL_PASSWORD: ${SERVICE_MYSQL_PASSWORD}
```

**Processed Output:**
```yaml
services:
  mysql:
    image: mysql:8.0
    container_name: stackvo-mysql
    environment:
      MYSQL_ROOT_PASSWORD: root
      MYSQL_DATABASE: stackvo
      MYSQL_USER: stackvo
      MYSQL_PASSWORD: stackvo
```

---

## Smart Volume Management

The generator creates volumes only for active services:

```bash
# .env
SERVICE_MYSQL_ENABLE=true
SERVICE_REDIS_ENABLE=true
SERVICE_POSTGRES_ENABLE=false  # Disabled
```

**Generated Volumes:**
```yaml
volumes:
  mysql-data:      # ✅ Active
  redis-data:      # ✅ Active
  # postgres-data  # ❌ Not created (disabled)
```

---

## Dynamic Route Generation

Traefik routes are automatically generated:

### Service Routes

```bash
# .env
SERVICE_MYSQL_ENABLE=true
SERVICE_RABBITMQ_ENABLE=true
SERVICE_RABBITMQ_URL=rabbitmq
```

**Generated Route:**
```yaml
http:
  routers:
    rabbitmq:
      rule: "Host(`rabbitmq.stackvo.loc`)"
      service: rabbitmq
      entryPoints:
        - websecure
      tls: {}
```

### Project Routes

```json
// projects/project1/stackvo.json
{
  "name": "project1",
  "domain": "project1.loc"
}
```

**Generated Route:**
```yaml
http:
  routers:
    project1:
      rule: "Host(`project1.loc`)"
      service: project1
      entryPoints:
        - websecure
      tls: {}
```

---

## CLI Integration

### Main CLI

**Location:** `cli/stackvo.sh`
**Size:** 98 lines

```bash
case "$COMMAND" in
    generate)
        bash "$CLI_DIR/commands/generate.sh" "$@"
        ;;
    up)
        docker compose "${COMPOSE_FILES[@]}" up -d
        ;;
    down)
        docker compose "${COMPOSE_FILES[@]}" down
        ;;
    # ...
esac
```

### Generate Commands

```bash
# Generate all configurations
./stackvo.sh generate

# Generate projects only
./stackvo.sh generate projects

# Generate services only
./stackvo.sh generate services
```

---

## Error Management

The generator gives informative messages in error situations:

```bash
# stackvo.json not found
log_warn "Skipping $project_name: stackvo.json not found"

# PHP version not specified
log_warn "PHP version not found, using default: ${DEFAULT_PHP_VERSION}"

# Template not found
log_error "Template not found: $template_file"
```

---

## Performance

The generator is optimized to run fast:

- **Pure Bash:** No PHP interpreter required
- **Parallel Processing:** Independent processes run in parallel
- **Cache:** Unchanged files are not regenerated
- **Minimal I/O:** Only necessary files are written

**Example Execution Time:**
- 5 services + 3 projects: ~2 seconds
- 20 services + 10 projects: ~5 seconds
- 40 services + 20 projects: ~10 seconds

---
