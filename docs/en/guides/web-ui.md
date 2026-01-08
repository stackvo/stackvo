---
title: Web UI Usage
description: Stackvo Web UI allows you to visually manage services and projects.
---

# Web UI Usage

Stackvo Web UI allows you to visually manage services and projects. This guide detailedly explains how to use the dashboard, view service and project lists, create, edit, and delete projects, access management tools like Adminer, PhpMyAdmin, PhpPgAdmin, view logs, and API endpoints. The Web UI offers a user-friendly interface alternative to the CLI.

---

## Access

```
https://stackvo.loc
```

**Note:** Don't forget to add the domain to the `/etc/hosts` file:
```
127.0.0.1  stackvo.loc
```

---

## Dashboard

### Overview

The dashboard shows the general status of the system:

- **Running Services:** Number of active containers
- **Projects:** Total number of projects
- **CPU Usage:** System CPU usage
- **Memory Usage:** System RAM usage
- **Disk Usage:** Docker volume disk usage

### Quick Access

Sections you can quickly access from the dashboard:

- **Services:** Service list and statuses
- **Projects:** Project list and management
- **Tools:** Management tools
- **Logs:** Container logs
- **Settings:** System settings

---

## Services Tab

### Service List

Displays a list of all services:

| Service | Status | Version | URL | Actions |
|--------|-------|---------|-----|---------|
| MySQL | Running | 8.0 | mysql.stackvo.loc | Start/Stop/Restart |
| Redis | Running | 7.0 | - | Start/Stop/Restart |
| RabbitMQ | Running | 3 | rabbitmq.stackvo.loc | Start/Stop/Restart |

### Service Details

Click on a service to view details:

- **Container Name:** stackvo-mysql
- **Image:** mysql:8.0
- **Status:** Up 2 hours
- **Ports:** 0.0.0.0:3306->3306/tcp
- **Network:** stackvo-net
- **Volumes:** mysql-data
- **Environment Variables:** MYSQL_ROOT_PASSWORD, MYSQL_DATABASE, etc.

### Service Controls

For each service:

- **Start:** Start the service
- **Stop:** Stop the service
- **Restart:** Restart the service
- **Logs:** View logs
- **Stats:** CPU, Memory, Network usage

---

## Projects Tab

### Project List

Displays a list of all projects:

| Project | Domain | PHP Version | Webserver | Status | Actions |
|-------|--------|-------------|-----------|--------|---------|
| project1 | project1.loc | 8.2 | nginx | Running | Open/Edit/Delete |
| laravel-app | laravel.loc | 8.2 | nginx | Running | Open/Edit/Delete |

### Creating New Project

Click on the **New Project** button:

1. **Project Name:** myproject
2. **Domain:** myproject.loc
3. **PHP Version:** 8.2
4. **Webserver:** nginx
5. **Document Root:** public
6. **PHP Extensions:** pdo, pdo_mysql, mysqli, gd, curl, zip, mbstring

Click on the **Create** button.

**Note:** You need to manually update the `/etc/hosts` file:
```bash
echo "127.0.0.1  myproject.loc" | sudo tee -a /etc/hosts
```

### Editing Project

Click on the **Edit** button of a project:

- Change PHP version
- Change webserver
- Add/remove PHP extensions
- Change document root

Click on the **Save** button.

### Deleting Project

Click on the **Delete** button of a project:

1. Confirmation window opens
2. Click on **Confirm** button
3. Project containers are stopped
4. Project directory is deleted (optional)

**Note:** You need to manually remove the domain from the `/etc/hosts` file.

---

## Tools Tab

### Management Tools

Stackvo offers various management tools:

#### Adminer

**URL:** `https://adminer.stackvo.loc`

Universal management interface for all databases.

**Connection:**
- System: MySQL / PostgreSQL / MongoDB
- Server: stackvo-mysql / stackvo-postgres / stackvo-mongo
- Username: stackvo
- Password: stackvo
- Database: stackvo

#### PhpMyAdmin

**URL:** `https://phpmyadmin.stackvo.loc`

Management interface for MySQL and MariaDB.

**Connection:**
- Server: stackvo-mysql
- Username: stackvo
- Password: stackvo

#### PhpPgAdmin

**URL:** `https://phppgadmin.stackvo.loc`

Management interface for PostgreSQL.

#### PhpMongo

**URL:** `https://phpmongo.stackvo.loc`

Management interface for MongoDB.

#### PhpMemcachedAdmin

**URL:** `https://phpmemcachedadmin.stackvo.loc`

Management interface for Memcached.

#### OpCacheGUI

**URL:** `https://opcache.stackvo.loc`

PHP OPcache statistics and management.

#### Kafbat

**URL:** `https://kafbat.stackvo.loc`

Management interface for Kafka.

---

## Logs Tab

### Container Logs

View logs of all containers:

**Filters:**
- **Container:** Select specific container
- **Level:** INFO, WARNING, ERROR
- **Time Range:** Last 1 hour, 24 hours, 7 days

**Features:**
- Real-time log streaming
- Search/filter
- Download logs
- Clear logs

### Log Viewing

```
[2024-12-16 10:00:00] INFO: MySQL started successfully
[2024-12-16 10:00:01] INFO: Redis connected
[2024-12-16 10:00:02] WARNING: Slow query detected (2.5s)
[2024-12-16 10:00:03] ERROR: Connection refused to RabbitMQ
```

---

## Settings Tab

### Global Settings

**Editing .env File:**

Edit the `.env` file via UI:

- **Traefik Settings**
- **Default Project Settings**
- **Docker Network**
- **Security Settings**
- **Port Mappings**
- **Service Versions**

Click on **Save** button and run `./stackvo.sh generate`.

### System Information

- **Docker Version:** 24.0.7
- **Docker Compose Version:** 2.23.0
- **Stackvo Version:** 1.0.0
- **OS:** Ubuntu 22.04
- **Total Containers:** 15
- **Total Volumes:** 8
- **Total Networks:** 1

---

## API Endpoints

Web UI uses the following API endpoints:

### Services API

```
GET /api/services.php
```

Returns list of all services.

### Projects API

```
GET /api/projects.php
```

Returns list of all projects.

### Docker Stats API

```
GET /api/docker-stats.php
```

Returns container statistics.

### Control API

```
POST /api/control.php
```

Controls containers (start/stop/restart).

**Payload:**
```json
{
  "action": "restart",
  "container": "stackvo-mysql"
}
```

### Create Project API

```
POST /api/create-project.php
```

Creates a new project.

**Payload:**
```json
{
  "name": "myproject",
  "domain": "myproject.loc",
  "php_version": "8.2",
  "webserver": "nginx",
  "document_root": "public",
  "php_extensions": ["pdo", "pdo_mysql", "mysqli"]
}
```

### Delete Project API

```
POST /api/delete-project.php
```

Deletes a project.

**Payload:**
```json
{
  "name": "myproject"
}
```

---

## Troubleshooting

### UI Not Opening

```bash
# Check container status
docker ps | grep stackvo-ui

# Check logs
docker logs stackvo-ui

# Check hosts file
cat /etc/hosts | grep stackvo.loc

# Restart
docker restart stackvo-ui
```

### API Errors

```bash
# Check PHP logs
docker logs stackvo-ui

# Test API endpoint
curl https://stackvo.loc/api/services.php

# Check permissions
docker exec stackvo-ui ls -la /var/www/html/api/
```

### Slow Performance

```bash
# Container stats
docker stats stackvo-ui

# Increase resource limits
# In docker-compose.yml:
# resources:
#   limits:
#     memory: 512M
#     cpus: '1.0'
```
