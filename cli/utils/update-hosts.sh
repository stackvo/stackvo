#!/bin/bash

###################################################################
# STACKVO HOSTS FILE UPDATER (DEPRECATED)
# This script is deprecated and will be removed in future versions.
# Please use Traefik for automatic domain routing.
###################################################################

echo "🔧 Stackvo - Hosts File Updater"
echo ""
echo "⚠️  This script is DEPRECATED and will be removed in future versions."
echo "    Stackvo now uses Traefik for automatic domain routing."
echo "    Manual /etc/hosts management is no longer required."
echo ""
echo "ℹ️  If you need to access your projects:"
echo "    1. Make sure Traefik is running: ./cli/stackvo.sh up"
echo "    2. Access via domain: https://your-project.loc"
echo "    3. Traefik handles all routing automatically"
echo ""
exit 0
