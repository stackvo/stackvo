###################################################################
# STACKVO MEILISEARCH COMPOSE TEMPLATE
###################################################################

services:
  meilisearch:
    image: "getmeili/meilisearch:{{ SERVICE_MEILISEARCH_VERSION }}"
    container_name: "stackvo-meilisearch"
    restart: unless-stopped

    environment:
      MEILI_MASTER_KEY: "{{ SERVICE_MEILISEARCH_MASTER_KEY | default('stackvo-master-key') }}"
      MEILI_ENV: "development"
      MEILI_NO_ANALYTICS: "true"

    ports:
      - "{{ HOST_PORT_MEILISEARCH | default('7700') }}:7700"

    volumes:
      - stackvo-meili-data:/meili_data
      - ../logs/meilisearch:/meili_data/logs

    networks:
      - "{{ DOCKER_DEFAULT_NETWORK }}"

volumes:
  stackvo-meili-data:
