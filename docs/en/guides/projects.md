---
title: Project Management
description: Step-by-step description of creating, configuring, and managing projects with Stackvo.
---

# Project Management

This guide explains step-by-step how to create, configure, and manage projects with Stackvo. It covers all processes from creating a new project to framework projects like Laravel, Symfony, WordPress, from webserver selection to multi-language support, from custom configurations to project update and deletion operations. stackvo.json configuration and best practices are explained for each project.

---

## Creating a New Project

### Creating a Project via CLI

#### 1. Create Project Directory

```bash
# Create project directory and document root
mkdir -p projects/myproject/public
```

#### 2. Create stackvo.json

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

#### 3. Create Test File

```bash
# Simple PHP file
echo "<?php phpinfo();" > projects/myproject/public/index.php

# Or HTML file
echo "<h1>Welcome to My Project</h1>" > projects/myproject/public/index.html
```

#### 4. Run Generator

```bash
# Generate only projects
./stackvo.sh generate projects

# Or generate all
./stackvo.sh generate
```

#### 5. Start Services

```bash
./stackvo.sh up
```

#### 6. Update Hosts File

```bash
# Linux/macOS
echo "127.0.0.1  myproject.loc" | sudo tee -a /etc/hosts

# Windows (PowerShell - Run as Administrator)
Add-Content C:\Windows\System32\drivers\etc\hosts "127.0.0.1  myproject.loc"
```

#### 7. Open in Browser

```
https://myproject.loc
```

---

## Framework Projects

### Laravel Project

```bash
# 1. Install Laravel via Composer
composer create-project laravel/laravel projects/laravel-app

# 2. Create stackvo.json
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

# 3. Edit .env file
nano projects/laravel-app/.env

# Database settings
# DB_CONNECTION=mysql
# DB_HOST=stackvo-mysql
# DB_PORT=3306
# DB_DATABASE=laravel
# DB_USERNAME=stackvo
# DB_PASSWORD=stackvo

# Cache settings
# CACHE_DRIVER=redis
# REDIS_HOST=stackvo-redis
# REDIS_PORT=6379

# 4. Generate and start
./stackvo.sh generate projects
./stackvo.sh up

# 5. Hosts
echo "127.0.0.1  laravel.loc" | sudo tee -a /etc/hosts

# 6. Laravel key generate
docker exec stackvo-laravel-app-php php artisan key:generate

# 7. Migration
docker exec stackvo-laravel-app-php php artisan migrate
```

### Symfony Project

```bash
# 1. Create project via Symfony CLI
symfony new projects/symfony-app --webapp

# 2. Create stackvo.json
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

# 3. Create .env.local
cat > projects/symfony-app/.env.local <<EOF
DATABASE_URL="postgresql://stackvo:root@stackvo-postgres:5432/symfony?serverVersion=14&charset=utf8"
EOF

# 4. Generate and start
./stackvo.sh generate projects
./stackvo.sh up

# 5. Hosts
echo "127.0.0.1  symfony.loc" | sudo tee -a /etc/hosts
```

### WordPress Project

```bash
# 1. Download WordPress
wget https://wordpress.org/latest.tar.gz
tar -xzf latest.tar.gz
mv wordpress projects/wordpress-site

# 2. Create stackvo.json
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

# 3. Create wp-config.php
cp projects/wordpress-site/wp-config-sample.php projects/wordpress-site/wp-config.php
nano projects/wordpress-site/wp-config.php

# Database settings
# define('DB_NAME', 'wordpress');
# define('DB_USER', 'stackvo');
# define('DB_PASSWORD', 'stackvo');
# define('DB_HOST', 'stackvo-mysql');

# 4. Generate and start
./stackvo.sh generate projects
./stackvo.sh up

# 5. Hosts
echo "127.0.0.1  wordpress.loc" | sudo tee -a /etc/hosts

# 6. Complete installation
# Open https://wordpress.loc
```

---

## Webserver Selection

### Nginx (Default)

```json
{
  "webserver": "nginx"
}
```

**Advantages:**
- High performance
- Low memory usage
- Reverse proxy support

### Apache

```json
{
  "webserver": "apache"
}
```

**Advantages:**
- .htaccess support
- Modular structure
- Large community

### Caddy

```json
{
  "webserver": "caddy"
}
```

**Advantages:**
- Automatic HTTPS
- Modern configuration
- HTTP/3 support

### Ferron

```json
{
  "webserver": "ferron"
}
```

**Advantages:**
- Lightweight and fast
- YAML configuration
- Modern architecture

---

## Custom Configurations

### Nginx Configuration

```bash
# 1. Create .stackvo directory
mkdir -p projects/myproject/.stackvo

# 2. Create nginx.conf
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

# 3. Generate and restart
./stackvo.sh generate projects
./stackvo.sh restart
```

### PHP Configuration

```bash
# Create php.ini
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

# Generate and restart
./stackvo.sh generate projects
docker restart stackvo-myproject-php
```

---

## Multi-Language Support

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

## Updating Projects

### Changing Configuration

```bash
# 1. Edit stackvo.json
nano projects/myproject/stackvo.json

# Change PHP version
# "version": "8.2" → "8.3"

# 2. Regenerate projects
./stackvo.sh generate projects

# 3. Recreate container
docker compose -f generated/stackvo.yml \
  -f generated/docker-compose.dynamic.yml \
  -f generated/docker-compose.projects.yml \
  up -d --force-recreate stackvo-myproject-php
```

### Changing Webserver

```bash
# 1. Edit stackvo.json
nano projects/myproject/stackvo.json

# "webserver": "nginx" → "apache"

# 2. Regenerate projects
./stackvo.sh generate projects

# 3. Recreate containers
./stackvo.sh restart
```

---

## Deleting Projects

### Via CLI

```bash
# 1. Stop containers
./stackvo.sh down

# 2. Delete project directory
rm -rf projects/myproject

# 3. Regenerate
./stackvo.sh generate projects

# 4. Start
./stackvo.sh up

# 5. Remove from hosts file
sudo nano /etc/hosts
# Remove line: 127.0.0.1  myproject.loc
```

---

## Troubleshooting

### Container Not Starting

```bash
# Check logs
docker logs stackvo-myproject-web
docker logs stackvo-myproject-php

# Check stackvo.json
cat projects/myproject/stackvo.json

# Syntax check
docker exec stackvo-myproject-web nginx -t
```

### 404 Error

```bash
# Document root check
docker exec stackvo-myproject-web ls -la /var/www/html/public

# Nginx configuration check
docker exec stackvo-myproject-web cat /etc/nginx/conf.d/default.conf

# PHP-FPM connection check
docker exec stackvo-myproject-web nc -zv myproject-php 9000
```

### Permission Error

```bash
# Fix file ownership
sudo chown -R $USER:$USER projects/myproject

# Inside container
docker exec stackvo-myproject-php chown -R www-data:www-data /var/www/html
```
