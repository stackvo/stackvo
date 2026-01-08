# Examples

Real-world usage scenarios and example projects. This section detailedly demonstrates how to use Stackvo in real-world scenarios ranging from e-commerce platforms to blog platforms, SaaS applications to microservice architectures, real-time chat applications to CI/CD pipelines with detailed code examples. Each example presents a fully working project architecture.

---

## E-Commerce Platform

### Architecture

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

### Services

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

### Projects

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
    
    // Process order
    processOrder($order);
    
    // Send email
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

## Blog Platform

### Features

- Multi-tenant blog system
- Markdown support
- Full-text search (Elasticsearch)
- Image optimization
- CDN integration

### Stack

- **Backend:** Symfony
- **Database:** PostgreSQL
- **Cache:** Redis
- **Search:** Elasticsearch
- **Queue:** RabbitMQ

### Configuration

```bash
# .env
SERVICE_POSTGRES_ENABLE=true
SERVICE_REDIS_ENABLE=true
SERVICE_ELASTICSEARCH_ENABLE=true
SERVICE_KIBANA_ENABLE=true
SERVICE_RABBITMQ_ENABLE=true

./stackvo.sh generate
./stackvo.sh up
```

### Project

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

### Elasticsearch Integration

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

## SaaS Application

### Architecture

Multi-tenant SaaS application:

- Tenant isolation (database per tenant)
- Subscription management
- Usage tracking
- Billing integration

### Database Strategy

Separate database for each tenant:

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
        // 1. Create database
        $this->createDatabase($tenantId);
        
        // 2. Run migration
        $this->runMigrations($tenantId);
        
        // 3. Seed data
        $this->seedData($tenantId, $plan);
        
        // 4. Warmup cache
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

## Microservice Architecture

### Services

1. **API Gateway** - Routing and authentication
2. **User Service** - User management
3. **Product Service** - Product catalog
4. **Order Service** - Order management
5. **Payment Service** - Payment processing
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

// Usage
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

// Usage
$client = new HttpClient($registry);
$user = $client->call('user-service', 'users/123');
```

---

## Real-time Chat Application

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
        
        // Save message to MongoDB
        $this->saveMessage($data);
        
        // Send to all clients
        foreach ($this->clients as $client) {
            $client->send($msg);
        }
        
        // Publish to Redis (for scaling)
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

./stackvo.sh generate
./stackvo.sh up
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