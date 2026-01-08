---
title: Linux Installation
description: Stackvo installation on Ubuntu, Debian, CentOS, Arch, and other Linux distributions
---

# Linux Installation

Linux offers the best performance and most seamless experience for Stackvo. This guide explains step-by-step Docker and Stackvo installation on Ubuntu, Debian, CentOS, Rocky Linux, Arch, and other popular Linux distributions. Thanks to native Docker support, it runs faster and more efficiently than Windows and macOS.

---

!!! tip "Checked System Requirements?"
    Check the [System Requirements](../started/requirements.md) page before starting the installation.

---

## Docker Installation

### 1. System Update

```bash
# Ubuntu/Debian
sudo apt update && sudo apt upgrade -y

# CentOS/RHEL
sudo yum update -y

# Rocky/Alma
sudo dnf update -y

# Arch/Manjaro
sudo pacman -Syu
```

### 2. Docker Installation

=== "Ubuntu/Debian"

    ```bash
    # Required packages
    sudo apt install -y apt-transport-https ca-certificates curl gnupg git jq
    
    # Docker GPG key
    curl -fsSL https://download.docker.com/linux/ubuntu/gpg | \
      sudo gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg
    
    # Docker repository
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] \
      https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" | \
      sudo tee /etc/apt/sources.list.d/docker.list > /dev/null
    
    # Install Docker
    sudo apt update
    sudo apt install -y docker-ce docker-ce-cli containerd.io docker-compose-plugin
    ```

=== "CentOS/RHEL"

    ```bash
    # Required packages
    sudo yum install -y yum-utils git jq
    
    # Docker repository
    sudo yum-config-manager --add-repo https://download.docker.com/linux/centos/docker-ce.repo
    
    # Install Docker
    sudo yum install -y docker-ce docker-ce-cli containerd.io docker-compose-plugin
    ```

=== "Rocky/Alma"

    ```bash
    # Required packages
    sudo dnf install -y dnf-plugins-core git jq
    
    # Docker repository
    sudo dnf config-manager --add-repo https://download.docker.com/linux/centos/docker-ce.repo
    
    # Install Docker
    sudo dnf install -y docker-ce docker-ce-cli containerd.io docker-compose-plugin
    ```

=== "Arch/Manjaro"

    ```bash
    # Install Docker (available in repository)
    sudo pacman -S docker docker-compose git jq
    ```

### 3. Start Docker Service

```bash
sudo systemctl start docker
sudo systemctl enable docker
```

### 4. User Permissions

```bash
# Add to Docker group
sudo usermod -aG docker $USER

# Reload session
newgrp docker
```

### 5. Docker Verification

```bash
# Version check
docker --version
docker compose version

# Test
docker run hello-world
```

---

## Stackvo Installation

After Docker installation is complete, follow the [Quick Start](../started/quick-start.md) page to install Stackvo.

**Summary:**

```bash
# Clone repository
git clone https://github.com/stackvo/stackvo.git
cd stackvo

# Configuration
cp .env.example .env

# Start
./stackvo.sh generate
./stackvo.sh up
```

---

## Installation Verification

```bash
# Container status
docker ps

# Open Web UI
```

In your browser:

- **Stackvo Dashboard:** https://stackvo.loc
- **Traefik Dashboard:** http://traefik.stackvo.loc

!!! success "Installation Completed!"
    You can now proceed to the [Quick Start](../started/quick-start.md) page to create your first project.

---

## Common Issues

### Permission Denied

```bash
# Add to Docker group
sudo usermod -aG docker $USER
newgrp docker
```

### Port Conflict

```bash
# Stop conflicting service
sudo systemctl stop apache2
sudo systemctl stop nginx
```