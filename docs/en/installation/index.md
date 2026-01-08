---
title: Installation
description: Stackvo installation guide - Step-by-step installation for all platforms
---

# Installation

Installing Stackvo on your computer is quite easy and supported on all major operating systems. This section details all steps from Docker installation to Stackvo configuration on Linux, macOS, and Windows platforms. Guides specifically prepared for each operating system cover everything from system requirements to installation verification.

---

## Operating System Selection

Stackvo works on all major operating systems. Select your operating system:

<div class="grid cards" markdown>

-   :fontawesome-brands-linux:{ .lg .middle } __Linux__

    ---

    Valid installation steps for all popular Linux distributions

    [:octicons-arrow-right-24: Linux Installation](linux.md)

-   :fontawesome-brands-apple:{ .lg .middle } __macOS__

    ---

    Compatible for Intel and Apple Silicon (M series) processors

    [:octicons-arrow-right-24: macOS Installation](macos.md)

-   :fontawesome-brands-windows:{ .lg .middle } __Windows__

    ---

    Runs on WSL2 (Windows Subsystem for Linux)

    [:octicons-arrow-right-24: Windows Installation](windows.md)

</div>

---

!!! tip "Checked System Requirements?"
    Check the [System Requirements](../started/requirements.md) page before starting the installation.

---

## Quick Installation Way

If Docker is already installed on your system:

```bash
# 1. Clone the repository
git clone https://github.com/stackvo/stackvo.git
cd stackvo

# 2. Configuration
cp .env.example .env

# 3. Run installation script
./stackvo.sh install

# 4. Start
./stackvo.sh generate
./stackvo.sh up
```

!!! success "Installation Completed!"
Web UI: [https://stackvo.loc](https://stackvo.loc)

---

## Post-Installation Settings

### Editing Hosts File

=== "Linux/macOS"

    ```bash
    sudo nano /etc/hosts
    ```

    Add:
    ```
    127.0.0.1  stackvo.loc
    127.0.0.1  traefik.stackvo.loc
    ```

=== "Windows"

    As administrator:
    ```
    notepad C:\Windows\System32\drivers\etc\hosts
    ```

    Add:
    ```
    127.0.0.1  stackvo.loc
    127.0.0.1  traefik.stackvo.loc
    ```


---

## Installation Verification

Verify that the installation was successful:

### Service Check

```bash
# Status of all services
./stackvo.sh ps

# Check logs
./stackvo.sh logs
```

### Web UI Check

Open in your browser:

- **Stackvo Dashboard:** https://stackvo.loc/
- **Traefik Dashboard:** http://traefik.stackvo.loc

!!! success "Installation Completed!"
    You can now proceed to the [Quick Start](../started/quick-start.md) page to create your first project.
