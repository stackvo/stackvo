###################################################################
# STACKVO COUCHDB COMPOSE TEMPLATE
###################################################################

services:
  couchdb:
    image: "couchdb:{{ SERVICE_COUCHDB_VERSION }}"
    container_name: "stackvo-couchdb"
    restart: unless-stopped

    environment:
      COUCHDB_USER: "{{ SERVICE_COUCHDB_USER | default('admin') }}"
      COUCHDB_PASSWORD: "{{ SERVICE_COUCHDB_PASSWORD | default('stackvo') }}"

    volumes:
      - stackvo-couchdb-data:/opt/couchdb/data
      - stackvo-couchdb-config:/opt/couchdb/etc/local.d
      - ./logs/couchdb:/opt/couchdb/var/log

    ports:
      - "{{ HOST_PORT_COUCHDB | default('5984') }}:5984"

    networks:
      - "{{ DOCKER_DEFAULT_NETWORK }}"

    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:5984/_up"]
      interval: 30s
      timeout: 10s
      retries: 5

volumes:
  stackvo-couchdb-data:
  stackvo-couchdb-config:
