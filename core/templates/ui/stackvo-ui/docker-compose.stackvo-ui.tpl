###################################################################
# STACKVO WEB UI COMPOSE TEMPLATE
# Nginx + PHP-FPM container for serving the web interface
###################################################################

services:
  stackvo-ui:
    build:
      context: ../core/templates/ui/stackvo-ui
      dockerfile: Dockerfile
    container_name: "stackvo-ui"
    restart: unless-stopped
    
    environment:
      # Host path for volume mappings when running generate inside container
      HOST_STACKVO_ROOT: {{ STACKVO_ROOT }}
    
    volumes:
      - ../.ui:/usr/share/nginx/html
      - ../:/app:ro
      - ../projects:/app/projects:rw
      - ../generated:/app/generated:rw
      - /var/run/docker.sock:/var/run/docker.sock
      - ../.ui/logs:/usr/share/nginx/html/logs:rw
      - ../.ui/cache:/usr/share/nginx/html/cache:rw
    
    networks:
      - {{ DOCKER_DEFAULT_NETWORK }}
    
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.stackvo-ui.rule=Host(`stackvo.loc`)"
      - "traefik.http.routers.stackvo-ui.entrypoints=websecure"
      - "traefik.http.routers.stackvo-ui.tls=true"
      - "traefik.http.services.stackvo-ui.loadbalancer.server.port=80"
