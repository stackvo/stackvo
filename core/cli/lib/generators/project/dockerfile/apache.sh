#!/bin/bash
###################################################################
# STACKVO APACHE DOCKERFILE GENERATOR MODULE
# Apache + mod_php Dockerfile generation
###################################################################

##
# Generate Apache Dockerfile
#
# Parameters:
#   $1 - Dockerfile path
#   $2 - PHP version
#   $3 - APT packages (space-separated)
#   $4 - Configure commands
#   $5 - docker-php-ext-install extensions
#   $6 - PECL extensions
#   $7 - Project name
##
generate_apache_dockerfile() {
    local dockerfile=$1
    local php_version=$2
    local apt_packages=$3
    local configure_commands=$4
    local docker_ext_install=$5
    local pecl_install=$6
    local project_name=$7
    
    # Default tools
    local default_tools=${PHP_DEFAULT_TOOLS:-""}
    local composer_version=${PHP_TOOL_COMPOSER_VERSION:-latest}
    local nodejs_version=${PHP_TOOL_NODEJS_VERSION:-20}
    
    # Dockerfile header
    generate_dockerfile_header "$project_name" "Apache + mod_php" "$php_version" "apache" > "$dockerfile"
    
    # System dependencies
    generate_system_dependencies_install "$apt_packages" >> "$dockerfile"
    
    # Configure commands
    if [ -n "$configure_commands" ]; then
        echo -e "$configure_commands" >> "$dockerfile"
        echo "" >> "$dockerfile"
    fi
    
    # PHP extensions
    generate_php_extension_install "$docker_ext_install" "$pecl_install" >> "$dockerfile"
    
    # Development tools
    generate_development_tools_install "$default_tools" "$composer_version" "$nodejs_version" >> "$dockerfile"
    
    # Apache modules
    generate_apache_modules >> "$dockerfile"
    
    # Workdir
    echo "" >> "$dockerfile"
    echo "WORKDIR /var/www/html" >> "$dockerfile"
}

##
# Enable Apache modules
#
# Output:
#   Dockerfile RUN command
##
generate_apache_modules() {
    cat <<'EOF'

# Enable Apache modules
RUN a2enmod rewrite

EOF
}
