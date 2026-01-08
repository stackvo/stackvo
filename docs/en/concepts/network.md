---
title: Network
description: Examine this section to understand Stackvo network architecture and operating principles.
---

# Network

Stackvo runs all containers on a single Docker network: `stackvo-net`. This page details how the bridge network in the 172.30.0.0/16 subnet works, hostname-based communication between containers, port mapping, network isolation, and troubleshooting methods. All services and projects can easily communicate on the same network.

---

## stackvo-net

**Type:** Bridge
**Subnet:** 172.30.0.0/16
**Gateway:** 172.30.0.1

### Network Definition

```yaml
networks:
  stackvo-net:
    driver: bridge
    ipam:
      config:
        - subnet: 172.30.0.0/16
```

---

## Network Architecture

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
│   └── ... (other services)
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

## Container Communication

### Hostname Based Communication

Containers can find each other via hostname:

```php
<?php
// Connection from PHP to MySQL
$host = 'stackvo-mysql';  // Container hostname
$port = 3306;

$pdo = new PDO("mysql:host=$host;port=$port;dbname=stackvo", 'stackvo', 'stackvo');
```

```php
<?php
// Connection from PHP to Redis
$redis = new Redis();
$redis->connect('stackvo-redis', 6379);
```

```php
<?php
// Connection from PHP to RabbitMQ
use PhpAmqpLib\Connection\AMQPStreamConnection;

$connection = new AMQPStreamConnection(
    'stackvo-rabbitmq',  // Hostname
    5672,                   // Port
    'admin',                // User
    'admin'                 // Password
);
```

### Port Mapping

**Internal port** is used for communication between containers:

| Service | Container Hostname | Internal Port | Host Port |
|--------|-------------------|---------------|-----------|
| MySQL | stackvo-mysql | 3306 | 3306 |
| PostgreSQL | stackvo-postgres | 5432 | 5433 |
| MongoDB | stackvo-mongo | 27017 | 27017 |
| Redis | stackvo-redis | 6379 | 6379 |
| RabbitMQ | stackvo-rabbitmq | 5672 | 5672 |
| Kafka | stackvo-kafka | 9092 | 9094 |

**Note:** **Internal port** is used for communication between containers, **host port** is used for access from host.

---

## Communication Flow

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

## Network Isolation

### Advantages

1. **Security:** Containers run in an isolated environment
2. **Easy Service Discovery:** Hostname based communication
3. **No Port Conflicts:** Each container uses its own port
4. **Simple Management:** Single network, easy troubleshooting

### Access to Outside World

Containers have internet access:

```bash
# Inside container
docker exec -it stackvo-mysql ping google.com
docker exec -it stackvo-php curl https://api.example.com
```

---

## Network Configuration

### Changing Subnet

Subnet can be changed in `.env` file:

```bash
DOCKER_NETWORK_SUBNET=172.30.0.0/16
```

**Note:** You must recreate the network to change the subnet:

```bash
./stackvo.sh down
docker network rm stackvo-net
./stackvo.sh generate
./stackvo.sh up
```

### Custom Network

Edit `core/compose/base.yml` template for custom network settings:

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

### Containers Cannot See Each Other

```bash
# Check network
docker network inspect stackvo-net

# Verify container is connected to network
docker inspect stackvo-mysql | grep -A 10 Networks

# Ping test
docker exec stackvo-php ping stackvo-mysql
```

### DNS Resolution Issue

```bash
# DNS test inside container
docker exec stackvo-php nslookup stackvo-mysql
docker exec stackvo-php cat /etc/resolv.conf
```

### Network Connectivity

```bash
# Service access test from container
docker exec stackvo-php nc -zv stackvo-mysql 3306
docker exec stackvo-php nc -zv stackvo-redis 6379
```

### Recreating Network

```bash
# Stop all containers
./stackvo.sh down

# Remove network
docker network rm stackvo-net

# Recreate
./stackvo.sh generate
./stackvo.sh up
```

---

## Network Monitoring

### Active Connections

```bash
# List all containers in network
docker network inspect stackvo-net --format '{{range .Containers}}{{.Name}} - {{.IPv4Address}}{{"\n"}}{{end}}'
```

### Network Statistics

```bash
# Container network statistics
docker stats --no-stream --format "table {{.Container}}\t{{.NetIO}}"
```

### Traffic Monitoring

```bash
# Monitor network traffic with tcpdump
docker run --rm --net=container:stackvo-mysql nicolaka/netshoot tcpdump -i any port 3306
```

---

## Best Practices

### 1. Hostname Usage

❌ **Incorrect:**
```php
$host = '172.30.0.5';  // Using IP address
```

✅ **Correct:**
```php
$host = 'stackvo-mysql';  // Use hostname
```

### 2. Internal Port Usage

❌ **Incorrect:**
```php
$port = 5433;  // Host port
```

✅ **Correct:**
```php
$port = 5432;  // Internal port
```

### 3. Connection String

✅ **Correct:**
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
