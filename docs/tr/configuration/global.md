# Global Konfigürasyon

Global konfigürasyon, `.env` dosyası üzerinden yönetilir ve tüm Stackvo sistemini etkiler. Bu sayfa, 364 satırlık `.env` dosyasının tüm bölümlerini, Traefik ayarlarından servis konfigürasyonlarına, Docker network ayarlarından güvenlik parametrelerine kadar her şeyi detaylı olarak açıklamaktadır. Global ayarlar, tüm projeler ve servisler için varsayılan değerleri belirler.

---

## .env Dosyası

`.env` dosyası, Stackvo'un ana konfigürasyon dosyasıdır. 364 satır ve 11 ana bölümden oluşur.

### Dosya Yapısı

```bash
# 1. Traefik Ayarları (~20 satır)
# 2. Varsayılan Proje Ayarları (~5 satır)
# 3. Stackvo UI Ayarları (~10 satır)
# 4. Docker Network (~5 satır)
# 5. Host System Mappings (~5 satır)
# 6. Security Settings (~5 satır)
# 7. Port Mappings (~10 satır)
# 8. CLI Behavior (~10 satır)
# 9. Supported Languages (~30 satır)
# 10. Stackvo Web UI Tools (~40 satır)
# 11. Services (~180 satır)
```

---

## Traefik Ayarları

Reverse proxy ve SSL/TLS konfigürasyonu.

```bash
# Global domain suffix
DEFAULT_TLD_SUFFIX=stackvo.loc

# SSL/TLS
SSL_ENABLE=true
REDIRECT_TO_HTTPS=true

# Let's Encrypt (sadece public domain'ler için)
LETSENCRYPT_ENABLE=false
LETSENCRYPT_EMAIL=admin@stackvo.loc

# Traefik subdomain
TRAEFIK_URL=traefik
```

**Açıklamalar:**
- `DEFAULT_TLD_SUFFIX`: Tüm servislerin domain suffix'i (örn: `mysql.stackvo.loc`)
- `SSL_ENABLE`: SSL/TLS desteğini aktifleştirir
- `REDIRECT_TO_HTTPS`: HTTP isteklerini HTTPS'e yönlendirir
- `LETSENCRYPT_ENABLE`: Let's Encrypt sertifikaları (`.loc` gibi local domain'lerde çalışmaz)

---

## Varsayılan Proje Ayarları

Yeni projeler için varsayılan değerler.

```bash
DEFAULT_PHP_VERSION=8.2
DEFAULT_WEBSERVER=nginx
DEFAULT_DOCUMENT_ROOT=public
```

---

## Stackvo UI Ayarları

Web UI performans ve davranış ayarları.

```bash
DEFAULT_TIMEOUT=30
SYSTEM_COMMAND_TIMEOUT=120
CACHE_ENABLE=true
CACHE_TTL=5
LOG_ENABLE=true
LOG_LEVEL=DEBUG
```

---

## Docker Network

Docker network ve container ayarları.

```bash
DOCKER_DEFAULT_NETWORK=stackvo-net
DOCKER_PRUNE_ON_REBUILD=false
DOCKER_FORCE_RECREATE=true
DOCKER_REMOVE_ORPHANS=true
```

---

## Host System Mappings

Host sistem ile container mapping'leri.

```bash
HOST_USER_ID=1000
HOST_GROUP_ID=1000
HOST_TIMEZONE=Europe/Istanbul
```

---

## Security Settings

Güvenlik konfigürasyonları.

```bash
ALLOW_HTTPD=true
ALLOW_NGINX=true
ALLOWED_PHP_VERSIONS=7.4,8.0,8.1,8.2,8.3,8.4
```

---

## Port Mappings

Host port yönlendirmeleri.

```bash
HOST_PORT_POSTGRES=5433
HOST_PORT_PERCONA=3308
HOST_PORT_ADMINER=8082
HOST_PORT_KAFKA=9094
HOST_PORT_TOMCAT=8081
```

---

## CLI Behavior

CLI davranış ayarları.

```bash
STACKVO_VERBOSE=false
STACKVO_STRICT=true
STACKVO_SHOW_BANNER=true
STACKVO_DRY_RUN=false
STACKVO_VERSION=1.0.0
STACKVO_GENERATE_LOG=core/generator.log
```

---

## Supported Languages

Stackvo 6 programlama dilini destekler: **PHP, Python, Go, Ruby, Rust, Node.js**

```bash
SUPPORTED_LANGUAGES=php,python,go,ruby,rust,nodejs
```

Desteklenen versiyonlar ve detaylar için [Giriş](../guides/projects.md#coklu-dil-destegi) sayfasına bakın.

---

## Stackvo Web UI Tools

Yönetim araçları konfigürasyonu:

| Tool | Enable | Version | URL | Erişim |
|------|--------|---------|-----|--------|
| **Adminer** | `TOOLS_ADMINER_ENABLE=true` | `4.8.1` | `adminer` | `https://adminer.stackvo.loc` |
| **PhpMyAdmin** | `TOOLS_PHPMYADMIN_ENABLE=true` | `5.2.1` | `phpmyadmin` | `https://phpmyadmin.stackvo.loc` |
| **PhpPgAdmin** | `TOOLS_PHPPGADMIN_ENABLE=true` | `7.13.0` | `phppgadmin` | `https://phppgadmin.stackvo.loc` |
| **PhpMongo** | `TOOLS_PHPMONGO_ENABLE=true` | `1.3.3` | `phpmongo` | `https://phpmongo.stackvo.loc` |
| **PhpMemcachedAdmin** | `TOOLS_PHPMEMCACHEDADMIN_ENABLE=true` | `1.3.0` | `phpmemcachedadmin` | `https://phpmemcachedadmin.stackvo.loc` |
| **OpCacheGUI** | `TOOLS_OPCACHE_ENABLE=true` | `3.6.0` | `opcache` | `https://opcache.stackvo.loc` |
| **Kafbat** | `TOOLS_KAFBAT_ENABLE=true` | `1.4.2` | `kafbat` | `https://kafbat.stackvo.loc` |

---

## Services

40+ servis konfigürasyonu. Her servis için `.env` dosyasında `SERVICE_*_ENABLE`, `SERVICE_*_VERSION` ve servise özel parametreler bulunur.

```bash
# MySQL
SERVICE_MYSQL_ENABLE=true
SERVICE_MYSQL_VERSION=8.0
SERVICE_MYSQL_ROOT_PASSWORD=root
SERVICE_MYSQL_DATABASE=stackvo
SERVICE_MYSQL_USER=stackvo
SERVICE_MYSQL_PASSWORD=stackvo

# Redis
SERVICE_REDIS_ENABLE=true
SERVICE_REDIS_VERSION=7.0
SERVICE_REDIS_PASSWORD=

# RabbitMQ
SERVICE_RABBITMQ_ENABLE=true
SERVICE_RABBITMQ_VERSION=3
SERVICE_RABBITMQ_DEFAULT_USER=admin
SERVICE_RABBITMQ_DEFAULT_PASS=admin
```

**Detaylı servis konfigürasyonları ve bağlantı bilgileri için [Servisler Referansı](../references/services.md) sayfasına bakın.**