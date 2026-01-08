---
title: En İyi Uygulamalar
description: Production ortamında Stackvo kullanımı için en iyi uygulamalar ve tavsiyeler.
---

# En İyi Uygulamalar

Production ortamında Stackvo kullanımı için en iyi uygulamalar ve tavsiyeler. Bu kılavuz, environment separation, version control, secrets management gibi genel ilkelerden Docker resource limits, health checks, logging gibi Docker best practices'e, güvenlik, performans, monitoring, deployment ve maintenance konularında profesyonel yaklaşımları detaylı olarak açıklamaktadır.

---

## Genel İlkeler

### 1. Environment Separation

Development, staging ve production ortamları için farklı `.env` dosyaları kullanın.

```bash
# Development
.env

# Staging
.env.staging

# Production
.env.production
```

**Kullanım:**
```bash
# Staging için
cp .env .env.staging
nano .env.staging

# Generate
ENV_FILE=.env.staging ./stackvo.sh generate
```

### 2. Version Control

`.env` dosyasını version control'e eklemeyin, sadece `.env.example` ekleyin.

```bash
# .gitignore
.env
.env.local
.env.*.local

# Version control'e ekle
.env.example
```

### 3. Secrets Management

Hassas bilgileri environment variable'larda saklayın.

```bash
# .env
SERVICE_MYSQL_ROOT_PASSWORD=${MYSQL_ROOT_PASSWORD}
SERVICE_RABBITMQ_DEFAULT_PASS=${RABBITMQ_PASSWORD}
```

---

## Docker Best Practices

### 1. Resource Limits

Container'lar için resource limitleri belirleyin.

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

Container'lar için health check tanımlayın.

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

Container loglarını düzenli olarak temizleyin.

```bash
# Log rotation
docker run --log-driver json-file \
  --log-opt max-size=10m \
  --log-opt max-file=3 \
  myimage
```

### 4. Volume Backup

Volume'ları düzenli olarak yedekleyin.

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

### 1. Güçlü Şifreler

Tüm servisler için güçlü şifreler kullanın.

```bash
# .env
SERVICE_MYSQL_ROOT_PASSWORD=$(openssl rand -base64 32)
SERVICE_RABBITMQ_DEFAULT_PASS=$(openssl rand -base64 32)
```

### 2. Network İzolasyonu

Sadece gerekli portları expose edin.

```yaml
services:
  mysql:
    ports:
      - "127.0.0.1:3306:3306"  # Sadece localhost'tan erişim
```

### 3. SSL/TLS

Production'da her zaman SSL/TLS kullanın.

```bash
# .env
SSL_ENABLE=true
REDIRECT_TO_HTTPS=true
LETSENCRYPT_ENABLE=true
LETSENCRYPT_EMAIL=admin@yourdomain.com
```

### 4. Firewall

Gereksiz portları kapatın.

```bash
# UFW ile
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw deny 3306/tcp  # MySQL'i dışarıdan kapat
sudo ufw enable
```

---

## Performance Best Practices

### 1. OPcache

PHP OPcache'i production'da aktif edin.

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

Sık kullanılan verileri Redis'te cache'leyin.

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

Veritabanı sorgularını optimize edin.

```sql
-- Sık kullanılan kolonlara index ekle
CREATE INDEX idx_email ON users(email);
CREATE INDEX idx_created_at ON posts(created_at);

-- Slow query log
SET GLOBAL slow_query_log = 'ON';
SET GLOBAL long_query_time = 2;
```

### 4. Connection Pooling

Database connection pooling kullanın.

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

Container'ları düzenli olarak izleyin.

```bash
# Resource kullanımı
docker stats

# Health check
docker ps --filter "health=unhealthy"

# Logs
docker logs -f --tail=100 stackvo-mysql
```

### 2. Grafana Dashboard

Grafana ile görsel monitoring kurun.

```bash
# .env
SERVICE_GRAFANA_ENABLE=true

# Prometheus data source ekle
# Grafana'da dashboard oluştur
```

### 3. Alerting

Kritik durumlar için alert kurun.

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

Rolling update ile zero downtime deployment.

```bash
# Yeni versiyonu pull et
docker compose pull

# Rolling update
docker compose up -d --no-deps --scale web=2 web
docker compose up -d --no-deps --scale web=1 --remove-orphans web
```

### 2. Blue-Green Deployment

İki ortam arasında geçiş yapın.

```bash
# Blue environment
docker compose -f docker-compose.blue.yml up -d

# Test et
curl https://blue.example.com

# Green'e geç
docker compose -f docker-compose.green.yml up -d

# Traefik'te route değiştir
# Blue'yu kapat
docker compose -f docker-compose.blue.yml down
```

### 3. Database Migrations

Migration'ları dikkatli yapın.

```bash
# Backup al
docker exec stackvo-mysql mysqldump -u root -proot mydb > backup.sql

# Migration çalıştır
docker exec stackvo-laravel-app-php php artisan migrate

# Rollback planı hazır olsun
docker exec stackvo-laravel-app-php php artisan migrate:rollback
```

---

## Maintenance Best Practices

### 1. Regular Updates

Image'ları düzenli güncelleyin.

```bash
# Image'ları güncelle
docker compose pull

# Restart
docker compose up -d
```

### 2. Cleanup

Kullanılmayan kaynakları temizleyin.

```bash
# Unused images
docker image prune -a

# Unused volumes
docker volume prune

# Unused networks
docker network prune

# Tümü
docker system prune -a --volumes
```

### 3. Log Rotation

Logları düzenli temizleyin.

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

Development için ayrı konfigürasyon.

```bash
# .env.local
SERVICE_MYSQL_VERSION=8.0
DEFAULT_PHP_VERSION=8.3
STACKVO_VERBOSE=true
```

### 2. Hot Reload

Code değişikliklerini otomatik yansıtın.

```yaml
# docker-compose.override.yml
services:
  web:
    volumes:
      - ./projects/myproject:/var/www/html:cached
```

### 3. Debugging

Xdebug ile debugging.

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

Yapılandırılmış log formatı kullanın.

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

Sentry gibi error tracking servisi kullanın.

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

Blackfire ile profiling yapın.

```bash
# .env
SERVICE_BLACKFIRE_ENABLE=true

# Profiling
blackfire curl https://myproject.loc
```