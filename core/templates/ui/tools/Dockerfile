FROM php:8.2-fpm-alpine

# Install nginx, supervisor, Docker CLI, and bash (required for generate.sh)
RUN apk add --no-cache nginx supervisor docker-cli docker-cli-compose bash curl git unzip

# Conditional Java installation (only if Kafbat enabled)


# Install base PHP Extensions (always needed for PHP-based tools)
RUN apk add --no-cache postgresql-dev libzip-dev && \
    docker-php-ext-install mysqli pdo pdo_mysql opcache

# Conditional PostgreSQL extension (only if PhpPgAdmin enabled)


# Conditional Redis extension (if needed by any tool)
RUN apk add --no-cache autoconf build-base openssl-dev zlib-dev && \
    pecl install redis && \
    docker-php-ext-enable redis && \
    apk del autoconf build-base

# Conditional Memcached extension (only if PhpMemcachedAdmin enabled)


# Conditional MongoDB extension (only if PhpMongo enabled)


# Configure Nginx
COPY nginx.conf /etc/nginx/nginx.conf
RUN mkdir -p /run/nginx

# Configure PHP (Suppress all warnings for legacy tools)
RUN echo "error_reporting = E_ERROR | E_PARSE" > /usr/local/etc/php/conf.d/error_reporting.ini && \
    echo "display_errors = Off" >> /usr/local/etc/php/conf.d/error_reporting.ini && \
    echo "log_errors = On" >> /usr/local/etc/php/conf.d/error_reporting.ini

# Setup Web Root
WORKDIR /var/www/html

# --- CONDITIONAL TOOL INSTALLATIONS ---


# --- 2. PhpMyAdmin v5.2.1 ---
RUN curl -L https://files.phpmyadmin.net/phpMyAdmin/5.2.1/phpMyAdmin-5.2.1-all-languages.zip -o pma.zip && \
    unzip pma.zip && \
    mv phpMyAdmin-5.2.1-all-languages phpmyadmin && \
    rm pma.zip && \
    cp phpmyadmin/config.sample.inc.php phpmyadmin/config.inc.php && \
    sed -i "s/\$cfg\['blowfish_secret'\] = '';/\$cfg\['blowfish_secret'\] = 'stackvo-secret-key-12345678901234567890';/" phpmyadmin/config.inc.php && \
    echo "\$cfg['Servers'][\$i]['host'] = 'stackvo-mysql';" >> phpmyadmin/config.inc.php && \
    echo "\$cfg['Servers'][\$i]['port'] = '3306';" >> phpmyadmin/config.inc.php && \
    echo "\$cfg['Servers'][\$i]['auth_type'] = 'cookie';" >> phpmyadmin/config.inc.php && \
    echo "\$cfg['AllowArbitraryServer'] = true;" >> phpmyadmin/config.inc.php






# Permissions
RUN chown -R www-data:www-data /var/www/html

# Copy Supervisord configuration
COPY supervisord.conf /etc/supervisord.conf

EXPOSE 80 8080

# Start all services with Supervisord
CMD ["/usr/bin/supervisord", "-c", "/etc/supervisord.conf"]
