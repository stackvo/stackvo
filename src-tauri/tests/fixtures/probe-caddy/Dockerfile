# syntax=docker/dockerfile:1.4

# Auto-generated Dockerfile for probe-caddy
# Web Server: Caddy + PHP-FPM
# PHP Version: 8.4
FROM php:8.4-fpm

# Install Caddy and Supervisord with BuildKit cache
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && apt-get install -y \
    debian-keyring \
    debian-archive-keyring \
    apt-transport-https \
    curl \
    supervisor \
    && curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg \
    && curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | tee /etc/apt/sources.list.d/caddy-stable.list \
    && apt-get update \
    && apt-get install -y caddy \
    && rm -rf /var/lib/apt/lists/*

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
    libzip-dev \
    && rm -rf /var/lib/apt/lists/*

RUN docker-php-ext-configure gd --with-freetype --with-jpeg

# Install PHP extensions
RUN docker-php-ext-install pdo
RUN docker-php-ext-install pdo_mysql
RUN docker-php-ext-install mbstring curl zip gd intl bcmath

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


# Configure PHP-FPM to listen on TCP port 127.0.0.1:9000
RUN sed -i 's|listen = .*|listen = 127.0.0.1:9000|' /usr/local/etc/php-fpm.d/www.conf

# Copy Caddyfile
COPY Caddyfile /etc/caddy/Caddyfile

# Copy Supervisord configuration
COPY supervisord.conf /etc/supervisor/conf.d/supervisord.conf

# Create entrypoint script to ensure log directories exist at runtime
RUN echo '#!/bin/bash' > /entrypoint.sh && \
    echo 'mkdir -p /var/log/caddy /var/log/php-fpm' >> /entrypoint.sh && \
    echo 'touch /var/log/caddy/access.log /var/log/caddy/error.log' >> /entrypoint.sh && \
    echo 'chmod 666 /var/log/caddy/*.log' >> /entrypoint.sh && \
    echo 'exec "$@"' >> /entrypoint.sh && \
    chmod +x /entrypoint.sh

WORKDIR /var/www/html

# Use entrypoint to create log directories before starting supervisord
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/usr/bin/supervisord", "-c", "/etc/supervisor/conf.d/supervisord.conf"]
