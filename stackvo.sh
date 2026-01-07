#!/usr/bin/env bash

###################################################################
# STACKVO CLI WRAPPER SCRIPT
# This script forwards all commands to core/cli/stackvo.sh
###################################################################

# Get the directory where this script is located (root directory)
WRAPPER_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Execute the actual CLI script with all arguments
exec "$WRAPPER_DIR/core/cli/stackvo.sh" "$@"
