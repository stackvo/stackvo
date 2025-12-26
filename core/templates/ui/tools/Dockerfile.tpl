FROM php:8.2-fpm-alpine

# Install nginx, supervisor, Docker CLI, and bash (required for generate.sh)
RUN apk add --no-cache nginx supervisor docker-cli docker-cli-compose bash curl git unzip

# Conditional Java installation (only if Kafbat enabled)
{{#TOOLS_KAFBAT_ENABLE}}
RUN apk add --no-cache openjdk21-jre
{{/TOOLS_KAFBAT_ENABLE}}

# Install base PHP Extensions (always needed for PHP-based tools)
RUN apk add --no-cache postgresql-dev libzip-dev && \
    docker-php-ext-install mysqli pdo pdo_mysql opcache

# Conditional PostgreSQL extension (only if PhpPgAdmin enabled)
{{#TOOLS_PHPPGADMIN_ENABLE}}
RUN docker-php-ext-install pdo_pgsql pgsql
{{/TOOLS_PHPPGADMIN_ENABLE}}

# Conditional Redis extension (if needed by any tool)
RUN apk add --no-cache autoconf build-base openssl-dev zlib-dev && \
    pecl install redis && \
    docker-php-ext-enable redis && \
    apk del autoconf build-base

# Conditional Memcached extension (only if PhpMemcachedAdmin enabled)
{{#TOOLS_PHPMEMCACHEDADMIN_ENABLE}}
RUN apk add --no-cache autoconf build-base libmemcached-dev zlib-dev openssl-dev && \
    pecl install memcached && \
    docker-php-ext-enable memcached && \
    apk del autoconf build-base
{{/TOOLS_PHPMEMCACHEDADMIN_ENABLE}}

# Conditional MongoDB extension (only if PhpMongo enabled)
{{#TOOLS_PHPMONGO_ENABLE}}
RUN apk add --no-cache autoconf build-base openssl-dev && \
    pecl install mongodb && \
    docker-php-ext-enable mongodb && \
    apk del autoconf build-base
{{/TOOLS_PHPMONGO_ENABLE}}

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

{{#TOOLS_ADMINER_ENABLE}}
# --- 1. Adminer v{{ TOOLS_ADMINER_VERSION }} ---
RUN mkdir -p adminer && \
    curl -L -o adminer/index.php https://github.com/vrana/adminer/releases/download/v{{ TOOLS_ADMINER_VERSION }}/adminer-{{ TOOLS_ADMINER_VERSION }}.php

{{/TOOLS_ADMINER_ENABLE}}
{{#TOOLS_PHPMYADMIN_ENABLE}}
# --- 2. PhpMyAdmin v{{ TOOLS_PHPMYADMIN_VERSION }} ---
RUN curl -L https://files.phpmyadmin.net/phpMyAdmin/{{ TOOLS_PHPMYADMIN_VERSION }}/phpMyAdmin-{{ TOOLS_PHPMYADMIN_VERSION }}-all-languages.zip -o pma.zip && \
    unzip pma.zip && \
    mv phpMyAdmin-{{ TOOLS_PHPMYADMIN_VERSION }}-all-languages phpmyadmin && \
    rm pma.zip && \
    cp phpmyadmin/config.sample.inc.php phpmyadmin/config.inc.php && \
    sed -i "s/\$cfg\['blowfish_secret'\] = '';/\$cfg\['blowfish_secret'\] = 'stackvo-secret-key-12345678901234567890';/" phpmyadmin/config.inc.php && \
    echo "\$cfg['Servers'][\$i]['host'] = 'stackvo-mysql';" >> phpmyadmin/config.inc.php && \
    echo "\$cfg['Servers'][\$i]['port'] = '3306';" >> phpmyadmin/config.inc.php && \
    echo "\$cfg['Servers'][\$i]['auth_type'] = 'cookie';" >> phpmyadmin/config.inc.php && \
    echo "\$cfg['AllowArbitraryServer'] = true;" >> phpmyadmin/config.inc.php

{{/TOOLS_PHPMYADMIN_ENABLE}}
{{#TOOLS_PHPMEMCACHEDADMIN_ENABLE}}
# --- 3. PhpMemcachedAdmin v{{ TOOLS_PHPMEMCACHEDADMIN_VERSION }} ---
RUN mkdir -p phpmemcachedadmin && \
    curl -L https://github.com/elijaa/phpmemcachedadmin/archive/refs/heads/master.zip -o pma_mem.zip && \
    unzip pma_mem.zip && \
    mv phpmemcachedadmin-master/* phpmemcachedadmin/ && \
    rm -rf phpmemcachedadmin-master pma_mem.zip

{{/TOOLS_PHPMEMCACHEDADMIN_ENABLE}}
{{#TOOLS_OPCACHE_ENABLE}}
# --- 4. OpCacheGUI v{{ TOOLS_OPCACHE_VERSION }} ---
RUN mkdir -p opcache && \
    curl -L https://raw.githubusercontent.com/amnuts/opcache-gui/master/index.php -o opcache/index.php

{{/TOOLS_OPCACHE_ENABLE}}
{{#TOOLS_PHPMONGO_ENABLE}}
# Install Composer (only if MongoDB-PHP-GUI enabled)
RUN curl -sS https://getcomposer.org/installer | php -- --install-dir=/usr/local/bin --filename=composer

# --- 5. MongoDB-PHP-GUI v{{ TOOLS_PHPMONGO_VERSION }} ---
RUN git clone https://github.com/SamuelTallet/MongoDB-PHP-GUI.git phpmongo && \
    cd phpmongo && \
    composer install --no-dev --optimize-autoloader && \
    echo "<?php" > config.php && \
    echo "define('MONGO_HOST', 'stackvo-mongo');" >> config.php && \
    echo "define('MONGO_PORT', 27017);" >> config.php && \
    echo "define('MONGO_USER', 'root');" >> config.php && \
    echo "define('MONGO_PASS', 'root');" >> config.php && \
    echo "define('MONGO_AUTH_DB', 'admin');" >> config.php

{{/TOOLS_PHPMONGO_ENABLE}}
{{#TOOLS_PHPPGADMIN_ENABLE}}
# --- 6. PhpPgAdmin v{{ TOOLS_PHPPGADMIN_VERSION }} ---
RUN curl -L https://github.com/phppgadmin/phppgadmin/releases/download/REL_7-13-0/phpPgAdmin-{{ TOOLS_PHPPGADMIN_VERSION }}.zip -o ppa.zip && \
    unzip ppa.zip && \
    mv phpPgAdmin-{{ TOOLS_PHPPGADMIN_VERSION }} phppgadmin && \
    rm ppa.zip && \
    cp phppgadmin/conf/config.inc.php-dist phppgadmin/conf/config.inc.php && \
    sed -i "s/\$conf\['servers'\]\[0\]\['host'\] = '';/\$conf\['servers'\]\[0\]\['host'\] = 'stackvo-postgres';/" phppgadmin/conf/config.inc.php

{{/TOOLS_PHPPGADMIN_ENABLE}}
{{#TOOLS_KAFBAT_ENABLE}}
# --- 7. Kafbat Kafka UI v{{ TOOLS_KAFBAT_VERSION }} ---
COPY --from=ghcr.io/kafbat/kafka-ui:v{{ TOOLS_KAFBAT_VERSION }} /api.jar /opt/kafbat/kafka-ui.jar

{{/TOOLS_KAFBAT_ENABLE}}
# Permissions
RUN chown -R www-data:www-data /var/www/html

# Copy Supervisord configuration
COPY supervisord.conf /etc/supervisord.conf

EXPOSE 80 8080

# Start all services with Supervisord
CMD ["/usr/bin/supervisord", "-c", "/etc/supervisord.conf"]
