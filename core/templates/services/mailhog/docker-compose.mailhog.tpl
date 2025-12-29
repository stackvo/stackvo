###################################################################
# STACKVO MAILHOG COMPOSE TEMPLATE
###################################################################

services:
  mailhog:
    profiles: ["services", "mailhog"]  # --services ile tümü, --profile mailhog ile sadece bu servis
    image: "mailhog/mailhog:{{ SERVICE_MAILHOG_VERSION }}"
    container_name: "stackvo-mailhog"
    restart: unless-stopped

    ports:
      - "{{ HOST_PORT_MAILHOG_SMTP | default('1025') }}:1025"
      - "{{ HOST_PORT_MAILHOG_UI | default('8025') }}:8025"

    volumes:
      - ../logs/mailhog:/var/log/mailhog

    networks:
      - "{{ DOCKER_DEFAULT_NETWORK }}"
