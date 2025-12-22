#!/usr/bin/env bash

###################################################################
# STACKVO PULL SCRIPT
# Pulls all images sequentially to avoid rate limits
###################################################################

# Load common library for shared paths and variables
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib/common.sh"

echo "📥 Pulling all Docker images (one by one to avoid rate limits)..."
echo ""
echo "⚠️  If you hit rate limits, login to Docker Hub:"
echo "   docker login"
echo ""

# Get all services
services=$(docker compose "${COMPOSE_FILES[@]}" config --services 2>/dev/null)

if [ -z "$services" ]; then
    echo "❌ No services found. Run 'stackvo generate' first."
    exit 1
fi

# Pull each service one by one
total=$(echo "$services" | wc -l)
current=0

echo "$services" | while read -r service; do
    if [ -n "$service" ]; then
        current=$((current + 1))
        echo "[$current/$total] Pulling $service..."
        docker compose "${COMPOSE_FILES[@]}" pull "$service" || echo "  ⚠ Failed (rate limit or network issue)"
        sleep 1  # Small delay to avoid rate limits
    fi
done

echo ""
echo "✅ Pull complete! Now run: stackvo up"
