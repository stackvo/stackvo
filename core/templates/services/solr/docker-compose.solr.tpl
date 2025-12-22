###################################################################
# STACKVO APACHE SOLR COMPOSE TEMPLATE
###################################################################

services:
  solr:
    image: "solr:{{ SERVICE_SOLR_VERSION }}"
    container_name: "stackvo-solr"
    restart: unless-stopped

    environment:
      # Solr ana dizini
      SOLR_HOME: /var/solr
      # JVM Memory ayarları
      SOLR_JAVA_MEM: "{{ SERVICE_SOLR_JAVA_MEM | default('-Xms512m -Xmx512m') }}"
      # Analytics kapatma
      SOLR_OPTS: >-
        {{ SERVICE_SOLR_OPTS | default('-Dsolr.disable.shardsyslog=true -Dsolr.jetty.inetaccess.allowall=true') }}

    command:
      - solr-precreate
      - "{{ SERVICE_SOLR_DEFAULT_CORE | default('stackvo-core') }}"

    volumes:
      # Solr data directory
      - stackvo-solr-data:/var/solr
      # Varsayılan veya override solr configsets
      - ./stackvo-config/solr/configsets:/opt/solr/server/solr/configsets
      # Solr logs
      - ./logs/solr:/var/solr/logs

    ports:
      - "{{ HOST_PORT_SOLR | default('8983') }}:8983"

    networks:
      - "{{ DOCKER_DEFAULT_NETWORK }}"

volumes:
  stackvo-solr-data:
