<div align="center">

# 🚀 StackVo

**Docker-Based Local Development Environment for Modern LAMP and MEAN Stacks**

![Status](https://img.shields.io/badge/status-active-success.svg)
![Release](https://img.shields.io/github/v/release/stackvo/stackvo)
![GitHub Issues](https://img.shields.io/github/issues/stackvo/stackvo)
![GitHub Closed Issues](https://img.shields.io/github/issues-closed/stackvo/stackvo)
![GitHub Pull Requests](https://img.shields.io/github/issues-pr/stackvo/stackvo)
![GitHub Contributors](https://img.shields.io/github/contributors/stackvo/stackvo)
![Security](https://img.shields.io/badge/security-policy-success?logo=security&logoColor=white)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

![Docker](https://img.shields.io/badge/Docker-Required-2496ED?logo=docker&logoColor=white)
![Bash](https://img.shields.io/badge/Bash-3.x+-4EAA25?logo=gnubash&logoColor=white)
![Traefik](https://img.shields.io/badge/Traefik-Reverse_Proxy-24A1C1?logo=traefikproxy&logoColor=white)

[🇬🇧 English](README.md) |
[🇹🇷 Türkçe](README_TR.md)

</div>

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

**System Requirements:**

- **Docker:** 20.10+ (Docker Desktop on macOS/Windows, Docker Engine on Linux)
- **Docker Compose:** 2.0+ (v2 plugin format - `docker compose` not `docker-compose`)
- **Bash:** 3.2+ (pre-installed on macOS and Linux, use WSL2 or Git Bash on Windows)
- **RAM:** 4GB minimum, 8GB+ recommended
- **Disk Space:** 10GB+ free space

**Supported Operating Systems:**

- ✅ **macOS** 10.15+ (Catalina or later) - Intel & Apple Silicon
- ✅ **Linux** - Ubuntu 20.04+, Debian 11+, Fedora 35+, Arch Linux
- ✅ **Windows** 10/11 with WSL2 (Ubuntu 20.04+ in WSL)

**Not Supported:**

- ❌ Native Windows (without WSL2)
- ❌ macOS < 10.15
- ❌ Docker Compose v1 (deprecated)

### Installation

**Step 1: Clone and Setup**

```bash
# Clone the repository
git clone https://github.com/stackvo/stackvo.git
cd stackvo

# Copy environment file
cp .env.example .env
```

**Step 2: Install CLI**

```bash
# Install Stackvo CLI globally
./stackvo.sh install

# Verify installation
stackvo --help
```

**Step 3: Generate Configuration**

```bash
# Generate all configurations
stackvo generate

# This will create:
# - generated/stackvo.yml (Traefik + UI)
# - generated/docker-compose.dynamic.yml (Services)
# - generated/docker-compose.projects.yml (Projects)
```

**Step 4: Start Services**

```bash
# Start core services (Traefik + UI)
stackvo up

# Wait for services to start (~30 seconds)
# Check status
stackvo ps
```

**Step 5: Configure Hosts File**

```bash
# Add Stackvo UI domain to hosts file
echo "127.0.0.1  stackvo.loc" | sudo tee -a /etc/hosts
```

**Step 6: Access Web UI**

Open your browser and visit: **https://stackvo.loc**

> **Note:** You'll see a SSL warning because we use self-signed certificates in development. Click "Advanced" → "Proceed to site" to continue.

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
./stackvo.sh generate

# Restart services
./stackvo.sh restart

# Add to hosts file
echo "127.0.0.1  myproject.loc" | sudo tee -a /etc/hosts

# Open in browser: https://myproject.loc
```

---

## 📚 Basic Commands

```bash
# Installation and Configuration
./stackvo.sh install               # Install CLI to system
./stackvo.sh generate              # Generate all configurations
./stackvo.sh generate projects     # Generate only projects
./stackvo.sh generate services     # Generate only services

# Container Management
./stackvo.sh up                    # Start core services (minimal)
./stackvo.sh up --all              # Start all services and projects
./stackvo.sh up --services         # Start core + all services
./stackvo.sh up --projects         # Start core + all projects
./stackvo.sh up --profile mysql    # Start core + MySQL
./stackvo.sh down                  # Stop all services
./stackvo.sh restart               # Restart all services
./stackvo.sh ps                    # List running services

# Logs and Others
./stackvo.sh logs                  # Watch all logs
./stackvo.sh logs mysql            # Watch specific service log
./stackvo.sh pull                  # Pull Docker images
./stackvo.sh uninstall             # Uninstall Stackvo
```

> **Note:** After running `./stackvo.sh install`, you can use `stackvo` command directly from anywhere:
>
> ```bash
> stackvo up
> stackvo generate
> stackvo logs
> ```

---

## 🛠️ Supported Services

| Category              | Count | Services                                       |
| --------------------- | ----- | ---------------------------------------------- |
| **Databases**         | 5     | MySQL, MariaDB, PostgreSQL, MongoDB, Cassandra |
| **Cache Systems**     | 2     | Redis, Memcached                               |
| **Message Queues**    | 2     | RabbitMQ, Kafka                                |
| **Search & Indexing** | 2     | Elasticsearch, Kibana                          |
| **Monitoring**        | 1     | Grafana                                        |
| **Developer Tools**   | 2     | MailHog, Blackfire                             |

> **Total 14 services** • For details: [Services Documentation](docs/en/references/services.md)

---

## 🎨 Web UI Dashboard

Stackvo provides a modern web interface built with Vue.js 3 and Vuetify 3:

- **Real-time Monitoring** - CPU, Memory, Storage, Network
- **Services Management** - Start/Stop/Restart, Port mappings, Logs
- **Projects Management** - Create, delete, configure projects
- **Tools Access** - Adminer, PhpMyAdmin, RabbitMQ UI and more

**Access:** `https://stackvo.loc`

### 📸 Screenshots

<table>
  <tr>
    <td width="50%">
      <img src="https://github.com/stackvo/stackvo/blob/main/docs/screenshots/1-Dashboard.png?raw=true" alt="Dashboard" />
      <p align="center"><b>Dashboard</b></p>
    </td>
    <td width="50%">
      <img src="https://github.com/stackvo/stackvo/blob/main/docs/screenshots/2-Projects-list.png?raw=true" alt="Projects List" />
      <p align="center"><b>Projects List</b></p>
    </td>
  </tr>
  <tr>
    <td width="50%">
      <img src="https://github.com/stackvo/stackvo/blob/main/docs/screenshots/3-Projects-detail.png?raw=true" alt="Project Detail" />
      <p align="center"><b>Project Detail</b></p>
    </td>
    <td width="50%">
      <img src="https://github.com/stackvo/stackvo/blob/main/docs/screenshots/4-Projects-new.png?raw=true" alt="New Project" />
      <p align="center"><b>New Project</b></p>
    </td>
  </tr>
  <tr>
    <td width="50%">
      <img src="https://github.com/stackvo/stackvo/blob/main/docs/screenshots/5-Services-list.png?raw=true" alt="Services List" />
      <p align="center"><b>Services List</b></p>
    </td>
    <td width="50%">
      <img src="https://github.com/stackvo/stackvo/blob/main/docs/screenshots/6-Services-detail.png?raw=true" alt="Service Detail" />
      <p align="center"><b>Service Detail</b></p>
    </td>
  </tr>
</table>

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

For detailed contribution guidelines, including coding standards, commit message format, and changelog generation workflow, see the [Contributing Guide](CONTRIBUTING.md).

### Quick Contribution Steps

1. Fork this repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'feat: add amazing feature'`)
4. Push your branch (`git push origin feature/amazing-feature`)
5. Create a Pull Request

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
