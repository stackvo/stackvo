# Project Configuration

A `stackvo.json` file is required for every Stackvo project. This page detailedly explains all fields of the project-based configuration file, PHP and other runtime versions, webserver options, document root settings, and multi-language support. Project configuration allows each project to be customized independently.

---

## stackvo.json File

Project configuration is done with a `stackvo.json` file for each project in the `projects/` directory.

### Minimum Configuration

```json
{
  "name": "myproject",
  "domain": "myproject.loc",
  "php": {
    "version": "8.2"
  },
  "webserver": "nginx",
  "document_root": "public"
}
```

### Full Configuration

```json
{
  "name": "myproject",
  "domain": "myproject.loc",
  "php": {
    "version": "8.2",
    "extensions": [
      "pdo",
      "pdo_mysql",
      "mysqli",
      "gd",
      "curl",
      "zip",
      "mbstring",
      "xml",
      "json",
      "redis"
    ]
  },
  "webserver": "nginx",
  "document_root": "public"
}
```

---

## Configuration Fields

### name (required)

Project name. Used for container names.

```json
{
  "name": "myproject"
}
```

**Container names:**
- `stackvo-myproject-php`
- `stackvo-myproject-web`

### domain (required)

The domain where the project will be accessed.

```json
{
  "domain": "myproject.loc"
}
```

**Note:** Should be added to `/etc/hosts` file:
```
127.0.0.1  myproject.loc
```

### php (optional)

PHP runtime configuration.

```json
{
  "php": {
    "version": "8.2",
    "extensions": ["pdo", "pdo_mysql", "mysqli", "gd", "curl", "zip", "mbstring"]
  }
}
```

**Supported versions:** 5.6, 7.0, 7.1, 7.2, 7.3, 7.4, 8.0, 8.1, 8.2, 8.3, 8.4, 8.5

**Default:** `DEFAULT_PHP_VERSION` in `.env` file (8.2)

### webserver (optional)

Webserver selection.

```json
{
  "webserver": "nginx"
}
```

**Options:**
- `nginx` (default)
- `apache`
- `caddy`

### document_root (optional)

Document root directory.

```json
{
  "document_root": "public"
}
```

**Default:** `public`

---

## Multi-Language Support

### PHP + Node.js

```json
{
  "name": "fullstack-app",
  "domain": "fullstack.loc",
  "php": {
    "version": "8.3"
  },
  "nodejs": {
    "version": "14.23"
  },
  "webserver": "nginx",
  "document_root": "public"
}
```

### PHP + Python

```json
{
  "name": "ml-app",
  "domain": "ml.loc",
  "php": {
    "version": "8.2"
  },
  "python": {
    "version": "3.14"
  },
  "webserver": "nginx",
  "document_root": "public"
}
```

### All Languages

```json
{
  "name": "polyglot-app",
  "domain": "polyglot.loc",
  "php": {
    "version": "8.3"
  },
  "nodejs": {
    "version": "14.23"
  },
  "python": {
    "version": "3.14"
  },
  "golang": {
    "version": "1.23"
  },
  "ruby": {
    "version": "3.3"
  },
  "rust": {
    "version": "1.62"
  },
  "webserver": "caddy",
  "document_root": "public"
}
```

---

## Webserver Options

### Nginx

```json
{
  "webserver": "nginx"
}
```

- **Image:** `nginx:alpine`
- **Config:** `.stackvo/nginx.conf` or auto-generated
- **Template:** `core/templates/servers/nginx/default.conf`

### Apache

```json
{
  "webserver": "apache"
}
```

- **Image:** `php:{version}-apache`
- **Config:** `.stackvo/apache.conf` or auto-generated
- **Template:** `core/templates/servers/apache/default.conf`

### Caddy

```json
{
  "webserver": "caddy"
}
```

- **Image:** `caddy:latest`
- **Config:** `.stackvo/Caddyfile` or auto-generated
- **Template:** `core/templates/servers/caddy/Caddyfile`

---

## Project Directory Structure

### Basic Structure

```
projects/myproject/
├── stackvo.json          # Project configuration (REQUIRED)
├── public/                 # Document root
│   └── index.php
├── src/                    # Source codes
├── vendor/                 # Composer dependencies
└── composer.json
```

### With Custom Configurations

```
projects/myproject/
├── stackvo.json
├── .stackvo/             # Custom configurations (optional)
│   ├── nginx.conf          # Custom Nginx config
│   ├── php.ini             # Custom PHP config
│   └── php-fpm.conf        # Custom PHP-FPM config
├── public/
│   └── index.php
└── ...
```
