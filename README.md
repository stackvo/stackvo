<div align="center">

# 🚀 StackVo

**Docker-Based Local Development Environment for Modern LAMP and MEAN Stacks**

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE.md)
[![Docker](https://img.shields.io/badge/Docker-Required-2496ED?logo=docker)](https://www.docker.com/)
[![Bash](https://img.shields.io/badge/Bash-3.x+-4EAA25?logo=gnubash)](https://www.gnu.org/software/bash/)
[![Traefik](https://img.shields.io/badge/Traefik-Reverse_Proxy-24A1C1?logo=traefikproxy)](https://traefik.io/)

[🇹🇷 Türkçe](README_TR.md)

</div>

---

## 📖 About

**Stackvo** is a Docker-based, fully customizable and modular development environment management system for your modern web development projects. With its pure Bash generator system, you can manage 40+ services with a single command.

### ✨ Key Features

- 🐳 **40+ Ready Services** - MySQL, PostgreSQL, MongoDB, Redis, RabbitMQ and more
- 🌐 **Multi-Language Support** - PHP, Node.js, Python, Go, Ruby, Rust (6 languages)
- 🔧 **3 Web Server Options** - Nginx, Apache, Caddy
- 🎯 **Pure Bash Generator** - Bash 3.x+ compatible, macOS and Linux support
- 🔒 **Traefik Reverse Proxy** - Automatic SSL/TLS, routing and load balancing
- 🎨 **Modern Web UI** - Real-time monitoring with Vue.js 3 + Vuetify 3
- 📦 **Single Network Architecture** - All services on stackvo-net
- 🚀 **Modular Structure** - Easily enable/disable services via .env
- 🔄 **Dynamic Configuration** - Automatic Docker Compose and Traefik routing
- ⚡ **Zero-Config** - Works immediately with default settings

---

## 🚀 Quick Start

### Requirements

- Docker 20.10+
- Docker Compose 2.0+
- Bash 3.2+
- 4GB+ RAM
- 10GB+ Disk space

### Installation

```bash
# 1. Clone the repository
git clone https://github.com/stackvo/stackvo.git
cd stackvo

# 2. Copy environment file
cp .env.example .env

# 3. Install CLI
./cli/stackvo.sh install

# 4. Generate configuration
./cli/stackvo.sh generate

# 5. Start services
./cli/stackvo.sh up

# 6. Update hosts file
echo "127.0.0.1  stackvo.loc" | sudo tee -a /etc/hosts

# 7. Access Web UI
# https://stackvo.loc
```

### Create Your First Project

```bash
# Create project folder
mkdir -p projects/myproject/public

# Create stackvo.json file
cat > projects/myproject/stackvo.json <<'EOF'
{
  "name": "myproject",
  "domain": "myproject.loc",
  "php": {
    "version": "8.2",
    "extensions": ["pdo", "pdo_mysql", "mbstring"]
  },
  "webserver": "nginx",
  "document_root": "public"
}
EOF

# Create test file
echo "<?php phpinfo();" > projects/myproject/public/index.php

# Regenerate configuration
./cli/stackvo.sh generate

# Restart services
./cli/stackvo.sh restart

# Add to hosts file
echo "127.0.0.1  myproject.loc" | sudo tee -a /etc/hosts

# Open in browser: https://myproject.loc
```

---

## 📚 Basic Commands

```bash
# Installation and Configuration
./cli/stackvo.sh install               # Install CLI to system
./cli/stackvo.sh generate              # Generate all configurations
./cli/stackvo.sh generate projects     # Generate only projects
./cli/stackvo.sh generate services     # Generate only services

# Container Management
./cli/stackvo.sh up                    # Start all services
./cli/stackvo.sh down                  # Stop all services
./cli/stackvo.sh restart               # Restart all services
./cli/stackvo.sh ps                    # List running services

# Logs and Others
./cli/stackvo.sh logs                  # Watch all logs
./cli/stackvo.sh logs mysql            # Watch specific service log
./cli/stackvo.sh pull                  # Pull Docker images
./cli/stackvo.sh doctor                # System health check
./cli/stackvo.sh uninstall             # Uninstall Stackvo
```

---

## 🛠️ Supported Services

| Category                | Count | Services                                                                       |
| ----------------------- | ----- | ------------------------------------------------------------------------------ |
| **Databases**           | 8     | MySQL, MariaDB, PostgreSQL, MongoDB, Cassandra, Percona, CouchDB, Couchbase    |
| **Cache Systems**       | 2     | Redis, Memcached                                                               |
| **Message Queues**      | 4     | RabbitMQ, Apache ActiveMQ, Kafka, NATS                                         |
| **Search & Indexing**   | 4     | Elasticsearch, Kibana, Meilisearch, Solr                                       |
| **Monitoring & QA**     | 5     | Grafana, Netdata, SonarQube, Sentry, Logstash                                  |
| **Developer Tools**     | 8     | Adminer, PhpMyAdmin, PhpPgAdmin, PhpMongo, MailHog, Ngrok, Selenium, Blackfire |
| **Application Servers** | 2     | Tomcat, Kong API Gateway                                                       |

> **Total 33+ services** • For details: [Services Documentation](docs/en/references/services.md)

---

## 🎨 Web UI Dashboard

Stackvo provides a modern web interface built with Vue.js 3 and Vuetify 3:

- **Real-time Monitoring** - CPU, Memory, Storage, Network
- **Services Management** - Start/Stop/Restart, Port mappings, Logs
- **Projects Management** - Create, delete, configure projects
- **Tools Access** - Adminer, PhpMyAdmin, RabbitMQ UI and more

**Access:** `https://stackvo.loc`

---

## 📖 Documentation

Visit the [docs](docs/en) directory for detailed documentation:

- **[Getting Started](docs/en/started/introduction.md)** - Introduction to Stackvo and core concepts
- **[Installation](docs/en/installation/index.md)** - Detailed installation guide
- **[Quick Start](docs/en/started/quick-start.md)** - Create your first project
- **[Configuration](docs/en/configuration/index.md)** - .env and stackvo.json settings
- **[CLI Reference](docs/en/references/cli.md)** - All CLI commands
- **[Services](docs/en/references/services.md)** - All supported services
- **[Architecture](docs/en/concepts/architecture.md)** - System architecture and design
- **[Troubleshooting](docs/en/community/troubleshooting.md)** - Common issues

---

## 🤝 Contributing

Stackvo is an open source project and we welcome your contributions!

1. Fork this repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'feat: add amazing feature'`)
4. Push your branch (`git push origin feature/amazing-feature`)
5. Create a Pull Request

For details, see the [Contributing Guide](docs/en/community/contributing.md).

---

## 📝 License

This project is licensed under the MIT License. See [LICENSE.md](LICENSE.md) for details.

---

## 🔗 Links

- **Documentation:** [stackvo.github.io/stackvo](https://stackvo.github.io/stackvo/)
- **GitHub:** [github.com/stackvo/stackvo](https://github.com/stackvo/stackvo)
- **Issues:** [github.com/stackvo/stackvo/issues](https://github.com/stackvo/stackvo/issues)
- **Discussions:** [github.com/stackvo/stackvo/discussions](https://github.com/stackvo/stackvo/discussions)
- **Changelog:** [CHANGELOG.md](docs/en/changelog.md)

---

## 💬 Support

For questions or issues:

- 📖 Check the [Documentation](docs/en) pages
- 🐛 Open an [Issue](https://github.com/stackvo/stackvo/issues)
- 💬 Ask questions in [Discussions](https://github.com/stackvo/stackvo/discussions)
- 📧 Read the [Support Guide](docs/en/community/support.md)

---
