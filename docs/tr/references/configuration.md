# Konfigürasyon Referansı

.env dosyası ve tüm konfigürasyon seçeneklerinin detaylı referansı. Bu sayfa, 364 satırlık .env dosyasının 11 ana bölümünü (Traefik, varsayılan proje ayarları, Stackvo UI, Docker network, host system mappings, security, port mappings, CLI behavior, desteklenen diller, tools ve servisler) detaylı olarak açıklamaktadır. Her parametre için tip, varsayılan değer ve açıklama verilmektedir.

## .env Dosyası

Stackvo'un ana konfigürasyon dosyası. 364 satır, 11 ana bölüm.

---

## Traefik Ayarları

### DEFAULT_TLD_SUFFIX

**Tip:** String  
**Varsayılan:** `stackvo.loc`  
**Açıklama:** Tüm servislerin domain suffix'i

```bash
DEFAULT_TLD_SUFFIX=stackvo.loc
```

### SSL_ENABLE

**Tip:** Boolean  
**Varsayılan:** `true`  
**Açıklama:** SSL/TLS desteğini aktifleştirir

```bash
SSL_ENABLE=true
```

### REDIRECT_TO_HTTPS

**Tip:** Boolean  
**Varsayılan:** `true`  
**Açıklama:** HTTP isteklerini HTTPS'e yönlendirir

```bash
REDIRECT_TO_HTTPS=true
```

### LETSENCRYPT_ENABLE

**Tip:** Boolean  
**Varsayılan:** `false`  
**Açıklama:** Let's Encrypt sertifikaları (production için)

```bash
LETSENCRYPT_ENABLE=false
LETSENCRYPT_EMAIL=admin@stackvo.loc
```

### TRAEFIK_URL

**Tip:** String  
**Varsayılan:** `traefik`  
**Açıklama:** Traefik dashboard subdomain'i

```bash
TRAEFIK_URL=traefik
# Erişim: https://traefik.stackvo.loc
```

---

## Varsayılan Proje Ayarları

### DEFAULT_PHP_VERSION

**Tip:** String  
**Varsayılan:** `8.2`  
**Geçerli Değerler:** 5.6, 7.0-7.4, 8.0-8.5  
**Açıklama:** Yeni projeler için varsayılan PHP versiyonu

```bash
DEFAULT_PHP_VERSION=8.2
```

### DEFAULT_WEBSERVER

**Tip:** String  
**Varsayılan:** `nginx`  
**Geçerli Değerler:** nginx, apache, caddy, ferron  
**Açıklama:** Yeni projeler için varsayılan webserver

```bash
DEFAULT_WEBSERVER=nginx
```

### DEFAULT_DOCUMENT_ROOT

**Tip:** String  
**Varsayılan:** `public`  
**Açıklama:** Yeni projeler için varsayılan document root

```bash
DEFAULT_DOCUMENT_ROOT=public
```

---

## Stackvo UI Ayarları

### DEFAULT_TIMEOUT

**Tip:** Integer  
**Varsayılan:** `30`  
**Birim:** Saniye  
**Açıklama:** API request timeout

```bash
DEFAULT_TIMEOUT=30
```

### SYSTEM_COMMAND_TIMEOUT

**Tip:** Integer  
**Varsayılan:** `120`  
**Birim:** Saniye  
**Açıklama:** Sistem komutları timeout

```bash
SYSTEM_COMMAND_TIMEOUT=120
```

### CACHE_ENABLE

**Tip:** Boolean  
**Varsayılan:** `true`  
**Açıklama:** UI cache'i aktifleştirir

```bash
CACHE_ENABLE=true
CACHE_TTL=5
```

### LOG_ENABLE

**Tip:** Boolean  
**Varsayılan:** `true`  
**Açıklama:** Loglama aktifleştirir

```bash
LOG_ENABLE=true
LOG_LEVEL=DEBUG
```

---

## Docker Network

### DOCKER_DEFAULT_NETWORK

**Tip:** String  
**Varsayılan:** `stackvo-net`  
**Açıklama:** Docker network adı

```bash
DOCKER_DEFAULT_NETWORK=stackvo-net
```

### DOCKER_NETWORK_SUBNET

**Tip:** String (CIDR)  
**Varsayılan:** `172.30.0.0/16`  
**Açıklama:** Docker network subnet

```bash
DOCKER_NETWORK_SUBNET=172.30.0.0/16
```

### DOCKER_PRUNE_ON_REBUILD

**Tip:** Boolean  
**Varsayılan:** `false`  
**Açıklama:** Rebuild'de prune çalıştır

```bash
DOCKER_PRUNE_ON_REBUILD=false
```

### DOCKER_FORCE_RECREATE

**Tip:** Boolean  
**Varsayılan:** `true`  
**Açıklama:** Container'ları force recreate

```bash
DOCKER_FORCE_RECREATE=true
```

### DOCKER_REMOVE_ORPHANS

**Tip:** Boolean  
**Varsayılan:** `true`  
**Açıklama:** Orphan container'ları kaldır

```bash
DOCKER_REMOVE_ORPHANS=true
```

---

## Host System Mappings

### HOST_USER_ID

**Tip:** Integer  
**Varsayılan:** `1000`  
**Açıklama:** Host user ID

```bash
HOST_USER_ID=1000
```

### HOST_GROUP_ID

**Tip:** Integer  
**Varsayılan:** `1000`  
**Açıklama:** Host group ID

```bash
HOST_GROUP_ID=1000
```

### HOST_TIMEZONE

**Tip:** String  
**Varsayılan:** `Europe/Istanbul`  
**Açıklama:** Timezone

```bash
HOST_TIMEZONE=Europe/Istanbul
```

---

## Security Settings

### ALLOW_HTTPD

**Tip:** Boolean  
**Varsayılan:** `true`  
**Açıklama:** Apache kullanımına izin ver

```bash
ALLOW_HTTPD=true
```

### ALLOW_NGINX

**Tip:** Boolean  
**Varsayılan:** `true`  
**Açıklama:** Nginx kullanımına izin ver

```bash
ALLOW_NGINX=true
```

### ALLOWED_PHP_VERSIONS

**Tip:** String (comma-separated)  
**Varsayılan:** `7.4,8.0,8.1,8.2,8.3,8.4`  
**Açıklama:** İzin verilen PHP versiyonları

```bash
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

### STACKVO_VERBOSE

**Tip:** Boolean  
**Varsayılan:** `false`  
**Açıklama:** Detaylı çıktı

```bash
STACKVO_VERBOSE=false
```

### STACKVO_STRICT

**Tip:** Boolean  
**Varsayılan:** `true`  
**Açıklama:** Strict mode

```bash
STACKVO_STRICT=true
```

### STACKVO_SHOW_BANNER

**Tip:** Boolean  
**Varsayılan:** `true`  
**Açıklama:** Banner göster

```bash
STACKVO_SHOW_BANNER=true
```

### STACKVO_DRY_RUN

**Tip:** Boolean  
**Varsayılan:** `false`  
**Açıklama:** Dry run mode

```bash
STACKVO_DRY_RUN=false
```

### STACKVO_VERSION

**Tip:** String  
**Varsayılan:** `1.0.0`  
**Açıklama:** Stackvo versiyonu

```bash
STACKVO_VERSION=1.0.0
```

### STACKVO_GENERATE_LOG

**Tip:** String  
**Varsayılan:** `core/generator.log`  
**Açıklama:** Generator log dosyası

```bash
STACKVO_GENERATE_LOG=core/generator.log
```

---

## Supported Languages

### SUPPORTED_LANGUAGES

**Tip:** String (comma-separated)  
**Varsayılan:** `php,python,go,ruby,rust,nodejs`  
**Açıklama:** Desteklenen diller

```bash
SUPPORTED_LANGUAGES=php,python,go,ruby,rust,nodejs
```

Her dil için:
- `SUPPORTED_LANGUAGES_{LANG}_VERSIONS` - Versiyonlar
- `SUPPORTED_LANGUAGES_{LANG}_DEFAULT` - Varsayılan versiyon
- `SUPPORTED_LANGUAGES_PHP_EXTENSIONS` - PHP extension'ları (sadece PHP için)

---

## Services

Her servis için 3 temel ayar:

### SERVICE_{NAME}_ENABLE

**Tip:** Boolean  
**Açıklama:** Servisi aktifleştirir

```bash
SERVICE_MYSQL_ENABLE=true
```

### SERVICE_{NAME}_VERSION

**Tip:** String  
**Açıklama:** Servis versiyonu

```bash
SERVICE_MYSQL_VERSION=8.0
```

### SERVICE_{NAME}_URL

**Tip:** String  
**Açıklama:** Servis subdomain'i

```bash
SERVICE_RABBITMQ_URL=rabbitmq
# Erişim: https://rabbitmq.stackvo.loc
```

Servis-specific ayarlar için [Servisler Referansı](services.md) sayfasına bakın.

---

