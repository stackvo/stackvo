###################################################################
# STACKVO KIBANA COMPOSE TEMPLATE
###################################################################

services:
  kibana:
    profiles: ["services", "kibana"]  # --services ile tümü, --profile kibana ile sadece bu servis
    image: "kibana:{{ SERVICE_KIBANA_VERSION }}"
    container_name: "stackvo-kibana"
    restart: unless-stopped

    environment:
      ELASTICSEARCH_HOSTS: "{{ SERVICE_KIBANA_ELASTICSEARCH_HOSTS | default('http://stackvo-elasticsearch:9200') }}"
      SERVER_NAME: "{{ SERVICE_KIBANA_SERVER_NAME | default('stackvo-kibana') }}"
      SERVER_HOST: "{{ SERVICE_KIBANA_SERVER_HOST | default('0.0.0.0') }}"

    volumes:
      - stackvo-kibana-data:/usr/share/kibana/data
      - ../logs/services/kibana:/usr/share/kibana/logs

    ports:
      - "{{ HOST_PORT_KIBANA | default('5601') }}:5601"

    networks:
      - "{{ DOCKER_DEFAULT_NETWORK }}"

    healthcheck:
      test: ["CMD-SHELL", "curl -f http://localhost:5601/api/status || exit 1"]
      interval: 30s
      timeout: 10s
      retries: 5

volumes:
  stackvo-kibana-data:
