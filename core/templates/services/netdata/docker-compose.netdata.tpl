###################################################################
# STACKVO NETDATA COMPOSE TEMPLATE
###################################################################

services:
  netdata:
    image: "netdata/netdata:{{ SERVICE_NETDATA_VERSION }}"
    container_name: "stackvo-netdata"
    restart: unless-stopped

    cap_add:
      - SYS_PTRACE

    security_opt:
      - apparmor:unconfined

    ports:
      - "{{ HOST_PORT_NETDATA | default('19999') }}:19999"

    volumes:
      - netdata-config:/etc/netdata
      - netdata-lib:/var/lib/netdata
      - netdata-cache:/var/cache/netdata
      - /etc/passwd:/host/etc/passwd:ro
      - /etc/group:/host/etc/group:ro
      - /proc:/host/proc:ro
      - /sys:/host/sys:ro
      - /etc/os-release:/host/etc/os-release:ro
      - ./logs/netdata:/var/log/netdata

    networks:
      - "{{ DOCKER_DEFAULT_NETWORK }}"

volumes:
  netdata-config:
  netdata-lib:
  netdata-cache:
