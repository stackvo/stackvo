---
title: Mimari
description: Stackvo mimarisi ve çalışma prensiplerini anlamak için bu bölümü inceleyin.
---

# Mimari

Stackvo, üç katmanlı Docker Compose mimarisi ile modüler ve esnek bir yapı sunar. Bu sayfa, base layer (Traefik), services layer (infrastructure) ve projects layer (applications) olmak üzere üç katmanın nasıl çalıştığını, birbirleriyle nasıl etkileştiğini ve compose merge stratejisini detaylı olarak açıklamaktadır. Katmanlı mimari, kolay bakım ve bağımsız güncelleme imkanı sağlar.

---

## Üç Katmanlı Docker Compose Sistemi

```
┌─────────────────────────────────────────────────────────┐
│              generated/stackvo.yml                    │
│              (Base Layer - Traefik)                     │
│  • Traefik Reverse Proxy                                │
│  • stackvo-net Network (172.30.0.0/16)                │
│  • Temel routing ve SSL yapılandırması                  │
│  • Template: core/compose/base.yml                      │
└─────────────────────────────────────────────────────────┘
                         ↓ merge
┌─────────────────────────────────────────────────────────┐
│      generated/docker-compose.dynamic.yml               │
│      (Services Layer - Infrastructure)                  │
│  • 40+ Servis (MySQL, Redis, RabbitMQ, etc.)            │
│  • Templates: core/templates/services/*/                │
│  • Otomatik volume tanımları                            │
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
              ✅ Tam Entegre Stack
```

---

## Katmanlar

### 1. Base Layer (Traefik)

**Dosya:** `generated/stackvo.yml`

**Sorumluluklar:**
- Traefik reverse proxy container'ı
- `stackvo-net` Docker network oluşturma
- Temel SSL/TLS konfigürasyonu
- HTTP → HTTPS yönlendirme

**Template:** `core/compose/base.yml`

**Örnek:**
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

**Dosya:** `generated/docker-compose.dynamic.yml`

**Sorumluluklar:**
- 40+ altyapı servisi (MySQL, PostgreSQL, Redis, RabbitMQ, vb.)
- Servis-specific volume tanımları
- Traefik routing labels
- Servisler arası bağımlılıklar

**Generator:** `cli/lib/generators/compose.sh`

**Templates:** `core/templates/services/*/`

**Örnek:**
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

**Dosya:** `generated/docker-compose.projects.yml`

**Sorumluluklar:**
- Kullanıcı projelerinin container'ları
- PHP-FPM ve webserver (Nginx/Apache/Caddy/Ferron) container'ları
- Proje-specific volume mount'ları
- Domain routing (Traefik labels)

**Generator:** `cli/lib/generators/project.sh`

**Örnek:**
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

## Network Mimarisi

### stackvo-net

Tüm container'lar tek bir Docker network üzerinde çalışır:

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
└── ... (diğer servisler)
```

**Avantajlar:**
- Container'lar birbirlerini hostname ile bulabilir
- İzolasyon ve güvenlik
- Kolay servis keşfi
- Basit network yönetimi

**İletişim Örneği:**
```
External → Traefik (80/443) → Nginx → PHP-FPM
PHP → MySQL (stackvo-mysql:3306)
PHP → Redis (stackvo-redis:6379)
PHP → RabbitMQ (stackvo-rabbitmq:5672)
```

---

## Dizin Yapısı

```
stackvo/
├── .env                          # Ana konfigürasyon
├── core/
│   ├── cli/                          # CLI komutları
│   ├── stackvo.sh              # Ana CLI
│   ├── commands/                 # Komut scriptleri
│   ├── lib/                      # Kütüphaneler
│   │   └── generators/           # Generator modülleri
│   └── utils/                    # Yardımcı scriptler
│
├── core/                         # Core dosyalar
│   ├── ui/                       # Stackvo Web UI
│   │   ├── client/               # Vue.js frontend
│   │   ├── server/               # Node.js backend
│   │   └── dist/                 # Build output
│   ├── compose/
│   │   └── base.yml              # Traefik base template
│   ├── templates/                # Servis ve webserver template'leri
│   │   ├── services/             # 40+ servis template
│   │   ├── servers/              # Webserver template'leri
│   │   └── ui/                   # UI template'leri
│   ├── traefik/                  # Traefik konfigürasyonu
│   │   ├── traefik.yml
│   │   └── dynamic/
│   │       └── routes.yml        # Auto-generated routes
│   ├── certs/                    # SSL sertifikaları
│   └── generated/                # Auto-generated configs
│       └── configs/
│
├── generated/                    # Auto-generated compose files
│   ├── stackvo.yml
│   ├── docker-compose.dynamic.yml
│   └── docker-compose.projects.yml
│
├── projects/                     # Kullanıcı projeleri
│   └── project1/
│       ├── stackvo.json
│       ├── .stackvo/
│       └── public/
│
└── logs/                         # Container logları
    ├── services/
    └── projects/
```

---

## Compose Merge Stratejisi

Stackvo, üç compose dosyasını merge ederek çalışır:

```bash
docker compose \
  -f generated/stackvo.yml \
  -f generated/docker-compose.dynamic.yml \
  -f generated/docker-compose.projects.yml \
  up -d
```

**Avantajlar:**
- Modüler yapı
- Kolay bakım
- Bağımsız güncelleme
- Temiz separation of concerns

---

## Lifecycle

### 1. Konfigürasyon

```bash
# .env dosyasını düzenle
nano .env
```

### 2. Generation

```bash
# Generator'ı çalıştır
./core/cli/stackvo.sh generate
```

**Generator şunları yapar:**

1. `.env` dosyasını okur
2. SSL sertifikaları oluşturur (yoksa)
3. `generated/stackvo.yml` oluşturur
4. `generated/docker-compose.dynamic.yml` oluşturur
5. `generated/docker-compose.projects.yml` oluşturur
6. `core/traefik/dynamic/routes.yml` oluşturur
7. `core/generated/configs/` dizininde servis konfigürasyonları oluşturur

### 3. Deployment

```bash
# Servisleri başlat
./core/cli/stackvo.sh up
```

### 4. Management

```bash
# Durumu kontrol et
./core/cli/stackvo.sh ps

# Logları izle
./core/cli/stackvo.sh logs

# Yeniden başlat
./core/cli/stackvo.sh restart
```

---

