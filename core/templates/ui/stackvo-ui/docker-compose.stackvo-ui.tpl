###################################################################
# STACKVO WEB UI COMPOSE TEMPLATE
# Nginx + Node.js container for serving the web interface
###################################################################

services:
  stackvo-ui:
    build:
      context: ..
      dockerfile: core/templates/ui/stackvo-ui/Dockerfile
    container_name: "stackvo-ui"
    restart: unless-stopped
    
    environment:
      # Node.js environment
      NODE_ENV: production
      # Stackvo root directory for backend
      STACKVO_ROOT: /app
      # Projects directory
      PROJECTS_DIR: /app/projects
      # Generated directory
      GENERATED_DIR: /app/generated
      # Host path for volume mappings when running generate inside container
      HOST_STACKVO_ROOT: {{ STACKVO_ROOT }}
    
    volumes:
      # Mount entire stackvo directory for Docker operations
      - ../:/app:ro
      # Mount projects directory (read-write for project management)
      - ../projects:/app/projects:rw
      # Mount generated directory (read-write for generate command)
      - ../generated:/app/generated:rw
      # Mount Docker socket for Docker API access
      - /var/run/docker.sock:/var/run/docker.sock
    
    networks:
      - {{ DOCKER_DEFAULT_NETWORK }}
    
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.stackvo-ui.rule=Host(`stackvo.loc`)"
      - "traefik.http.routers.stackvo-ui.entrypoints=websecure"
      - "traefik.http.routers.stackvo-ui.tls=true"
      - "traefik.http.services.stackvo-ui.loadbalancer.server.port=80"
