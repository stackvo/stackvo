#!/usr/bin/env bash

###################################################################
# STACKVO UNINSTALLER
# Removes all containers, volumes, and installation
###################################################################

# Load common library and logger
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib/common.sh"
source "$SCRIPT_DIR/../lib/logger.sh"

echo -e "${RED}⚠️  STACKVO UNINSTALLER${NC}"
echo ""
echo "This will remove:"
echo "  - All Stackvo Docker containers"
echo "  - All Stackvo Docker images"
echo "  - All Stackvo Docker volumes (DATABASE DATA WILL BE DELETED!)"
echo "  - Stackvo Docker network (stackvo-net)"
echo "  - System-wide 'stackvo' command"
echo "  - Generated configuration files"
echo "  - Tools generated files (Dockerfile, nginx.conf, supervisord.conf)"
echo "  - SSL certificates (core/certs/)"
echo "  - Log files (logs/)"
echo ""
echo -e "${YELLOW}⚠️  WARNING: All Stackvo database data will be deleted!${NC}"
echo ""
read -p "Do you want to continue? (yes/no): " -r
echo

if [[ ! $REPLY =~ ^[Yy][Ee][Ss]$ ]]; then
    echo "Cancelled."
    exit 0
fi

echo -e "${RED}[1/9]${NC} Stopping containers..."
cd "$STACKVO_ROOT"

# Collect image IDs before removing containers
echo "  Collecting image IDs from Stackvo containers..."
STACKVO_IMAGES=$(docker ps -a --filter "name=stackvo-" --format "{{.Image}}" | sort -u)

if [ -f "generated/stackvo.yml" ]; then
    docker compose \
        -f generated/stackvo.yml \
        -f generated/docker-compose.dynamic.yml \
        -f generated/docker-compose.projects.yml \
        down -v --remove-orphans
else
    echo "  Warning: Generated files not found, trying to stop containers by name..."
    docker ps --format "{{.Names}}" | grep "stackvo-" | xargs -r docker rm -f
fi

echo -e "${RED}[2/9]${NC} Removing Docker images..."
# Remove images that were used by Stackvo containers
if [ -n "$STACKVO_IMAGES" ]; then
    echo "  Removing images used by Stackvo containers..."
    echo "$STACKVO_IMAGES" | xargs -r docker rmi -f 2>/dev/null || true
else
    echo "  No Stackvo images found to remove"
fi

echo -e "${RED}[3/9]${NC} Removing volumes..."
docker volume ls --format "{{.Name}}" | grep "stackvo" | xargs -r docker volume rm 2>/dev/null || true

echo -e "${RED}[4/9]${NC} Removing network..."
docker network rm stackvo-net 2>/dev/null || true

echo -e "${RED}[5/9]${NC} Removing system command..."
sudo rm -f /usr/local/bin/stackvo 2>/dev/null || true

echo -e "${RED}[6/9]${NC} Removing generated files..."
# Remove new generated directory (use sudo in case files are root-owned)
sudo rm -rf "$STACKVO_ROOT/generated/"
# Clean up old locations (if they exist)
sudo rm -f "$STACKVO_ROOT/stackvo.yml"
sudo rm -f "$STACKVO_ROOT/docker-compose.dynamic.yml"
sudo rm -f "$STACKVO_ROOT/docker-compose.projects.yml"
sudo rm -rf "$STACKVO_ROOT/core/traefik/"
sudo rm -rf "$STACKVO_ROOT/core/generated-configs/"
sudo rm -rf "$STACKVO_ROOT/core/generated/"
sudo rm -rf "$STACKVO_ROOT/core/certs/"

echo -e "${RED}[7/9]${NC} Removing tools generated files..."
# Remove generated tools files
rm -f "$STACKVO_ROOT/core/templates/ui/tools/Dockerfile"
rm -f "$STACKVO_ROOT/core/templates/ui/tools/nginx.conf"
rm -f "$STACKVO_ROOT/core/templates/ui/tools/supervisord.conf"
# Remove backup files
rm -f "$STACKVO_ROOT/core/templates/ui/tools/Dockerfile.backup"
rm -f "$STACKVO_ROOT/core/templates/ui/tools/nginx.conf.backup"
rm -f "$STACKVO_ROOT/core/templates/ui/tools/supervisord.conf.backup"
# Note: tpl/ directory is preserved (contains source templates)

echo -e "${RED}[9/9]${NC} Removing SSL certificates..."
sudo rm -rf "$STACKVO_ROOT/generated/certs/" 2>/dev/null || true
rm -rf "$STACKVO_ROOT/core/certs/" 2>/dev/null || true

echo -e "${RED}[9/9]${NC} Removing log files..."
sudo rm -rf "$STACKVO_ROOT/logs/"

echo ""
echo -e "${GREEN}✔ Stackvo successfully uninstalled!${NC}"
echo ""
echo "Removed files:"
echo "  ✓ Stackvo Docker containers and volumes"
echo "  ✓ Stackvo Docker images"
echo "  ✓ Stackvo Docker network (stackvo-net)"
echo "  ✓ System command (/usr/local/bin/stackvo)"
echo "  ✓ All generated files (core/generated/)"
echo "  ✓ Tools generated files"
echo "  ✓ Log files"
echo ""
echo "Preserved files:"
echo "  • Project files (projects/) - User data"
echo "  • Configuration (.env) - Base settings"
echo "  • Template files (core/templates/) - Source files"
echo ""
echo "To reinstall Stackvo:"
echo "  1. ./cli/stackvo.sh generate"
echo "  2. ./cli/stackvo.sh up"
echo ""
