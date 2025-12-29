###################################################################
# STACKVO BLACKFIRE AGENT COMPOSE TEMPLATE
###################################################################

services:
  blackfire:
    profiles: ["services", "blackfire"]  # --services ile tümü, --profile blackfire ile sadece bu servis
    image: "blackfire/blackfire:{{ SERVICE_BLACKFIRE_VERSION }}"
    container_name: "stackvo-blackfire"
    restart: unless-stopped

    environment:
      BLACKFIRE_SERVER_ID: "{{ SERVICE_BLACKFIRE_SERVER_ID }}"
      BLACKFIRE_SERVER_TOKEN: "{{ SERVICE_BLACKFIRE_SERVER_TOKEN }}"
      BLACKFIRE_LOG_LEVEL: "{{ SERVICE_BLACKFIRE_LOG_LEVEL | default('1') }}"

    ports:
      - "{{ HOST_PORT_BLACKFIRE | default('8707') }}:8707"

    volumes:
      - ../logs/blackfire:/var/log/blackfire

    networks:
      - "{{ DOCKER_DEFAULT_NETWORK }}"
