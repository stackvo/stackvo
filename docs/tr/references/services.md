# Servisler Referansı

14 desteklenen servislerin detaylı referansı. Bu sayfa, veritabanlarından (MySQL, PostgreSQL, MongoDB, MariaDB, Cassandra) cache sistemlerine (Redis, Memcached), message queue'lardan (RabbitMQ, Kafka) arama ve indeksleme araçlarına (Elasticsearch, Kibana), monitoring araçlarından (Grafana) developer tools'a (MailHog, Blackfire) kadar tüm servislerin image bilgilerini, port ayarlarını, konfigürasyon parametrelerini ve bağlantı detaylarını içermektedir.

## Servis Kategorileri

- [Veritabanları](#veritabanları) (5)
- [Cache Sistemleri](#cache-sistemleri) (2)
- [Message Queues](#message-queues) (2)
- [Arama ve İndeksleme](#arama-ve-indeksleme) (2)
- [Monitoring](#monitoring) (1)
- [Developer Tools](#developer-tools) (2)

---

## Veritabanları

### MySQL

**Image:** `mysql:{version}`  
**Default Version:** 8.0  
**Ports:** 3306

**Konfigürasyon:**
```bash
SERVICE_MYSQL_ENABLE=true
SERVICE_MYSQL_VERSION=8.0
SERVICE_MYSQL_ROOT_PASSWORD=root
SERVICE_MYSQL_DATABASE=stackvo
SERVICE_MYSQL_USER=stackvo
SERVICE_MYSQL_PASSWORD=stackvo
```

**Bağlantı:**
- Host: `stackvo-mysql`
- Port: `3306`
- Database: `stackvo`
- User: `stackvo` / Password: `stackvo`
- Root: `root` / Password: `root`

**Management UI:** `https://phpmyadmin.stackvo.loc`

### MariaDB

**Image:** `mariadb:{version}`  
**Default Version:** 10.6  
**Ports:** 3307 (host), 3306 (container)

**Konfigürasyon:**
```bash
SERVICE_MARIADB_ENABLE=true
SERVICE_MARIADB_VERSION=10.6
SERVICE_MARIADB_ROOT_PASSWORD=root
SERVICE_MARIADB_DATABASE=stackvo
SERVICE_MARIADB_USER=stackvo
SERVICE_MARIADB_PASSWORD=stackvo
```

### PostgreSQL

**Image:** `postgres:{version}`  
**Default Version:** 14  
**Ports:** 5433 (host), 5432 (container)

**Konfigürasyon:**
```bash
SERVICE_POSTGRES_ENABLE=true
SERVICE_POSTGRES_VERSION=14
SERVICE_POSTGRES_PASSWORD=root
SERVICE_POSTGRES_DB=stackvo
SERVICE_POSTGRES_USER=stackvo
```

**Management UI:** `https://phppgadmin.stackvo.loc`

### MongoDB

**Image:** `mongo:{version}`  
**Default Version:** 5.0  
**Ports:** 27017

**Konfigürasyon:**
```bash
SERVICE_MONGO_ENABLE=true
SERVICE_MONGO_VERSION=5.0
SERVICE_MONGO_INITDB_ROOT_USERNAME=root
SERVICE_MONGO_INITDB_ROOT_PASSWORD=root
```

**Management UI:** `https://phpmongo.stackvo.loc`

### Cassandra

**Image:** `cassandra:{version}`  
**Default Version:** latest  
**Ports:** 9042

**Konfigürasyon:**
```bash
SERVICE_CASSANDRA_ENABLE=false
SERVICE_CASSANDRA_VERSION=latest
```

---

## Cache Sistemleri

### Redis

**Image:** `redis:{version}`  
**Default Version:** 7.0  
**Ports:** 6379

**Konfigürasyon:**
```bash
SERVICE_REDIS_ENABLE=true
SERVICE_REDIS_VERSION=7.0
SERVICE_REDIS_PASSWORD=
```

**Bağlantı:**
- Host: `stackvo-redis`
- Port: `6379`

### Memcached

**Image:** `memcached:{version}`  
**Default Version:** 1.6  
**Ports:** 11211

**Konfigürasyon:**
```bash
SERVICE_MEMCACHED_ENABLE=true
SERVICE_MEMCACHED_VERSION=1.6
```

**Management UI:** `https://phpmemcachedadmin.stackvo.loc`

---

## Message Queues

### RabbitMQ

**Image:** `rabbitmq:{version}-management`  
**Default Version:** 3  
**Ports:** 5672 (AMQP), 15672 (Management)

**Konfigürasyon:**
```bash
SERVICE_RABBITMQ_ENABLE=true
SERVICE_RABBITMQ_VERSION=3
SERVICE_RABBITMQ_URL=rabbitmq
SERVICE_RABBITMQ_DEFAULT_USER=admin
SERVICE_RABBITMQ_DEFAULT_PASS=admin
```

**Management UI:** `https://rabbitmq.stackvo.loc`

### Kafka

**Image:** `confluentinc/cp-kafka:{version}`  
**Default Version:** 7.5.0  
**Ports:** 9092 (internal), 9094 (external)

**Konfigürasyon:**
```bash
SERVICE_KAFKA_ENABLE=false
SERVICE_KAFKA_VERSION=7.5.0
```

**Management UI:** `https://kafbat.stackvo.loc`

---

## Arama ve İndeksleme

### Elasticsearch

**Image:** `elasticsearch:{version}`  
**Default Version:** 8.11.3  
**Ports:** 9200, 9300

**Konfigürasyon:**
```bash
SERVICE_ELASTICSEARCH_ENABLE=false
SERVICE_ELASTICSEARCH_VERSION=8.11.3
```

### Kibana

**Image:** `kibana:{version}`  
**Default Version:** 8.11.3  
**Ports:** 5601

**Konfigürasyon:**
```bash
SERVICE_KIBANA_ENABLE=false
SERVICE_KIBANA_VERSION=8.11.3
```

**Erişim:** `https://kibana.stackvo.loc`

---

## Monitoring

### Grafana

**Image:** `grafana/grafana:{version}`  
**Default Version:** latest  
**Ports:** 3000

**Konfigürasyon:**
```bash
SERVICE_GRAFANA_ENABLE=false
SERVICE_GRAFANA_VERSION=latest
SERVICE_GRAFANA_ADMIN_USER=admin
SERVICE_GRAFANA_ADMIN_PASSWORD=admin
```

**Erişim:** `https://grafana.stackvo.loc`

---

## Developer Tools

### MailHog

**Image:** `mailhog/mailhog:{version}`  
**Default Version:** latest  
**Ports:** 1025 (SMTP), 8025 (Web)

**Konfigürasyon:**
```bash
SERVICE_MAILHOG_ENABLE=false
SERVICE_MAILHOG_VERSION=latest
```

**Erişim:** `https://mailhog.stackvo.loc`

### Blackfire

**Image:** `blackfire/blackfire:{version}`  
**Default Version:** latest  
**Ports:** 8707

**Konfigürasyon:**
```bash
SERVICE_BLACKFIRE_ENABLE=false
SERVICE_BLACKFIRE_VERSION=latest
SERVICE_BLACKFIRE_SERVER_ID=
SERVICE_BLACKFIRE_SERVER_TOKEN=
```

---
