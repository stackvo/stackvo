---
title: macOS Installation
description: Stackvo installation on macOS - Intel and Apple Silicon (M1/M2/M3) support
---

# macOS Installation

Docker Desktop is used for Stackvo installation on macOS. This guide detailedly explains Docker Desktop installation, system settings, and Stackvo configuration on both Intel and Apple Silicon (M1/M2/M3) Mac computers. Easy installation options with Homebrew are also available.

---

!!! tip "Checked System Requirements?"
    Check the [System Requirements](../started/requirements.md) page before starting the installation.

---

## Docker Desktop Installation

### 1. Downloading Docker Desktop

=== "Apple Silicon (M1/M2/M3)"

    ```bash
    # Open in browser:
    https://desktop.docker.com/mac/main/arm64/Docker.dmg

    # Or with Homebrew (recommended):
    brew install --cask docker
    ```

=== "Intel"

    ```bash
    # Open in browser:
    https://desktop.docker.com/mac/main/amd64/Docker.dmg

    # Or with Homebrew (recommended):
    brew install --cask docker
    ```

### 2. Installing Docker Desktop

1. Open the DMG file
2. Drag Docker.app to Applications
3. Start Docker from Applications folder
4. It will ask for admin password on first launch

!!! warning "First Launch"
    Docker Desktop may take a few minutes on first launch.

### 3. Docker Desktop Settings

After Docker Desktop opens:

**Settings** (⚙️) → **Resources**

| Resource | Minimum | Recommended |
|--------|---------|----------|
| **CPU** | 2 cores | 4 cores |
| **Memory** | 4 GB | 8 GB |
| **Disk** | 20 GB | 50 GB |

Click on **Apply & Restart** button.

### 4. Docker Verification

```bash
# Version check
docker --version
docker compose version

# Test
docker run hello-world
```

---

## Installation with Homebrew (Recommended)

```bash
# If Homebrew is not installed:
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Docker Desktop + Required tools
brew install --cask docker
brew install git jq curl

# Start Docker Desktop
open -a Docker
```

---

## Stackvo Installation

After Docker Desktop installation is complete, follow the [Quick Start](../started/quick-start.md) page to install Stackvo.

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
```

In your browser:

- **Stackvo Dashboard:** https://stackvo.loc
- **Traefik Dashboard:** http://traefik.stackvo.loc

!!! success "Installation Completed!"
    You can now proceed to the [Quick Start](../started/quick-start.md) page to create your first project.

---

## Common Issues

### Docker Desktop Not Starting

```bash
# Completely close Docker Desktop
pkill -SIGHUP -f Docker

# Restart
open -a Docker
```

### Rosetta 2 Required (Apple Silicon)

```bash
# Install Rosetta 2
softwareupdate --install-rosetta --agree-to-license
```

### Port Conflict

```bash
# Find and stop conflicting process
sudo lsof -i :80
sudo kill -9 <PID>
```