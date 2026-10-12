# syntax=docker/dockerfile:1.4

# Auto-generated Dockerfile for probe-swoole
# Server: Swoole
# PHP Version: 8.4
# Note: Uses php-cli image - Swoole IS the HTTP server
FROM php:8.4-cli

# Install System Packages with BuildKit cache
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && apt-get install -y \
    libcurl4-openssl-dev \
    libfreetype6-dev \
    libicu-dev \
    libjpeg-dev \
    libonig-dev \
    libpng-dev \
    libssl-dev \
    libzip-dev \
    && rm -rf /var/lib/apt/lists/*

RUN docker-php-ext-configure gd --with-freetype --with-jpeg

# Install PHP extensions
RUN docker-php-ext-install pdo
RUN docker-php-ext-install pdo_mysql
RUN docker-php-ext-install mbstring curl zip gd intl bcmath pcntl

# Install PECL extensions
# Note: Versions are pinned for stability; each install is retried because
# pecl.php.net occasionally fails to serve package metadata.

RUN --mount=type=cache,target=/tmp/pear,sharing=locked \
    for i in 1 2 3; do pecl install igbinary-3.2.16 && break; \
      [ "$i" = 3 ] && exit 1; echo "pecl install failed (attempt $i/3), retrying..."; sleep $((i * 5)); done \
    && docker-php-ext-enable igbinary

RUN --mount=type=cache,target=/tmp/pear,sharing=locked \
    for i in 1 2 3; do pecl install redis-6.3.0 && break; \
      [ "$i" = 3 ] && exit 1; echo "pecl install failed (attempt $i/3), retrying..."; sleep $((i * 5)); done \
    && docker-php-ext-enable redis

RUN --mount=type=cache,target=/tmp/pear,sharing=locked \
    for i in 1 2 3; do pecl install xdebug-3.4.0 && break; \
      [ "$i" = 3 ] && exit 1; echo "pecl install failed (attempt $i/3), retrying..."; sleep $((i * 5)); done \
    && docker-php-ext-enable xdebug

RUN --mount=type=cache,target=/tmp/pear,sharing=locked \
    for i in 1 2 3; do pecl install swoole-6.1.6 && break; \
      [ "$i" = 3 ] && exit 1; echo "pecl install failed (attempt $i/3), retrying..."; sleep $((i * 5)); done \
    && docker-php-ext-enable swoole


# Install System Packages
RUN apt-get update && apt-get install -y \
    git \
    wget \
    unzip \
    default-mysql-client \
    postgresql-client \
    redis-tools \
    strace \
    vim \
    nano \
    curl \
    iputils-ping \
    net-tools \
    telnet \
    htop \
    procps \
    tar \
    gzip \
    bzip2 \
    p7zip-full \
    && rm -rf /var/lib/apt/lists/*


# Install Development Tools
# Install Composer
COPY --from=composer:latest /usr/bin/composer /usr/bin/composer

# Install Node.js 20.x
RUN curl -fsSL --retry 5 --retry-all-errors --retry-delay 3 -o /tmp/nodesource_setup.sh https://deb.nodesource.com/setup_20.x \
    && bash /tmp/nodesource_setup.sh \
    && rm -f /tmp/nodesource_setup.sh \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*


# Expose Swoole port
EXPOSE 8000

# Swoole fallback server for non-Laravel projects
RUN { \
    echo '<?php'; \
    echo '$http = new Swoole\\HTTP\\Server("0.0.0.0", 8000);'; \
    echo '$http->set(["document_root" => "/var/www/html/public", "enable_static_handler" => true]);'; \
    echo '$http->on("request", function ($request, $response) {'; \
    echo '    ob_start();'; \
    echo '    include "/var/www/html/public/index.php";'; \
    echo '    $response->end(ob_get_clean());'; \
    echo '});'; \
    echo '$http->start();'; \
    } > /swoole-server.php

# Entrypoint: Laravel Octane if available, otherwise standalone Swoole HTTP server
RUN { \
    echo '#!/bin/bash'; \
    echo 'cd /var/www/html'; \
    echo 'if [ -f artisan ]; then'; \
    echo '    exec php artisan octane:start --server=swoole --host=0.0.0.0 --port=8000'; \
    echo 'else'; \
    echo '    exec php /swoole-server.php'; \
    echo 'fi'; \
    } > /swoole-entrypoint.sh && chmod +x /swoole-entrypoint.sh

WORKDIR /var/www/html

CMD ["/swoole-entrypoint.sh"]
