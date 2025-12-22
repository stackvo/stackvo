#!/bin/bash
###################################################################
# PHP EXTENSION METADATA
# Her extension için gerekli sistem bağımlılıklarını tanımlar
###################################################################

##
# Returns required system packages for extension
#
# Parameters:
#   $1 - Extension name
#
# Output:
#   Space-separated package list
##
get_extension_packages() {
    local ext=$1
    
    case "$ext" in
        gd)
            echo "libpng-dev libjpeg-dev libfreetype6-dev"
            ;;
        zip)
            echo "libzip-dev"
            ;;
        curl)
            echo "libcurl4-openssl-dev"
            ;;
        mbstring)
            echo "libonig-dev"
            ;;
        pgsql|pdo_pgsql)
            echo "libpq-dev"
            ;;
        intl)
            echo "libicu-dev"
            ;;
        soap)
            echo "libxml2-dev"
            ;;
        ldap)
            echo "libldap2-dev"
            ;;
        imap)
            echo "libc-client-dev libkrb5-dev"
            ;;
        bz2)
            echo "libbz2-dev"
            ;;
        gmp)
            echo "libgmp-dev"
            ;;
        imagick)
            echo "libmagickwand-dev"
            ;;
        redis)
            echo ""  # PECL extension, apt paketi gerektirmez
            ;;
        memcached)
            echo "libmemcached-dev zlib1g-dev"
            ;;
        mongodb)
            echo ""  # PECL extension
            ;;
        xdebug)
            echo ""  # PECL extension
            ;;
        swoole)
            echo ""  # PECL extension
            ;;
        xmlrpc)
            echo "libxml2-dev"
            ;;
        xsl)
            echo "libxslt1-dev"
            ;;
        tidy)
            echo "libtidy-dev"
            ;;
        snmp)
            echo "libsnmp-dev"
            ;;
        enchant)
            echo "libenchant-2-dev"
            ;;
        pspell)
            echo "libpspell-dev"
            ;;
        gettext)
            echo "gettext"
            ;;
        *)
            echo ""  # Bağımlılık gerektirmeyen extension'lar
            ;;
    esac
}

##
# Returns special configure command for extension
#
# Parameters:
#   $1 - Extension name
#
# Output:
#   Configure command (if any)
##
get_extension_configure() {
    local ext=$1
    
    case "$ext" in
        gd)
            echo "docker-php-ext-configure gd --with-freetype --with-jpeg"
            ;;
        imap)
            echo "docker-php-ext-configure imap --with-kerberos --with-imap-ssl"
            ;;
        ldap)
            echo "docker-php-ext-configure ldap --with-libdir=lib/x86_64-linux-gnu"
            ;;
        *)
            echo ""
            ;;
    esac
}

##
# Checks if extension should be installed via PECL
#
# Parameters:
#   $1 - Extension name
#
# Output:
#   0 = PECL, 1 = docker-php-ext-install
##
is_pecl_extension() {
    local ext=$1
    
    case "$ext" in
        redis|memcached|mongodb|xdebug|swoole|igbinary|msgpack|apcu|imagick)
            return 0
            ;;
        *)
            return 1
            ;;
    esac
}

##
# Returns installation commands for development tool
#
# Parameters:
#   $1 - Tool name
#   $2 - Composer version (optional)
#   $3 - Node.js version (optional)
#
# Output:
#   Installation commands
##
get_tool_install_commands() {
    local tool=$1
    local composer_version=${2:-latest}
    local nodejs_version=${3:-20}
    
    case "$tool" in
        composer)
            echo "# Install Composer"
            echo "COPY --from=composer:${composer_version} /usr/bin/composer /usr/bin/composer"
            ;;
        nodejs)
            echo "# Install Node.js ${nodejs_version}.x"
            echo "RUN curl -fsSL https://deb.nodesource.com/setup_${nodejs_version}.x | bash - \\"
            echo "    && apt-get install -y nodejs \\"
            echo "    && rm -rf /var/lib/apt/lists/*"
            ;;
        git)
            echo "# Install Git"
            echo "RUN apt-get update && apt-get install -y git && rm -rf /var/lib/apt/lists/*"
            ;;
        wget)
            echo "# Install Wget"
            echo "RUN apt-get update && apt-get install -y wget && rm -rf /var/lib/apt/lists/*"
            ;;
        unzip)
            echo "# Install Unzip"
            echo "RUN apt-get update && apt-get install -y unzip && rm -rf /var/lib/apt/lists/*"
            ;;
        *)
            echo ""
            ;;
    esac
}
