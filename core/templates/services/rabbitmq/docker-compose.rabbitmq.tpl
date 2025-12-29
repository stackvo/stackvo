###################################################################
# STACKVO RABBITMQ COMPOSE TEMPLATE
###################################################################

services:
  rabbitmq:
    profiles: ["services", "rabbitmq"]  # --services ile tümü, --profile rabbitmq ile sadece RabbitMQ
    image: "rabbitmq:{{ SERVICE_RABBITMQ_VERSION }}-management"
    container_name: "stackvo-rabbitmq"
    restart: unless-stopped

    environment:
      RABBITMQ_DEFAULT_USER: "{{ SERVICE_RABBITMQ_DEFAULT_USER }}"
      RABBITMQ_DEFAULT_PASS: "{{ SERVICE_RABBITMQ_DEFAULT_PASS }}"
      RABBITMQ_DEFAULT_VHOST: "/"

    volumes:
      - stackvo-rabbitmq-data:/var/lib/rabbitmq
      - stackvo-rabbitmq-logs:/var/log/rabbitmq

    ports:
      - "{{ HOST_PORT_RABBITMQ | default('5672') }}:5672"
      - "{{ HOST_PORT_RABBITMQ_MGMT | default('15672') }}:15672"

    networks:
      - "{{ DOCKER_DEFAULT_NETWORK }}"

volumes:
  stackvo-rabbitmq-data:
  stackvo-rabbitmq-logs:
