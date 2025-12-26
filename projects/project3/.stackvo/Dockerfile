# Auto-generated Dockerfile for project3
# Web Server: Apache + mod_php
# PHP Version: 8.0
FROM php:8.0-apache

# Install system dependencies
RUN apt-get update && apt-get install -y \
     libcurl4-openssl-dev libfreetype6-dev libjpeg-dev libonig-dev libpng-dev libzip-dev  \
    && rm -rf /var/lib/apt/lists/*


RUN docker-php-ext-configure gd --with-freetype --with-jpeg

# Install PHP extensions
RUN docker-php-ext-install pdo pdo_mysql mysqli gd curl zip mbstring


# Install Development Tools
# Install Composer
COPY --from=composer:latest /usr/bin/composer /usr/bin/composer

# Install Node.js 20.x
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*


# Enable Apache modules
RUN a2enmod rewrite

WORKDIR /var/www/html
