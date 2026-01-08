---
title: Traefik
description: Stackvo's reverse proxy and SSL/TLS management system. Provides automatic service discovery and routing.
---

# Traefik

Traefik is Stackvo's reverse proxy and SSL/TLS management system. This page detailedly explains Traefik, a modern reverse proxy and load balancer, its automatic service discovery, dynamic configuration, routing with Docker labels, SSL/TLS certificate management, middleware usage, and dashboard features. Traefik automatically discovers containers and creates routes.

---

## What is Traefik?

Traefik is a modern reverse proxy and load balancer. It automatically discovers and routes Docker containers.

**Advantages:**
- Automatic service discovery (Docker labels)
- Dynamic configuration (no restart required)
- SSL/TLS management (Let's Encrypt support)
- HTTP → HTTPS redirection
- Load balancing
- Web dashboard

---

## Traefik Container

```yaml
services:
  traefik:
    image: traefik:v2.10
    container_name: stackvo-traefik
    restart: unless-stopped
    
    networks:
      - stackvo-net
    
    ports:
      - "80:80"       # HTTP
      - "443:443"     # HTTPS
      - "8080:8080"   # Dashboard
    
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro
      - ./core/traefik/traefik.yml:/etc/traefik/traefik.yml:ro
      - ./core/traefik/dynamic:/etc/traefik/dynamic:ro
      - ./core/certs:/etc/traefik/certs:ro
```

---

## Configuration

### Static Configuration

**File:** `core/traefik/traefik.yml`

```yaml
# Entrypoints
entryPoints:
  web:
    address: ":80"
    http:
      redirections:
        entryPoint:
          to: websecure
          scheme: https
  
  websecure:
    address: ":443"

# Providers
providers:
  docker:
    endpoint: "unix:///var/run/docker.sock"
    exposedByDefault: false
    network: stackvo-net
  
  file:
    directory: /etc/traefik/dynamic
    watch: true

# API and Dashboard
api:
  dashboard: true
  insecure: true

# Logging
log:
  level: INFO

# Access logs
accessLog:
  filePath: /var/log/traefik/access.log
```

### Dynamic Configuration

**File:** `core/traefik/dynamic/routes.yml` (auto-generated)

```yaml
http:
  routers:
    # Service routes
    mysql:
      rule: "Host(`mysql.stackvo.loc`)"
      service: mysql
      entryPoints:
        - websecure
      tls: {}
    
    rabbitmq:
      rule: "Host(`rabbitmq.stackvo.loc`)"
      service: rabbitmq
      entryPoints:
        - websecure
      tls: {}
    
    # Project routes
    project1:
      rule: "Host(`project1.loc`)"
      service: project1
      entryPoints:
        - websecure
      tls: {}
  
  services:
    mysql:
      loadBalancer:
        servers:
          - url: "http://stackvo-mysql:3306"
    
    rabbitmq:
      loadBalancer:
        servers:
          - url: "http://stackvo-rabbitmq:15672"
    
    project1:
      loadBalancer:
        servers:
          - url: "http://stackvo-project1-web:80"

# TLS Configuration
tls:
  certificates:
    - certFile: /etc/traefik/certs/stackvo-wildcard.crt
      keyFile: /etc/traefik/certs/stackvo-wildcard.key
```

---

## Docker Labels

Traefik performs automatic routing by reading labels on Docker containers.

### Service Labels

```yaml
services:
  rabbitmq:
    image: rabbitmq:3-management
    container_name: stackvo-rabbitmq
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.rabbitmq.rule=Host(`rabbitmq.stackvo.loc`)"
      - "traefik.http.routers.rabbitmq.entrypoints=websecure"
      - "traefik.http.routers.rabbitmq.tls=true"
      - "traefik.http.services.rabbitmq.loadbalancer.server.port=15672"
```

### Project Labels

```yaml
services:
  project1-web:
    image: nginx:alpine
    container_name: stackvo-project1-web
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.project1.rule=Host(`project1.loc`)"
      - "traefik.http.routers.project1.entrypoints=websecure"
      - "traefik.http.routers.project1.tls=true"
      - "traefik.http.services.project1.loadbalancer.server.port=80"
```

---

## SSL/TLS Management

### Self-Signed Certificates

Stackvo uses self-signed certificates for local development:

```bash
# Generate certificates
./core/cli/utils/generate-ssl-certs.sh
```

**Generated Files:**
- `core/certs/stackvo-wildcard.crt`
- `core/certs/stackvo-wildcard.key`

**Wildcard Domain:**
```
*.stackvo.loc
*.loc
```

### Let's Encrypt (Production)

Let's Encrypt can be used in production environment:

```yaml
# .env
LETSENCRYPT_ENABLE=true
LETSENCRYPT_EMAIL=admin@yourdomain.com
```

**Note:** Let's Encrypt only works for public domains (does not work on local domains like `.loc`).

---

## Routing

### Host-based Routing

```yaml
# Domain based routing
rule: "Host(`project1.loc`)"
rule: "Host(`mysql.stackvo.loc`)"
```

### Path-based Routing

```yaml
# Path based routing
rule: "Host(`stackvo.loc`) && PathPrefix(`/api`)"
rule: "Host(`stackvo.loc`) && Path(`/admin`)"
```

### Multiple Domains

```yaml
# Multiple domains
rule: "Host(`project1.loc`) || Host(`www.project1.loc`)"
```

---

## Middleware

### HTTP → HTTPS Redirect

```yaml
entryPoints:
  web:
    address: ":80"
    http:
      redirections:
        entryPoint:
          to: websecure
          scheme: https
```

### Headers

```yaml
http:
  middlewares:
    security-headers:
      headers:
        customResponseHeaders:
          X-Frame-Options: "SAMEORIGIN"
          X-Content-Type-Options: "nosniff"
          X-XSS-Protection: "1; mode=block"
```

### Rate Limiting

```yaml
http:
  middlewares:
    rate-limit:
      rateLimit:
        average: 100
        burst: 50
```

---

## Dashboard

Access to Traefik dashboard:

```
http://localhost:8080
```

**Dashboard Features:**
- Active routers
- Active services
- Middlewares
- TLS certificates
- Real-time metrics

---

## Automatic Service Discovery

Traefik automatically discovers Docker containers:

### 1. Start Container

```bash
docker run -d \
  --name my-service \
  --network stackvo-net \
  --label "traefik.enable=true" \
  --label "traefik.http.routers.my-service.rule=Host(\`my-service.loc\`)" \
  --label "traefik.http.routers.my-service.entrypoints=websecure" \
  --label "traefik.http.routers.my-service.tls=true" \
  nginx:alpine
```

### 2. Traefik Auto-Discovery

Traefik automatically discovers the container and creates a route.

### 3. Access

```
https://my-service.loc
```

**Note:** Don't forget to add the domain to the `/etc/hosts` file:
```
127.0.0.1  my-service.loc
```

---

## Load Balancing

Traefik balances load between multiple instances:

```yaml
services:
  app-1:
    image: myapp:latest
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.myapp.rule=Host(`myapp.loc`)"
      - "traefik.http.services.myapp.loadbalancer.server.port=80"
  
  app-2:
    image: myapp:latest
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.myapp.rule=Host(`myapp.loc`)"
      - "traefik.http.services.myapp.loadbalancer.server.port=80"
```

Traefik distributes requests for `myapp.loc` between `app-1` and `app-2`.

---

## Troubleshooting

### Traefik Logs

```bash
# Container logs
docker logs stackvo-traefik

# Access logs
cat logs/traefik/access.log

# Error logs
docker logs stackvo-traefik 2>&1 | grep ERROR
```

### Dashboard Check

```
http://localhost:8080
```

In Dashboard:
- Check if routers are active
- Check if services are healthy
- Check if TLS certificates are loaded

### Route Test

```bash
# HTTP request
curl -H "Host: project1.loc" http://localhost

# HTTPS request
curl -k -H "Host: project1.loc" https://localhost
```

### DNS Check

```bash
# /etc/hosts check
cat /etc/hosts | grep stackvo
cat /etc/hosts | grep project1
```

### Container Network Check

```bash
# Check if Traefik is connected to network
docker inspect stackvo-traefik | grep -A 10 Networks

# Check if container is seen by Traefik
docker exec stackvo-traefik wget -O- http://stackvo-project1-web
```

---

## Best Practices

### 1. Label Usage

✅ **Correct:**
```yaml
labels:
  - "traefik.enable=true"
  - "traefik.http.routers.myapp.rule=Host(`myapp.loc`)"
  - "traefik.http.routers.myapp.entrypoints=websecure"
  - "traefik.http.routers.myapp.tls=true"
```

### 2. Port Specification

If container exposes multiple ports, specify the port:

```yaml
labels:
  - "traefik.http.services.myapp.loadbalancer.server.port=8080"
```

### 3. Network

All containers must be in `stackvo-net` network:

```yaml
networks:
  - stackvo-net
```

### 4. HTTPS Usage

Always use HTTPS in production:

```yaml
entryPoints:
  - websecure
```
