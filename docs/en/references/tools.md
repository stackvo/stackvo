# Tools Reference

Comprehensive reference for all management and admin tools available in Stackvo. These web-based tools provide graphical interfaces for managing databases, cache systems, message queues, and PHP performance. All tools are accessible via Stackvo Web UI and run inside the tools container.

---

## Tool Categories

- Database Management Tools (4)
- Cache Management Tools (1)
- Message Queue Management Tools (1)
- Performance Monitoring Tools (1)

---

## Database Management Tools

### Adminer

**Version:** 4.8.1  
**URL:** `https://adminer.stackvo.loc`  
**Environment Variable:** `TOOLS_ADMINER_ENABLE`

**Description:**  
Adminer is a full-featured database management tool written in PHP. It is a lightweight alternative to phpMyAdmin and supports multiple database systems in a single interface.

**Supported Databases:**
- MySQL
- MariaDB
- PostgreSQL
- SQLite
- MongoDB
- Oracle
- MS SQL
- Elasticsearch

**Key Features:**
- Universal database interface
- Lightweight structure (single PHP file)
- Support for multiple database systems
- Import/export data in various formats (SQL, CSV, XML)
- Run custom SQL queries
- Manage tables, views, triggers, and stored procedures
- User and privilege management
- Database schema visualization

**Connection Examples:**

MySQL/MariaDB:
```
System: MySQL
Server: stackvo-mysql
Username: stackvo
Password: stackvo
Database: stackvo
```

PostgreSQL:
```
System: PostgreSQL
Server: stackvo-postgres
Username: stackvo
Password: root
Database: stackvo
```

MongoDB:
```
System: MongoDB
Server: stackvo-mongo
Username: root
Password: root
Database: stackvo
```

**Configuration:**
```bash
TOOLS_ADMINER_ENABLE=true
TOOLS_ADMINER_VERSION=4.8.1
TOOLS_ADMINER_URL=adminer
```

---

### PhpMyAdmin

**Version:** 5.2.1  
**URL:** `https://phpmyadmin.stackvo.loc`  
**Environment Variable:** `TOOLS_PHPMYADMIN_ENABLE`

**Description:**  
PhpMyAdmin is the most popular web-based management tool for MySQL and MariaDB databases. It provides a comprehensive database management interface with advanced features.

**Supported Databases:**
- MySQL
- MariaDB

**Key Features:**
- Intuitive web interface for MySQL/MariaDB
- View, create, and modify databases, tables, fields, and indexes
- Execute SQL statements and batch queries
- Import/export data (SQL, CSV, XML, PDF, Excel, etc.)
- User and privilege management
- Server configuration and status monitoring
- Visual query builder
- Database search and replace
- Bookmark frequently used queries
- Manage multiple servers

**Connection:**
```
Server: stackvo-mysql (or stackvo-mariadb)
Username: stackvo
Password: stackvo
```

**Root Access:**
```
Username: root
Password: root
```

**Configuration:**
```bash
TOOLS_PHPMYADMIN_ENABLE=true
TOOLS_PHPMYADMIN_VERSION=5.2.1
TOOLS_PHPMYADMIN_URL=phpmyadmin
```

---

### PhpPgAdmin

**Version:** 7.13.0  
**URL:** `https://phppgadmin.stackvo.loc`  
**Environment Variable:** `TOOLS_PHPPGADMIN_ENABLE`

**Description:**  
PhpPgAdmin is a web-based management tool for PostgreSQL databases. It provides a user-friendly interface to manage PostgreSQL servers, databases, and objects.

**Supported Databases:**
- PostgreSQL

**Key Features:**
- Complete PostgreSQL database management
- View and modify databases, schemas, tables, and views
- Execute SQL queries with syntax highlighting
- Import/export data
- User, group, and privilege management
- Create and manage functions, triggers, and sequences
- Visual schema browser
- Advanced search features
- Support for PostgreSQL-specific features (array, JSON, etc.)

**Connection:**
```
Server: stackvo-postgres
Username: stackvo
Password: root
Database: stackvo
```

**Configuration:**
```bash
TOOLS_PHPPGADMIN_ENABLE=true
TOOLS_PHPPGADMIN_VERSION=7.13.0
TOOLS_PHPPGADMIN_URL=phppgadmin
```

---

### PhpMongo

**Version:** 1.3.3  
**URL:** `https://phpmongo.stackvo.loc`  
**Environment Variable:** `TOOLS_PHPMONGO_ENABLE`

**Description:**  
PhpMongo is a web-based MongoDB management tool that provides an intuitive interface to manage MongoDB databases, collections, and documents.

**Supported Databases:**
- MongoDB

**Key Features:**
- MongoDB database and collection management
- Document CRUD operations (Create, Read, Update, Delete)
- JSON document viewer and editor
- Execute MongoDB queries
- Import/export collections
- Index management
- User and role management
- Database statistics and monitoring
- GridFS file management

**Connection:**
```
Server: stackvo-mongo
Port: 27017
Username: root
Password: root
Database: stackvo
Authentication Database: admin
```

**Configuration:**
```bash
TOOLS_PHPMONGO_ENABLE=true
TOOLS_PHPMONGO_VERSION=1.3.3
TOOLS_PHPMONGO_URL=phpmongo
```

---

## Cache Management Tools

### PhpMemcachedAdmin

**Version:** 1.3.0  
**URL:** `https://phpmemcachedadmin.stackvo.loc`  
**Environment Variable:** `TOOLS_PHPMEMCACHEDADMIN_ENABLE`

**Description:**  
PhpMemcachedAdmin is a web-based management tool for Memcached servers. It provides real-time monitoring and management capabilities for your cache infrastructure.

**Supported Systems:**
- Memcached

**Key Features:**
- Real-time Memcached server monitoring
- View cache statistics (hit rate, memory usage, connections)
- View cached items and their values
- Delete individual cache items or flush entire cache
- Multiple server support
- Visual charts for cache performance
- Memory usage visualization
- Connection monitoring

**Connection:**
```
Server: stackvo-memcached
Port: 11211
```

**Configuration:**
```bash
TOOLS_PHPMEMCACHEDADMIN_ENABLE=true
TOOLS_PHPMEMCACHEDADMIN_VERSION=1.3.0
TOOLS_PHPMEMCACHEDADMIN_URL=phpmemcachedadmin
```

---

## Message Queue Management Tools

### Kafbat (Kafka UI)

**Version:** 1.4.2  
**URL:** `https://kafbat.stackvo.loc`  
**Environment Variable:** `TOOLS_KAFBAT_ENABLE`

**Description:**  
Kafbat (formerly Kafka UI) is a modern web interface to manage and monitor Apache Kafka clusters. It provides comprehensive tools to work with topics, messages, consumer groups, and cluster configuration.

**Supported Systems:**
- Apache Kafka
- Kafka Connect
- Schema Registry

**Key Features:**
- Kafka cluster monitoring and management
- Topic creation, configuration, and deletion
- View and search messages in topics
- Send messages to topics
- Consumer group monitoring and management
- Partition and replica management
- Kafka Connect connector management
- Schema Registry integration
- ACL (Access Control List) management
- Real-time metrics and statistics

**Connection:**
```
Kafka Broker: stackvo-kafka:9092
```

**Configuration:**
```bash
TOOLS_KAFBAT_ENABLE=true
TOOLS_KAFBAT_VERSION=1.4.2
TOOLS_KAFBAT_URL=kafbat
```

**Note:** Kafbat requires Kafka service to be enabled:
```bash
SERVICE_KAFKA_ENABLE=true
```

---

## Performance Monitoring Tools

### OpCache GUI

**Version:** 3.6.0  
**URL:** `https://opcache.stackvo.loc`  
**Environment Variable:** `TOOLS_OPCACHE_ENABLE`

**Description:**  
OpCache GUI is a web-based interface to monitor and manage PHP OPcache. It provides detailed statistics about cached scripts, memory usage, and cache performance.

**Supported Systems:**
- PHP OPcache

**Key Features:**
- Real-time OPcache statistics
- Memory usage visualization
- Detailed cached file list
- Cache hit/miss rate monitoring
- Invalidate specific cached files
- Reset entire cache
- Configuration summary
- Performance graphs and charts
- Memory fragmentation analysis

**Usage:**
- Access URL to view OPcache statistics
- Monitor cache efficiency and memory usage
- Identify frequently cached scripts
- Clear cache when needed during development

**Configuration:**
```bash
TOOLS_OPCACHE_ENABLE=true
TOOLS_OPCACHE_VERSION=3.6.0
TOOLS_OPCACHE_URL=opcache
```

**Note:** OPcache statistics are collected from all PHP project containers running in Stackvo.

---

## Accessing Tools

### Via Web UI

1. Open Stackvo Web UI: `https://stackvo.loc`
2. Go to **Tools** tab
3. Click on the desired tool to open in a new tab

### Direct Access

All tools can be accessed directly via their URLs:

```
https://adminer.stackvo.loc
https://phpmyadmin.stackvo.loc
https://phppgadmin.stackvo.loc
https://phpmongo.stackvo.loc
https://phpmemcachedadmin.stackvo.loc
https://opcache.stackvo.loc
https://kafbat.stackvo.loc
```

**Important:** Make sure to add these domains to your `/etc/hosts` file:

```bash
127.0.0.1  adminer.stackvo.loc
127.0.0.1  phpmyadmin.stackvo.loc
127.0.0.1  phppgadmin.stackvo.loc
127.0.0.1  phpmongo.stackvo.loc
127.0.0.1  phpmemcachedadmin.stackvo.loc
127.0.0.1  opcache.stackvo.loc
127.0.0.1  kafbat.stackvo.loc
```

---

## Enabling/Disabling Tools

### Enabling a Tool

Edit `.env` file and set the tool's enable flag to `true`:

```bash
# Enable Adminer
TOOLS_ADMINER_ENABLE=true

# Enable PhpMyAdmin
TOOLS_PHPMYADMIN_ENABLE=true
```

### Disabling a Tool

Set enable flag to `false`:

```bash
# Disable Kafbat
TOOLS_KAFBAT_ENABLE=false
```

### Applying Changes

After changing `.env` file, regenerate configuration and restart:

```bash
./stackvo.sh generate
./stackvo.sh restart
```

---

## Troubleshooting

### Tool Not Accessible

```bash
# Check if tools container is running
docker ps | grep stackvo-tools

# Check container logs
docker logs stackvo-tools

# Verify hosts file
cat /etc/hosts | grep stackvo.loc

# Restart tools container
docker restart stackvo-tools
```

### Connection Errors

```bash
# Verify service is running
docker ps | grep stackvo-mysql

# Check network connectivity
docker exec stackvo-tools ping stackvo-mysql

# Verify service credentials in .env file
cat .env | grep SERVICE_MYSQL
```

### Performance Issues

```bash
# Check container resource usage
docker stats stackvo-tools

# View detailed logs
docker logs -f stackvo-tools

# Restart container
docker restart stackvo-tools
```

---

## Security Considerations

1. **Production Usage:** These tools are designed for development environments. For production consider:
   - Disabling tools or restricting access
   - Using strong authentication
   - Applying IP whitelisting
   - Using VPN or SSH tunneling

2. **Credentials:** Default credentials are set in `.env` file. Change them for production:
   ```bash
   SERVICE_MYSQL_ROOT_PASSWORD=strong_password_here
   SERVICE_POSTGRES_PASSWORD=strong_password_here
   ```

3. **SSL/TLS:** When `SSL_ENABLE=true` in `.env` file, all tools are accessible via HTTPS

---
