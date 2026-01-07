#!/bin/bash
###################################################################
# STACKVO PROJECT CONFIG PARSER MODULE
# JSON parsing and validation operations
###################################################################

##
# Sanitize project name for Traefik
# Replace dots with dashes since Traefik uses dots as separators
#
# Parameters:
#   $1 - Project name
#
# Output:
#   Sanitized project name (dots replaced with dashes)
##
sanitize_project_name_for_traefik() {
    local project_name=$1
    # Replace dots with dashes (for Traefik compatibility)
    echo "$project_name" | tr '.' '-'
}

##
# Parse project JSON configuration
#
# Parameters:
#   $1 - stackvo.json file path
#   $2 - Project name
#
# Output:
#   Config values in KEY=VALUE format
##
parse_project_config() {
    local project_json=$1
    local project_name=$2
    
    # Extract values from JSON (using macOS BSD awk compatible sed)
    local php_version=$(sed -n 's/.*"version"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$project_json")
    local web_server=$(sed -n 's/.*"webserver"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$project_json")
    local project_domain=$(sed -n 's/.*"domain"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$project_json")
    local document_root=$(sed -n 's/.*"document_root"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$project_json")
    
    # Parse extension array (extract values within square brackets)
    local extensions=$(sed -n 's/.*"extensions"[[:space:]]*:[[:space:]]*\[\([^]]*\)\].*/\1/p' "$project_json" | tr -d '\",' | tr -s ' ')
    
    # Apply default values
    if [ -z "$document_root" ]; then
        document_root="public"
    fi
    
    if [ -z "$extensions" ]; then
        extensions=$(get_default_extensions)
    fi
    
    # Use defaults for missing values
    if [ -z "$php_version" ]; then
        log_warn "PHP version not found ($project_json), using default: ${DEFAULT_PHP_VERSION:-$CONST_DEFAULT_PHP_VERSION}"
        php_version="${DEFAULT_PHP_VERSION:-$CONST_DEFAULT_PHP_VERSION}"
    fi
    
    if [ -z "$web_server" ]; then
        log_warn "Webserver not found ($project_json), using default: ${DEFAULT_WEBSERVER:-$CONST_DEFAULT_WEBSERVER}"
        web_server="${DEFAULT_WEBSERVER:-$CONST_DEFAULT_WEBSERVER}"
    fi
    
    if [ -z "$project_domain" ]; then
        log_error "Domain not found ($project_json) - project: $project_name"
        return 1
    fi
    
    # Output config values
    echo "PHP_VERSION=$php_version"
    echo "WEB_SERVER=$web_server"
    echo "DOMAIN=$project_domain"
    echo "DOCUMENT_ROOT=$document_root"
    echo "EXTENSIONS=$extensions"
}

##
# Return default PHP extension list
#
# Output:
#   Default extension list (space-separated)
##
get_default_extensions() {
    echo "pdo pdo_mysql mysqli gd curl zip mbstring"
}

##
# Extract config values to variables
# Gets values from parse_project_config() output
#
# Parameters:
#   $1 - Config string (parse_project_config output)
#
# Global Variables:
#   php_version, web_server, project_domain, document_root, extensions
##
extract_config_values() {
    local config=$1
    
    php_version=$(echo "$config" | grep "^PHP_VERSION=" | cut -d= -f2)
    web_server=$(echo "$config" | grep "^WEB_SERVER=" | cut -d= -f2)
    project_domain=$(echo "$config" | grep "^DOMAIN=" | cut -d= -f2)
    document_root=$(echo "$config" | grep "^DOCUMENT_ROOT=" | cut -d= -f2)
    extensions=$(echo "$config" | grep "^EXTENSIONS=" | cut -d= -f2-)
}

##
# Validate project configuration
#
# Parameters:
#   $1 - Project name
#   $2 - PHP version
#   $3 - Web server
#   $4 - Domain
#
# Output:
#   0 = valid, 1 = invalid
##
validate_project_config() {
    local project_name=$1
    local php_version=$2
    local web_server=$3
    local project_domain=$4
    
    # Check required fields
    if [ -z "$project_name" ] || [ -z "$php_version" ] || [ -z "$web_server" ] || [ -z "$project_domain" ]; then
        log_error "Missing required fields - project: $project_name"
        return 1
    fi
    
    # Check web server type
    case "$web_server" in
        nginx|apache|caddy)
            return 0
            ;;
        *)
            log_warn "Unknown web server: $web_server (project: $project_name), nginx will be used"
            return 0
            ;;
    esac
}
