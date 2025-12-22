###################################################################
# STACKVO ACTIVEMQ CLASSIC COMPOSE TEMPLATE
###################################################################

services:
  activemq:
    image: "apache/activemq-classic:{{ SERVICE_ACTIVEMQ_VERSION }}"
    container_name: "stackvo-activemq"
    restart: unless-stopped

    environment:
      ACTIVEMQ_ADMIN_LOGIN: "{{ SERVICE_ACTIVEMQ_ADMIN_USER }}"
      ACTIVEMQ_ADMIN_PASSWORD: "{{ SERVICE_ACTIVEMQ_ADMIN_PASSWORD }}"

    ports:
      - "{{ SERVICE_ACTIVEMQ_HOST_PORT_OPENWIRE | default('61616') }}:61616"
      - "{{ SERVICE_ACTIVEMQ_HOST_PORT_AMQP | default('5672') }}:5672"
      - "{{ SERVICE_ACTIVEMQ_HOST_PORT_STOMP | default('61613') }}:61613"
      - "{{ SERVICE_ACTIVEMQ_HOST_PORT_MQTT | default('1883') }}:1883"
      - "{{ SERVICE_ACTIVEMQ_HOST_PORT_WS | default('61614') }}:61614"
      - "{{ SERVICE_ACTIVEMQ_HOST_PORT_UI | default('8161') }}:8161"

    volumes:
      - stackvo-activemq-data:/opt/apache-activemq/data
      - stackvo-activemq-conf:/opt/apache-activemq/conf
      - ./logs/activemq:/opt/apache-activemq/data

    networks:
      - "{{ DOCKER_DEFAULT_NETWORK }}"

volumes:
  stackvo-activemq-data:
  stackvo-activemq-conf:
