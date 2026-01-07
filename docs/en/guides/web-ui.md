---
title: Web UI Kullanımı
description: Stackvo Web UI, servisleri ve projeleri görsel olarak yönetmenizi sağlar.
---

# Web UI Kullanımı

Stackvo Web UI, servisleri ve projeleri görsel olarak yönetmenizi sağlar. Bu kılavuz, dashboard'ın nasıl kullanılacağını, servis ve proje listelerinin nasıl görüntüleneğini, yeni proje oluşturma, düzenleme ve silme işlemlerini, Adminer, PhpMyAdmin, PhpPgAdmin gibi management araçlarına erişimi, log görüntülemeyi ve API endpoint'lerini detaylı olarak açıklamaktadır. Web UI, CLI'ya alternatif kullanıcı dostu bir arayüz sunar.

---

## Erişim

```
https://stackvo.loc
```

**Not:** `/etc/hosts` dosyasına domain eklemeyi unutmayın:
```
127.0.0.1  stackvo.loc
```

---

## Dashboard

### Ana Sayfa

Dashboard, sistemin genel durumunu gösterir:

- **Çalışan Servisler:** Aktif container sayısı
- **Projeler:** Toplam proje sayısı
- **CPU Kullanımı:** Sistem CPU kullanımı
- **Memory Kullanımı:** Sistem RAM kullanımı
- **Disk Kullanımı:** Docker volume disk kullanımı

### Hızlı Erişim

Dashboard'dan hızlıca erişebileceğiniz bölümler:

- **Services:** Servis listesi ve durumları
- **Projects:** Proje listesi ve yönetimi
- **Tools:** Management araçları
- **Logs:** Container logları
- **Settings:** Sistem ayarları

---

## Services Sekmesi

### Servis Listesi

Tüm servislerin listesini görüntüler:

| Servis | Durum | Version | URL | Actions |
|--------|-------|---------|-----|---------|
| MySQL | Running | 8.0 | mysql.stackvo.loc | Start/Stop/Restart |
| Redis | Running | 7.0 | - | Start/Stop/Restart |
| RabbitMQ | Running | 3 | rabbitmq.stackvo.loc | Start/Stop/Restart |

### Servis Detayları

Bir servise tıklayarak detayları görüntüleyin:

- **Container Name:** stackvo-mysql
- **Image:** mysql:8.0
- **Status:** Up 2 hours
- **Ports:** 0.0.0.0:3306->3306/tcp
- **Network:** stackvo-net
- **Volumes:** mysql-data
- **Environment Variables:** MYSQL_ROOT_PASSWORD, MYSQL_DATABASE, vb.

### Servis Kontrolleri

Her servis için:

- **Start:** Servisi başlat
- **Stop:** Servisi durdur
- **Restart:** Servisi yeniden başlat
- **Logs:** Logları görüntüle
- **Stats:** CPU, Memory, Network kullanımı

---

## Projects Sekmesi

### Proje Listesi

Tüm projelerin listesini görüntüler:

| Proje | Domain | PHP Version | Webserver | Status | Actions |
|-------|--------|-------------|-----------|--------|---------|
| project1 | project1.loc | 8.2 | nginx | Running | Open/Edit/Delete |
| laravel-app | laravel.loc | 8.2 | nginx | Running | Open/Edit/Delete |

### Yeni Proje Oluşturma

**New Project** butonuna tıklayın:

1. **Project Name:** myproject
2. **Domain:** myproject.loc
3. **PHP Version:** 8.2
4. **Webserver:** nginx
5. **Document Root:** public
6. **PHP Extensions:** pdo, pdo_mysql, mysqli, gd, curl, zip, mbstring

**Create** butonuna tıklayın.

**Not:** `/etc/hosts` dosyasını manuel güncellemeniz gerekir:
```bash
echo "127.0.0.1  myproject.loc" | sudo tee -a /etc/hosts
```

### Proje Düzenleme

Bir projenin **Edit** butonuna tıklayın:

- PHP versiyonunu değiştirin
- Webserver'ı değiştirin
- PHP extension'ları ekleyin/çıkarın
- Document root'u değiştirin

**Save** butonuna tıklayın.

### Proje Silme

Bir projenin **Delete** butonuna tıklayın:

1. Onay penceresi açılır
2. **Confirm** butonuna tıklayın
3. Proje container'ları durdurulur
4. Proje dizini silinir (opsiyonel)

**Not:** `/etc/hosts` dosyasından domain'i manuel kaldırmanız gerekir.

---

## Tools Sekmesi

### Management Araçları

Stackvo, çeşitli management araçları sunar:

#### Adminer

**URL:** `https://adminer.stackvo.loc`

Tüm veritabanları için universal management arayüzü.

**Bağlantı:**
- System: MySQL / PostgreSQL / MongoDB
- Server: stackvo-mysql / stackvo-postgres / stackvo-mongo
- Username: stackvo
- Password: stackvo
- Database: stackvo

#### PhpMyAdmin

**URL:** `https://phpmyadmin.stackvo.loc`

MySQL ve MariaDB için management arayüzü.

**Bağlantı:**
- Server: stackvo-mysql
- Username: stackvo
- Password: stackvo

#### PhpPgAdmin

**URL:** `https://phppgadmin.stackvo.loc`

PostgreSQL için management arayüzü.

#### PhpMongo

**URL:** `https://phpmongo.stackvo.loc`

MongoDB için management arayüzü.

#### PhpMemcachedAdmin

**URL:** `https://phpmemcachedadmin.stackvo.loc`

Memcached için management arayüzü.

#### OpCacheGUI

**URL:** `https://opcache.stackvo.loc`

PHP OPcache istatistikleri ve yönetimi.

#### Kafbat

**URL:** `https://kafbat.stackvo.loc`

Kafka için management arayüzü.

---

## Logs Sekmesi

### Container Logları

Tüm container'ların loglarını görüntüleyin:

**Filtreler:**
- **Container:** Belirli container seç
- **Level:** INFO, WARNING, ERROR
- **Time Range:** Son 1 saat, 24 saat, 7 gün

**Özellikler:**
- Real-time log streaming
- Search/filter
- Download logs
- Clear logs

### Log Görüntüleme

```
[2024-12-16 10:00:00] INFO: MySQL started successfully
[2024-12-16 10:00:01] INFO: Redis connected
[2024-12-16 10:00:02] WARNING: Slow query detected (2.5s)
[2024-12-16 10:00:03] ERROR: Connection refused to RabbitMQ
```

---

## Settings Sekmesi

### Global Settings

**.env Dosyası Düzenleme:**

UI üzerinden `.env` dosyasını düzenleyin:

- **Traefik Settings**
- **Default Project Settings**
- **Docker Network**
- **Security Settings**
- **Port Mappings**
- **Service Versions**

**Save** butonuna tıklayın ve `./core/cli/stackvo.sh generate` çalıştırın.

### System Information

- **Docker Version:** 24.0.7
- **Docker Compose Version:** 2.23.0
- **Stackvo Version:** 1.0.0
- **OS:** Ubuntu 22.04
- **Total Containers:** 15
- **Total Volumes:** 8
- **Total Networks:** 1

---

## API Endpoints

Web UI, aşağıdaki API endpoint'lerini kullanır:

### Services API

```
GET /api/services.php
```

Tüm servislerin listesini döner.

### Projects API

```
GET /api/projects.php
```

Tüm projelerin listesini döner.

### Docker Stats API

```
GET /api/docker-stats.php
```

Container istatistiklerini döner.

### Control API

```
POST /api/control.php
```

Container'ları kontrol eder (start/stop/restart).

**Payload:**
```json
{
  "action": "restart",
  "container": "stackvo-mysql"
}
```

### Create Project API

```
POST /api/create-project.php
```

Yeni proje oluşturur.

**Payload:**
```json
{
  "name": "myproject",
  "domain": "myproject.loc",
  "php_version": "8.2",
  "webserver": "nginx",
  "document_root": "public",
  "php_extensions": ["pdo", "pdo_mysql", "mysqli"]
}
```

### Delete Project API

```
POST /api/delete-project.php
```

Projeyi siler.

**Payload:**
```json
{
  "name": "myproject"
}
```

---

## Troubleshooting

### UI Açılmıyor

```bash
# Container durumunu kontrol et
docker ps | grep stackvo-ui

# Logları kontrol et
docker logs stackvo-ui

# Hosts dosyasını kontrol et
cat /etc/hosts | grep stackvo.loc

# Yeniden başlat
docker restart stackvo-ui
```

### API Hataları

```bash
# PHP loglarını kontrol et
docker logs stackvo-ui

# API endpoint'i test et
curl https://stackvo.loc/api/services.php

# Permissions kontrolü
docker exec stackvo-ui ls -la /var/www/html/api/
```

### Slow Performance

```bash
# Container stats
docker stats stackvo-ui

# Resource limitleri artır
# docker-compose.yml'de:
# resources:
#   limits:
#     memory: 512M
#     cpus: '1.0'
```

---

