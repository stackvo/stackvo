---
title: Network
description: Stackvo network mimarisi ve çalışma prensiplerini anlamak için bu bölümü inceleyin.
---

# Network

Stackvo, tüm container'ları tek bir Docker network üzerinde çalıştırır: `stackvo-net`. Bu sayfa, 172.30.0.0/16 subnet'ındaki bridge network'ün nasıl çalıştığını, container'lar arası hostname bazlı iletişimi, port mapping'ı, network izolasyonunu ve troubleshooting yöntemlerini detaylı olarak açıklamaktadır. Tüm servisler ve projeler aynı network üzerinde kolay iletişim kurar.

---

## stackvo-net

**Tip:** Bridge  
**Subnet:** 172.30.0.0/16  
**Gateway:** 172.30.0.1

### Network Tanımı

```yaml
networks:
  stackvo-net:
    driver: bridge
    ipam:
      config:
        - subnet: 172.30.0.0/16
```

---

## Network Mimarisi

```
stackvo-net (172.30.0.0/16)
├── 172.30.0.1 (Gateway)
│
├── Traefik (Reverse Proxy)
│   └── Ports: 80, 443, 8080
│
├── Infrastructure Services
│   ├── MySQL (stackvo-mysql:3306)
│   ├── MariaDB (stackvo-mariadb:3306)
│   ├── PostgreSQL (stackvo-postgres:5432)
│   ├── MongoDB (stackvo-mongo:27017)
│   ├── Redis (stackvo-redis:6379)
│   ├── Memcached (stackvo-memcached:11211)
│   ├── RabbitMQ (stackvo-rabbitmq:5672)
│   ├── Kafka (stackvo-kafka:9092)
│   ├── Elasticsearch (stackvo-elasticsearch:9200)
│   └── ... (diğer servisler)
│
├── Stackvo UI
│   ├── stackvo-ui (Web UI)
│   └── stackvo-tools (Management Tools)
│
└── User Projects
    ├── Project1
    │   ├── stackvo-project1-php:9000
    │   └── stackvo-project1-web:80
    ├── Project2
    │   ├── stackvo-project2-php:9000
    │   └── stackvo-project2-web:80
    └── ...
```

---

## Container İletişimi

### Hostname Bazlı İletişim

Container'lar birbirlerini hostname ile bulabilir:

```php
<?php
// PHP'den MySQL'e bağlantı
$host = 'stackvo-mysql';  // Container hostname
$port = 3306;

$pdo = new PDO("mysql:host=$host;port=$port;dbname=stackvo", 'stackvo', 'stackvo');
```

```php
<?php
// PHP'den Redis'e bağlantı
$redis = new Redis();
$redis->connect('stackvo-redis', 6379);
```

```php
<?php
// PHP'den RabbitMQ'ya bağlantı
use PhpAmqpLib\Connection\AMQPStreamConnection;

$connection = new AMQPStreamConnection(
    'stackvo-rabbitmq',  // Hostname
    5672,                   // Port
    'admin',                // User
    'admin'                 // Password
);
```

### Port Mapping

Container'lar arasında iletişim için **internal port** kullanılır:

| Servis | Container Hostname | Internal Port | Host Port |
|--------|-------------------|---------------|-----------|
| MySQL | stackvo-mysql | 3306 | 3306 |
| PostgreSQL | stackvo-postgres | 5432 | 5433 |
| MongoDB | stackvo-mongo | 27017 | 27017 |
| Redis | stackvo-redis | 6379 | 6379 |
| RabbitMQ | stackvo-rabbitmq | 5672 | 5672 |
| Kafka | stackvo-kafka | 9092 | 9094 |

**Not:** Container'lar arası iletişimde **internal port** kullanılır, host'tan erişimde **host port** kullanılır.

---

## İletişim Akışı

### External → Application

```
1. Browser/Client
   ↓ HTTPS (443)
2. Traefik (Reverse Proxy)
   ↓ HTTP (80)
3. Nginx/Apache/Caddy/Ferron (Webserver)
   ↓ FastCGI (9000)
4. PHP-FPM
```

### Application → Services

```
PHP-FPM
├─→ MySQL (stackvo-mysql:3306)
├─→ PostgreSQL (stackvo-postgres:5432)
├─→ MongoDB (stackvo-mongo:27017)
├─→ Redis (stackvo-redis:6379)
├─→ Memcached (stackvo-memcached:11211)
├─→ RabbitMQ (stackvo-rabbitmq:5672)
├─→ Kafka (stackvo-kafka:9092)
└─→ Elasticsearch (stackvo-elasticsearch:9200)
```

---

## Network İzolasyonu

### Avantajlar

1. **Güvenlik:** Container'lar izole ortamda çalışır
2. **Kolay Servis Keşfi:** Hostname bazlı iletişim
3. **Port Çakışması Yok:** Her container kendi portunu kullanır
4. **Basit Yönetim:** Tek network, kolay troubleshooting

### Dış Dünyaya Erişim

Container'lar internet erişimine sahiptir:

```bash
# Container içinden
docker exec -it stackvo-mysql ping google.com
docker exec -it stackvo-php curl https://api.example.com
```

---

## Network Konfigürasyonu

### Subnet Değiştirme

`.env` dosyasında subnet değiştirilebilir:

```bash
DOCKER_NETWORK_SUBNET=172.30.0.0/16
```

**Not:** Subnet değişikliği için network'ü yeniden oluşturmanız gerekir:

```bash
./core/cli/stackvo.sh down
docker network rm stackvo-net
./core/cli/stackvo.sh generate
./core/cli/stackvo.sh up
```

### Custom Network

Özel network ayarları için `core/compose/base.yml` template'ini düzenleyin:

```yaml
networks:
  stackvo-net:
    driver: bridge
    ipam:
      driver: default
      config:
        - subnet: 172.30.0.0/16
          gateway: 172.30.0.1
    driver_opts:
      com.docker.network.bridge.name: stackvo-br0
      com.docker.network.bridge.enable_ip_masquerade: "true"
```

---

## Network Troubleshooting

### Container'lar Birbirini Görmüyor

```bash
# Network'ü kontrol et
docker network inspect stackvo-net

# Container'ın network'e bağlı olduğunu doğrula
docker inspect stackvo-mysql | grep -A 10 Networks

# Ping testi
docker exec stackvo-php ping stackvo-mysql
```

### DNS Çözümleme Sorunu

```bash
# Container içinden DNS testi
docker exec stackvo-php nslookup stackvo-mysql
docker exec stackvo-php cat /etc/resolv.conf
```

### Network Connectivity

```bash
# Container'dan servis erişimi testi
docker exec stackvo-php nc -zv stackvo-mysql 3306
docker exec stackvo-php nc -zv stackvo-redis 6379
```

### Network Yeniden Oluşturma

```bash
# Tüm container'ları durdur
./core/cli/stackvo.sh down

# Network'ü sil
docker network rm stackvo-net

# Yeniden oluştur
./core/cli/stackvo.sh generate
./core/cli/stackvo.sh up
```

---

## Network Monitoring

### Aktif Bağlantılar

```bash
# Network'teki tüm container'ları listele
docker network inspect stackvo-net --format '{{range .Containers}}{{.Name}} - {{.IPv4Address}}{{"\n"}}{{end}}'
```

### Network İstatistikleri

```bash
# Container network istatistikleri
docker stats --no-stream --format "table {{.Container}}\t{{.NetIO}}"
```

### Traffic Monitoring

```bash
# tcpdump ile network trafiği izleme
docker run --rm --net=container:stackvo-mysql nicolaka/netshoot tcpdump -i any port 3306
```

---

## Best Practices

### 1. Hostname Kullanımı

❌ **Yanlış:**
```php
$host = '172.30.0.5';  // IP adresi kullanma
```

✅ **Doğru:**
```php
$host = 'stackvo-mysql';  // Hostname kullan
```

### 2. Internal Port Kullanımı

❌ **Yanlış:**
```php
$port = 5433;  // Host port
```

✅ **Doğru:**
```php
$port = 5432;  // Internal port
```

### 3. Connection String

✅ **Doğru:**
```php
// MySQL
$dsn = 'mysql:host=stackvo-mysql;port=3306;dbname=stackvo';

// PostgreSQL
$dsn = 'pgsql:host=stackvo-postgres;port=5432;dbname=stackvo';

// MongoDB
$uri = 'mongodb://root:root@stackvo-mongo:27017/stackvo?authSource=admin';

// Redis
$redis->connect('stackvo-redis', 6379);
```