---
title: System Requirements
description: Detailed information about minimum, recommended, and professional hardware requirements, supported operating systems (Linux, macOS, Windows), Docker and Docker Compose versions, network ports, and system check to run Stackvo smoothly.
---

# System Requirements

To run Stackvo smoothly and efficiently, your system must meet certain hardware and software requirements. This page detailedly explains all system requirements for minimum, recommended, and professional usage, supported operating systems, Docker versions, and network settings.

---

## Hardware Requirements

### Minimum Requirements

!!! warning "Minimum Configuration"
You can only run basic services with this configuration.

| Component    | Minimum | Description              |
| ------------ | ------- | ------------------------ |
| **CPU**      | 2 Core  | Dual-core processor      |
| **RAM**      | 4 GB    | For System + Docker      |
| **Disk**     | 20 GB   | Free disk space          |
| **Internet** | Yes     | Required for first install|

**Runnable Services:**

- MySQL or PostgreSQL (1 unit)
- Redis
- Nginx
- 1-2 small projects

### Recommended Requirements

!!! success "Recommended Configuration"
Recommended configuration for comfortable development.

| Component    | Recommended | Description           |
| ------------ | ----------- | --------------------- |
| **CPU**      | 4 Core      | Quad-core processor   |
| **RAM**      | 8 GB        | For multiple services |
| **Disk**     | 50 GB       | SSD recommended       |
| **Internet** | Fast        | For image downloads   |

**Runnable Services:**

- 10-15 services concurrently
- 3-5 medium-sized projects
- Monitoring tools

### Professional Requirements

!!! tip "Professional Configuration"
Run all services and multiple projects comfortably.

| Component    | Professional | Description        |
| ------------ | ------------ | ------------------ |
| **CPU**      | 8+ Core      | Multi-core processor|
| **RAM**      | 16+ GB       | For all services   |
| **Disk**     | 100+ GB      | NVMe SSD recommended|
| **Internet** | Very Fast    | Fiber connection   |

**Runnable Services:**

- 40+ services concurrently
- 10+ projects
- All monitoring and logging tools

---

## Operating System Requirements

### Linux

!!! success "Best Performance"
    Linux offers the best performance for Docker.

| Distribution | Minimum Version | Kernel |
|---------|------------------|--------|
| **Ubuntu** | 20.04 LTS+ | 4.4+ |
| **Debian** | 10+ | 4.4+ |
| **CentOS/RHEL** | 7+ | 3.10+ |
| **Rocky/Alma** | 8+ | 3.10+ |
| **Arch/Manjaro** | Rolling | 5.0+ |
| **Fedora** | 35+ | 5.0+ |

### macOS

!!! info "Docker Desktop Required"
    Docker Desktop must be used on macOS.

| Version | Chip Support |
|----------|--------------|
| **macOS 12+** (Monterey, Ventura, Sonoma) | Intel x86_64, Apple Silicon (M1/M2/M3) |

**Note:** Rosetta 2 might be required for Apple Silicon.

### Windows

!!! warning "WSL2 Mandatory"
    WSL2 (Windows Subsystem for Linux 2) must be used on Windows.

| Version | Requirement |
|----------|------------|
| **Windows 10 Pro/Enterprise** | Build 19041+ |
| **Windows 11 Pro/Enterprise** | All versions |

**Requirements:** WSL2 enabled + Ubuntu 20.04+ WSL distro + Docker Desktop 4.0+

---

## Docker Requirements

### Docker Engine

!!! danger "Critical Requirement"
Docker Engine must be installed!

**Minimum Version:**

```bash
Docker Engine: 20.10.0+
```

**Recommended Version:**

```bash
Docker Engine: 24.0.0+
```

**Check:**

```bash
docker --version
# Output: Docker version 24.0.7, build afdd53b
```

### Docker Compose

!!! danger "Critical Requirement"
Docker Compose must be installed!

**Minimum Version:**

```bash
Docker Compose: 2.0.0+
```

**Recommended Version:**

```bash
Docker Compose: 2.20.0+
```

**Check:**

```bash
docker compose version
# Output: Docker Compose version v2.23.0
```

!!! warning "Old Version Warning"
Use `docker compose` (v2.x) instead of `docker-compose` (v1.x)!

---

## Network Requirements

### Critical Ports

Ports required for Stackvo to run:

| Port | Service | Description |
|------|--------|----------|
| **80** | Traefik | HTTP |
| **443** | Traefik | HTTPS |
| **8080** | Traefik Dashboard | Management panel |

!!! warning "Port Conflict"
    These ports must not be used by another application!

**Port Check:**
```bash
# Linux/macOS
sudo lsof -i :80
sudo lsof -i :443

# Windows (PowerShell)
netstat -ano | findstr :80
```

### Internet Connection

- **First Install:** ~5-10 GB download for Docker images
- **Normal Usage:** Optional (only for updates)

---

## Software Requirements

### Mandatory Software

```bash
# Bash 4.0+
bash --version

# Git 2.0+
git --version

# Curl 7.0+
curl --version

# jq 1.5+ (JSON parser)
jq --version
```

### Optional Tools

- **IDE:** VS Code, PhpStorm, WebStorm
- **Terminal:** htop, ncdu, lazydocker

---

## System Check

Stackvo provides a script that automatically checks system requirements:

```bash
cd stackvo
./core/cli/check-requirements.sh
```

**Example Output:**

```
✅ Operating System: Ubuntu 22.04 LTS
✅ Docker Engine: 24.0.7
✅ Docker Compose: 2.23.0
✅ Bash: 5.1.16
✅ Git: 2.34.1
✅ RAM: 16 GB (Sufficient)
✅ Disk: 120 GB free (Sufficient)
⚠️  Port 80: In use (Apache running)

Total: 8/9 checks passed
```

!!! tip "Ready to Install?"
    If all checks are successful, you can proceed to the [Installation](../installation/index.md) page.
