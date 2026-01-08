---
title: Architecture
description: Examine this section to understand Stackvo architecture and operating principles.
---

# Architecture

Stackvo offers a modular and flexible structure with a three-layer Docker Compose architecture. This page details how the three layers—base layer (Traefik), services layer (infrastructure), and projects layer (applications)—work, interact with each other, and the compose merge strategy. The layered architecture provides ease of maintenance and independent updates.

---

## Three-Layer Docker Compose System

```
┌─────────────────────────────────────────────────────────┐
│              generated/stackvo.yml                    │
│              (Base Layer - Traefik)                     │
│  • Traefik Reverse Proxy                                │
│  • stackvo-net Network (172.30.0.0/16)                │
│  • Basic routing and SSL configuration                  │
│  • Template: core/compose/base.yml                      │
└─────────────────────────────────────────────────────────┘
                         ↓ merge
┌─────────────────────────────────────────────────────────┐
│      generated/docker-compose.dynamic.yml               │
│      (Services Layer - Infrastructure)                  │
│  • 40+ Services (MySQL, Redis, RabbitMQ, etc.)          │
│  • Templates: core/templates/services/*/                │
│  • Automatic volume definitions                         │
│  • Generator: cli/lib/generators/compose.sh             │
└─────────────────────────────────────────────────────────┘
                         ↓ merge
┌─────────────────────────────────────────────────────────┐
│      generated/docker-compose.projects.yml              │
│      (Projects Layer - Applications)                    │
│  • PHP-FPM Containers (project-name-php)                │
│  • Webserver Containers (project-name-web)              │
│  • Traefik routing labels                               │
│  • Project-specific volumes                             │
│  • Generator: cli/lib/generators/project.sh             │
└─────────────────────────────────────────────────────────┘
                         ↓
              ✅ Fully Integrated Stack
```

---

## Layers

### 1. Base Layer (Traefik)

**File:** `generated/stackvo.yml`

**Responsibilities:**
- Traefik reverse proxy container
- `stackvo-net` Docker network creation
- Basic SSL/TLS configuration
- HTTP → HTTPS redirection

**Template:** `core/compose/base.yml`

**Example:**
```yaml
services:
  traefik:
    image: traefik:v2.10
    container_name: stackvo-traefik
    restart: unless-stopped
    networks:
      - stackvo-net
    ports:
      - "80:80"
      - "443:443"
      - "8080:8080"
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro
      - ./core/traefik/traefik.yml:/etc/traefik/traefik.yml:ro
      - ./core/traefik/dynamic:/etc/traefik/dynamic:ro
      - ./core/certs:/etc/traefik/certs:ro

networks:
  stackvo-net:
    driver: bridge
    ipam:
      config:
        - subnet: 172.30.0.0/16
```

### 2. Services Layer (Infrastructure)

**File:** `generated/docker-compose.dynamic.yml`

**Responsibilities:**
- 40+ infrastructure services (MySQL, PostgreSQL, Redis, RabbitMQ, etc.)
- Service-specific volume definitions
- Traefik routing labels
- Inter-service dependencies

**Generator:** `cli/lib/generators/compose.sh`

**Templates:** `core/templates/services/*/`

**Example:**
```yaml
services:
  mysql:
    image: mysql:8.0
    container_name: stackvo-mysql
    restart: unless-stopped
    environment:
      MYSQL_ROOT_PASSWORD: root
      MYSQL_DATABASE: stackvo
      MYSQL_USER: stackvo
      MYSQL_PASSWORD: stackvo
    volumes:
      - mysql-data:/var/lib/mysql
      - ./logs/services/mysql:/var/log/mysql
    networks:
      - stackvo-net
    labels:
      - "traefik.enable=false"

volumes:
  mysql-data:
```

### 3. Projects Layer (Applications)

**File:** `generated/docker-compose.projects.yml`

**Responsibilities:**
- User project containers
- PHP-FPM and webserver (Nginx/Apache/Caddy/Ferron) containers
- Project-specific volume mounts
- Domain routing (Traefik labels)

**Generator:** `cli/lib/generators/project.sh`

**Example:**
```yaml
services:
  project1-php:
    image: php:8.2-fpm
    container_name: stackvo-project1-php
    restart: unless-stopped
    volumes:
      - ./projects/project1:/var/www/html
      - ./logs/projects/project1:/var/log/project1
    networks:
      - stackvo-net

  project1-web:
    image: nginx:alpine
    container_name: stackvo-project1-web
    restart: unless-stopped
    volumes:
      - ./projects/project1:/var/www/html
      - ./core/generated/configs/project1-nginx.conf:/etc/nginx/conf.d/default.conf:ro
    networks:
      - stackvo-net
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.project1.rule=Host(`project1.loc`)"
      - "traefik.http.routers.project1.entrypoints=websecure"
      - "traefik.http.routers.project1.tls=true"
    depends_on:
      - project1-php
```

---

## Network Architecture

### stackvo-net

All containers run on a single Docker network:

```
stackvo-net (172.30.0.0/16)
├── 172.30.0.1 (Gateway)
├── Traefik (Reverse Proxy)
├── MySQL (stackvo-mysql)
├── Redis (stackvo-redis)
├── RabbitMQ (stackvo-rabbitmq)
├── Elasticsearch (stackvo-elasticsearch)
├── Stackvo UI (stackvo-ui)
├── Tools Container (stackvo-tools)
├── Project1-PHP (stackvo-project1-php)
├── Project1-Web (stackvo-project1-web)
└── ... (other services)
```

**Advantages:**
- Containers can find each other via hostname
- Isolation and security
- Easy service discovery
- Simple network management

**Communication Example:**
```
External → Traefik (80/443) → Nginx → PHP-FPM
PHP → MySQL (stackvo-mysql:3306)
PHP → Redis (stackvo-redis:6379)
PHP → RabbitMQ (stackvo-rabbitmq:5672)
```

---

## Directory Structure

```
stackvo/
├── .env                          # Main configuration
├── core/
│   ├── cli/                          # CLI commands
│   ├── stackvo.sh              # Main CLI
│   ├── commands/                 # Command scripts
│   ├── lib/                      # Libraries
│   │   └── generators/           # Generator modules
│   └── utils/                    # Utility scripts
│
├── core/                         # Core files
│   ├── ui/                       # Stackvo Web UI
│   │   ├── client/               # Vue.js frontend
│   │   ├── server/               # Node.js backend
│   │   └── dist/                 # Build output
│   ├── compose/
│   │   └── base.yml              # Traefik base template
│   ├── templates/                # Service and webserver templates
│   │   ├── services/             # 40+ service templates
│   │   ├── servers/              # Webserver templates
│   │   └── ui/                   # UI templates
│   ├── traefik/                  # Traefik configuration
│   │   ├── traefik.yml
│   │   └── dynamic/
│   │       └── routes.yml        # Auto-generated routes
│   ├── certs/                    # SSL certificates
│   └── generated/                # Auto-generated configs
│       └── configs/
│
├── generated/                    # Auto-generated compose files
│   ├── stackvo.yml
│   ├── docker-compose.dynamic.yml
│   └── docker-compose.projects.yml
│
├── projects/                     # User projects
│   └── project1/
│       ├── stackvo.json
│       ├── .stackvo/
│       └── public/
│
└── logs/                         # Container logs
    ├── services/
    └── projects/
```

---

## Compose Merge Strategy

Stackvo works by merging three compose files:

```bash
docker compose \
  -f generated/stackvo.yml \
  -f generated/docker-compose.dynamic.yml \
  -f generated/docker-compose.projects.yml \
  up -d
```

**Advantages:**
- Modular structure
- Easy maintenance
- Independent updates
- Clean separation of concerns

---

## Lifecycle

### 1. Configuration

```bash
# Edit .env file
nano .env
```

### 2. Generation

```bash
# Run generator
./stackvo.sh generate
```

**Generator does the following:**

1. Reads `.env` file
2. Generates SSL certificates (if nonexistent)
3. Generates `generated/stackvo.yml`
4. Generates `generated/docker-compose.dynamic.yml`
5. Generates `generated/docker-compose.projects.yml`
6. Generates `core/traefik/dynamic/routes.yml`
7. Generates service configurations in `core/generated/configs/` directory

### 3. Deployment

```bash
# Start services
./stackvo.sh up
```

### 4. Management

```bash
# Check status
./stackvo.sh ps

# Monitor logs
./stackvo.sh logs

# Restart
./stackvo.sh restart
```
