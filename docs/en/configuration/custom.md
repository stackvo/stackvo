# Custom Configurations

You can create custom webserver and runtime configurations for each project in the `.stackvo/` directory. This page details how to create custom configuration files for webservers like Nginx, Apache, Caddy, how to customize PHP and PHP-FPM settings, and configuration priority order. Custom configurations override auto-generated settings.

---

## Configuration Priority

The generator searches for configuration files in the following order:

1. **Custom configuration:** `projects/myproject/.stackvo/nginx.conf`
2. **In Project root:** `projects/myproject/nginx.conf`
3. **Auto-generated:** `core/generated/configs/myproject-nginx.conf`

---

## .stackvo/ Directory

```
projects/myproject/
├── .stackvo/             # Custom configurations (optional)
│   ├── nginx.conf          # Custom Nginx configuration
│   ├── apache.conf         # Custom Apache configuration
│   ├── Caddyfile           # Custom Caddy configuration
│   ├── ferron.yaml         # Custom Ferron configuration
│   ├── php.ini             # Custom PHP configuration
│   └── php-fpm.conf        # Custom PHP-FPM configuration
├── stackvo.json
└── public/
    └── index.php
```

---

## Nginx Configuration

### Creating Custom Nginx Config

```bash
mkdir -p projects/myproject/.stackvo
nano projects/myproject/.stackvo/nginx.conf
```

### Example Nginx Configuration

```nginx
server {
    listen 80;
    server_name myproject.loc;
    root /var/www/html/public;
    index index.php index.html;

    # Logging
    access_log /var/log/nginx/access.log;
    error_log /var/log/nginx/error.log;

    # Gzip compression
    gzip on;
    gzip_types text/plain text/css application/json application/javascript text/xml application/xml;

    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;

    location / {
        try_files $uri $uri/ /index.php?$query_string;
    }

    location ~ \.php$ {
        fastcgi_pass myproject-php:9000;
        fastcgi_index index.php;
        fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;
        include fastcgi_params;
        
        # PHP settings
        fastcgi_read_timeout 300;
        fastcgi_buffer_size 128k;
        fastcgi_buffers 4 256k;
    }

    location ~ /\.ht {
        deny all;
    }

    # Cache static files
    location ~* \.(jpg|jpeg|png|gif|ico|css|js|svg|woff|woff2|ttf|eot)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
}
```

### Nginx for Laravel

```nginx
server {
    listen 80;
    server_name laravel.loc;
    root /var/www/html/public;
    index index.php;

    add_header X-Frame-Options "SAMEORIGIN";
    add_header X-Content-Type-Options "nosniff";

    charset utf-8;

    location / {
        try_files $uri $uri/ /index.php?$query_string;
    }

    location = /favicon.ico { access_log off; log_not_found off; }
    location = /robots.txt  { access_log off; log_not_found off; }

    error_page 404 /index.php;

    location ~ \.php$ {
        fastcgi_pass laravel-php:9000;
        fastcgi_param SCRIPT_FILENAME $realpath_root$fastcgi_script_name;
        include fastcgi_params;
    }

    location ~ /\.(?!well-known).* {
        deny all;
    }
}
```

---

## Apache Configuration

### Creating Custom Apache Config

```bash
mkdir -p projects/myproject/.stackvo
nano projects/myproject/.stackvo/apache.conf
```

### Example Apache Configuration

```apache
<VirtualHost *:80>
    ServerName myproject.loc
    DocumentRoot /var/www/html/public

    <Directory /var/www/html/public>
        Options Indexes FollowSymLinks
        AllowOverride All
        Require all granted
    </Directory>

    # Logging
    ErrorLog /var/log/apache2/error.log
    CustomLog /var/log/apache2/access.log combined

    # Security headers
    Header always set X-Frame-Options "SAMEORIGIN"
    Header always set X-Content-Type-Options "nosniff"
    Header always set X-XSS-Protection "1; mode=block"

    # Compression
    <IfModule mod_deflate.c>
        AddOutputFilterByType DEFLATE text/html text/plain text/xml text/css text/javascript application/javascript
    </IfModule>

    # Cache static files
    <IfModule mod_expires.c>
        ExpiresActive On
        ExpiresByType image/jpg "access plus 1 year"
        ExpiresByType image/jpeg "access plus 1 year"
        ExpiresByType image/gif "access plus 1 year"
        ExpiresByType image/png "access plus 1 year"
        ExpiresByType text/css "access plus 1 month"
        ExpiresByType application/javascript "access plus 1 month"
    </IfModule>
</VirtualHost>
```

---

## Caddy Configuration

### Creating Custom Caddyfile

```bash
mkdir -p projects/myproject/.stackvo
nano projects/myproject/.stackvo/Caddyfile
```

### Example Caddyfile

```caddy
:80 {
    root * /var/www/html/public
    encode gzip

    # PHP-FPM
    php_fastcgi myproject-php:9000

    # File server
    file_server

    # Security headers
    header {
        X-Frame-Options "SAMEORIGIN"
        X-Content-Type-Options "nosniff"
        X-XSS-Protection "1; mode=block"
    }

    # Logging
    log {
        output file /var/log/caddy/access.log
    }
}
```

---

## PHP Configuration

### Creating Custom php.ini

```bash
mkdir -p projects/myproject/.stackvo
nano projects/myproject/.stackvo/php.ini
```

### Example php.ini (Development)

```ini
; Memory
memory_limit = 256M

; Upload
upload_max_filesize = 64M
post_max_size = 64M

; Execution
max_execution_time = 300
max_input_time = 300

; Error reporting
display_errors = On
display_startup_errors = On
error_reporting = E_ALL

; Timezone
date.timezone = Europe/Istanbul

; OPcache
opcache.enable = 1
opcache.memory_consumption = 128
opcache.interned_strings_buffer = 8
opcache.max_accelerated_files = 10000
opcache.revalidate_freq = 2
opcache.fast_shutdown = 1

; Session
session.save_handler = redis
session.save_path = "tcp://stackvo-redis:6379"

; Extensions
extension=pdo.so
extension=pdo_mysql.so
extension=mysqli.so
extension=redis.so
extension=gd.so
extension=curl.so
extension=zip.so
extension=mbstring.so
```

### Production php.ini

```ini
memory_limit = 512M
upload_max_filesize = 128M
post_max_size = 128M
max_execution_time = 60

; Security
display_errors = Off
display_startup_errors = Off
error_reporting = E_ALL & ~E_DEPRECATED & ~E_STRICT
log_errors = On
error_log = /var/log/php/error.log

; OPcache (production optimized)
opcache.enable = 1
opcache.memory_consumption = 256
opcache.interned_strings_buffer = 16
opcache.max_accelerated_files = 20000
opcache.revalidate_freq = 0
opcache.validate_timestamps = 0
opcache.fast_shutdown = 1
```

---

## PHP-FPM Configuration

### Creating Custom php-fpm.conf

```bash
mkdir -p projects/myproject/.stackvo
nano projects/myproject/.stackvo/php-fpm.conf
```

### Example php-fpm.conf

```ini
[www]
user = www-data
group = www-data

listen = 9000

; Process manager
pm = dynamic
pm.max_children = 50
pm.start_servers = 5
pm.min_spare_servers = 5
pm.max_spare_servers = 35
pm.max_requests = 500

; Logging
php_admin_value[error_log] = /var/log/php-fpm/error.log
php_admin_flag[log_errors] = on

; Timeouts
request_terminate_timeout = 300

; Slow log
slowlog = /var/log/php-fpm/slow.log
request_slowlog_timeout = 10s

; Environment variables
env[HOSTNAME] = $HOSTNAME
env[PATH] = /usr/local/bin:/usr/bin:/bin
env[TMP] = /tmp
env[TMPDIR] = /tmp
env[TEMP] = /tmp
```

---

## Applying Configurations

After creating custom configurations:

```bash
# 1. Regenerate projects
./stackvo.sh generate projects

# 2. Restart containers
./stackvo.sh restart

# 3. Or restart only the relevant project
docker restart stackvo-myproject-web
docker restart stackvo-myproject-php
```

---

## Configuration Verification

### Nginx Syntax Check

```bash
docker exec stackvo-myproject-web nginx -t
```

### Apache Syntax Check

```bash
docker exec stackvo-myproject-web apachectl configtest
```

### PHP Config Check

```bash
docker exec stackvo-myproject-php php -i | grep "Configuration File"
docker exec stackvo-myproject-php php -i | grep "memory_limit"
```

---

## Troubleshooting

### Configuration Not Loading

```bash
# Check volume mounts
docker inspect stackvo-myproject-web | grep Mounts -A 20

# Check file permissions
ls -la projects/myproject/.stackvo/
```

### Syntax Error

```bash
# Check logs
docker logs stackvo-myproject-web
docker logs stackvo-myproject-php

# Enter container
docker exec -it stackvo-myproject-web sh
cat /etc/nginx/conf.d/default.conf
```