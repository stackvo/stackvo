---
title: Traefik
description: Stackvo'un reverse proxy ve SSL/TLS yönetim sistemidir. Otomatik servis keşfi ve routing sağlar.
---

# Traefik

Traefik, Stackvo'un reverse proxy ve SSL/TLS yönetim sistemidir. Bu sayfa, modern reverse proxy ve load balancer olan Traefik'in otomatik servis keşfi, dinamik konfigürasyon, Docker labels ile routing, SSL/TLS sertifika yönetimi, middleware kullanımı ve dashboard özelliklerini detaylı olarak açıklamaktadır. Traefik, container'ları otomatik olarak keşfeder ve route oluşturur.

---

## Traefik Nedir?

Traefik, modern bir reverse proxy ve load balancer'dır. Docker container'larını otomatik olarak keşfeder ve route'lar.

**Avantajlar:**
- Otomatik servis keşfi (Docker labels)
- Dinamik konfigürasyon (yeniden başlatma gerektirmez)
- SSL/TLS yönetimi (Let's Encrypt desteği)
- HTTP → HTTPS yönlendirme
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

## Konfigürasyon

### Static Configuration

**Dosya:** `core/traefik/traefik.yml`

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

**Dosya:** `core/traefik/dynamic/routes.yml` (auto-generated)

```yaml
http:
  routers:
    # Servis route'ları
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
    
    # Proje route'ları
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

Traefik, Docker container'larındaki label'ları okuyarak otomatik routing yapar.

### Servis Labels

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

### Proje Labels

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

## SSL/TLS Yönetimi

### Self-Signed Sertifikalar

Stackvo, local development için self-signed sertifikalar kullanır:

```bash
# Sertifika oluşturma
./cli/utils/generate-ssl-certs.sh
```

**Oluşturulan Dosyalar:**
- `core/certs/stackvo-wildcard.crt`
- `core/certs/stackvo-wildcard.key`

**Wildcard Domain:**
```
*.stackvo.loc
*.loc
```

### Let's Encrypt (Production)

Production ortamında Let's Encrypt kullanılabilir:

```yaml
# .env
LETSENCRYPT_ENABLE=true
LETSENCRYPT_EMAIL=admin@yourdomain.com
```

**Not:** Let's Encrypt sadece public domain'ler için çalışır (`.loc` gibi local domain'lerde çalışmaz).

---

## Routing

### Host-based Routing

```yaml
# Domain bazlı routing
rule: "Host(`project1.loc`)"
rule: "Host(`mysql.stackvo.loc`)"
```

### Path-based Routing

```yaml
# Path bazlı routing
rule: "Host(`stackvo.loc`) && PathPrefix(`/api`)"
rule: "Host(`stackvo.loc`) && Path(`/admin`)"
```

### Multiple Domains

```yaml
# Birden fazla domain
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

Traefik dashboard'a erişim:

```
http://localhost:8080
```

**Dashboard Özellikleri:**
- Aktif router'lar
- Aktif servisler
- Middleware'ler
- TLS sertifikaları
- Real-time metrics

---

## Otomatik Servis Keşfi

Traefik, Docker container'larını otomatik olarak keşfeder:

### 1. Container Başlatma

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

### 2. Traefik Otomatik Keşif

Traefik, container'ı otomatik olarak keşfeder ve route oluşturur.

### 3. Erişim

```
https://my-service.loc
```

**Not:** `/etc/hosts` dosyasına domain eklemeyi unutmayın:
```
127.0.0.1  my-service.loc
```

---

## Load Balancing

Traefik, birden fazla instance arasında load balancing yapar:

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

Traefik, `myapp.loc` için gelen istekleri `app-1` ve `app-2` arasında dağıtır.

---

## Troubleshooting

### Traefik Logları

```bash
# Container logları
docker logs stackvo-traefik

# Access logları
cat logs/traefik/access.log

# Error logları
docker logs stackvo-traefik 2>&1 | grep ERROR
```

### Dashboard Kontrol

```
http://localhost:8080
```

Dashboard'da:
- Router'ların aktif olduğunu kontrol edin
- Service'lerin healthy olduğunu kontrol edin
- TLS sertifikalarının yüklendiğini kontrol edin

### Route Test

```bash
# HTTP isteği
curl -H "Host: project1.loc" http://localhost

# HTTPS isteği
curl -k -H "Host: project1.loc" https://localhost
```

### DNS Kontrol

```bash
# /etc/hosts kontrolü
cat /etc/hosts | grep stackvo
cat /etc/hosts | grep project1
```

### Container Network Kontrol

```bash
# Traefik'in network'e bağlı olduğunu kontrol et
docker inspect stackvo-traefik | grep -A 10 Networks

# Container'ın Traefik tarafından görüldüğünü kontrol et
docker exec stackvo-traefik wget -O- http://stackvo-project1-web
```

---

## Best Practices

### 1. Label Kullanımı

✅ **Doğru:**
```yaml
labels:
  - "traefik.enable=true"
  - "traefik.http.routers.myapp.rule=Host(`myapp.loc`)"
  - "traefik.http.routers.myapp.entrypoints=websecure"
  - "traefik.http.routers.myapp.tls=true"
```

### 2. Port Belirtme

Eğer container birden fazla port expose ediyorsa, port belirtin:

```yaml
labels:
  - "traefik.http.services.myapp.loadbalancer.server.port=8080"
```

### 3. Network

Tüm container'lar `stackvo-net` network'ünde olmalı:

```yaml
networks:
  - stackvo-net
```

### 4. HTTPS Kullanımı

Production'da her zaman HTTPS kullanın:

```yaml
entryPoints:
  - websecure
```

---

