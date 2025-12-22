# Proje Konfigürasyonu

Her Stackvo projesi için `stackvo.json` dosyası gereklidir. Bu sayfa, proje bazlı konfigürasyon dosyasının tüm alanlarını, PHP ve diğer runtime versiyonlarını, webserver seçeneklerini, document root ayarlarını ve çoklu dil desteğini detaylı olarak açıklamaktadır. Proje konfigürasyonu, her projenin bağımsız olarak özelleştirilmesini sağlar.

## stackvo.json Dosyası

Proje konfigürasyonu, `projects/` dizinindeki her proje için `stackvo.json` dosyası ile yapılır.

### Minimum Konfigürasyon

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

### Tam Konfigürasyon

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

## Konfigürasyon Alanları

### name (zorunlu)

Proje adı. Container isimleri için kullanılır.

```json
{
  "name": "myproject"
}
```

**Container isimleri:**
- `stackvo-myproject-php`
- `stackvo-myproject-web`

### domain (zorunlu)

Projenin erişileceği domain.

```json
{
  "domain": "myproject.loc"
}
```

**Not:** `/etc/hosts` dosyasına eklenmeli:
```
127.0.0.1  myproject.loc
```

### php (opsiyonel)

PHP runtime konfigürasyonu.

```json
{
  "php": {
    "version": "8.2",
    "extensions": ["pdo", "pdo_mysql", "mysqli", "gd", "curl", "zip", "mbstring"]
  }
}
```

**Desteklenen versiyonlar:** 5.6, 7.0, 7.1, 7.2, 7.3, 7.4, 8.0, 8.1, 8.2, 8.3, 8.4, 8.5

**Varsayılan:** `.env` dosyasındaki `DEFAULT_PHP_VERSION` (8.2)

### webserver (opsiyonel)

Webserver seçimi.

```json
{
  "webserver": "nginx"
}
```

**Seçenekler:**
- `nginx` (varsayılan)
- `apache`
- `caddy`

### document_root (opsiyonel)

Document root dizini.

```json
{
  "document_root": "public"
}
```

**Varsayılan:** `public`

---

## Çoklu Dil Desteği

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

### Tüm Diller

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

## Webserver Seçenekleri

### Nginx

```json
{
  "webserver": "nginx"
}
```

- **Image:** `nginx:alpine`
- **Config:** `.stackvo/nginx.conf` veya auto-generated
- **Template:** `core/templates/servers/nginx/default.conf`

### Apache

```json
{
  "webserver": "apache"
}
```

- **Image:** `php:{version}-apache`
- **Config:** `.stackvo/apache.conf` veya auto-generated
- **Template:** `core/templates/servers/apache/default.conf`

### Caddy

```json
{
  "webserver": "caddy"
}
```

- **Image:** `caddy:latest`
- **Config:** `.stackvo/Caddyfile` veya auto-generated
- **Template:** `core/templates/servers/caddy/Caddyfile`

---

## Proje Dizin Yapısı

### Temel Yapı

```
projects/myproject/
├── stackvo.json          # Proje konfigürasyonu (ZORUNLU)
├── public/                 # Document root
│   └── index.php
├── src/                    # Kaynak kodlar
├── vendor/                 # Composer dependencies
└── composer.json
```

### Özel Konfigürasyonlarla

```
projects/myproject/
├── stackvo.json
├── .stackvo/             # Özel konfigürasyonlar (opsiyonel)
│   ├── nginx.conf          # Özel Nginx config
│   ├── php.ini             # Özel PHP config
│   └── php-fpm.conf        # Özel PHP-FPM config
├── public/
│   └── index.php
└── ...
```
