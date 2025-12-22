###################################################################
# STACKVO GRAFANA COMPOSE TEMPLATE
###################################################################

services:
  grafana:
    image: "grafana/grafana:{{ SERVICE_GRAFANA_VERSION }}"
    container_name: "stackvo-grafana"
    restart: unless-stopped

    environment:
      GF_SECURITY_ADMIN_USER: "{{ SERVICE_GRAFANA_ADMIN_USER }}"
      GF_SECURITY_ADMIN_PASSWORD: "{{ SERVICE_GRAFANA_ADMIN_PASSWORD }}"
      GF_INSTALL_PLUGINS: ""
      GF_SERVER_ROOT_URL: "http://grafana.stackvo.{{ DEFAULT_TLD_SUFFIX | default('loc') }}"

    volumes:
      - stackvo-grafana-data:/var/lib/grafana
      - stackvo-grafana-config:/etc/grafana
      - ./logs/grafana:/var/log/grafana

    ports:
      - "{{ HOST_PORT_GRAFANA | default('3001') }}:3000"

    networks:
      - "{{ DOCKER_DEFAULT_NETWORK }}"

    user: "{{ HOST_USER_ID | default('472') }}"

volumes:
  stackvo-grafana-data:
  stackvo-grafana-config:
