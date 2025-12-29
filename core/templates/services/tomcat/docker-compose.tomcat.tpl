###################################################################
# STACKVO TOMCAT COMPOSE TEMPLATE
###################################################################

services:
  tomcat:
    profiles: ["services", "tomcat"]  # --services ile tümü, --profile tomcat ile sadece bu servis
    image: "tomcat:{{ SERVICE_TOMCAT_VERSION }}"
    container_name: "stackvo-tomcat"
    restart: unless-stopped

    volumes:
      - ./core/templates/appserver/tomcat/webapps:/usr/local/tomcat/webapps
      - ../logs/tomcat:/usr/local/tomcat/logs

    ports:
      - "{{ HOST_PORT_TOMCAT | default('8080') }}:8080"

    networks:
      - "{{ DOCKER_DEFAULT_NETWORK }}"

    environment:
      CATALINA_OPTS: "{{ SERVICE_TOMCAT_CATALINA_OPTS | default('-Xms512M -Xmx1024M') }}"
      JAVA_OPTS: "{{ SERVICE_TOMCAT_JAVA_OPTS | default('-Djava.awt.headless=true') }}"


