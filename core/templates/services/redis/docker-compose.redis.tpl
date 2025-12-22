###################################################################
# STACKVO REDIS COMPOSE TEMPLATE
###################################################################

services:
  redis:
    image: "redis:{{ SERVICE_REDIS_VERSION }}"
    container_name: "stackvo-redis"
    restart: unless-stopped

    command: ["redis-server", "/etc/redis/redis.conf"]

    volumes:
      - stackvo-redis-data:/data
      - ./core/generated-configs/redis.conf:/etc/redis/redis.conf:ro
      - ./logs/redis:/var/log/redis

    ports:
      - "{{ HOST_PORT_REDIS | default('6379') }}:6379"

    networks:
      - "{{ DOCKER_DEFAULT_NETWORK }}"

volumes:
  stackvo-redis-data:
