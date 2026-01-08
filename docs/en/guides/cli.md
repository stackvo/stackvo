---
title: CLI Usage
description: Stackvo CLI is the main tool for all system management. This guide shows you how to use CLI commands.
---

# CLI Usage

Stackvo CLI is the main tool for all system management. This guide details how to use basic commands like generate, up, down, restart, ps, logs, advanced features like verbose mode and dry run, adding new services, creating projects, and troubleshooting methods. The CLI simplifies and automates Docker Compose commands.

---

## Installation

### Installing CLI to System

```bash
# Go to Stackvo directory
cd /path/to/stackvo

# Install CLI
./stackvo.sh install
```

This command adds the `stackvo` command as a symbolic link to the `/usr/local/bin/` directory.

**Verification:**
```bash
# Run from any directory
stackvo --help
```

---

## Basic Commands

### generate

Generates configuration files.

```bash
# Generate all configurations
./stackvo.sh generate

# Generate only projects
./stackvo.sh generate projects

# Generate only services
./stackvo.sh generate services
```

**What it does:**
1. Reads `.env` file
2. Generates SSL certificates (if missing)
3. Generates `generated/stackvo.yml`
4. Generates `generated/docker-compose.dynamic.yml`
5. Generates `generated/docker-compose.projects.yml`
6. Generates `core/traefik/dynamic/routes.yml`
7. Generates service configurations

**Example Output:**
```
✅ SSL certificates found
✅ Generated stackvo.yml
✅ Generated docker-compose.dynamic.yml
✅ Generated docker-compose.projects.yml
✅ Generated Traefik routes
✅ Generated 15 service configurations
✅ Generation completed!
```

### up

Starts services. By default, it only starts core services (Traefik + UI).

**Syntax:**
```bash
./stackvo.sh up [OPTIONS]
```

**Options:**
- (empty) - Minimal mode: Only core services (Traefik + UI)
- `--all` - Start all services and projects
- `--services` - Start Core + all services
- `--projects` - Start Core + all projects
- `--profile <name>` - Start Core + specific profile

**Examples:**
```bash
# Minimal mode - Only Traefik + UI
./stackvo.sh up

# Start all services and projects
./stackvo.sh up --all

# Start Core + all services
./stackvo.sh up --services

# Start Core + all projects
./stackvo.sh up --projects

# Start Core + only MySQL
./stackvo.sh up --profile mysql

# Start Core + specific project
./stackvo.sh up --profile project-myproject

# Multiple profiles
./stackvo.sh up --profile mysql --profile redis
```

**Detailed Output:**
```bash
# Verbose mode
STACKVO_VERBOSE=true ./stackvo.sh up
```

**Starting Specific Services:**
```bash
# Use Docker Compose command directly
docker compose -f generated/stackvo.yml \
  -f generated/docker-compose.dynamic.yml \
  -f generated/docker-compose.projects.yml \
  up -d mysql redis
```

### down

Stops all services.

```bash
./stackvo.sh down
```

**Deleting Volumes too:**
```bash
./stackvo.sh down -v
```

**Removing Orphan Containers:**
```bash
./stackvo.sh down --remove-orphans
```

### restart

Restarts services.

```bash
# Restart all services
./stackvo.sh restart

# Restart specific services
docker compose -f generated/stackvo.yml \
  -f generated/docker-compose.dynamic.yml \
  restart mysql redis
```

### ps

Lists running services.

```bash
./stackvo.sh ps
```

**Example Output:**
```
NAME                      STATUS              PORTS
stackvo-traefik         Up 2 hours          0.0.0.0:80->80/tcp, 0.0.0.0:443->443/tcp
stackvo-mysql           Up 2 hours          0.0.0.0:3306->3306/tcp
stackvo-redis           Up 2 hours          0.0.0.0:6379->6379/tcp
stackvo-rabbitmq        Up 2 hours          0.0.0.0:5672->5672/tcp
stackvo-project1-php    Up 2 hours          9000/tcp
stackvo-project1-web    Up 2 hours          80/tcp
```

### logs

Displays container logs.

```bash
# Watch all logs
./stackvo.sh logs

# Watch specific service log
./stackvo.sh logs mysql

# Follow mode
./stackvo.sh logs -f mysql

# Last 100 lines
./stackvo.sh logs --tail=100 mysql

# Multiple services
./stackvo.sh logs mysql redis
```

**With Timestamp:**
```bash
./stackvo.sh logs -f --timestamps mysql
```

### pull

Pulls Docker images.

```bash
./stackvo.sh pull
```

**Pulling Specific Images:**
```bash
docker compose -f generated/stackvo.yml \
  -f generated/docker-compose.dynamic.yml \
  pull mysql redis
```

### doctor

Performs system health check.

```bash
stackvo doctor
```

**Checked Items:**
- Is Docker installed?
- Is Docker Compose installed?
- Is Docker daemon running?
- Are required ports open?
- Is there a `.env` file?
- Are there SSL certificates?

### uninstall

Uninstalls Stackvo.

```bash
./stackvo.sh uninstall
```

**What it does:**
1. Stops all containers
2. Deletes volumes (asks for confirmation)
3. Deletes network
4. Removes CLI symbolic link

---

## Advanced Usage

### Verbose Mode

For detailed output:

```bash
STACKVO_VERBOSE=true ./stackvo.sh generate
STACKVO_VERBOSE=true ./stackvo.sh up
```

### Dry Run

To see commands without executing them:

```bash
STACKVO_DRY_RUN=true ./stackvo.sh generate
```

### Custom .env File

```bash
# Use a different .env file
cp .env .env.production
nano .env.production

# Use with generate
ENV_FILE=.env.production ./stackvo.sh generate
```

### Working with Specific Compose Files

```bash
# Base layer only
docker compose -f generated/stackvo.yml up -d

# Base + services
docker compose -f generated/stackvo.yml \
  -f generated/docker-compose.dynamic.yml up -d

# All (default)
docker compose -f generated/stackvo.yml \
  -f generated/docker-compose.dynamic.yml \
  -f generated/docker-compose.projects.yml up -d
```

---

## Common Scenarios

### Adding New Service

```bash
# 1. Edit .env file
nano .env

# Enable Elasticsearch
# SERVICE_ELASTICSEARCH_ENABLE=true

# 2. Regenerate configurations
./stackvo.sh generate

# 3. Restart services
./stackvo.sh up
```

### Adding Project

```bash
# 1. Create project directory
mkdir -p projects/newproject/public

# 2. Create stackvo.json
cat > projects/newproject/stackvo.json <<EOF
{
  "name": "newproject",
  "domain": "newproject.loc",
  "php": {"version": "8.2"},
  "webserver": "nginx",
  "document_root": "public"
}
EOF

# 3. Test file
echo "<?php phpinfo();" > projects/newproject/public/index.php

# 4. Regenerate projects
./stackvo.sh generate projects

# 5. Restart services
./stackvo.sh restart

# 6. Update hosts file
echo "127.0.0.1  newproject.loc" | sudo tee -a /etc/hosts
```

### Changing Service Version

```bash
# 1. Edit .env file
nano .env

# Change MySQL version
# SERVICE_MYSQL_VERSION=8.0 → 8.1

# 2. Regenerate configurations
./stackvo.sh generate services

# 3. Recreate MySQL container
docker compose -f generated/stackvo.yml \
  -f generated/docker-compose.dynamic.yml \
  up -d --force-recreate mysql
```

### Resetting Entire System

```bash
# 1. Stop and remove all containers
./stackvo.sh down -v

# 2. Delete network
docker network rm stackvo-net

# 3. Delete generated files
rm -rf generated/*

# 4. Regenerate
./stackvo.sh generate

# 5. Start
./stackvo.sh up
```

### Taking Backup

```bash
# MySQL backup
docker exec stackvo-mysql mysqldump -u root -proot --all-databases > backup.sql

# PostgreSQL backup
docker exec stackvo-postgres pg_dumpall -U stackvo > backup.sql

# MongoDB backup
docker exec stackvo-mongo mongodump --username root --password root --authenticationDatabase admin --out /backup

# Redis backup
docker exec stackvo-redis redis-cli SAVE
docker cp stackvo-redis:/data/dump.rdb ./redis-backup.rdb
```

---

## Troubleshooting

### Container Not Starting

```bash
# Check logs
./stackvo.sh logs <container-name>

# Inspect container details
docker inspect stackvo-<container-name>

# Regenerate configuration
./stackvo.sh generate
./stackvo.sh down
./stackvo.sh up
```

### Port Conflict

```bash
# Find container using which port
docker ps --format "table {{.Names}}\t{{.Ports}}"

# Change port in .env file
nano .env

# Regenerate and start
./stackvo.sh generate
./stackvo.sh restart
```

### Permission Error

```bash
# Add user to docker group
sudo usermod -aG docker $USER
newgrp docker

# Or run with sudo
sudo ./stackvo.sh up
```
