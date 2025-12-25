###################################################################
# STACKVO LOGSTASH COMPOSE TEMPLATE
###################################################################

services:
  logstash:
    image: "logstash:{{ SERVICE_LOGSTASH_VERSION }}"
    container_name: "stackvo-logstash"
    restart: unless-stopped

    environment:
      LS_JAVA_OPTS: "{{ SERVICE_LOGSTASH_JAVA_OPTS | default('-Xmx512m -Xms512m') }}"

    volumes:
      - ./core/templates/monitoring/logstash/logstash.conf:/usr/share/logstash/pipeline/logstash.conf:ro
      - stackvo-logstash-data:/usr/share/logstash/data
      - ../logs/logstash:/usr/share/logstash/logs

    ports:
      - "{{ HOST_PORT_LOGSTASH_TCP | default('5001') }}:5000/tcp"
      - "{{ HOST_PORT_LOGSTASH_UDP | default('5001') }}:5000/udp"
      - "{{ HOST_PORT_LOGSTASH_BEATS | default('5044') }}:5044"
      - "{{ HOST_PORT_LOGSTASH_API | default('9600') }}:9600"

    networks:
      - "{{ DOCKER_DEFAULT_NETWORK }}"

    depends_on:
      - elasticsearch

volumes:
  stackvo-logstash-data:
