# Örnekler

Gerçek dünya kullanım senaryoları ve örnek projeler. Bu bölüm, e-ticaret platformundan blog platformuna, SaaS uygulamasından mikroservis mimarisine, real-time chat uygulamasından CI/CD pipeline'a kadar gerçek dünya senaryolarında Stackvo'un nasıl kullanılacağını detaylı kod örnekleriyle göstermektedir. Her örnek, tam çalışan bir proje mimarisi sunmaktadır.

## E-Ticaret Platformu

### Mimari

```
┌─────────────────────────────────────────────────┐
│                  Traefik                        │
│            (Reverse Proxy)                      │
└────────┬────────────────────────────────────────┘
         │
    ┌────┴────┬─────────┬──────────┬──────────┐
    │         │         │          │          │
┌───▼───┐ ┌──▼──┐  ┌───▼────┐ ┌───▼────┐ ┌──▼───┐
│ Web   │ │ API │  │ Admin  │ │ Worker │ │ Cron │
│ (Vue) │ │(PHP)│  │ (PHP)  │ │ (PHP)  │ │(PHP) │
└───┬───┘ └──┬──┘  └───┬────┘ └───┬────┘ └──┬───┘
    │        │         │          │         │
    └────────┴─────────┴──────────┴─────────┘
                       │
         ┌─────────────┼─────────────┐
         │             │             │
    ┌────▼────┐   ┌───▼────┐   ┌───▼────┐
    │  MySQL  │   │ Redis  │   │RabbitMQ│
    └─────────┘   └────────┘   └────────┘
```

### Servisler

**1. MySQL**
```bash
SERVICE_MYSQL_ENABLE=true
SERVICE_MYSQL_VERSION=8.0
```

**2. Redis**
```bash
SERVICE_REDIS_ENABLE=true
SERVICE_REDIS_VERSION=7.0
```

**3. RabbitMQ**
```bash
SERVICE_RABBITMQ_ENABLE=true
SERVICE_RABBITMQ_VERSION=3
```

### Projeler

**1. Web Frontend (Vue.js)**

```bash
mkdir -p projects/ecommerce-web/public
cat > projects/ecommerce-web/stackvo.json <<EOF
{
  "name": "ecommerce-web",
  "domain": "shop.loc",
  "nodejs": {"version": "14.23"},
  "webserver": "nginx",
  "document_root": "dist"
}
EOF
```

**2. API Backend (Laravel)**

```bash
composer create-project laravel/laravel projects/ecommerce-api

cat > projects/ecommerce-api/stackvo.json <<EOF
{
  "name": "ecommerce-api",
  "domain": "api.shop.loc",
  "php": {
    "version": "8.2",
    "extensions": ["pdo", "pdo_mysql", "redis", "gd", "zip"]
  },
  "webserver": "nginx",
  "document_root": "public"
}
EOF
```

**3. Admin Panel**

```bash
mkdir -p projects/ecommerce-admin/public

cat > projects/ecommerce-admin/stackvo.json <<EOF
{
  "name": "ecommerce-admin",
  "domain": "admin.shop.loc",
  "php": {"version": "8.2"},
  "webserver": "nginx",
  "document_root": "public"
}
EOF
```

**4. Background Worker**

```bash
cat > projects/ecommerce-api/worker.php <<'EOF'
<?php
require __DIR__ . '/vendor/autoload.php';

use PhpAmqpLib\Connection\AMQPStreamConnection;

$connection = new AMQPStreamConnection('stackvo-rabbitmq', 5672, 'admin', 'admin');
$channel = $connection->channel();

$channel->queue_declare('orders', false, true, false, false);

$callback = function ($msg) {
    $order = json_decode($msg->body, true);
    
    // Sipariş işleme
    processOrder($order);
    
    // Email gönder
    sendOrderEmail($order);
    
    $msg->ack();
};

$channel->basic_consume('orders', '', false, false, false, false, $callback);

while ($channel->is_consuming()) {
    $channel->wait();
}
EOF
```

---

## Blog Platformu

### Özellikler

- Multi-tenant blog sistemi
- Markdown desteği
- Full-text search (Elasticsearch)
- Image optimization
- CDN entegrasyonu

### Stack

- **Backend:** Symfony
- **Database:** PostgreSQL
- **Cache:** Redis
- **Search:** Elasticsearch
- **Queue:** RabbitMQ

### Konfigürasyon

```bash
# .env
SERVICE_POSTGRES_ENABLE=true
SERVICE_REDIS_ENABLE=true
SERVICE_ELASTICSEARCH_ENABLE=true
SERVICE_KIBANA_ENABLE=true
SERVICE_RABBITMQ_ENABLE=true

./core/cli/stackvo.sh generate
./core/cli/stackvo.sh up
```

### Proje

```bash
symfony new projects/blog-platform --webapp

cat > projects/blog-platform/stackvo.json <<EOF
{
  "name": "blog-platform",
  "domain": "blog.loc",
  "php": {
    "version": "8.3",
    "extensions": ["pdo", "pdo_pgsql", "redis", "gd", "imagick"]
  },
  "webserver": "nginx",
  "document_root": "public"
}
EOF
```

### Elasticsearch Entegrasyonu

```php
<?php
// config/packages/fos_elastica.yaml
fos_elastica:
    clients:
        default:
            host: stackvo-elasticsearch
            port: 9200
    indexes:
        blog:
            types:
                post:
                    properties:
                        title: ~
                        content: ~
                        tags: ~
```

---

## SaaS Uygulaması

### Mimari

Multi-tenant SaaS uygulaması:

- Tenant isolation (database per tenant)
- Subscription management
- Usage tracking
- Billing integration

### Database Stratejisi

Her tenant için ayrı database:

```php
<?php
// TenantManager.php
class TenantManager
{
    public function getTenantConnection($tenantId)
    {
        $config = [
            'host' => 'stackvo-mysql',
            'database' => "tenant_{$tenantId}",
            'username' => 'stackvo',
            'password' => 'stackvo'
        ];
        
        return new PDO(
            "mysql:host={$config['host']};dbname={$config['database']}",
            $config['username'],
            $config['password']
        );
    }
}
```

### Tenant Provisioning

```php
<?php
// ProvisionTenant.php
class ProvisionTenant
{
    public function provision($tenantId, $plan)
    {
        // 1. Database oluştur
        $this->createDatabase($tenantId);
        
        // 2. Migration çalıştır
        $this->runMigrations($tenantId);
        
        // 3. Seed data
        $this->seedData($tenantId, $plan);
        
        // 4. Cache hazırla
        $this->warmupCache($tenantId);
    }
    
    private function createDatabase($tenantId)
    {
        $pdo = new PDO('mysql:host=stackvo-mysql', 'root', 'root');
        $pdo->exec("CREATE DATABASE tenant_{$tenantId}");
    }
}
```

---

## Mikroservis Mimarisi

### Servisler

1. **API Gateway** - Routing ve authentication
2. **User Service** - Kullanıcı yönetimi
3. **Product Service** - Ürün kataloğu
4. **Order Service** - Sipariş yönetimi
5. **Payment Service** - Ödeme işlemleri
6. **Notification Service** - Email/SMS

### Service Discovery

```php
<?php
// ServiceRegistry.php
class ServiceRegistry
{
    private $redis;
    
    public function __construct()
    {
        $this->redis = new Redis();
        $this->redis->connect('stackvo-redis', 6379);
    }
    
    public function register($serviceName, $url)
    {
        $this->redis->hSet('services', $serviceName, $url);
    }
    
    public function discover($serviceName)
    {
        return $this->redis->hGet('services', $serviceName);
    }
}

// Kullanım
$registry = new ServiceRegistry();
$registry->register('user-service', 'http://stackvo-user-service-web');
$registry->register('product-service', 'http://stackvo-product-service-web');

$userServiceUrl = $registry->discover('user-service');
```

### Inter-Service Communication

```php
<?php
// HttpClient.php
class HttpClient
{
    private $registry;
    
    public function call($service, $endpoint, $data = [])
    {
        $url = $this->registry->discover($service);
        
        $ch = curl_init("$url/$endpoint");
        curl_setopt($ch, CURLOPT_RETURNTRANSFER, true);
        curl_setopt($ch, CURLOPT_POSTFIELDS, json_encode($data));
        curl_setopt($ch, CURLOPT_HTTPHEADER, [
            'Content-Type: application/json',
            'X-Service-Token: ' . $this->getServiceToken()
        ]);
        
        $response = curl_exec($ch);
        return json_decode($response, true);
    }
}

// Kullanım
$client = new HttpClient($registry);
$user = $client->call('user-service', 'users/123');
```

---

## Real-time Chat Uygulaması

### Stack

- **Backend:** PHP + WebSocket
- **Frontend:** Vue.js
- **Database:** MongoDB
- **Cache:** Redis
- **Queue:** RabbitMQ

### WebSocket Server

```php
<?php
// chat-server.php
use Ratchet\Server\IoServer;
use Ratchet\Http\HttpServer;
use Ratchet\WebSocket\WsServer;

require __DIR__ . '/vendor/autoload.php';

class ChatServer implements MessageComponentInterface
{
    protected $clients;
    protected $redis;
    
    public function __construct()
    {
        $this->clients = new \SplObjectStorage;
        $this->redis = new Redis();
        $this->redis->connect('stackvo-redis', 6379);
    }
    
    public function onMessage(ConnectionInterface $from, $msg)
    {
        $data = json_decode($msg, true);
        
        // Mesajı MongoDB'ye kaydet
        $this->saveMessage($data);
        
        // Tüm client'lara gönder
        foreach ($this->clients as $client) {
            $client->send($msg);
        }
        
        // Redis'e publish (scaling için)
        $this->redis->publish('chat', $msg);
    }
}

$server = IoServer::factory(
    new HttpServer(
        new WsServer(
            new ChatServer()
        )
    ),
    8080
);

$server->run();
```

---

## CI/CD Pipeline

### GitLab CI

```yaml
# .gitlab-ci.yml
stages:
  - test
  - build
  - deploy

test:
  stage: test
  script:
    - docker compose -f docker-compose.test.yml up -d
    - docker exec test-app vendor/bin/phpunit
    - docker compose -f docker-compose.test.yml down

build:
  stage: build
  script:
    - docker build -t myapp:$CI_COMMIT_SHA .
    - docker push myapp:$CI_COMMIT_SHA

deploy:
  stage: deploy
  script:
    - ssh user@server "cd /app && docker compose pull && docker compose up -d"
  only:
    - main
```

---

## Monitoring Stack

### Prometheus + Grafana

```bash
# .env
SERVICE_GRAFANA_ENABLE=true
SERVICE_PROMETHEUS_ENABLE=true

./core/cli/stackvo.sh generate
./core/cli/stackvo.sh up
```

### Metrics Export

```php
<?php
// metrics.php
$redis = new Redis();
$redis->connect('stackvo-redis', 6379);

header('Content-Type: text/plain');

echo "# HELP app_requests_total Total requests\n";
echo "# TYPE app_requests_total counter\n";
echo "app_requests_total " . $redis->get('requests:total') . "\n";

echo "# HELP app_users_active Active users\n";
echo "# TYPE app_users_active gauge\n";
echo "app_users_active " . $redis->sCard('users:active') . "\n";
```