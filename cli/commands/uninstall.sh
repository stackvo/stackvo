#!/usr/bin/env bash

###################################################################
# STACKVO UNINSTALLER
# Stackvo projesine ait tüm Docker kaynaklarını ve dosyaları temizler
###################################################################

# Load common library and logger
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib/common.sh"
source "$SCRIPT_DIR/../lib/logger.sh"

echo -e "${RED}⚠️  STACKVO UNINSTALLER${NC}"
echo ""
echo "This will remove:"
echo "  - All Stackvo Docker containers (stackvo-* prefix)"
echo "  - All Docker images used by containers"
echo "  - All Stackvo Docker volumes (stackvo-* prefix)"
echo "  - Stackvo Docker network (stackvo-net)"
echo "  - System command (/usr/local/bin/stackvo)"
echo "  - All generated files and directories:"
echo "    • generated/ directory (compose files, configs, Dockerfiles)"
echo "    • logs/ directory (all log files)"
echo "    • cache/ directory (if exists)"
echo "    • projects/ directory (user projects)"
echo ""
echo -e "${YELLOW}⚠️  WARNING: All database data and project files will be deleted!${NC}"
echo -e "${YELLOW}⚠️  WARNING: Images used by containers will also be removed (mysql, redis, etc.)${NC}"
echo ""
read -p "Do you want to continue? (yes/no): " -r
echo

if [[ ! $REPLY =~ ^[Yy][Ee][Ss]$ ]]; then
    echo "Cancelled."
    exit 0
fi

echo -e "${RED}[1/8]${NC} Stopping containers..."
cd "$STACKVO_ROOT"

# Detect images used by containers before removal
echo "  Detecting images used by Stackvo containers..."
STACKVO_IMAGES=$(docker ps -a --filter "name=stackvo-" --format "{{.Image}}" | sort -u)

# Stop with Docker Compose (if compose files exist)
if [ -f "generated/stackvo.yml" ]; then
    echo "  Stopping containers with Docker Compose..."
    docker compose \
        -f generated/stackvo.yml \
        -f generated/docker-compose.dynamic.yml \
        -f generated/docker-compose.projects.yml \
        down -v --remove-orphans 2>/dev/null || true
fi

# Manually stop and remove stackvo-* prefixed containers
echo "  Removing Stackvo containers..."
docker ps -a --format "{{.Names}}" | grep "^stackvo-" | xargs -r docker rm -f 2>/dev/null || true

echo -e "${RED}[2/8]${NC} Removing Docker images..."
# Remove all images used by containers (including official images)
if [ -n "$STACKVO_IMAGES" ]; then
    echo "  Removing images used by containers..."
    for img in $STACKVO_IMAGES; do
        echo "    - Removing: $img"
        docker rmi -f "$img" 2>/dev/null || true
    done
else
    echo "  No images to remove"
fi

# Remove dangling images (untagged images with <none>)
echo "  Removing dangling images (<none>)..."
docker images -f "dangling=true" -q | xargs -r docker rmi -f 2>/dev/null || true

# Remove Stackvo build cache
echo "  Removing Stackvo build cache..."
docker builder prune -af --filter "label=project=stackvo" 2>/dev/null || true
docker builder prune -af --filter "label!=project=stackvo" 2>/dev/null || true


echo -e "${RED}[3/8]${NC} Removing Docker volumes..."
docker volume ls --format "{{.Name}}" | grep "stackvo" | xargs -r docker volume rm 2>/dev/null || true

echo -e "${RED}[4/8]${NC} Removing Docker network..."
docker network rm stackvo-net 2>/dev/null || true

echo -e "${RED}[5/8]${NC} Removing system command..."
sudo rm -f /usr/local/bin/stackvo 2>/dev/null || true

echo -e "${RED}[6/8]${NC} Cleaning generated directory..."
# Remove generated directory completely (use sudo - some files may be root-owned)
sudo rm -rf "$STACKVO_ROOT/generated/" 2>/dev/null || true

# Clean up old locations (if they exist)
sudo rm -f "$STACKVO_ROOT/stackvo.yml" 2>/dev/null || true
sudo rm -f "$STACKVO_ROOT/docker-compose.dynamic.yml" 2>/dev/null || true
sudo rm -f "$STACKVO_ROOT/docker-compose.projects.yml" 2>/dev/null || true
sudo rm -rf "$STACKVO_ROOT/core/traefik/" 2>/dev/null || true
sudo rm -rf "$STACKVO_ROOT/core/generated-configs/" 2>/dev/null || true
sudo rm -rf "$STACKVO_ROOT/core/generated/" 2>/dev/null || true
sudo rm -rf "$STACKVO_ROOT/core/certs/" 2>/dev/null || true

echo -e "${RED}[7/8]${NC} Cleaning log files..."
sudo rm -rf "$STACKVO_ROOT/logs/" 2>/dev/null || true

echo -e "${RED}[8/8]${NC} Cleaning project files..."
sudo rm -rf "$STACKVO_ROOT/projects/" 2>/dev/null || true

# Clean cache directory if exists
if [ -d "$STACKVO_ROOT/cache/" ]; then
    echo "  Cleaning cache directory..."
    sudo rm -rf "$STACKVO_ROOT/cache/" 2>/dev/null || true
fi

# Clean tools generated files
if [ -d "$STACKVO_ROOT/core/templates/ui/tools/" ]; then
    echo "  Cleaning tools generated files..."
    rm -f "$STACKVO_ROOT/core/templates/ui/tools/Dockerfile" 2>/dev/null || true
    rm -f "$STACKVO_ROOT/core/templates/ui/tools/nginx.conf" 2>/dev/null || true
    rm -f "$STACKVO_ROOT/core/templates/ui/tools/supervisord.conf" 2>/dev/null || true
    rm -f "$STACKVO_ROOT/core/templates/ui/tools/Dockerfile.backup" 2>/dev/null || true
    rm -f "$STACKVO_ROOT/core/templates/ui/tools/nginx.conf.backup" 2>/dev/null || true
    rm -f "$STACKVO_ROOT/core/templates/ui/tools/supervisord.conf.backup" 2>/dev/null || true
fi

echo ""
echo -e "${GREEN}✔ Stackvo successfully uninstalled!${NC}"
echo ""
echo "Removed resources:"
echo "  ✓ All Stackvo Docker containers"
echo "  ✓ All images used by containers"
echo "  ✓ All Stackvo Docker volumes"
echo "  ✓ Stackvo Docker network (stackvo-net)"
echo "  ✓ System command (/usr/local/bin/stackvo)"
echo "  ✓ Generated directory and all contents"
echo "  ✓ Log files (logs/)"
echo "  ✓ Project files (projects/)"
echo "  ✓ Cache files (if any)"
echo "  ✓ Tools generated files"
echo ""
echo "Preserved resources:"
echo "  • Configuration file (.env)"
echo "  • Template files (core/templates/)"
echo "  • CLI commands (cli/)"
echo ""
echo "To reinstall Stackvo:"
echo "  1. ./cli/stackvo.sh install"
echo "  2. ./cli/stackvo.sh generate"
echo "  3. ./cli/stackvo.sh up"
echo ""
