###################################################################
# STACKVO ELASTICSEARCH COMPOSE TEMPLATE
###################################################################

services:
  elasticsearch:
    image: "docker.elastic.co/elasticsearch/elasticsearch:{{ SERVICE_ELASTICSEARCH_VERSION }}"
    container_name: "stackvo-elasticsearch"
    restart: unless-stopped

    environment:
      - discovery.type=single-node
      - ES_JAVA_OPTS={{ ES_JAVA_OPTS | default('-Xms1g -Xmx1g') }}
      - xpack.security.enabled={{ ELASTIC_SECURITY | default('false') }}
      - xpack.security.enrollment.enabled={{ ELASTIC_ENROLLMENT | default('false') }}
      - cluster.name=stackvo-es
      - network.host=0.0.0.0

    ulimits:
      memlock: -1
      nofile: 65536

    volumes:
      - stackvo-elasticsearch-data:/usr/share/elasticsearch/data
      - ../logs/elasticsearch:/usr/share/elasticsearch/logs

    ports:
      - "{{ HOST_PORT_ELASTICSEARCH | default('9200') }}:9200"

    networks:
      - "{{ DOCKER_DEFAULT_NETWORK }}"

volumes:
  stackvo-elasticsearch-data:
