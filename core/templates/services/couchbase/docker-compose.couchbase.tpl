###################################################################
# STACKVO COUCHBASE COMPOSE TEMPLATE
###################################################################

services:
  couchbase:
    image: "couchbase:{{ SERVICE_COUCHBASE_VERSION }}"
    container_name: "stackvo-couchbase"
    restart: unless-stopped

    environment:
      COUCHBASE_ADMINISTRATOR_USERNAME: "{{ SERVICE_COUCHBASE_ADMIN_USER | default('Administrator') }}"
      COUCHBASE_ADMINISTRATOR_PASSWORD: "{{ SERVICE_COUCHBASE_ADMIN_PASSWORD | default('stackvo') }}"

    volumes:
      - stackvo-couchbase-data:/opt/couchbase/var
      - ../logs/couchbase:/opt/couchbase/var/lib/couchbase/logs

    ports:
      - "{{ HOST_PORT_COUCHBASE_WEB | default('8091') }}:8091"
      - "{{ HOST_PORT_COUCHBASE_API | default('8092') }}:8092"
      - "{{ HOST_PORT_COUCHBASE_INTERNAL | default('8093') }}:8093"
      - "{{ HOST_PORT_COUCHBASE_QUERY | default('8094') }}:8094"
      - "{{ HOST_PORT_COUCHBASE_FTS | default('8095') }}:8095"
      - "{{ HOST_PORT_COUCHBASE_CLIENT | default('11210') }}:11210"

    networks:
      - "{{ DOCKER_DEFAULT_NETWORK }}"

volumes:
  stackvo-couchbase-data:
