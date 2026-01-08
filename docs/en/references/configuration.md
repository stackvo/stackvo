# Configuration Reference

Detailed reference for .env file and all configuration options. This page detailedly explains the 11 main sections of the 364-line .env file (Traefik, default project settings, Stackvo UI, Docker network, host system mappings, security, port mappings, CLI behavior, supported languages, tools, and services). Type, default value and description are provided for each parameter.

## .env File

Main configuration file of Stackvo. 364 lines, 11 main sections.

---

## Traefik Settings

### DEFAULT_TLD_SUFFIX

**Type:** String  
**Default:** `stackvo.loc`  
**Description:** Domain suffix for all services

```bash
DEFAULT_TLD_SUFFIX=stackvo.loc
```

### SSL_ENABLE

**Type:** Boolean  
**Default:** `true`  
**Description:** Enables SSL/TLS support

```bash
SSL_ENABLE=true
```

### REDIRECT_TO_HTTPS

**Type:** Boolean  
**Default:** `true`  
**Description:** Redirects HTTP requests to HTTPS

```bash
REDIRECT_TO_HTTPS=true
```

### LETSENCRYPT_ENABLE

**Type:** Boolean  
**Default:** `false`  
**Description:** Let's Encrypt certificates (for production)

```bash
LETSENCRYPT_ENABLE=false
LETSENCRYPT_EMAIL=admin@stackvo.loc
```

### TRAEFIK_URL

**Type:** String  
**Default:** `traefik`  
**Description:** Traefik dashboard subdomain

```bash
TRAEFIK_URL=traefik
# Access: https://traefik.stackvo.loc
```

---

## Default Project Settings

### DEFAULT_PHP_VERSION

**Type:** String  
**Default:** `8.2`  
**Valid Values:** 5.6, 7.0-7.4, 8.0-8.5  
**Description:** Default PHP version for new projects

```bash
DEFAULT_PHP_VERSION=8.2
```

### DEFAULT_WEBSERVER

**Type:** String  
**Default:** `nginx`  
**Valid Values:** nginx, apache, caddy, ferron  
**Description:** Default webserver for new projects

```bash
DEFAULT_WEBSERVER=nginx
```

### DEFAULT_DOCUMENT_ROOT

**Type:** String  
**Default:** `public`  
**Description:** Default document root for new projects

```bash
DEFAULT_DOCUMENT_ROOT=public
```

---

## Stackvo UI Settings

### DEFAULT_TIMEOUT

**Type:** Integer  
**Default:** `30`  
**Unit:** Seconds  
**Description:** API request timeout

```bash
DEFAULT_TIMEOUT=30
```

### SYSTEM_COMMAND_TIMEOUT

**Type:** Integer  
**Default:** `120`  
**Unit:** Seconds  
**Description:** System commands timeout

```bash
SYSTEM_COMMAND_TIMEOUT=120
```

### CACHE_ENABLE

**Type:** Boolean  
**Default:** `true`  
**Description:** Enables UI cache

```bash
CACHE_ENABLE=true
CACHE_TTL=5
```

### LOG_ENABLE

**Type:** Boolean  
**Default:** `true`  
**Description:** Enables logging

```bash
LOG_ENABLE=true
LOG_LEVEL=DEBUG
```

---

## Docker Network

### DOCKER_DEFAULT_NETWORK

**Type:** String  
**Default:** `stackvo-net`  
**Description:** Docker network name

```bash
DOCKER_DEFAULT_NETWORK=stackvo-net
```

### DOCKER_NETWORK_SUBNET

**Type:** String (CIDR)  
**Default:** `172.30.0.0/16`  
**Description:** Docker network subnet

```bash
DOCKER_NETWORK_SUBNET=172.30.0.0/16
```

### DOCKER_PRUNE_ON_REBUILD

**Type:** Boolean  
**Default:** `false`  
**Description:** Run prune on rebuild

```bash
DOCKER_PRUNE_ON_REBUILD=false
```

### DOCKER_FORCE_RECREATE

**Type:** Boolean  
**Default:** `true`  
**Description:** Force recreate containers

```bash
DOCKER_FORCE_RECREATE=true
```

### DOCKER_REMOVE_ORPHANS

**Type:** Boolean  
**Default:** `true`  
**Description:** Remove orphan containers

```bash
DOCKER_REMOVE_ORPHANS=true
```

---

## Host System Mappings

### HOST_USER_ID

**Type:** Integer  
**Default:** `1000`  
**Description:** Host user ID

```bash
HOST_USER_ID=1000
```

### HOST_GROUP_ID

**Type:** Integer  
**Default:** `1000`  
**Description:** Host group ID

```bash
HOST_GROUP_ID=1000
```

### HOST_TIMEZONE

**Type:** String  
**Default:** `Europe/Istanbul`  
**Description:** Timezone

```bash
HOST_TIMEZONE=Europe/Istanbul
```

---

## Security Settings

### ALLOW_HTTPD

**Type:** Boolean  
**Default:** `true`  
**Description:** Allow Apache usage

```bash
ALLOW_HTTPD=true
```

### ALLOW_NGINX

**Type:** Boolean  
**Default:** `true`  
**Description:** Allow Nginx usage

```bash
ALLOW_NGINX=true
```

### ALLOWED_PHP_VERSIONS

**Type:** String (comma-separated)  
**Default:** `7.4,8.0,8.1,8.2,8.3,8.4`  
**Description:** Allowed PHP versions

```bash
ALLOWED_PHP_VERSIONS=7.4,8.0,8.1,8.2,8.3,8.4
```

---

## Port Mappings

Host port forwardings.

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

**Type:** Boolean  
**Default:** `false`  
**Description:** Verbose output

```bash
STACKVO_VERBOSE=false
```

### STACKVO_STRICT

**Type:** Boolean  
**Default:** `true`  
**Description:** Strict mode

```bash
STACKVO_STRICT=true
```

### STACKVO_SHOW_BANNER

**Type:** Boolean  
**Default:** `true`  
**Description:** Show banner

```bash
STACKVO_SHOW_BANNER=true
```

### STACKVO_DRY_RUN

**Type:** Boolean  
**Default:** `false`  
**Description:** Dry run mode

```bash
STACKVO_DRY_RUN=false
```

### STACKVO_VERSION

**Type:** String  
**Default:** `1.0.0`  
**Description:** Stackvo version

```bash
STACKVO_VERSION=1.0.0
```

### STACKVO_GENERATE_LOG

**Type:** String  
**Default:** `core/generator.log`  
**Description:** Generator log file

```bash
STACKVO_GENERATE_LOG=core/generator.log
```

---

## Supported Languages

### SUPPORTED_LANGUAGES

**Type:** String (comma-separated)  
**Default:** `php,python,go,ruby,rust,nodejs`  
**Description:** Supported languages

```bash
SUPPORTED_LANGUAGES=php,python,go,ruby,rust,nodejs
```

For each language:
- `SUPPORTED_LANGUAGES_{LANG}_VERSIONS` - Versions
- `SUPPORTED_LANGUAGES_{LANG}_DEFAULT` - Default version
- `SUPPORTED_LANGUAGES_PHP_EXTENSIONS` - PHP extensions (only for PHP)

---

## Services

3 basic settings for each service:

### SERVICE_{NAME}_ENABLE

**Type:** Boolean  
**Description:** Enables the service

```bash
SERVICE_MYSQL_ENABLE=true
```

### SERVICE_{NAME}_VERSION

**Type:** String  
**Description:** Service version

```bash
SERVICE_MYSQL_VERSION=8.0
```

### SERVICE_{NAME}_URL

**Type:** String  
**Description:** Service subdomain

```bash
SERVICE_RABBITMQ_URL=rabbitmq
# Access: https://rabbitmq.stackvo.loc
```

See [Services Reference](services.md) page for service-specific settings.

---
