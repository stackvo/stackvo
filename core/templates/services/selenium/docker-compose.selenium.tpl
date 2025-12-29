###################################################################
# STACKVO SELENIUM CHROME COMPOSE TEMPLATE
###################################################################

services:
  selenium:
    profiles: ["services", "selenium"]  # --services ile tümü, --profile selenium ile sadece bu servis
    image: "selenium/standalone-chrome:{{ SERVICE_SELENIUM_VERSION }}"
    container_name: "stackvo-selenium"
    restart: unless-stopped

    shm_size: "{{ SERVICE_SELENIUM_SHM_SIZE | default('2g') }}"

    ports:
      - "{{ HOST_PORT_SELENIUM | default('4444') }}:4444"
      - "{{ HOST_PORT_SELENIUM_VNC | default('7900') }}:7900"

    environment:
      SCREEN_WIDTH: "{{ SERVICE_SELENIUM_SCREEN_WIDTH | default('1920') }}"
      SCREEN_HEIGHT: "{{ SERVICE_SELENIUM_SCREEN_HEIGHT | default('1080') }}"
      SCREEN_DEPTH: "{{ SERVICE_SELENIUM_SCREEN_DEPTH | default('24') }}"

    volumes:
      - ../logs/services/selenium:/var/log/selenium

    networks:
      - "{{ DOCKER_DEFAULT_NETWORK }}"
