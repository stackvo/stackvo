#!/bin/bash

###################################################################
# HOSTS FILE UPDATER
# Adds project domains to /etc/hosts for local development
###################################################################

echo "🔧 Stackvo - Hosts File Updater"
echo ""
echo "This will add the following entries to /etc/hosts:"
echo "  127.0.0.1  project1.loc"
echo "  127.0.0.1  project2.loc"
echo ""
echo "You will need to enter your password (sudo required)"
echo ""

# Backup hosts file
sudo cp /etc/hosts /etc/hosts.backup

# Check if entries already exist
if grep -q "project1.loc" /etc/hosts && grep -q "project2.loc" /etc/hosts; then
    echo "✅ Entries already exist in /etc/hosts"
else
    echo "Adding entries to /etc/hosts..."
    
    # Add entries
    echo "" | sudo tee -a /etc/hosts > /dev/null
    echo "# Stackvo Projects" | sudo tee -a /etc/hosts > /dev/null
    echo "127.0.0.1  project1.loc" | sudo tee -a /etc/hosts > /dev/null
    echo "127.0.0.1  project2.loc" | sudo tee -a /etc/hosts > /dev/null
    
    echo "✅ Entries added successfully!"
fi

echo ""
echo "You can now access:"
echo "  • http://project1.loc"
echo "  • http://project2.loc"
echo "  • http://traefik.localhost:8080 (Dashboard)"
echo ""
