---
title: Best Practices
description: Best practices and recommendations for using Stackvo in production environment.
---

# Best Practices

Best practices and recommendations for using Stackvo in production environment. This guide detailedly explains professional approaches on topics from general principles like environment separation, version control, secrets management to Docker best practices like resource limits, health checks, logging, and security, performance, monitoring, deployment, and maintenance.

---

## General Principles

### 1. Environment Separation

Use different `.env` files for development, staging, and production environments.

```bash
# Development
.env

# Staging
.env.staging

# Production
.env.production
```

**Usage:**
```bash
# For Staging
cp .env .env.staging
nano .env.staging

# Generate
ENV_FILE=.env.staging ./stackvo.sh generate
```

### 2. Version Control

Do not add `.env` file to version control, only add `.env.example`.

```bash
# .gitignore
.env
.env.local
.env.*.local

# Add to version control
.env.example
```

### 3. Secrets Management

Store sensitive information in environment variables.

```bash
# .env
SERVICE_MYSQL_ROOT_PASSWORD=${MYSQL_ROOT_PASSWORD}
SERVICE_RABBITMQ_DEFAULT_PASS=${RABBITMQ_PASSWORD}
```

---

## Docker Best Practices

### 1. Resource Limits

Set resource limits for containers.

```yaml
# docker-compose.yml
services:
  mysql:
    image: mysql:8.0
    deploy:
      resources:
        limits:
          cpus: '2.0'
          memory: 2G
        reservations:
          cpus: '1.0'
          memory: 1G
```

### 2. Health Checks

Define health checks for containers.

```yaml
services:
  mysql:
    image: mysql:8.0
    healthcheck:
      test: ["CMD", "mysqladmin", "ping", "-h", "localhost"]
      interval: 10s
      timeout: 5s
      retries: 5
```

### 3. Logging

Clean container logs regularly.

```bash
# Log rotation
docker run --log-driver json-file \
  --log-opt max-size=10m \
  --log-opt max-file=3 \
  myimage
```

### 4. Volume Backup

Backup volumes regularly.

```bash
# MySQL backup
docker exec stackvo-mysql mysqldump -u root -proot --all-databases > backup-$(date +%Y%m%d).sql

# Volume backup
docker run --rm \
  -v stackvo_mysql-data:/data \
  -v $(pwd):/backup \
  ubuntu tar czf /backup/mysql-data-$(date +%Y%m%d).tar.gz /data
```

---

## Security Best Practices

### 1. Strong Passwords

Use strong passwords for all services.

```bash
# .env
SERVICE_MYSQL_ROOT_PASSWORD=$(openssl rand -base64 32)
SERVICE_RABBITMQ_DEFAULT_PASS=$(openssl rand -base64 32)
```

### 2. Network Isolation

Expose only necessary ports.

```yaml
services:
  mysql:
    ports:
      - "127.0.0.1:3306:3306"  # Access only from localhost
```

### 3. SSL/TLS

Always use SSL/TLS in production.

```bash
# .env
SSL_ENABLE=true
REDIRECT_TO_HTTPS=true
LETSENCRYPT_ENABLE=true
LETSENCRYPT_EMAIL=admin@yourdomain.com
```

### 4. Firewall

Close unnecessary ports.

```bash
# With UFW
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw deny 3306/tcp  # Close MySQL to outside
sudo ufw enable
```

---

## Performance Best Practices

### 1. OPcache

Enable PHP OPcache in production.

```ini
; php.ini
opcache.enable=1
opcache.memory_consumption=256
opcache.interned_strings_buffer=16
opcache.max_accelerated_files=20000
opcache.revalidate_freq=0
opcache.validate_timestamps=0
opcache.fast_shutdown=1
```

### 2. Redis Cache

Cache frequently used data in Redis.

```php
<?php
$redis = new Redis();
$redis->connect('stackvo-redis', 6379);

$cacheKey = 'user:123';
$user = $redis->get($cacheKey);

if (!$user) {
    $user = $db->query("SELECT * FROM users WHERE id = 123")->fetch();
    $redis->setex($cacheKey, 3600, json_encode($user));
}
```

### 3. Database Indexing

Optimize database queries.

```sql
-- Add index to frequently used columns
CREATE INDEX idx_email ON users(email);
CREATE INDEX idx_created_at ON posts(created_at);

-- Slow query log
SET GLOBAL slow_query_log = 'ON';
SET GLOBAL long_query_time = 2;
```

### 4. Connection Pooling

Use database connection pooling.

```php
<?php
// PDO persistent connection
$pdo = new PDO(
    'mysql:host=stackvo-mysql;dbname=mydb',
    'user',
    'pass',
    [PDO::ATTR_PERSISTENT => true]
);
```

---

## Monitoring Best Practices

### 1. Container Monitoring

Monitor containers regularly.

```bash
# Resource usage
docker stats

# Health check
docker ps --filter "health=unhealthy"

# Logs
docker logs -f --tail=100 stackvo-mysql
```

### 2. Grafana Dashboard

Set up visual monitoring with Grafana.

```bash
# .env
SERVICE_GRAFANA_ENABLE=true

# Add Prometheus data source
# Create dashboard in Grafana
```

### 3. Alerting

Set up alerts for critical situations.

```yaml
# Prometheus alert rules
groups:
  - name: containers
    rules:
      - alert: ContainerDown
        expr: up == 0
        for: 5m
        annotations:
          summary: "Container {{ $labels.instance }} is down"
```

---

## Deployment Best Practices

### 1. Zero Downtime Deployment

Zero downtime deployment with rolling update.

```bash
# Pull new version
docker compose pull

# Rolling update
docker compose up -d --no-deps --scale web=2 web
docker compose up -d --no-deps --scale web=1 --remove-orphans web
```

### 2. Blue-Green Deployment

Switch between two environments.

```bash
# Blue environment
docker compose -f docker-compose.blue.yml up -d

# Test
curl https://blue.example.com

# Switch to Green
docker compose -f docker-compose.green.yml up -d

# Switch route in Traefik
# Close Blue
docker compose -f docker-compose.blue.yml down
```

### 3. Database Migrations

Perform migrations carefully.

```bash
# Backup
docker exec stackvo-mysql mysqldump -u root -proot mydb > backup.sql

# Run migration
docker exec stackvo-laravel-app-php php artisan migrate

# Have a rollback plan ready
docker exec stackvo-laravel-app-php php artisan migrate:rollback
```

---

## Maintenance Best Practices

### 1. Regular Updates

Update images regularly.

```bash
# Update images
docker compose pull

# Restart
docker compose up -d
```

### 2. Cleanup

Clean up unused resources.

```bash
# Unused images
docker image prune -a

# Unused volumes
docker volume prune

# Unused networks
docker network prune

# All
docker system prune -a --volumes
```

### 3. Log Rotation

Clean logs regularly.

```bash
# Docker log cleanup
truncate -s 0 $(docker inspect --format='{{.LogPath}}' stackvo-mysql)

# Logrotate
cat > /etc/logrotate.d/docker-containers <<EOF
/var/lib/docker/containers/*/*.log {
  rotate 7
  daily
  compress
  missingok
  delaycompress
  copytruncate
}
EOF
```

---

## Development Best Practices

### 1. Local Development

Separate configuration for development.

```bash
# .env.local
SERVICE_MYSQL_VERSION=8.0
DEFAULT_PHP_VERSION=8.3
STACKVO_VERBOSE=true
```

### 2. Hot Reload

Automatically reflect code changes.

```yaml
# docker-compose.override.yml
services:
  web:
    volumes:
      - ./projects/myproject:/var/www/html:cached
```

### 3. Debugging

Debugging with Xdebug.

```ini
; php.ini
zend_extension=xdebug.so
xdebug.mode=debug
xdebug.client_host=host.docker.internal
xdebug.client_port=9003
```

---

## Troubleshooting Best Practices

### 1. Structured Logging

Use structured log format.

```php
<?php
error_log(json_encode([
    'level' => 'ERROR',
    'message' => 'Database connection failed',
    'context' => ['host' => 'stackvo-mysql'],
    'timestamp' => date('c')
]));
```

### 2. Error Tracking

Use an error tracking service like Sentry.

```php
<?php
\Sentry\init(['dsn' => 'https://...']);

try {
    // Code
} catch (\Exception $e) {
    \Sentry\captureException($e);
}
```

### 3. Performance Profiling

Perform profiling with Blackfire.

```bash
# .env
SERVICE_BLACKFIRE_ENABLE=true

# Profiling
blackfire curl https://myproject.loc
```