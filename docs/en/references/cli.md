# CLI Commands Reference

Complete reference for Stackvo CLI commands. This page detailedly explains the syntax, parameters, options, and usage examples of generate, up, down, restart, ps, logs, pull, doctor, install, and uninstall commands. It also includes environment variables, exit codes, and Docker Compose equivalents.

## Installation

```bash
./stackvo.sh install
```

After installation, the `stackvo` command can be used system-wide.

---

## Commands

### generate

Generates configuration files.

**Syntax:**
```bash
./stackvo.sh generate [MODE] [OPTIONS]
```

**Modes:**
- (empty) - Generate all
- `projects` - Generate only projects
- `services` - Generate only services

**Options:**
- `--uninstall-tools` - Remove tools configurations

**Examples:**
```bash
# Generate all
./stackvo.sh generate

# Generate only projects
./stackvo.sh generate projects

# Generate only services
./stackvo.sh generate services

# Uninstall tools
./stackvo.sh generate --uninstall-tools
```

**Output Files:**
- `generated/stackvo.yml`
- `generated/docker-compose.dynamic.yml`
- `generated/docker-compose.projects.yml`
- `core/traefik/dynamic/routes.yml`
- `core/generated/configs/*`

---

### up

Starts services. Minimal mode (only core services) by default.

**Syntax:**
```bash
./stackvo.sh up [MODE_OPTIONS]
```

**Mode Options:**
- (empty) - Minimal mode: Only core services (Traefik + UI)
- `--all` - Start all services and projects (old behavior)
- `--services` - Start Core + all services
- `--projects` - Start Core + all projects
- `--profile <name>` - Start Core + a specific profile (can be used multiple times)

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

# Core + start only MySQL
./stackvo.sh up --profile mysql

# Core + start MySQL and Redis
./stackvo.sh up --profile mysql --profile redis

# Core + start a specific project
./stackvo.sh up --profile project-myproject
```

**Profile Naming:**
- For services: `mysql`, `redis`, `postgres`, `mongodb`, etc.
- For projects: `project-{project-name}` (e.g., `project-myproject`)

**Note:** 
- Default behavior changed: `up` command now only starts core services.
- Use `--all` parameter for old behavior.
- Profiles use Docker Compose profile feature.

**Equivalent Docker Compose:**
```bash
# Minimal mode
docker compose -f generated/stackvo.yml \
  -f generated/docker-compose.dynamic.yml \
  -f generated/docker-compose.projects.yml \
  --profile core up -d

# With specific profile
docker compose -f generated/stackvo.yml \
  -f generated/docker-compose.dynamic.yml \
  -f generated/docker-compose.projects.yml \
  --profile core --profile mysql up -d
```

---

### down

Stops all services.

**Syntax:**
```bash
./stackvo.sh down [OPTIONS]
```

**Options:**
- `-v, --volumes` - Remove volumes as well
- `--remove-orphans` - Remove orphan containers

**Examples:**
```bash
# Stop services
./stackvo.sh down

# Stop with volumes
./stackvo.sh down -v

# Remove orphans
./stackvo.sh down --remove-orphans
```

---

### restart

Restarts services.

**Syntax:**
```bash
./stackvo.sh restart [SERVICE...]
```

**Examples:**
```bash
# Restart all services
./stackvo.sh restart

# Restart specific services
docker compose -f generated/stackvo.yml \
  -f generated/docker-compose.dynamic.yml \
  restart mysql redis
```

---

### ps

Lists running services.

**Syntax:**
```bash
./stackvo.sh ps [OPTIONS]
```

**Options:**
- `-a, --all` - Show all containers (including stopped ones)
- `--format` - Output format

**Examples:**
```bash
# List running services
./stackvo.sh ps

# List all containers
./stackvo.sh ps -a

# Custom format
./stackvo.sh ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
```

**Output:**
```
NAME                      STATUS              PORTS
stackvo-traefik         Up 2 hours          0.0.0.0:80->80/tcp
stackvo-mysql           Up 2 hours          0.0.0.0:3306->3306/tcp
stackvo-redis           Up 2 hours          0.0.0.0:6379->6379/tcp
```

---

### logs

Displays container logs.

**Syntax:**
```bash
./stackvo.sh logs [OPTIONS] [SERVICE...]
```

**Options:**
- `-f, --follow` - Watch logs live
- `--tail=N` - Show last N lines
- `--timestamps` - Add timestamps
- `--since` - Logs since a specific time

**Examples:**
```bash
# Show all logs
./stackvo.sh logs

# Watch MySQL logs
./stackvo.sh logs -f mysql

# Last 100 lines
./stackvo.sh logs --tail=100 mysql

# With timestamps
./stackvo.sh logs --timestamps mysql

# Logs from last 1 hour
./stackvo.sh logs --since=1h mysql
```

---

### pull

Pulls Docker images.

**Syntax:**
```bash
./stackvo.sh pull [SERVICE...]
```

**Examples:**
```bash
# Pull all images
./stackvo.sh pull

# Pull specific images
docker compose -f generated/stackvo.yml \
  -f generated/docker-compose.dynamic.yml \
  pull mysql redis
```

---

### doctor

Performs system health check.

**Syntax:**
```bash
stackvo doctor
```

**Checks:**
- ✓ Is Docker installed?
- ✓ Is Docker Compose installed?
- ✓ Is Docker daemon running?
- ✓ Are required ports available?
- ✓ Does `.env` file exist?
- ✓ Do SSL certificates exist?
- ✓ Does `generated/` directory exist?

**Example Output:**
```
✓ Docker is installed (version 24.0.7)
✓ Docker Compose is installed (version 2.23.0)
✓ Docker daemon is running
✓ Port 80 is available
✓ Port 443 is available
✓ .env file exists
✓ SSL certificates found
✓ Generated directory exists

All checks passed!
```

---

### install

Installs Stackvo CLI to the system.

**Syntax:**
```bash
./stackvo.sh install
```

**What it does:**
- Creates symbolic link to `/usr/local/bin/stackvo`
- Makes CLI available system-wide

**Requirements:**
- Sudo privileges

---

### uninstall

Uninstalls Stackvo.

**Syntax:**
```bash
./stackvo.sh uninstall
```

**What it does:**
1. Stops all containers
2. Deletes volumes (asks for confirmation)
3. Deletes network
4. Removes CLI symbolic link

**Warning:** This operation cannot be undone!

---

## Environment Variables

Environment variables controlling CLI behavior:

### STACKVO_VERBOSE

Detailed output.

```bash
STACKVO_VERBOSE=true ./stackvo.sh generate
```

### STACKVO_DRY_RUN

Show without executing commands.

```bash
STACKVO_DRY_RUN=true ./stackvo.sh generate
```

### ENV_FILE

Use different .env file.

```bash
ENV_FILE=.env.production ./stackvo.sh generate
```

---

## Exit Codes

| Code | Description |
|------|----------|
| 0 | Success |
| 1 | General error |
| 2 | Usage error |
| 126 | Command invoked cannot execute |
| 127 | Command not found |
| 130 | Script terminated by Control-C |

---
