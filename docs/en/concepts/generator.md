---
title: Generator Sistemi
description: Stackvo generator sistemi ve çalışma prensiplerini anlamak için bu bölümü inceleyin.
---

# Generator Sistemi

Stackvo'un generator sistemi, Pure Bash ile yazılmıştır ve PHP bağımlılığı gerektirmez. Bu sayfa, Bash 3.x+ uyumlu generator'ın nasıl çalıştığını, 5 ana modülünü (compose.sh, project.sh, traefik.sh, tools.sh, config.sh), template processor'ı, akıllı volume yönetimini ve dinamik route generation'ı detaylı olarak açıklamaktadır. Generator, .env dosyasından Docker Compose dosyalarını otomatik oluşturur.

---

## Generator Workflow

```bash
┌─────────────┐
│   .env      │  → Konfigürasyon kaynağı
└──────┬──────┘
       │
       ↓
┌──────────────────────────────────────────────────┐
│     cli/commands/generate.sh                     │
│     (Orchestrator)                               │
│                                                  │
│  1. load_env()           → .env yükle           │
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

## Generator Modülleri

Generator sistemi 5 ana modülden oluşur:

### 1. compose.sh

**Konum:** `cli/lib/generators/compose.sh`  
**Boyut:** 6.2 KB

**Sorumluluklar:**
- `generated/docker-compose.dynamic.yml` oluşturma
- Aktif servisleri `.env` dosyasından okuma
- Servis template'lerini işleme
- Volume tanımlarını otomatik oluşturma

**Ana Fonksiyonlar:**
```bash
generate_dynamic_compose()    # Ana fonksiyon
generate_service()            # Tek bir servis için compose entry
process_service_template()    # Template işleme
create_volume_definitions()   # Volume tanımları
```

**Örnek Çıktı:**
```yaml
services:
  mysql:
    image: mysql:8.0
    container_name: stackvo-mysql
    # ... (template'ten gelen konfigürasyon)

volumes:
  mysql-data:
  redis-data:
  # ... (sadece aktif servislerin volume'ları)
```

### 2. project.sh

**Konum:** `cli/lib/generators/project.sh`  
**Boyut:** 14.6 KB

**Sorumluluklar:**
- `generated/docker-compose.projects.yml` oluşturma
- `projects/` dizinindeki tüm projeleri tarama
- Her proje için `stackvo.json` okuma
- PHP-FPM ve webserver container'ları oluşturma
- Özel konfigürasyonları tespit etme

**Ana Fonksiyonlar:**
```bash
generate_projects()              # Ana fonksiyon
parse_project_config()           # stackvo.json parse
generate_php_container()         # PHP-FPM container
generate_web_container()         # Webserver container
generate_nginx_container()       # Nginx specific
generate_apache_container()      # Apache specific
generate_caddy_container()       # Caddy specific
generate_ferron_container()      # Ferron specific
```

**Konfigürasyon Önceliği:**
1. `projects/myproject/.stackvo/nginx.conf` (özel)
2. `projects/myproject/nginx.conf` (proje root)
3. `core/generated/configs/myproject-nginx.conf` (auto-generated)

### 3. traefik.sh

**Konum:** `cli/lib/generators/traefik.sh`  
**Boyut:** 7 KB

**Sorumluluklar:**
- `generated/stackvo.yml` oluşturma (Traefik base)
- `core/traefik/dynamic/routes.yml` oluşturma
- Servis ve proje route'larını otomatik oluşturma
- SSL/TLS konfigürasyonu

**Ana Fonksiyonlar:**
```bash
generate_base_compose()          # Traefik base compose
generate_traefik_config()        # Traefik static config
generate_traefik_routes()        # Dynamic routes
generate_service_route()         # Servis route'u
generate_project_route()         # Proje route'u
```

**Örnek Route:**
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

**Konum:** `cli/lib/generators/tools.sh`  
**Boyut:** 4.8 KB

**Sorumluluklar:**
- Stackvo UI Tools container konfigürasyonu
- Adminer, PhpMyAdmin, PhpPgAdmin, vb. araçlar
- Tools container için Nginx konfigürasyonu

**Ana Fonksiyonlar:**
```bash
generate_tools_configs()         # Ana fonksiyon
generate_tool_config()           # Tek bir tool için config
generate_tools_nginx_conf()      # Tools Nginx config
```

### 5. config.sh

**Konum:** `cli/lib/generators/config.sh`  
**Boyut:** 2.8 KB

**Sorumluluklar:**
- Servis-specific konfigürasyon dosyaları
- `core/generated/configs/` dizininde dosya oluşturma

**Ana Fonksiyonlar:**
```bash
generate_module_configs()        # Ana fonksiyon
generate_service_config()        # Servis config
```

---

## Template Processor

**Konum:** `cli/lib/template-processor.sh`  
**Boyut:** 4 KB

Template işleme için `envsubst` kullanılır:

```bash
process_template() {
    local template_file=$1
    local output_file=$2
    
    # Environment değişkenlerini template'e uygula
    envsubst < "$template_file" > "$output_file"
}
```

**Template Örneği:**
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

**İşlenmiş Çıktı:**
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

## Akıllı Volume Yönetimi

Generator, sadece aktif servislerin volume'larını oluşturur:

```bash
# .env
SERVICE_MYSQL_ENABLE=true
SERVICE_REDIS_ENABLE=true
SERVICE_POSTGRES_ENABLE=false  # Devre dışı
```

**Oluşturulan Volumes:**
```yaml
volumes:
  mysql-data:      # ✅ Aktif
  redis-data:      # ✅ Aktif
  # postgres-data  # ❌ Oluşturulmaz (devre dışı)
```

---

## Dinamik Route Generation

Traefik route'ları otomatik oluşturulur:

### Servis Route'ları

```bash
# .env
SERVICE_MYSQL_ENABLE=true
SERVICE_RABBITMQ_ENABLE=true
SERVICE_RABBITMQ_URL=rabbitmq
```

**Oluşturulan Route:**
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

### Proje Route'ları

```json
// projects/project1/stackvo.json
{
  "name": "project1",
  "domain": "project1.loc"
}
```

**Oluşturulan Route:**
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

## CLI Entegrasyonu

### Ana CLI

**Konum:** `cli/stackvo.sh`  
**Boyut:** 98 satır

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

### Generate Komutları

```bash
# Tüm konfigürasyonları üret
./cli/stackvo.sh generate

# Sadece projeleri üret
./cli/stackvo.sh generate projects

# Sadece servisleri üret
./cli/stackvo.sh generate services
```

---

## Hata Yönetimi

Generator, hata durumlarında bilgilendirici mesajlar verir:

```bash
# stackvo.json bulunamadı
log_warn "Skipping $project_name: stackvo.json not found"

# PHP versiyonu belirtilmemiş
log_warn "PHP version not found, using default: ${DEFAULT_PHP_VERSION}"

# Template bulunamadı
log_error "Template not found: $template_file"
```

---

## Performans

Generator, hızlı çalışmak için optimize edilmiştir:

- **Pure Bash:** PHP interpreter gerekmez
- **Paralel İşleme:** Bağımsız işlemler paralel çalışır
- **Cache:** Değişmeyen dosyalar yeniden oluşturulmaz
- **Minimal I/O:** Sadece gerekli dosyalar yazılır

**Örnek Çalışma Süresi:**
- 5 servis + 3 proje: ~2 saniye
- 20 servis + 10 proje: ~5 saniye
- 40 servis + 20 proje: ~10 saniye

---

