#!/bin/bash
###################################################################
# STACKVO CONSTANTS
# All constant values (magic numbers, strings) are defined here
# CONST_ prefix distinguishes them from .env variables
###################################################################

# Default Values (Fallback)
readonly CONST_DEFAULT_PHP_VERSION="8.2"
readonly CONST_DEFAULT_WEBSERVER="nginx"

# File Paths (Relative to ROOT_DIR)
readonly CONST_PATH_TEMPLATES="core/templates"
readonly CONST_PATH_GENERATED="generated"
readonly CONST_PATH_GENERATED_CONFIGS="generated/configs"
readonly CONST_PATH_GENERATED_PROJECTS="generated/projects"
readonly CONST_PATH_TRAEFIK_CONFIG="generated/traefik"
readonly CONST_PATH_TRAEFIK_DYNAMIC="generated/traefik/dynamic"
readonly CONST_PATH_CERTS="generated/certs"
readonly CONST_PATH_PROJECTS="projects"

# File Names
readonly CONST_FILE_STACKVO_JSON="stackvo.json"
readonly CONST_FILE_STACKVO_YML="stackvo.yml"
readonly CONST_FILE_DYNAMIC_YML="docker-compose.dynamic.yml"
readonly CONST_FILE_PROJECTS_YML="docker-compose.projects.yml"
readonly CONST_FILE_TRAEFIK_CONFIG="traefik.yml"
readonly CONST_FILE_TRAEFIK_ROUTES="routes.yml"


# Container Prefix
readonly CONST_CONTAINER_PREFIX="stackvo-"

# Network
readonly CONST_DEFAULT_NETWORK="stackvo-net"


# Stackvo Config Directory
readonly CONST_STACKVO_CONFIG_DIR=".stackvo"

