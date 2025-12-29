###################################################################
# STACKVO SONARQUBE COMPOSE TEMPLATE
###################################################################

services:
  sonarqube:
    profiles: ["services", "sonarqube"]  # --services ile tümü, --profile sonarqube ile sadece bu servis
    image: "sonarqube:{{ SERVICE_SONARQUBE_VERSION }}"
    container_name: "stackvo-sonarqube"
    restart: unless-stopped

    environment:
      SONAR_ES_BOOTSTRAP_CHECKS_DISABLE: "true"
      SONAR_SEARCH_JAVAADDITIONALOPTS: "-Dnode.store.allow_mmap=false"
      SONAR_WEB_JAVAOPTS: "{{ SERVICE_SONARQUBE_JAVA_OPTS | default('-Xms512m -Xmx512m') }}"
      SONARQUBE_JDBC_URL: "{{ SERVICE_SONARQUBE_JDBC_URL | default('jdbc:postgresql://postgres/sonarqube') }}"
      SONARQUBE_JDBC_USERNAME: "{{ SERVICE_SONARQUBE_DB_USERNAME | default('sonar') }}"
      SONARQUBE_JDBC_PASSWORD: "{{ SERVICE_SONARQUBE_DB_PASSWORD | default('sonar') }}"

    ports:
      - "{{ HOST_PORT_SONARQUBE | default('9000') }}:9000"

    volumes:
      - stackvo-sonarqube-data:/opt/sonarqube/data
      - stackvo-sonarqube-extensions:/opt/sonarqube/extensions
      - ../logs/sonarqube:/opt/sonarqube/logs

    networks:
      - "{{ DOCKER_DEFAULT_NETWORK }}"

volumes:
  stackvo-sonarqube-data:
  stackvo-sonarqube-extensions:
