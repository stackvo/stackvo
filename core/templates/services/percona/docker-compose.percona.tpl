###################################################################
# STACKVO PERCONA COMPOSE TEMPLATE
###################################################################

services:
  percona:
    image: "percona:{{ SERVICE_PERCONA_VERSION }}"
    container_name: "stackvo-percona"
    platform: linux/amd64
    restart: unless-stopped
    
    environment:
      MYSQL_ROOT_PASSWORD: "{{ SERVICE_PERCONA_ROOT_PASSWORD | default('root') }}"
      MYSQL_DATABASE: "{{ SERVICE_PERCONA_DATABASE | default('stackvo') }}"
      MYSQL_USER: "{{ SERVICE_PERCONA_USER | default('stackvo') }}"
      MYSQL_PASSWORD: "{{ SERVICE_PERCONA_PASSWORD | default('stackvo') }}"

    volumes:
      - stackvo-percona-data:/var/lib/mysql
      - ./generated/configs/percona.cnf:/etc/mysql/conf.d/stackvo.cnf:ro
      - ../logs/percona:/var/log/mysql

    ports:
      - "{{ HOST_PORT_PERCONA | default('3307') }}:3306"

    networks:
      - "{{ DOCKER_DEFAULT_NETWORK }}"

volumes:
  stackvo-percona-data:
