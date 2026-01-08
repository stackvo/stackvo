---
title: Service Configuration
description: Shows how to configure and use services in Stackvo.
---

# Service Configuration

This guide detailedly shows how to configure and use services in Stackvo. It explains activation, connection from PHP, CLI access, and management UI usage for 14 services ranging from databases like MySQL, PostgreSQL, MongoDB to cache systems like Redis, Memcached, message queues like RabbitMQ, Kafka, and search and indexing tools like Elasticsearch, Kibana.

---

## Service Activation

### Activation in .env File

```bash
# Edit .env file
nano .env

# Enable service
SERVICE_MYSQL_ENABLE=true
SERVICE_REDIS_ENABLE=true
SERVICE_RABBITMQ_ENABLE=true

# Generate configurations
./stackvo.sh generate

# Start services
./stackvo.sh up
```

---

## Databases

### MySQL

**Activation:**
```bash
SERVICE_MYSQL_ENABLE=true
SERVICE_MYSQL_VERSION=8.0
SERVICE_MYSQL_ROOT_PASSWORD=root
SERVICE_MYSQL_DATABASE=stackvo
SERVICE_MYSQL_USER=stackvo
SERVICE_MYSQL_PASSWORD=stackvo
```

**Connection from PHP:**
```php
<?php
$pdo = new PDO(
    'mysql:host=stackvo-mysql;port=3306;dbname=stackvo',
    'stackvo',
    'stackvo'
);
```

**CLI Access:**
```bash
# Inside container
docker exec -it stackvo-mysql mysql -u root -proot

# From host
mysql -h 127.0.0.1 -P 3306 -u stackvo -pstackvo stackvo
```

**Management UI:**
```
https://phpmyadmin.stackvo.loc
```

### PostgreSQL

**Activation:**
```bash
SERVICE_POSTGRES_ENABLE=true
SERVICE_POSTGRES_VERSION=14
SERVICE_POSTGRES_PASSWORD=root
SERVICE_POSTGRES_DB=stackvo
SERVICE_POSTGRES_USER=stackvo
```

**Connection from PHP:**
```php
<?php
$pdo = new PDO(
    'pgsql:host=stackvo-postgres;port=5432;dbname=stackvo',
    'stackvo',
    'root'
);
```

**CLI Access:**
```bash
# Inside container
docker exec -it stackvo-postgres psql -U stackvo -d stackvo

# From host
psql -h 127.0.0.1 -p 5433 -U stackvo -d stackvo
```

**Management UI:**
```
https://phppgadmin.stackvo.loc
```

### MongoDB

**Activation:**
```bash
SERVICE_MONGO_ENABLE=true
SERVICE_MONGO_VERSION=5.0
SERVICE_MONGO_INITDB_ROOT_USERNAME=root
SERVICE_MONGO_INITDB_ROOT_PASSWORD=root
```

**Connection from PHP:**
```php
<?php
$client = new MongoDB\Client(
    'mongodb://root:root@stackvo-mongo:27017/stackvo?authSource=admin'
);
$db = $client->stackvo;
```

**CLI Access:**
```bash
# Inside container
docker exec -it stackvo-mongo mongosh -u root -p root --authenticationDatabase admin
```

**Management UI:**
```
https://phpmongo.stackvo.loc
```

---

## Cache Systems

### Redis

**Activation:**
```bash
SERVICE_REDIS_ENABLE=true
SERVICE_REDIS_VERSION=7.0
SERVICE_REDIS_PASSWORD=
```

**Usage from PHP:**
```php
<?php
$redis = new Redis();
$redis->connect('stackvo-redis', 6379);

// Set
$redis->set('key', 'value');

// Get
$value = $redis->get('key');

// Expire
$redis->setex('key', 3600, 'value');
```

**With Laravel:**
```php
// config/database.php
'redis' => [
    'client' => 'phpredis',
    'default' => [
        'host' => 'stackvo-redis',
        'password' => null,
        'port' => 6379,
        'database' => 0,
    ],
],
```

**CLI Access:**
```bash
# Redis CLI
docker exec -it stackvo-redis redis-cli

# Commands
> SET key value
> GET key
> KEYS *
> FLUSHALL
```

### Memcached

**Activation:**
```bash
SERVICE_MEMCACHED_ENABLE=true
SERVICE_MEMCACHED_VERSION=1.6
```

**Usage from PHP:**
```php
<?php
$memcached = new Memcached();
$memcached->addServer('stackvo-memcached', 11211);

// Set
$memcached->set('key', 'value', 3600);

// Get
$value = $memcached->get('key');
```

**Management UI:**
```
https://phpmemcachedadmin.stackvo.loc
```

---

## Message Queues

### RabbitMQ

**Activation:**
```bash
SERVICE_RABBITMQ_ENABLE=true
SERVICE_RABBITMQ_VERSION=3
SERVICE_RABBITMQ_DEFAULT_USER=admin
SERVICE_RABBITMQ_DEFAULT_PASS=admin
```

**Usage from PHP:**
```php
<?php
use PhpAmqpLib\Connection\AMQPStreamConnection;
use PhpAmqpLib\Message\AMQPMessage;

$connection = new AMQPStreamConnection(
    'stackvo-rabbitmq',
    5672,
    'admin',
    'admin'
);
$channel = $connection->channel();

// Declare queue
$channel->queue_declare('hello', false, false, false, false);

// Publish message
$msg = new AMQPMessage('Hello World!');
$channel->basic_publish($msg, '', 'hello');

// Consume message
$callback = function ($msg) {
    echo 'Received: ', $msg->body, "\n";
};
$channel->basic_consume('hello', '', false, true, false, false, $callback);
```

**Management UI:**
```
https://rabbitmq.stackvo.loc
User: admin
Password: admin
```

### Kafka

**Activation:**
```bash
SERVICE_KAFKA_ENABLE=true
SERVICE_KAFKA_VERSION=7.5.0
```

**Usage from PHP:**
```php
<?php
// rdkafka extension required
$conf = new RdKafka\Conf();
$conf->set('metadata.broker.list', 'stackvo-kafka:9092');

// Producer
$producer = new RdKafka\Producer($conf);
$topic = $producer->newTopic('test');
$topic->produce(RD_KAFKA_PARTITION_UA, 0, 'Message payload');
$producer->flush(10000);
```

**Management UI:**
```
https://kafbat.stackvo.loc
```

---

## Search and Indexing

### Elasticsearch

**Activation:**
```bash
SERVICE_ELASTICSEARCH_ENABLE=true
SERVICE_ELASTICSEARCH_VERSION=8.11.3
```

**Usage from PHP:**
```php
<?php
use Elasticsearch\ClientBuilder;

$client = ClientBuilder::create()
    ->setHosts(['stackvo-elasticsearch:9200'])
    ->build();

// Create index
$client->indices()->create(['index' => 'my_index']);

// Add document
$client->index([
    'index' => 'my_index',
    'id' => '1',
    'body' => ['title' => 'Test Document']
]);

// Search
$results = $client->search([
    'index' => 'my_index',
    'body' => [
        'query' => [
            'match' => ['title' => 'test']
        ]
    ]
]);
```

### Kibana

**Activation:**
```bash
SERVICE_KIBANA_ENABLE=true
SERVICE_KIBANA_VERSION=8.11.3
```

**Access:**
```
https://kibana.stackvo.loc
```

---

## Monitoring

### Grafana

**Activation:**
```bash
SERVICE_GRAFANA_ENABLE=true
SERVICE_GRAFANA_VERSION=latest
SERVICE_GRAFANA_ADMIN_USER=admin
SERVICE_GRAFANA_ADMIN_PASSWORD=admin
```

**Access:**
```
https://grafana.stackvo.loc
User: admin
Password: admin
```

---

## Developer Tools

### MailHog

**Activation:**
```bash
SERVICE_MAILHOG_ENABLE=true
SERVICE_MAILHOG_VERSION=latest
```

**PHP Configuration:**
```ini
; php.ini
sendmail_path = "/usr/sbin/sendmail -S stackvo-mailhog:1025"
```

**Access:**
```
https://mailhog.stackvo.loc
```

### Blackfire

**Activation:**
```bash
SERVICE_BLACKFIRE_ENABLE=true
SERVICE_BLACKFIRE_VERSION=latest
SERVICE_BLACKFIRE_SERVER_ID=
SERVICE_BLACKFIRE_SERVER_TOKEN=
```

**PHP Extension:**
```bash
# Blackfire PHP extension is automatically installed
# Credentials must be set in .env file
```

---

## Service Management

### Starting/Stopping Services

```bash
# Start specific service
docker compose -f generated/stackvo.yml \
  -f generated/docker-compose.dynamic.yml \
  up -d mysql

# Stop specific service
docker compose -f generated/stackvo.yml \
  -f generated/docker-compose.dynamic.yml \
  stop mysql

# Restart specific service
docker compose -f generated/stackvo.yml \
  -f generated/docker-compose.dynamic.yml \
  restart mysql
```

### Service Logs

```bash
# Follow service logs
docker logs -f stackvo-mysql
docker logs -f stackvo-redis
docker logs -f stackvo-rabbitmq
```

### Service Status

```bash
# List running services
docker ps --filter "name=stackvo-"

# Inspect specific service
docker inspect stackvo-mysql
```

---

## Troubleshooting

### Service Not Starting

```bash
# Check logs
docker logs stackvo-<service-name>

# Check port conflict
docker ps --format "table {{.Names}}\t{{.Ports}}"

# Regenerate configuration
./stackvo.sh generate
./stackvo.sh restart
```

### Connection Error

```bash
# Check network
docker network inspect stackvo-net

# Ping test
docker exec stackvo-php ping stackvo-mysql

# Port test
docker exec stackvo-php nc -zv stackvo-mysql 3306
```

### Data Loss

```bash
# List volumes
docker volume ls | grep stackvo

# Volume backup
docker run --rm -v stackvo_mysql-data:/data -v $(pwd):/backup ubuntu tar czf /backup/mysql-backup.tar.gz /data

# Volume restore
docker run --rm -v stackvo_mysql-data:/data -v $(pwd):/backup ubuntu tar xzf /backup/mysql-backup.tar.gz -C /
```
