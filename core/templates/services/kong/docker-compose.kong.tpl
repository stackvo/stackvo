###################################################################
# STACKVO KONG COMPOSE TEMPLATE
###################################################################

services:
  kong:
    image: "kong:{{ SERVICE_KONG_VERSION }}"
    container_name: "stackvo-kong"
    restart: unless-stopped

    environment:
      KONG_DATABASE: "{{ SERVICE_KONG_DATABASE | default('off') }}"
      KONG_PROXY_ACCESS_LOG: "/dev/stdout"
      KONG_ADMIN_ACCESS_LOG: "/dev/stdout"
      KONG_PROXY_ERROR_LOG: "/dev/stderr"
      KONG_ADMIN_ERROR_LOG: "/dev/stderr"
      KONG_ADMIN_LISTEN: "{{ SERVICE_KONG_ADMIN_LISTEN | default('0.0.0.0:8001') }}"
      KONG_DECLARATIVE_CONFIG: "/kong/declarative/kong.yml"

    volumes:
      - ./core/templates/appserver/kong/kong.yml:/kong/declarative/kong.yml:ro
      - ./logs/kong:/usr/local/kong/logs

    ports:
      - "{{ HOST_PORT_KONG_PROXY | default('8000') }}:8000"
      - "{{ HOST_PORT_KONG_PROXY_SSL | default('8443') }}:8443"
      - "{{ HOST_PORT_KONG_ADMIN | default('8001') }}:8001"
      - "{{ HOST_PORT_KONG_ADMIN_SSL | default('8444') }}:8444"

    networks:
      - "{{ DOCKER_DEFAULT_NETWORK }}"

    healthcheck:
      test: ["CMD", "kong", "health"]
      interval: 10s
      timeout: 10s
      retries: 10
