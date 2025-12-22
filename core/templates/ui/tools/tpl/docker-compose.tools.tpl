###################################################################
# STACKVO UNIFIED TOOLS COMPOSE TEMPLATE
###################################################################

services:
  stackvo-tools:
    build:
      context: ../core/templates/ui/tools
      dockerfile: Dockerfile
    container_name: "stackvo-tools"
    restart: unless-stopped

    environment:
      # Pass necessary environment variables for tools configuration
      PMA_HOST: stackvo-mysql
      PMA_PORT: 3306
      PMA_ARBITRARY: 1

    networks:
      - "{{ DOCKER_DEFAULT_NETWORK }}"
