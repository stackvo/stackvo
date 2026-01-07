---
title: Servis Konfigürasyonu
description: Stackvo'da servisleri nasıl yapılandıracağınızı ve kullanacağınızı gösterir.
---

# Servis Konfigürasyonu

Bu kılavuz, Stackvo'da servisleri nasıl yapılandıracağınızı ve kullanacağınızı detaylı olarak göstermektedir. MySQL, PostgreSQL, MongoDB gibi veritabanlarından Redis, Memcached gibi cache sistemlerine, RabbitMQ, Kafka gibi message queue'lardan Elasticsearch, Kibana gibi arama ve indeksleme araçlarına kadar 14 servisin aktivasyonu, PHP'den bağlantısı, CLI erişimi ve management UI kullanımı açıklanmaktadır.

---

## Servis Aktivasyonu

### .env Dosyasında Aktivasyon

```bash
# .env dosyasını düzenle
nano .env

# Servisi aktif et
SERVICE_MYSQL_ENABLE=true
SERVICE_REDIS_ENABLE=true
SERVICE_RABBITMQ_ENABLE=true

# Konfigürasyonları üret
./core/cli/stackvo.sh generate

# Servisleri başlat
./core/cli/stackvo.sh up
```

---

## Veritabanları

### MySQL

**Aktivasyon:**
```bash
SERVICE_MYSQL_ENABLE=true
SERVICE_MYSQL_VERSION=8.0
SERVICE_MYSQL_ROOT_PASSWORD=root
SERVICE_MYSQL_DATABASE=stackvo
SERVICE_MYSQL_USER=stackvo
SERVICE_MYSQL_PASSWORD=stackvo
```

**PHP'den Bağlantı:**
```php
<?php
$pdo = new PDO(
    'mysql:host=stackvo-mysql;port=3306;dbname=stackvo',
    'stackvo',
    'stackvo'
);
```

**CLI Erişimi:**
```bash
# Container içinden
docker exec -it stackvo-mysql mysql -u root -proot

# Host'tan
mysql -h 127.0.0.1 -P 3306 -u stackvo -pstackvo stackvo
```

**Management UI:**
```
https://phpmyadmin.stackvo.loc
```

### PostgreSQL

**Aktivasyon:**
```bash
SERVICE_POSTGRES_ENABLE=true
SERVICE_POSTGRES_VERSION=14
SERVICE_POSTGRES_PASSWORD=root
SERVICE_POSTGRES_DB=stackvo
SERVICE_POSTGRES_USER=stackvo
```

**PHP'den Bağlantı:**
```php
<?php
$pdo = new PDO(
    'pgsql:host=stackvo-postgres;port=5432;dbname=stackvo',
    'stackvo',
    'root'
);
```

**CLI Erişimi:**
```bash
# Container içinden
docker exec -it stackvo-postgres psql -U stackvo -d stackvo

# Host'tan
psql -h 127.0.0.1 -p 5433 -U stackvo -d stackvo
```

**Management UI:**
```
https://phppgadmin.stackvo.loc
```

### MongoDB

**Aktivasyon:**
```bash
SERVICE_MONGO_ENABLE=true
SERVICE_MONGO_VERSION=5.0
SERVICE_MONGO_INITDB_ROOT_USERNAME=root
SERVICE_MONGO_INITDB_ROOT_PASSWORD=root
```

**PHP'den Bağlantı:**
```php
<?php
$client = new MongoDB\Client(
    'mongodb://root:root@stackvo-mongo:27017/stackvo?authSource=admin'
);
$db = $client->stackvo;
```

**CLI Erişimi:**
```bash
# Container içinden
docker exec -it stackvo-mongo mongosh -u root -p root --authenticationDatabase admin
```

**Management UI:**
```
https://phpmongo.stackvo.loc
```

---

## Cache Sistemleri

### Redis

**Aktivasyon:**
```bash
SERVICE_REDIS_ENABLE=true
SERVICE_REDIS_VERSION=7.0
SERVICE_REDIS_PASSWORD=
```

**PHP'den Kullanım:**
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

**Laravel ile:**
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

**CLI Erişimi:**
```bash
# Redis CLI
docker exec -it stackvo-redis redis-cli

# Komutlar
> SET key value
> GET key
> KEYS *
> FLUSHALL
```

### Memcached

**Aktivasyon:**
```bash
SERVICE_MEMCACHED_ENABLE=true
SERVICE_MEMCACHED_VERSION=1.6
```

**PHP'den Kullanım:**
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

**Aktivasyon:**
```bash
SERVICE_RABBITMQ_ENABLE=true
SERVICE_RABBITMQ_VERSION=3
SERVICE_RABBITMQ_DEFAULT_USER=admin
SERVICE_RABBITMQ_DEFAULT_PASS=admin
```

**PHP'den Kullanım:**
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

// Queue oluştur
$channel->queue_declare('hello', false, false, false, false);

// Mesaj gönder
$msg = new AMQPMessage('Hello World!');
$channel->basic_publish($msg, '', 'hello');

// Mesaj al
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

**Aktivasyon:**
```bash
SERVICE_KAFKA_ENABLE=true
SERVICE_KAFKA_VERSION=7.5.0
```

**PHP'den Kullanım:**
```php
<?php
// rdkafka extension gerekli
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

## Arama ve İndeksleme

### Elasticsearch

**Aktivasyon:**
```bash
SERVICE_ELASTICSEARCH_ENABLE=true
SERVICE_ELASTICSEARCH_VERSION=8.11.3
```

**PHP'den Kullanım:**
```php
<?php
use Elasticsearch\ClientBuilder;

$client = ClientBuilder::create()
    ->setHosts(['stackvo-elasticsearch:9200'])
    ->build();

// Index oluştur
$client->indices()->create(['index' => 'my_index']);

// Document ekle
$client->index([
    'index' => 'my_index',
    'id' => '1',
    'body' => ['title' => 'Test Document']
]);

// Arama
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

**Aktivasyon:**
```bash
SERVICE_KIBANA_ENABLE=true
SERVICE_KIBANA_VERSION=8.11.3
```

**Erişim:**
```
https://kibana.stackvo.loc
```

---

## Monitoring

### Grafana

**Aktivasyon:**
```bash
SERVICE_GRAFANA_ENABLE=true
SERVICE_GRAFANA_VERSION=latest
SERVICE_GRAFANA_ADMIN_USER=admin
SERVICE_GRAFANA_ADMIN_PASSWORD=admin
```

**Erişim:**
```
https://grafana.stackvo.loc
User: admin
Password: admin
```

---

## Developer Tools

### MailHog

**Aktivasyon:**
```bash
SERVICE_MAILHOG_ENABLE=true
SERVICE_MAILHOG_VERSION=latest
```

**PHP Konfigürasyonu:**
```ini
; php.ini
sendmail_path = "/usr/sbin/sendmail -S stackvo-mailhog:1025"
```

**Erişim:**
```
https://mailhog.stackvo.loc
```

### Blackfire

**Aktivasyon:**
```bash
SERVICE_BLACKFIRE_ENABLE=true
SERVICE_BLACKFIRE_VERSION=latest
SERVICE_BLACKFIRE_SERVER_ID=
SERVICE_BLACKFIRE_SERVER_TOKEN=
```

**PHP Extension:**
```bash
# Blackfire PHP extension otomatik olarak yüklenir
# .env dosyasında credentials ayarlanmalıdır
```

---

## Servis Yönetimi

### Servis Başlatma/Durdurma

```bash
# Belirli servisi başlat
docker compose -f generated/stackvo.yml \
  -f generated/docker-compose.dynamic.yml \
  up -d mysql

# Belirli servisi durdur
docker compose -f generated/stackvo.yml \
  -f generated/docker-compose.dynamic.yml \
  stop mysql

# Belirli servisi yeniden başlat
docker compose -f generated/stackvo.yml \
  -f generated/docker-compose.dynamic.yml \
  restart mysql
```

### Servis Logları

```bash
# Servis loglarını izle
docker logs -f stackvo-mysql
docker logs -f stackvo-redis
docker logs -f stackvo-rabbitmq
```

### Servis Durumu

```bash
# Çalışan servisleri listele
docker ps --filter "name=stackvo-"

# Belirli servis detayları
docker inspect stackvo-mysql
```

---

## Troubleshooting

### Servis Başlamıyor

```bash
# Logları kontrol et
docker logs stackvo-<service-name>

# Port çakışması kontrolü
docker ps --format "table {{.Names}}\t{{.Ports}}"

# Konfigürasyonu yeniden üret
./core/cli/stackvo.sh generate
./core/cli/stackvo.sh restart
```

### Bağlantı Hatası

```bash
# Network kontrolü
docker network inspect stackvo-net

# Ping testi
docker exec stackvo-php ping stackvo-mysql

# Port testi
docker exec stackvo-php nc -zv stackvo-mysql 3306
```

### Data Kaybı

```bash
# Volume'ları listele
docker volume ls | grep stackvo

# Volume backup
docker run --rm -v stackvo_mysql-data:/data -v $(pwd):/backup ubuntu tar czf /backup/mysql-backup.tar.gz /data

# Volume restore
docker run --rm -v stackvo_mysql-data:/data -v $(pwd):/backup ubuntu tar xzf /backup/mysql-backup.tar.gz -C /
```