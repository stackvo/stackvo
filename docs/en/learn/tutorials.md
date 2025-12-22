---
title: Eğitimler    
description: Stackvo'u öğrenmek için eğitimler
---

# Eğitimler

Stackvo'u adım adım öğrenmek için eğitimler. Bu bölüm, başlangıç seviyesinden ileri seviyeye kadar pratik örneklerle Stackvo kullanımını öğretmektedir. İlk PHP projesinden MySQL ve Redis kullanımına, Laravel kurulumundan RabbitMQ ile asenkron işlere, mikroservis mimarisine kadar gerçek dünya senaryolarıyla uygulamalı eğitimler sunulmaktadır.

---

## Başlangıç Seviyesi

### 1. İlk Projenizi Oluşturun

Bu eğitim, sıfırdan bir PHP projesi oluşturmanızı gösterir.

#### Hedef

Basit bir PHP projesi oluşturmak ve tarayıcıda çalıştırmak.

#### Adımlar

**1. Proje Dizini Oluşturun**

```bash
mkdir -p projects/hello-world/public
```

**2. stackvo.json Oluşturun**

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

**3. index.php Oluşturun**

```bash
cat > projects/hello-world/public/index.php <<'EOF'
<?php
echo "<h1>Hello, Stackvo!</h1>";
echo "<p>PHP Version: " . phpversion() . "</p>";
EOF
```

**4. Generate ve Start**

```bash
./cli/stackvo.sh generate projects
./cli/stackvo.sh up
```

**5. Hosts Dosyasını Güncelleyin**

```bash
echo "127.0.0.1  hello.loc" | sudo tee -a /etc/hosts
```

**6. Tarayıcıda Açın**

```
https://hello.loc
```

**Sonuç:** "Hello, Stackvo!" mesajını görmelisiniz.

---

### 2. MySQL ile Çalışma

Bu eğitim, MySQL veritabanına bağlanmayı ve veri eklemeyi gösterir.

#### Hedef

MySQL veritabanı kullanarak basit bir TODO uygulaması yapmak.

#### Adımlar

**1. MySQL'i Aktif Edin**

```bash
# .env dosyasını düzenle
nano .env

# MySQL'i aktif et
SERVICE_MYSQL_ENABLE=true

# Generate ve start
./cli/stackvo.sh generate
./cli/stackvo.sh up
```

**2. Veritabanı Oluşturun**

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
INSERT INTO tasks (title) VALUES ('İlk görev');
INSERT INTO tasks (title) VALUES ('İkinci görev');
EOF
```

**3. PHP Kodunu Yazın**

```bash
cat > projects/hello-world/public/todo.php <<'EOF'
<?php
$pdo = new PDO(
    'mysql:host=stackvo-mysql;port=3306;dbname=todo',
    'root',
    'root'
);

// Tüm görevleri al
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

**4. Tarayıcıda Açın**

```
https://hello.loc/todo.php
```

**Sonuç:** TODO listesini görmelisiniz.

---

### 3. Redis Cache Kullanımı

Bu eğitim, Redis ile cache kullanımını gösterir.

#### Hedef

Redis kullanarak sayfa yükleme süresini azaltmak.

#### Adımlar

**1. Redis'i Aktif Edin**

```bash
# .env dosyasını düzenle
nano .env

# Redis'i aktif et
SERVICE_REDIS_ENABLE=true

# Generate ve start
./cli/stackvo.sh generate
./cli/stackvo.sh up
```

**2. PHP Redis Extension Kontrolü**

```bash
docker exec stackvo-hello-world-php php -m | grep redis
```

**3. Cache Örneği**

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
    // Pahalı işlem simülasyonu
    sleep(2);
    $data = "Expensive calculation result: " . time();
    
    // Cache'e kaydet (60 saniye)
    $redis->setex($cacheKey, 60, $data);
    
    echo "<h1>Fresh Data (Cached for 60s)</h1>";
    echo "<p>Data: $data</p>";
}

echo "<p><a href='cache.php'>Refresh</a></p>";
EOF
```

**4. Test Edin**

```
https://hello.loc/cache.php
```

**Sonuç:** İlk yükleme 2 saniye sürer, sonraki yüklemeler anında olur.

---

## Orta Seviye

### 4. Laravel Projesi Kurulumu

Bu eğitim, Laravel projesi oluşturmayı gösterir.

#### Hedef

Tam fonksiyonel bir Laravel projesi kurmak.

#### Adımlar

**1. Composer ile Laravel Kurun**

```bash
composer create-project laravel/laravel projects/laravel-app
```

**2. stackvo.json Oluşturun**

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

**3. .env Dosyasını Düzenleyin**

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

**4. Veritabanı Oluşturun**

```bash
docker exec -it stackvo-mysql mysql -u root -proot -e "CREATE DATABASE laravel;"
```

**5. Generate ve Start**

```bash
./cli/stackvo.sh generate projects
./cli/stackvo.sh up
```

**6. Laravel Key Generate**

```bash
docker exec stackvo-laravel-app-php php artisan key:generate
```

**7. Migration Çalıştırın**

```bash
docker exec stackvo-laravel-app-php php artisan migrate
```

**8. Hosts Dosyasını Güncelleyin**

```bash
echo "127.0.0.1  laravel.loc" | sudo tee -a /etc/hosts
```

**9. Tarayıcıda Açın**

```
https://laravel.loc
```

**Sonuç:** Laravel welcome sayfasını görmelisiniz.

---

### 5. RabbitMQ ile Asenkron İşler

Bu eğitim, RabbitMQ kullanarak asenkron iş kuyruğu oluşturmayı gösterir.

#### Hedef

Email gönderme işini asenkron yapmak.

#### Adımlar

**1. RabbitMQ'yu Aktif Edin**

```bash
# .env
SERVICE_RABBITMQ_ENABLE=true

./cli/stackvo.sh generate
./cli/stackvo.sh up
```

**2. php-amqplib Kurun**

```bash
docker exec stackvo-laravel-app-php composer require php-amqplib/php-amqplib
```

**3. Producer Oluşturun**

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

**4. Consumer Oluşturun**

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
    
    // Email gönderme simülasyonu
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

**5. Worker'ı Çalıştırın**

```bash
docker exec -d stackvo-laravel-app-php php email-worker.php
```

**6. Test Edin**

```
https://laravel.loc/send-email.php
```

**Sonuç:** "Email queued successfully!" mesajı görünür, worker arka planda email'i işler.

---

## İleri Seviye

### 6. Mikroservis Mimarisi

Bu eğitim, birden fazla servisin birlikte çalıştığı bir mikroservis mimarisi oluşturmayı gösterir.

#### Hedef

API Gateway, Auth Service ve Product Service oluşturmak.

#### Mimari

```
Client → API Gateway → Auth Service
                    → Product Service → MySQL
```

#### Adımlar

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

// Auth kontrolü
$authResponse = file_get_contents("http://stackvo-auth-service-web/index.php?token=$token");
$auth = json_decode($authResponse, true);

if (!$auth['valid']) {
    http_response_code(401);
    echo json_encode(['error' => 'Unauthorized']);
    exit;
}

// Products'ı al
$productsResponse = file_get_contents("http://stackvo-product-service-web/index.php");
echo $productsResponse;
EOF
```

**4. Generate ve Start**

```bash
./cli/stackvo.sh generate projects
./cli/stackvo.sh up

echo "127.0.0.1  auth.loc products.loc api.loc" | sudo tee -a /etc/hosts
```

**5. Test Edin**

```bash
# Geçersiz token
curl -H "Authorization: invalid" https://api.loc
# {"error":"Unauthorized"}

# Geçerli token
curl -H "Authorization: valid-token" https://api.loc
# [{"id":1,"name":"Product 1","price":100},{"id":2,"name":"Product 2","price":200}]
```

---

