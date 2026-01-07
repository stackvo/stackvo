---
title: Proje Yönetimi
description: Stackvo ile proje oluşturma, yapılandırma ve yönetme işlemlerini adım adım açıklar.
---

# Proje Yönetimi

Bu kılavuz, Stackvo ile proje oluşturma, yapılandırma ve yönetme işlemlerini adım adım açıklamaktadır. Yeni proje oluşturmadan Laravel, Symfony, WordPress gibi framework projelerine, webserver seçiminden çoklu dil desteğine, özel konfigürasyonlardan proje güncelleme ve silme işlemlerine kadar tüm süreçleri kapsamaktadır. Her proje için stackvo.json konfigürasyonu ve best practices açıklanmaktadır.

---

## Yeni Proje Oluşturma

### CLI ile Proje Oluşturma

#### 1. Proje Dizini Oluşturma

```bash
# Proje dizini ve document root oluştur
mkdir -p projects/myproject/public
```

#### 2. stackvo.json Oluşturma

```bash
cat > projects/myproject/stackvo.json <<EOF
{
  "name": "myproject",
  "domain": "myproject.loc",
  "php": {
    "version": "8.2",
    "extensions": [
      "pdo",
      "pdo_mysql",
      "mysqli",
      "gd",
      "curl",
      "zip",
      "mbstring"
    ]
  },
  "webserver": "nginx",
  "document_root": "public"
}
EOF
```

#### 3. Test Dosyası Oluşturma

```bash
# Basit PHP dosyası
echo "<?php phpinfo();" > projects/myproject/public/index.php

# Veya HTML dosyası
echo "<h1>Welcome to My Project</h1>" > projects/myproject/public/index.html
```

#### 4. Generator Çalıştırma

```bash
# Sadece projeleri üret
./core/cli/stackvo.sh generate projects

# Veya tümünü üret
./core/cli/stackvo.sh generate
```

#### 5. Servisleri Başlatma

```bash
./core/cli/stackvo.sh up
```

#### 6. Hosts Dosyasını Güncelleme

```bash
# Linux/macOS
echo "127.0.0.1  myproject.loc" | sudo tee -a /etc/hosts

# Windows (PowerShell - Yönetici olarak)
Add-Content C:\Windows\System32\drivers\etc\hosts "127.0.0.1  myproject.loc"
```

#### 7. Tarayıcıda Açma

```
https://myproject.loc
```

---

## Framework Projeleri

### Laravel Projesi

```bash
# 1. Composer ile Laravel kur
composer create-project laravel/laravel projects/laravel-app

# 2. stackvo.json oluştur
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

# 3. .env dosyasını düzenle
nano projects/laravel-app/.env

# Database ayarları
# DB_CONNECTION=mysql
# DB_HOST=stackvo-mysql
# DB_PORT=3306
# DB_DATABASE=laravel
# DB_USERNAME=stackvo
# DB_PASSWORD=stackvo

# Cache ayarları
# CACHE_DRIVER=redis
# REDIS_HOST=stackvo-redis
# REDIS_PORT=6379

# 4. Generate ve start
./core/cli/stackvo.sh generate projects
./core/cli/stackvo.sh up

# 5. Hosts
echo "127.0.0.1  laravel.loc" | sudo tee -a /etc/hosts

# 6. Laravel key generate
docker exec stackvo-laravel-app-php php artisan key:generate

# 7. Migration
docker exec stackvo-laravel-app-php php artisan migrate
```

### Symfony Projesi

```bash
# 1. Symfony CLI ile proje oluştur
symfony new projects/symfony-app --webapp

# 2. stackvo.json oluştur
cat > projects/symfony-app/stackvo.json <<EOF
{
  "name": "symfony-app",
  "domain": "symfony.loc",
  "php": {
    "version": "8.3",
    "extensions": [
      "pdo",
      "pdo_pgsql",
      "intl",
      "xml",
      "curl",
      "zip",
      "mbstring",
      "opcache"
    ]
  },
  "webserver": "nginx",
  "document_root": "public"
}
EOF

# 3. .env.local oluştur
cat > projects/symfony-app/.env.local <<EOF
DATABASE_URL="postgresql://stackvo:root@stackvo-postgres:5432/symfony?serverVersion=14&charset=utf8"
EOF

# 4. Generate ve start
./core/cli/stackvo.sh generate projects
./core/cli/stackvo.sh up

# 5. Hosts
echo "127.0.0.1  symfony.loc" | sudo tee -a /etc/hosts
```

### WordPress Projesi

```bash
# 1. WordPress indir
wget https://wordpress.org/latest.tar.gz
tar -xzf latest.tar.gz
mv wordpress projects/wordpress-site

# 2. stackvo.json oluştur
cat > projects/wordpress-site/stackvo.json <<EOF
{
  "name": "wordpress-site",
  "domain": "wordpress.loc",
  "php": {
    "version": "8.1",
    "extensions": [
      "mysqli",
      "pdo",
      "pdo_mysql",
      "gd",
      "curl",
      "zip",
      "mbstring",
      "xml",
      "imagick"
    ]
  },
  "webserver": "apache",
  "document_root": "."
}
EOF

# 3. wp-config.php oluştur
cp projects/wordpress-site/wp-config-sample.php projects/wordpress-site/wp-config.php
nano projects/wordpress-site/wp-config.php

# Database ayarları
# define('DB_NAME', 'wordpress');
# define('DB_USER', 'stackvo');
# define('DB_PASSWORD', 'stackvo');
# define('DB_HOST', 'stackvo-mysql');

# 4. Generate ve start
./core/cli/stackvo.sh generate projects
./core/cli/stackvo.sh up

# 5. Hosts
echo "127.0.0.1  wordpress.loc" | sudo tee -a /etc/hosts

# 6. Kurulumu tamamla
# https://wordpress.loc adresini açın
```

---

## Webserver Seçimi

### Nginx (Varsayılan)

```json
{
  "webserver": "nginx"
}
```

**Avantajlar:**
- Yüksek performans
- Düşük bellek kullanımı
- Reverse proxy desteği

### Apache

```json
{
  "webserver": "apache"
}
```

**Avantajlar:**
- .htaccess desteği
- Modüler yapı
- Geniş topluluk

### Caddy

```json
{
  "webserver": "caddy"
}
```

**Avantajlar:**
- Otomatik HTTPS
- Modern konfigürasyon
- HTTP/3 desteği

### Ferron

```json
{
  "webserver": "ferron"
}
```

**Avantajlar:**
- Hafif ve hızlı
- YAML konfigürasyon
- Modern mimari

---

## Özel Konfigürasyonlar

### Nginx Konfigürasyonu

```bash
# 1. .stackvo dizini oluştur
mkdir -p projects/myproject/.stackvo

# 2. nginx.conf oluştur
cat > projects/myproject/.stackvo/nginx.conf <<'EOF'
server {
    listen 80;
    server_name myproject.loc;
    root /var/www/html/public;
    index index.php index.html;

    location / {
        try_files $uri $uri/ /index.php?$query_string;
    }

    location ~ \.php$ {
        fastcgi_pass myproject-php:9000;
        fastcgi_index index.php;
        fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;
        include fastcgi_params;
    }
}
EOF

# 3. Generate ve restart
./core/cli/stackvo.sh generate projects
./core/cli/stackvo.sh restart
```

### PHP Konfigürasyonu

```bash
# php.ini oluştur
cat > projects/myproject/.stackvo/php.ini <<EOF
memory_limit = 256M
upload_max_filesize = 64M
post_max_size = 64M
max_execution_time = 300
display_errors = On
error_reporting = E_ALL
date.timezone = Europe/Istanbul

opcache.enable = 1
opcache.memory_consumption = 128

session.save_handler = redis
session.save_path = "tcp://stackvo-redis:6379"
EOF

# Generate ve restart
./core/cli/stackvo.sh generate projects
docker restart stackvo-myproject-php
```

---

## Çoklu Dil Desteği

### PHP + Node.js

```json
{
  "name": "fullstack-app",
  "domain": "fullstack.loc",
  "php": {
    "version": "8.3"
  },
  "nodejs": {
    "version": "14.23"
  },
  "webserver": "nginx",
  "document_root": "public"
}
```

### PHP + Python

```json
{
  "name": "ml-app",
  "domain": "ml.loc",
  "php": {
    "version": "8.2"
  },
  "python": {
    "version": "3.14"
  },
  "webserver": "nginx",
  "document_root": "public"
}
```

---

## Proje Güncelleme

### Konfigürasyon Değiştirme

```bash
# 1. stackvo.json düzenle
nano projects/myproject/stackvo.json

# PHP versiyonunu değiştir
# "version": "8.2" → "8.3"

# 2. Projeleri yeniden üret
./core/cli/stackvo.sh generate projects

# 3. Container'ı yeniden oluştur
docker compose -f generated/stackvo.yml \
  -f generated/docker-compose.dynamic.yml \
  -f generated/docker-compose.projects.yml \
  up -d --force-recreate stackvo-myproject-php
```

### Webserver Değiştirme

```bash
# 1. stackvo.json düzenle
nano projects/myproject/stackvo.json

# "webserver": "nginx" → "apache"

# 2. Projeleri yeniden üret
./core/cli/stackvo.sh generate projects

# 3. Container'ları yeniden oluştur
./core/cli/stackvo.sh restart
```

---

## Proje Silme

### CLI ile

```bash
# 1. Container'ları durdur
./core/cli/stackvo.sh down

# 2. Proje dizinini sil
rm -rf projects/myproject

# 3. Yeniden üret
./core/cli/stackvo.sh generate projects

# 4. Başlat
./core/cli/stackvo.sh up

# 5. Hosts dosyasından kaldır
sudo nano /etc/hosts
# 127.0.0.1  myproject.loc satırını sil
```

---

## Troubleshooting

### Container Başlamıyor

```bash
# Logları kontrol et
docker logs stackvo-myproject-web
docker logs stackvo-myproject-php

# stackvo.json'ı kontrol et
cat projects/myproject/stackvo.json

# Syntax kontrolü
docker exec stackvo-myproject-web nginx -t
```

### 404 Hatası

```bash
# Document root kontrolü
docker exec stackvo-myproject-web ls -la /var/www/html/public

# Nginx konfigürasyonu kontrolü
docker exec stackvo-myproject-web cat /etc/nginx/conf.d/default.conf

# PHP-FPM bağlantısı kontrolü
docker exec stackvo-myproject-web nc -zv myproject-php 9000
```

### Permission Hatası

```bash
# Dosya sahipliğini düzelt
sudo chown -R $USER:$USER projects/myproject

# Container içinde
docker exec stackvo-myproject-php chown -R www-data:www-data /var/www/html
```

---

