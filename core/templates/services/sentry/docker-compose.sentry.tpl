###################################################################
# STACKVO SENTRY COMPOSE TEMPLATE
###################################################################

services:
  sentry:
    profiles: ["services", "sentry"]  # --services ile tümü, --profile sentry ile sadece bu servis
    image: "getsentry/sentry:{{ SERVICE_SENTRY_VERSION }}"
    container_name: "stackvo-sentry"
    restart: unless-stopped

    environment:
      SENTRY_SECRET_KEY: "{{ SERVICE_SENTRY_SECRET_KEY | default('stackvo-sentry-secret-key-change-me') }}"
      SENTRY_SINGLE_ORGANIZATION: "true"
      
      # Redis Configuration
      SENTRY_REDIS_HOST: "sentry-redis"
      SENTRY_REDIS_PORT: "6379"
      
      # PostgreSQL Configuration
      SENTRY_POSTGRES_HOST: "sentry-postgres"
      SENTRY_POSTGRES_PORT: "5432"
      SENTRY_DB_NAME: "sentry"
      SENTRY_DB_USER: "sentry"
      SENTRY_DB_PASSWORD: "{{ SERVICE_SENTRY_DB_PASSWORD | default('sentry') }}"
    
    command: >
      bash -c "
      sentry upgrade --noinput &&
      sentry run web
      "

    ports:
      - "{{ HOST_PORT_SENTRY | default('9001') }}:9000"

    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.sentry.rule=Host(`sentry.stackvo.loc`)"
      - "traefik.http.routers.sentry.service=sentry"
      - "traefik.http.routers.sentry.entrypoints=websecure"
      - "traefik.http.services.sentry.loadbalancer.server.port=9000"
      - "traefik.http.routers.sentry.tls=true"

    volumes:
      - ../logs/services/sentry:/var/log/sentry

    depends_on:
      - sentry-redis
      - sentry-postgres

    networks:
      - "{{ DOCKER_DEFAULT_NETWORK }}"

  sentry-redis:
    image: redis:7
    container_name: "stackvo-sentry-redis"
    restart: unless-stopped
    networks:
      - "{{ DOCKER_DEFAULT_NETWORK }}"

  sentry-postgres:
    image: postgres:15
    container_name: "stackvo-sentry-postgres"
    restart: unless-stopped
    environment:
      POSTGRES_DB: sentry
      POSTGRES_USER: sentry
      POSTGRES_PASSWORD: "{{ SERVICE_SENTRY_DB_PASSWORD | default('sentry') }}"
    volumes:
      - stackvo-sentry-postgres-data:/var/lib/postgresql/data
    networks:
      - "{{ DOCKER_DEFAULT_NETWORK }}"

volumes:
  stackvo-sentry-postgres-data:
