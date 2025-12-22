###################################################################
# STACKVO NGROK COMPOSE TEMPLATE
###################################################################

services:
  ngrok:
    image: "ngrok/ngrok:{{ SERVICE_NGROK_VERSION }}"
    container_name: "stackvo-ngrok"
    restart: unless-stopped

    environment:
      NGROK_AUTHTOKEN: "{{ SERVICE_NGROK_AUTHTOKEN }}"
      NGROK_ADDR: "{{ SERVICE_NGROK_ADDR | default('nginx:80') }}"
      NGROK_DOMAIN: "{{ SERVICE_NGROK_DOMAIN | default('') }}"
      NGROK_PROTOCOL: "{{ SERVICE_NGROK_PROTOCOL | default('http') }}"

    command:
      - "{{ SERVICE_NGROK_PROTOCOL | default('http') }}"
      - "{{ SERVICE_NGROK_ADDR | default('nginx:80') }}"

    ports:
      - "{{ HOST_PORT_NGROK | default('4040') }}:4040"

    volumes:
      - ./logs/ngrok:/var/log/ngrok

    networks:
      - "{{ DOCKER_DEFAULT_NETWORK }}"
