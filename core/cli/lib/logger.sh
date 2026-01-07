#!/bin/bash
###################################################################
# STACKVO LOGGER LIBRARY
# Common logging functions - used in all scripts
###################################################################

# Colors - made constant with readonly
readonly GREEN='\033[0;32m'
readonly BLUE='\033[0;34m'
readonly RED='\033[0;31m'
readonly YELLOW='\033[1;33m'
readonly NC='\033[0m'  # No Color

# Logging functions
log_info() { 
    echo -e "${BLUE}[INFO]${NC} $1" >&2
}

log_success() { 
    echo -e "${GREEN}[OK]${NC} $1" >&2
}

log_error() { 
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

log_warn() { 
    echo -e "${YELLOW}[WARN]${NC} $1" >&2
}
