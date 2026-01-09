#!/usr/bin/env bash

###################################################################
# STACKVO CLI WRAPPER SCRIPT
# This script forwards all commands to core/cli/stackvo.sh
###################################################################

# Get the directory where this script is located (root directory)
WRAPPER_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Ensure the actual CLI script is executable
chmod +x "$WRAPPER_DIR/core/cli/stackvo.sh" 2>/dev/null || true

# Execute the actual CLI script with all arguments
exec "$WRAPPER_DIR/core/cli/stackvo.sh" "$@"
