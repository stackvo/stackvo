---
title: Windows Installation
description: Stackvo installation on Windows 10/11 with WSL2
---

# Windows Installation

Stackvo runs on **WSL2 (Windows Subsystem for Linux 2)** on Windows. This guide detailedly explains the steps for WSL2 installation, Docker Desktop configuration, and running Stackvo within WSL2 on Windows 10 and Windows 11. Thanks to WSL2, you get a Linux-like experience.

---

!!! tip "Checked System Requirements?"
    Check the [System Requirements](../started/requirements.md) page before starting the installation.

!!! warning "WSL2 Required"
    Windows 10 Pro/Enterprise (Build 19041+) or Windows 11 is required.

---

## WSL2 Installation

### Automatic Installation (Recommended)

**Open PowerShell as Administrator:**

```powershell
# Install WSL2 (single command)
wsl --install

# Restart computer
Restart-Computer
```

!!! success "Single Command!"
    This command automatically installs WSL2, Ubuntu, and all requirements.

### First Launch

1. Open "Ubuntu" from Start menu
2. Enter username (lowercase, no spaces)
3. Set password (2 times)

---

## Docker Desktop Installation

### 1. Downloading Docker Desktop

Download [Docker Desktop for Windows](https://desktop.docker.com/win/main/amd64/Docker%20Desktop%20Installer.exe).

### 2. Installation

1. Run the downloaded `.exe` file
2. Check **"Use WSL 2 instead of Hyper-V"** option
3. Click "Install"
4. Restart computer when installation is complete

### 3. Docker Desktop Settings

**Settings** → **General:**
- ✅ Use the WSL 2 based engine

**Settings** → **Resources** → **WSL Integration:**
- ✅ Enable integration with my default WSL distro
- ✅ Ubuntu-22.04

**Settings** → **Resources:**

| Resource | Minimum | Recommended |
|--------|---------|----------|
| **CPU** | 2 cores | 4 cores |
| **Memory** | 4 GB | 8 GB |
| **Disk** | 30 GB | 50 GB |

**Apply & Restart**

---

## Stackvo Installation (Inside WSL2)

### Login to WSL2

```powershell
# Switch to WSL2 from PowerShell
wsl
```

### System Update

```bash
# Inside Ubuntu
sudo apt update && sudo apt upgrade -y
sudo apt install -y git curl jq
```

### Stackvo Installation

After Docker Desktop installation is complete, follow the [Quick Start](../started/quick-start.md) page to install Stackvo.

**Summary:**

```bash
# Inside WSL2
git clone https://github.com/stackvo/stackvo.git
cd stackvo

# Configuration
cp .env.example .env

# Start
./stackvo.sh generate
./stackvo.sh up
```

### Hosts File (Windows)

Open Notepad as administrator **on Windows**:

```
C:\Windows\System32\drivers\etc\hosts
```

Add:

```
127.0.0.1  stackvo.loc
127.0.0.1  traefik.stackvo.loc
```

---

## Installation Verification

### WSL2 Check

```powershell
# In PowerShell
wsl --list --verbose

# Output:
#   NAME            STATE           VERSION
# * Ubuntu-22.04    Running         2
```

### Docker Check

```bash
# Inside WSL2
docker --version
docker compose version
docker ps
```

### Web UI Check

Open in **Windows browser**:

- **Stackvo Dashboard:** https://stackvo.loc
- **Traefik Dashboard:** http://traefik.stackvo.loc

!!! success "Installation Completed!"
    You can now proceed to the [Quick Start](../started/quick-start.md) page to create your first project.

---

## Common Issues

### WSL2 Not Starting

**Error:** `WslRegisterDistribution failed with error: 0x80370102`

**Solution:** Virtualization must be enabled in BIOS

```powershell
# Enable Hyper-V
Enable-WindowsOptionalFeature -Online -FeatureName Microsoft-Hyper-V -All
Restart-Computer
```

### Docker Daemon Cannot Connect

**Solution:**
1. Check if Docker Desktop is running
2. Is Settings → Resources → WSL Integration enabled?
3. Restart WSL2: `wsl --shutdown` → `wsl`

### Port Conflict

```powershell
# Find conflicting service on Windows
netstat -ano | findstr :80

# Stop process
taskkill /PID <PID> /F
```