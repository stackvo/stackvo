# Global Configuration

Global configuration is managed via the `.env` file and affects the entire Stackvo system. This page detailedly explains all sections of the 364-line `.env` file, from Traefik settings to service configurations, from Docker network settings to security parameters. Global settings determine default values for all projects and services.

---

## .env File

The `.env` file is Stackvo's main configuration file. It consists of 364 lines and 11 main sections.

### File Structure

```bash
# 1. Traefik Settings (~20 lines)
# 2. Default Project Settings (~5 lines)
# 3. Stackvo UI Settings (~10 lines)
# 4. Docker Network (~5 lines)
# 5. Host System Mappings (~5 lines)
# 6. Security Settings (~5 lines)
# 7. Port Mappings (~10 lines)
# 8. CLI Behavior (~10 lines)
# 9. Supported Languages (~30 lines)
# 10. Stackvo Web UI Tools (~40 lines)
# 11. Services (~180 lines)
```

---

## Traefik Settings

Reverse proxy and SSL/TLS configuration.

```bash
# Global domain suffix
DEFAULT_TLD_SUFFIX=stackvo.loc

# SSL/TLS
SSL_ENABLE=true
REDIRECT_TO_HTTPS=true

# Let's Encrypt (only for public domains)
LETSENCRYPT_ENABLE=false
LETSENCRYPT_EMAIL=admin@stackvo.loc

# Traefik subdomain
TRAEFIK_URL=traefik
```

**Explanations:**
- `DEFAULT_TLD_SUFFIX`: Domain suffix for all services (e.g. `mysql.stackvo.loc`)
- `SSL_ENABLE`: Enables SSL/TLS support
- `REDIRECT_TO_HTTPS`: Redirects HTTP requests to HTTPS
- `LETSENCRYPT_ENABLE`: Let's Encrypt certificates (does not work on local domains like `.loc`)

---

## Default Project Settings

Default values for new projects.

```bash
DEFAULT_PHP_VERSION=8.2
DEFAULT_WEBSERVER=nginx
DEFAULT_DOCUMENT_ROOT=public
```

---

## Stackvo UI Settings

Web UI performance and behavior settings.

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

Docker network and container settings.

```bash
DOCKER_DEFAULT_NETWORK=stackvo-net
DOCKER_PRUNE_ON_REBUILD=false
DOCKER_FORCE_RECREATE=true
DOCKER_REMOVE_ORPHANS=true
```

---

## Host System Mappings

Host system mappings with container.

```bash
HOST_USER_ID=1000
HOST_GROUP_ID=1000
HOST_TIMEZONE=Europe/Istanbul
```

---

## Security Settings

Security configurations.

```bash
ALLOW_HTTPD=true
ALLOW_NGINX=true
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

CLI behavior settings.

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

Stackvo supports 6 programming languages: **PHP, Python, Go, Ruby, Rust, Node.js**

```bash
SUPPORTED_LANGUAGES=php,python,go,ruby,rust,nodejs
```

See [Introduction](../guides/projects.md#multi-language-support) page for supported versions and details.

---

## Stackvo Web UI Tools

Management tools configuration:

| Tool | Enable | Version | URL | Access |
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

40+ service configuration. Each service has `SERVICE_*_ENABLE`, `SERVICE_*_VERSION` and service-specific parameters in the `.env` file.

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

**See [Services Reference](../references/services.md) page for detailed service configurations and connection information.**