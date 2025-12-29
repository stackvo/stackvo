###################################################################
# STACKVO UNIFIED TOOLS COMPOSE TEMPLATE
###################################################################

services:
  stackvo-tools:
    profiles: ["services", "tools"]  # --services ile tümü, --profile tools ile sadece tools
    build:
      context: ../
      dockerfile: core/templates/ui/tools/Dockerfile
    container_name: "stackvo-tools"
    restart: unless-stopped

    environment:
      # Pass necessary environment variables for tools configuration
      PMA_HOST: stackvo-mysql
      PMA_PORT: 3306
      PMA_ARBITRARY: 1

    networks:
      - "{{ DOCKER_DEFAULT_NETWORK }}"
