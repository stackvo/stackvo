[supervisord]
nodaemon=true
user=root
logfile=/var/log/supervisord.log
pidfile=/var/run/supervisord.pid

[program:php-fpm]
command=php-fpm -F
autostart=true
autorestart=true
stdout_logfile=/dev/stdout
stdout_logfile_maxbytes=0
stderr_logfile=/dev/stderr
stderr_logfile_maxbytes=0

[program:nginx]
command=nginx -g 'daemon off;'
autostart=true
autorestart=true
stdout_logfile=/dev/stdout
stdout_logfile_maxbytes=0
stderr_logfile=/dev/stderr
stderr_logfile_maxbytes=0

{{#TOOLS_KAFBAT_ENABLE}}
[program:kafbat]
command=java -jar /opt/kafbat/kafka-ui.jar
autostart=true
autorestart=true
stdout_logfile=/dev/stdout
stdout_logfile_maxbytes=0
stderr_logfile=/dev/stderr
stderr_logfile_maxbytes=0
environment=DYNAMIC_CONFIG_ENABLED="true",KAFKA_CLUSTERS_0_NAME="stackvo-kafka",KAFKA_CLUSTERS_0_BOOTSTRAPSERVERS="stackvo-kafka:9092"
{{/TOOLS_KAFBAT_ENABLE}}
