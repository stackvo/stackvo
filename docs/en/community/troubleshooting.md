# Troubleshooting

Common causes and solutions. This page details all common problems and their step-by-step solutions, ranging from Docker problems (daemon, permission, port conflict) to generator and network problems, SSL/TLS and container problems to database and web server problems, CLI and volume problems to emergency scenarios. It includes symptoms and solution examples for each problem.

---

## General Troubleshooting

### System Check

```bash
# Stackvo doctor
stackvo doctor

# Docker check
docker --version
docker compose --version
docker ps

# Check logs
cat core/generator.log
```

---

## Docker Issues

### Docker daemon is not running

**Symptom:**
```
Cannot connect to the Docker daemon
```

**Solution:**
```bash
# Linux
sudo systemctl start docker
sudo systemctl enable docker

# macOS
open -a Docker

# WSL2
sudo service docker start
```

### Permission error

**Symptom:**
```
permission denied while trying to connect to the Docker daemon socket
```

**Solution:**
```bash
# Add user to docker group
sudo usermod -aG docker $USER
newgrp docker

# Or run with sudo
sudo ./stackvo.sh up
```

### Port conflict

**Symptom:**
```
Bind for 0.0.0.0:3306 failed: port is already allocated
```

**Solution:**
```bash
# Which process is using it?
sudo lsof -i :3306

# Change port in .env
nano .env
# HOST_PORT_MYSQL=3307

./stackvo.sh generate
./stackvo.sh restart
```

---

## Generator Issues

### Generate error

**Symptom:**
```
Error generating docker-compose files
```

**Solution:**
```bash
# Verbose mode
STACKVO_VERBOSE=true ./stackvo.sh generate

# Check logs
cat core/generator.log

# Template check
ls -la core/compose/
ls -la core/templates/
```

### stackvo.json parse error

**Symptom:**
```
Error parsing stackvo.json
```

**Solution:**
```bash
# JSON syntax check
cat projects/myproject/stackvo.json | jq .

# Example valid format
{
  "name": "myproject",
  "domain": "myproject.loc",
  "php": {"version": "8.2"},
  "webserver": "nginx",
  "document_root": "public"
}
```

---

## Network Issues

### Containers cannot see each other

**Symptom:**
```
Could not connect to stackvo-mysql
```

**Solution:**
```bash
# Network check
docker network inspect stackvo-net

# Is container connected to network?
docker inspect stackvo-mysql | grep -A 10 Networks

# Ping test
docker exec stackvo-php ping stackvo-mysql

# Recreate network
./stackvo.sh down
docker network rm stackvo-net
./stackvo.sh generate
./stackvo.sh up
```

### DNS resolution issue

**Symptom:**
```
Name or service not known
```

**Solution:**
```bash
# DNS test inside container
docker exec stackvo-php nslookup stackvo-mysql
docker exec stackvo-php cat /etc/resolv.conf

# Docker DNS restart
sudo systemctl restart docker
```

---

## SSL/TLS Issues

### SSL certificate error

**Symptom:**
```
SSL certificate problem: self signed certificate
```

**Solution:**
```bash
# Regenerate certificates
./core/cli/utils/generate-ssl-certs.sh

# Accept certificate in browser
# Chrome: Advanced → Proceed to site
# Firefox: Advanced → Accept the Risk
```

### Traefik SSL error

**Symptom:**
```
Traefik cannot find SSL certificates
```

**Solution:**
```bash
# Check certificate path
ls -la core/certs/

# Check Traefik config
cat core/traefik/traefik.yml

# Traefik restart
docker restart stackvo-traefik
```

---

## Container Issues

### Container is not starting

**Symptom:**
```
Container exited with code 1
```

**Solution:**
```bash
# Check logs
docker logs stackvo-mysql

# Container details
docker inspect stackvo-mysql

# Recreate
docker compose up -d --force-recreate stackvo-mysql
```

### Container is restarting continuously

**Symptom:**
```
Container is restarting continuously
```

**Solution:**
```bash
# Last 100 log lines
docker logs --tail=100 stackvo-mysql

# Health check
docker inspect --format='{{.State.Health.Status}}' stackvo-mysql

# Stop container and inspect logs
docker stop stackvo-mysql
docker logs stackvo-mysql
```

---

## Database Issues

### MySQL connection error

**Symptom:**
```
SQLSTATE[HY000] [2002] Connection refused
```

**Solution:**
```bash
# Is container running?
docker ps | grep mysql

# Connection details
Host: stackvo-mysql  # NOT localhost!
Port: 3306             # Internal port
User: stackvo
Password: stackvo

# Network test
docker exec stackvo-php nc -zv stackvo-mysql 3306
```

### PostgreSQL authentication error

**Symptom:**
```
FATAL: password authentication failed
```

**Solution:**
```bash
# Check .env
cat .env | grep POSTGRES

# Correct credentials
Host: stackvo-postgres
Port: 5432
User: stackvo
Password: root  # POSTGRES_PASSWORD in .env
```

### MongoDB connection timeout

**Symptom:**
```
MongoNetworkError: connection timed out
```

**Solution:**
```bash
# Check container
docker ps | grep mongo

# Connection string
mongodb://root:root@stackvo-mongo:27017/dbname?authSource=admin

# Network test
docker exec stackvo-php nc -zv stackvo-mongo 27017
```

---

## Web Server Issues

### 404 Not Found

**Symptom:**
```
404 Not Found - nginx
```

**Solution:**
```bash
# Document root check
docker exec stackvo-myproject-web ls -la /var/www/html/public

# Nginx config
docker exec stackvo-myproject-web cat /etc/nginx/conf.d/default.conf

# Nginx syntax test
docker exec stackvo-myproject-web nginx -t

# Nginx reload
docker exec stackvo-myproject-web nginx -s reload
```

### 502 Bad Gateway

**Symptom:**
```
502 Bad Gateway - nginx
```

**Solution:**
```bash
# Is PHP-FPM running?
docker ps | grep php

# PHP-FPM logs
docker logs stackvo-myproject-php

# FastCGI connection
docker exec stackvo-myproject-web nc -zv myproject-php 9000

# PHP-FPM restart
docker restart stackvo-myproject-php
```

### Permission denied

**Symptom:**
```
Permission denied: /var/www/html/storage
```

**Solution:**
```bash
# Permissions on Host
sudo chown -R $USER:$USER projects/myproject

# Inside Container
docker exec stackvo-myproject-php chown -R www-data:www-data /var/www/html
docker exec stackvo-myproject-php chmod -R 775 /var/www/html/storage
```

---

## CLI Issues

### Command not found

**Symptom:**
```
stackvo: command not found
```

**Solution:**
```bash
# Install CLI
./stackvo.sh install

# Or use full path
./stackvo.sh generate
```

### Script execution error

**Symptom:**
```
Permission denied: ./stackvo.sh
```

**Solution:**
```bash
# Make executable
chmod +x cli/stackvo.sh
chmod +x cli/commands/*.sh
chmod +x cli/lib/generators/*.sh
```

---

## Volume Issues

### Data loss

**Symptom:**
```
All database data is lost after restart
```

**Solution:**
```bash
# Check volumes
docker volume ls | grep stackvo

# Inspect volume
docker volume inspect stackvo_mysql-data

# Backup
docker run --rm \
  -v stackvo_mysql-data:/data \
  -v $(pwd):/backup \
  ubuntu tar czf /backup/mysql-backup.tar.gz /data
```

### Volume mount error

**Symptom:**
```
Error response from daemon: invalid mount config
```

**Solution:**
```bash
# Use absolute path
volumes:
  - /absolute/path/to/projects:/var/www/html

# Instead of relative path
volumes:
  - ./projects:/var/www/html  # ❌ Incorrect
```

---

## Emergency

### Reset entire system

```bash
# 1. Stop all containers
./stackvo.sh down -v

# 2. Remove network
docker network rm stackvo-net

# 3. Remove generated files
rm -rf generated/*

# 4. Recreate
./stackvo.sh generate
./stackvo.sh up
```

### Restore from backup

```bash
# MySQL
docker exec -i stackvo-mysql mysql -u root -proot < backup.sql

# PostgreSQL
docker exec -i stackvo-postgres psql -U stackvo < backup.sql

# Volume
docker run --rm \
  -v stackvo_mysql-data:/data \
  -v $(pwd):/backup \
  ubuntu tar xzf /backup/mysql-backup.tar.gz -C /
```

---

## Still unresolved?

1. **GitHub Issues:** [Report issue](https://github.com/stackvo/stackvo/issues/new)
2. **Discussions:** [Join discussions](https://github.com/stackvo/stackvo/discussions)
3. **Support:** [Get support](support.md)

**When opening an Issue:**
- Add error message
- Share `stackvo doctor` output
- Add logs
- Provide Environment information
