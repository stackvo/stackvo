# FAQ - Frequently Asked Questions

Frequently asked questions and answers about Stackvo. This page covers a wide range of questions from general inquiries to installation and usage, troubleshooting to performance optimization, security to services, Web UI to backup and update procedures. It includes quick solutions and practical examples.

---

## General Questions

### What is Stackvo?

Stackvo is a Docker-based, fully customizable, and modular development environment management system. It supports 40+ services and is written in pure Bash.

### Is Stackvo free?

Yes, Stackvo is completely free and open source (MIT License).

### Which operating systems does it work on?

- Linux (Ubuntu, Debian, CentOS, Arch)
- macOS
- Windows (WSL2)

---

## Installation

### What should I do if Docker is not installed?

Follow the steps on the [Installation Guide](../installation/index.md) page.

### I get an error during installation

```bash
# System check
stackvo doctor

# Check logs
cat core/generator.log
```

### I get a port conflict error

Change the ports in the `.env` file:

```bash
HOST_PORT_POSTGRES=5433
HOST_PORT_PERCONA=3308
```

---

## Usage

### How do I create a new project?

```bash
# 1. Project directory
mkdir -p projects/myproject/public

# 2. stackvo.json
cat > projects/myproject/stackvo.json <<EOF
{
  "name": "myproject",
  "domain": "myproject.loc",
  "php": {"version": "8.2"},
  "webserver": "nginx",
  "document_root": "public"
}
EOF

# 3. Generate and start
./stackvo.sh generate projects
./stackvo.sh up

# 4. Hosts
echo "127.0.0.1  myproject.loc" | sudo tee -a /etc/hosts
```

### How do I enable a service?

Edit the `.env` file:

```bash
SERVICE_REDIS_ENABLE=true
SERVICE_REDIS_VERSION=7.0
```

Then:

```bash
./stackvo.sh generate
./stackvo.sh up
```

### How do I change the PHP version?

In the `stackvo.json` file:

```json
{
  "php": {
    "version": "8.3"
  }
}
```

Then:

```bash
./stackvo.sh generate projects
./stackvo.sh restart
```

---

## Troubleshooting

### Container is not starting

```bash
# Check logs
docker logs stackvo-mysql

# Recreate
./stackvo.sh down
./stackvo.sh generate
./stackvo.sh up
```

### I get a 404 error

```bash
# Document root check
docker exec stackvo-myproject-web ls -la /var/www/html/public

# Nginx config check
docker exec stackvo-myproject-web nginx -t
```

### Database connection error

```bash
# Is container running?
docker ps | grep mysql

# Network check
docker exec stackvo-php ping stackvo-mysql

# Connection details
Host: stackvo-mysql
Port: 3306
User: stackvo
Password: stackvo
```

---

## Performance

### System runs slowly

```bash
# Resource usage
docker stats

# Remove unused containers
docker system prune -a
```

### Build time is long

```bash
# Use cache
docker compose build --parallel

# Pull images beforehand
./stackvo.sh pull
```

---

## Security

### Can I use it in production?

Yes, but:
- Use strong passwords
- Enable SSL/TLS
- Add firewall rules
- Close unnecessary ports

### How do I change passwords?

In the `.env` file:

```bash
SERVICE_MYSQL_ROOT_PASSWORD=$(openssl rand -base64 32)
SERVICE_RABBITMQ_DEFAULT_PASS=$(openssl rand -base64 32)
```

---

## Services

### Which databases are supported?

- MySQL (5.6 - 8.1)
- MariaDB (10.6)
- PostgreSQL (9.6 - 16)
- MongoDB (4.0 - 7.0)
- Cassandra
- Percona
- CouchDB
- Couchbase

### How do I install Redis Cluster?

Currently, single node Redis is supported. Custom configuration is required for Cluster.

### How do I use Elasticsearch?

```bash
# .env
SERVICE_ELASTICSEARCH_ENABLE=true
SERVICE_KIBANA_ENABLE=true

./stackvo.sh generate
./stackvo.sh up
```

Access:
- Elasticsearch: http://localhost:9200
- Kibana: https://kibana.stackvo.loc

---

## Web UI

### I cannot access Web UI

```bash
# Container check
docker ps | grep stackvo-ui

# Hosts check
cat /etc/hosts | grep stackvo.loc

# Restart
docker restart stackvo-ui
```

### API is not working

```bash
# PHP logs
docker logs stackvo-ui

# Permissions
docker exec stackvo-ui ls -la /var/www/html/api/
```

---

## Backup

### How do I backup the database?

**MySQL:**
```bash
docker exec stackvo-mysql mysqldump -u root -proot --all-databases > backup.sql
```

**PostgreSQL:**
```bash
docker exec stackvo-postgres pg_dumpall -U stackvo > backup.sql
```

**MongoDB:**
```bash
docker exec stackvo-mongo mongodump --username root --password root --out /backup
```

### How do I backup volumes?

```bash
docker run --rm \
  -v stackvo_mysql-data:/data \
  -v $(pwd):/backup \
  ubuntu tar czf /backup/mysql-backup.tar.gz /data
```

---

## Updates

### How do I update Stackvo?

```bash
# Pull
git pull origin main

# Re-generate
./stackvo.sh generate

# Restart
./stackvo.sh restart
```

### How do I update images?

```bash
# Update all images
./stackvo.sh pull

# Restart
./stackvo.sh up --force-recreate
```

---

## Other

### Can I run multiple projects?

Yes, you can run unlimited projects.

### Can I use a custom domain?

Yes, in the `stackvo.json` file:

```json
{
  "domain": "myapp.local"
}
```

Add to `/etc/hosts` file:

```
127.0.0.1  myapp.local
```

### How to generate SSL certificate?

```bash
./core/cli/utils/generate-ssl-certs.sh
```

---

## Still have questions?

- [GitHub Discussions](https://github.com/stackvo/stackvo/discussions)
- [GitHub Issues](https://github.com/stackvo/stackvo/issues)
- [Support](support.md)
