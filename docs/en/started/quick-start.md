---
title: Quick Start
description: Step-by-step guide to create your first project with Stackvo. Detailed explanation of the entire process from Docker installation to project configuration, hosts file editing to testing in the browser.
---

# Quick Start

This guide detailedly explains all the steps required to create your first project with Stackvo. You will learn everything step-by-step, from Docker installation to project configuration, hosts file editing to testing in the browser.

---

!!! warning "Installation Required"
    This guide assumes **installation is complete**. If you haven't installed it yet, follow the [Installation](../installation/index.md) page first.

**If installation is complete, continue:**

---

## Create Your First Project

### Laravel Project Example

#### 1. Create Project Directory

```bash
# Project folder
mkdir -p projects/mylaravel/public

# Add a simple index.php inside
cat > projects/mylaravel/public/index.php <<'EOF'
<?php
phpinfo();
EOF
```

#### 2. Project Configuration

```bash
# Create stackvo.json
cat > projects/mylaravel/stackvo.json <<'EOF'
{
  "name": "mylaravel",
  "domain": "mylaravel.loc",
  "php": {
    "version": "8.2",
    "extensions": [
      "pdo",
      "pdo_mysql",
      "mbstring",
      "xml",
      "curl",
      "zip"
    ]
  },
  "webserver": "nginx",
  "document_root": "public"
}
EOF
```

#### 3. Add to Hosts File

```bash
# /etc/hosts (Linux/macOS) or C:\Windows\System32\drivers\etc\hosts (Windows)
127.0.0.1  mylaravel.loc
```

#### 4. Start Project

```bash
# Regenerate configuration
./stackvo.sh generate

# Restart containers
./stackvo.sh restart

# Check project container
docker ps | grep mylaravel
```

#### 5. Open in Browser

[https://mylaravel.loc](https://mylaravel.loc)

!!! success "Your First Project is Ready!"
    You should see the PHP info page! 🎉