---
title: Tutorials
description: Tutorials to learn Stackvo
---

# Tutorials

Tutorials to learn Stackvo step-by-step. This section teaches Stackvo usage with practical examples from beginner to advanced levels. Hands-on tutorials with real-world scenarios are provided, ranging from your first PHP project to using MySQL and Redis, installing Laravel, asynchronous jobs with RabbitMQ, and microservice architecture.

---

## Beginner Level

### 1. Create Your First Project

This tutorial shows you how to create a PHP project from scratch.

#### Goal

Create a simple PHP project and run it in the browser.

#### Steps

**1. Create Project Directory**

```bash
mkdir -p projects/hello-world/public
```

**2. Create stackvo.json**

```bash
cat > projects/hello-world/stackvo.json <<EOF
{
  "name": "hello-world",
  "domain": "hello.loc",
  "php": {
    "version": "8.2"
  },
  "webserver": "nginx",
  "document_root": "public"
}
EOF
```

**3. Create index.php**

```bash
cat > projects/hello-world/public/index.php <<'EOF'
<?php
echo "<h1>Hello, Stackvo!</h1>";
echo "<p>PHP Version: " . phpversion() . "</p>";
EOF
```

**4. Generate and Start**

```bash
./stackvo.sh generate projects
./stackvo.sh up
```

**5. Update Hosts File**

```bash
echo "127.0.0.1  hello.loc" | sudo tee -a /etc/hosts
```

**6. Open in Browser**

```
https://hello.loc
```

**Result:** You should see the "Hello, Stackvo!" message.

---

### 2. Working with MySQL

This tutorial shows how to connect to a MySQL database and insert data.

#### Goal

Make a simple TODO application using a MySQL database.

#### Steps

**1. Enable MySQL**

```bash
# Edit .env file
nano .env

# Enable MySQL
SERVICE_MYSQL_ENABLE=true

# Generate and start
./stackvo.sh generate
./stackvo.sh up
```

**2. Create Database**

```bash
docker exec -it stackvo-mysql mysql -u root -proot <<EOF
CREATE DATABASE todo;
USE todo;
CREATE TABLE tasks (
    id INT AUTO_INCREMENT PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    completed BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
INSERT INTO tasks (title) VALUES ('First task');
INSERT INTO tasks (title) VALUES ('Second task');
EOF
```

**3. Write PHP Code**

```bash
cat > projects/hello-world/public/todo.php <<'EOF'
<?php
$pdo = new PDO(
    'mysql:host=stackvo-mysql;port=3306;dbname=todo',
    'root',
    'root'
);

// Get all tasks
$stmt = $pdo->query('SELECT * FROM tasks ORDER BY id DESC');
$tasks = $stmt->fetchAll(PDO::FETCH_ASSOC);
?>
<!DOCTYPE html>
<html>
<head>
    <title>TODO App</title>
    <style>
        body { font-family: Arial; max-width: 600px; margin: 50px auto; }
        .task { padding: 10px; border: 1px solid #ddd; margin: 5px 0; }
        .completed { text-decoration: line-through; color: #999; }
    </style>
</head>
<body>
    <h1>TODO List</h1>
    <?php foreach ($tasks as $task): ?>
        <div class="task <?= $task['completed'] ? 'completed' : '' ?>">
            <?= htmlspecialchars($task['title']) ?>
        </div>
    <?php endforeach; ?>
</body>
</html>
EOF
```

**4. Open in Browser**

```
https://hello.loc/todo.php
```

**Result:** You should see the TODO list.

---

### 3. Using Redis Cache

This tutorial shows how to use cache with Redis.

#### Goal

Reduce page load time using Redis.

#### Steps

**1. Enable Redis**

```bash
# Edit .env file
nano .env

# Enable Redis
SERVICE_REDIS_ENABLE=true

# Generate and start
./stackvo.sh generate
./stackvo.sh up
```

**2. Check PHP Redis Extension**

```bash
docker exec stackvo-hello-world-php php -m | grep redis
```

**3. Cache Example**

```bash
cat > projects/hello-world/public/cache.php <<'EOF'
<?php
$redis = new Redis();
$redis->connect('stackvo-redis', 6379);

$cacheKey = 'expensive_data';
$cachedData = $redis->get($cacheKey);

if ($cachedData) {
    echo "<h1>From Cache</h1>";
    echo "<p>Data: $cachedData</p>";
} else {
    // Expensive operation simulation
    sleep(2);
    $data = "Expensive calculation result: " . time();
    
    // Save to cache (60 seconds)
    $redis->setex($cacheKey, 60, $data);
    
    echo "<h1>Fresh Data (Cached for 60s)</h1>";
    echo "<p>Data: $data</p>";
}

echo "<p><a href='cache.php'>Refresh</a></p>";
EOF
```

**4. Test It**

```
https://hello.loc/cache.php
```

**Result:** First load takes 2 seconds, subsequent loads are instant.

---

## Intermediate Level

### 4. Laravel Project Installation

This tutorial shows how to create a Laravel project.

#### Goal

Install a fully functional Laravel project.

#### Steps

**1. Install Laravel with Composer**

```bash
composer create-project laravel/laravel projects/laravel-app
```

**2. Create stackvo.json**

```bash
cat > projects/laravel-app/stackvo.json <<EOF
{
  "name": "laravel-app",
  "domain": "laravel.loc",
  "php": {
    "version": "8.2",
    "extensions": [
      "pdo",
      "pdo_mysql",
      "mbstring",
      "xml",
      "curl",
      "zip",
      "bcmath",
      "gd",
      "redis"
    ]
  },
  "webserver": "nginx",
  "document_root": "public"
}
EOF
```

**3. Edit .env File**

```bash
cat > projects/laravel-app/.env <<EOF
APP_NAME=Laravel
APP_ENV=local
APP_KEY=
APP_DEBUG=true
APP_URL=https://laravel.loc

DB_CONNECTION=mysql
DB_HOST=stackvo-mysql
DB_PORT=3306
DB_DATABASE=laravel
DB_USERNAME=stackvo
DB_PASSWORD=stackvo

CACHE_DRIVER=redis
REDIS_HOST=stackvo-redis
REDIS_PORT=6379
EOF
```

**4. Create Database**

```bash
docker exec -it stackvo-mysql mysql -u root -proot -e "CREATE DATABASE laravel;"
```

**5. Generate and Start**

```bash
./stackvo.sh generate projects
./stackvo.sh up
```

**6. Generate Laravel Key**

```bash
docker exec stackvo-laravel-app-php php artisan key:generate
```

**7. Run Migration**

```bash
docker exec stackvo-laravel-app-php php artisan migrate
```

**8. Update Hosts File**

```bash
echo "127.0.0.1  laravel.loc" | sudo tee -a /etc/hosts
```

**9. Open in Browser**

```
https://laravel.loc
```

**Result:** You should see the Laravel welcome page.

---

### 5. Asynchronous Jobs with RabbitMQ

This tutorial shows how to create an asynchronous job queue using RabbitMQ.

#### Goal

Make sending emails asynchronous.

#### Steps

**1. Enable RabbitMQ**

```bash
# .env
SERVICE_RABBITMQ_ENABLE=true

./stackvo.sh generate
./stackvo.sh up
```

**2. Install php-amqplib**

```bash
docker exec stackvo-laravel-app-php composer require php-amqplib/php-amqplib
```

**3. Create Producer**

```bash
cat > projects/laravel-app/public/send-email.php <<'EOF'
<?php
require __DIR__ . '/../vendor/autoload.php';

use PhpAmqpLib\Connection\AMQPStreamConnection;
use PhpAmqpLib\Message\AMQPMessage;

$connection = new AMQPStreamConnection(
    'stackvo-rabbitmq',
    5672,
    'admin',
    'admin'
);
$channel = $connection->channel();

$channel->queue_declare('emails', false, true, false, false);

$emailData = json_encode([
    'to' => 'user@example.com',
    'subject' => 'Test Email',
    'body' => 'This is a test email'
]);

$msg = new AMQPMessage($emailData, ['delivery_mode' => 2]);
$channel->basic_publish($msg, '', 'emails');

echo "Email queued successfully!";

$channel->close();
$connection->close();
EOF
```

**4. Create Consumer**

```bash
cat > projects/laravel-app/email-worker.php <<'EOF'
<?php
require __DIR__ . '/vendor/autoload.php';

use PhpAmqpLib\Connection\AMQPStreamConnection;

$connection = new AMQPStreamConnection(
    'stackvo-rabbitmq',
    5672,
    'admin',
    'admin'
);
$channel = $connection->channel();

$channel->queue_declare('emails', false, true, false, false);

echo "Waiting for emails...\n";

$callback = function ($msg) {
    $emailData = json_decode($msg->body, true);
    echo "Sending email to: " . $emailData['to'] . "\n";
    
    // Email sending simulation
    sleep(2);
    
    echo "Email sent!\n";
    $msg->ack();
};

$channel->basic_qos(null, 1, null);
$channel->basic_consume('emails', '', false, false, false, false, $callback);

while ($channel->is_consuming()) {
    $channel->wait();
}
EOF
```

**5. Run Worker**

```bash
docker exec -d stackvo-laravel-app-php php email-worker.php
```

**6. Test It**

```
https://laravel.loc/send-email.php
```

**Result:** You will see "Email queued successfully!" message, worker processes the email in the background.

---

## Advanced Level

### 6. Microservice Architecture

This tutorial shows how to create a microservice architecture where multiple services work together.

#### Goal

Create API Gateway, Auth Service, and Product Service.

#### Architecture

```
Client → API Gateway → Auth Service
                    → Product Service → MySQL
```

#### Steps

**1. Auth Service**

```bash
mkdir -p projects/auth-service/public

cat > projects/auth-service/stackvo.json <<EOF
{
  "name": "auth-service",
  "domain": "auth.loc",
  "php": {"version": "8.3"},
  "webserver": "nginx",
  "document_root": "public"
}
EOF

cat > projects/auth-service/public/index.php <<'EOF'
<?php
header('Content-Type: application/json');

$token = $_GET['token'] ?? '';

if ($token === 'valid-token') {
    echo json_encode(['valid' => true, 'user_id' => 123]);
} else {
    http_response_code(401);
    echo json_encode(['valid' => false]);
}
EOF
```

**2. Product Service**

```bash
mkdir -p projects/product-service/public

cat > projects/product-service/stackvo.json <<EOF
{
  "name": "product-service",
  "domain": "products.loc",
  "php": {"version": "8.3"},
  "webserver": "nginx",
  "document_root": "public"
}
EOF

cat > projects/product-service/public/index.php <<'EOF'
<?php
header('Content-Type: application/json');

$products = [
    ['id' => 1, 'name' => 'Product 1', 'price' => 100],
    ['id' => 2, 'name' => 'Product 2', 'price' => 200],
];

echo json_encode($products);
EOF
```

**3. API Gateway**

```bash
mkdir -p projects/api-gateway/public

cat > projects/api-gateway/stackvo.json <<EOF
{
  "name": "api-gateway",
  "domain": "api.loc",
  "php": {"version": "8.3"},
  "webserver": "nginx",
  "document_root": "public"
}
EOF

cat > projects/api-gateway/public/index.php <<'EOF'
<?php
header('Content-Type: application/json');

$token = $_SERVER['HTTP_AUTHORIZATION'] ?? '';

// Auth check
$authResponse = file_get_contents("http://stackvo-auth-service-web/index.php?token=$token");
$auth = json_decode($authResponse, true);

if (!$auth['valid']) {
    http_response_code(401);
    echo json_encode(['error' => 'Unauthorized']);
    exit;
}

// Get Products
$productsResponse = file_get_contents("http://stackvo-product-service-web/index.php");
echo $productsResponse;
EOF
```

**4. Generate and Start**

```bash
./stackvo.sh generate projects
./stackvo.sh up

echo "127.0.0.1  auth.loc products.loc api.loc" | sudo tee -a /etc/hosts
```

**5. Test It**

```bash
# Invalid token
curl -H "Authorization: invalid" https://api.loc
# {"error":"Unauthorized"}

# Valid token
curl -H "Authorization: valid-token" https://api.loc
# [{"id":1,"name":"Product 1","price":100},{"id":2,"name":"Product 2","price":200}]
```
