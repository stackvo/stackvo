---
title: CLI Kullanımı
description: Stackvo CLI, tüm sistem yönetimi için ana araçtır. Bu kılavuz, CLI komutlarını nasıl kullanacağınızı gösterir.
---

# CLI Kullanımı

Stackvo CLI, tüm sistem yönetimi için ana araçtır. Bu kılavuz, generate, up, down, restart, ps, logs gibi temel komutların nasıl kullanılacağını, verbose mode ve dry run gibi ileri seviye özellikleri, yeni servis ekleme, proje oluşturma ve troubleshooting yöntemlerini detaylı olarak göstermektedir. CLI, Docker Compose komutlarını kolaylaştırır ve otomatikleştirir.

---

## Kurulum

### CLI'yi Sisteme Kurma

```bash
# Stackvo dizinine gidin
cd /path/to/stackvo

# CLI'yi kurun
./core/cli/stackvo.sh install
```

Bu komut, `stackvo` komutunu `/usr/local/bin/` dizinine sembolik link olarak ekler.

**Doğrulama:**
```bash
# Herhangi bir dizinden çalıştırın
stackvo --help
```

---

## Temel Komutlar

### generate

Konfigürasyon dosyalarını üretir.

```bash
# Tüm konfigürasyonları üret
./core/cli/stackvo.sh generate

# Sadece projeleri üret
./core/cli/stackvo.sh generate projects

# Sadece servisleri üret
./core/cli/stackvo.sh generate services
```

**Ne yapar:**
1. `.env` dosyasını okur
2. SSL sertifikaları oluşturur (yoksa)
3. `generated/stackvo.yml` oluşturur
4. `generated/docker-compose.dynamic.yml` oluşturur
5. `generated/docker-compose.projects.yml` oluşturur
6. `core/traefik/dynamic/routes.yml` oluşturur
7. Servis konfigürasyonları oluşturur

**Örnek Çıktı:**
```
✅ SSL certificates found
✅ Generated stackvo.yml
✅ Generated docker-compose.dynamic.yml
✅ Generated docker-compose.projects.yml
✅ Generated Traefik routes
✅ Generated 15 service configurations
✅ Generation completed!
```

### up

Starts services. By default, only starts core services (Traefik + UI).

**Syntax:**
```bash
./core/cli/stackvo.sh up [OPTIONS]
```

**Options:**
- (empty) - Minimal mode: Only core services (Traefik + UI)
- `--all` - Start all services and projects
- `--services` - Start core + all services
- `--projects` - Start core + all projects
- `--profile <name>` - Start core + specific profile

**Examples:**
```bash
# Minimal mode - Only Traefik + UI
./core/cli/stackvo.sh up

# Start all services and projects
./core/cli/stackvo.sh up --all

# Core + all services
./core/cli/stackvo.sh up --services

# Core + all projects
./core/cli/stackvo.sh up --projects

# Core + only MySQL
./core/cli/stackvo.sh up --profile mysql

# Core + specific project
./core/cli/stackvo.sh up --profile project-myproject

# Multiple profiles
./core/cli/stackvo.sh up --profile mysql --profile redis
```

**Verbose Output:**
```bash
# Verbose mode
STACKVO_VERBOSE=true ./core/cli/stackvo.sh up
```

**Starting Specific Services:**
```bash
# Use Docker Compose command directly
docker compose -f generated/stackvo.yml \
  -f generated/docker-compose.dynamic.yml \
  -f generated/docker-compose.projects.yml \
  up -d mysql redis
```

### down

Tüm servisleri durdurur.

```bash
./core/cli/stackvo.sh down
```

**Volume'ları da Silme:**
```bash
./core/cli/stackvo.sh down -v
```

**Orphan Container'ları Kaldırma:**
```bash
./core/cli/stackvo.sh down --remove-orphans
```

### restart

Servisleri yeniden başlatır.

```bash
# Tüm servisleri yeniden başlat
./core/cli/stackvo.sh restart

# Belirli servisleri yeniden başlat
docker compose -f generated/stackvo.yml \
  -f generated/docker-compose.dynamic.yml \
  restart mysql redis
```

### ps

Çalışan servisleri listeler.

```bash
./core/cli/stackvo.sh ps
```

**Örnek Çıktı:**
```
NAME                      STATUS              PORTS
stackvo-traefik         Up 2 hours          0.0.0.0:80->80/tcp, 0.0.0.0:443->443/tcp
stackvo-mysql           Up 2 hours          0.0.0.0:3306->3306/tcp
stackvo-redis           Up 2 hours          0.0.0.0:6379->6379/tcp
stackvo-rabbitmq        Up 2 hours          0.0.0.0:5672->5672/tcp
stackvo-project1-php    Up 2 hours          9000/tcp
stackvo-project1-web    Up 2 hours          80/tcp
```

### logs

Container loglarını görüntüler.

```bash
# Tüm logları izle
./core/cli/stackvo.sh logs

# Belirli servis logunu izle
./core/cli/stackvo.sh logs mysql

# Follow mode
./core/cli/stackvo.sh logs -f mysql

# Son 100 satır
./core/cli/stackvo.sh logs --tail=100 mysql

# Birden fazla servis
./core/cli/stackvo.sh logs mysql redis
```

**Zaman Damgası ile:**
```bash
./core/cli/stackvo.sh logs -f --timestamps mysql
```

### pull

Docker image'larını çeker.

```bash
./core/cli/stackvo.sh pull
```

**Belirli Image'ları Çekme:**
```bash
docker compose -f generated/stackvo.yml \
  -f generated/docker-compose.dynamic.yml \
  pull mysql redis
```

### doctor

Sistem sağlık kontrolü yapar.

```bash
stackvo doctor
```

**Kontrol Edilen Şeyler:**
- Docker kurulu mu?
- Docker Compose kurulu mu?
- Docker daemon çalışıyor mu?
- Gerekli portlar açık mı?
- `.env` dosyası var mı?
- SSL sertifikaları var mı?

### uninstall

Stackvo'u kaldırır.

```bash
./core/cli/stackvo.sh uninstall
```

**Ne yapar:**
1. Tüm container'ları durdurur
2. Volume'ları siler (onay ister)
3. Network'ü siler
4. CLI sembolik linkini kaldırır

---

## İleri Seviye Kullanım

### Verbose Mode

Detaylı çıktı için:

```bash
STACKVO_VERBOSE=true ./core/cli/stackvo.sh generate
STACKVO_VERBOSE=true ./core/cli/stackvo.sh up
```

### Dry Run

Komutları çalıştırmadan görmek için:

```bash
STACKVO_DRY_RUN=true ./core/cli/stackvo.sh generate
```

### Custom .env Dosyası

```bash
# Farklı bir .env dosyası kullan
cp .env .env.production
nano .env.production

# Generate ile kullan
ENV_FILE=.env.production ./core/cli/stackvo.sh generate
```

### Belirli Compose Dosyalarıyla Çalışma

```bash
# Sadece base layer
docker compose -f generated/stackvo.yml up -d

# Base + services
docker compose -f generated/stackvo.yml \
  -f generated/docker-compose.dynamic.yml up -d

# Tümü (varsayılan)
docker compose -f generated/stackvo.yml \
  -f generated/docker-compose.dynamic.yml \
  -f generated/docker-compose.projects.yml up -d
```

---

## Yaygın Senaryolar

### Yeni Servis Ekleme

```bash
# 1. .env dosyasını düzenle
nano .env

# Elasticsearch'ü aktif et
# SERVICE_ELASTICSEARCH_ENABLE=true

# 2. Konfigürasyonları yeniden üret
./core/cli/stackvo.sh generate

# 3. Servisleri yeniden başlat
./core/cli/stackvo.sh up
```

### Proje Ekleme

```bash
# 1. Proje dizini oluştur
mkdir -p projects/newproject/public

# 2. stackvo.json oluştur
cat > projects/newproject/stackvo.json <<EOF
{
  "name": "newproject",
  "domain": "newproject.loc",
  "php": {"version": "8.2"},
  "webserver": "nginx",
  "document_root": "public"
}
EOF

# 3. Test dosyası
echo "<?php phpinfo();" > projects/newproject/public/index.php

# 4. Projeleri yeniden üret
./core/cli/stackvo.sh generate projects

# 5. Servisleri yeniden başlat
./core/cli/stackvo.sh restart

# 6. Hosts dosyasını güncelle
echo "127.0.0.1  newproject.loc" | sudo tee -a /etc/hosts
```

### Servis Versiyonu Değiştirme

```bash
# 1. .env dosyasını düzenle
nano .env

# MySQL versiyonunu değiştir
# SERVICE_MYSQL_VERSION=8.0 → 8.1

# 2. Konfigürasyonları yeniden üret
./core/cli/stackvo.sh generate services

# 3. MySQL container'ını yeniden oluştur
docker compose -f generated/stackvo.yml \
  -f generated/docker-compose.dynamic.yml \
  up -d --force-recreate mysql
```

### Tüm Sistemi Sıfırlama

```bash
# 1. Tüm container'ları durdur ve sil
./core/cli/stackvo.sh down -v

# 2. Network'ü sil
docker network rm stackvo-net

# 3. Generated dosyaları sil
rm -rf generated/*

# 4. Yeniden üret
./core/cli/stackvo.sh generate

# 5. Başlat
./core/cli/stackvo.sh up
```

### Backup Alma

```bash
# MySQL backup
docker exec stackvo-mysql mysqldump -u root -proot --all-databases > backup.sql

# PostgreSQL backup
docker exec stackvo-postgres pg_dumpall -U stackvo > backup.sql

# MongoDB backup
docker exec stackvo-mongo mongodump --username root --password root --authenticationDatabase admin --out /backup

# Redis backup
docker exec stackvo-redis redis-cli SAVE
docker cp stackvo-redis:/data/dump.rdb ./redis-backup.rdb
```

---

## Troubleshooting

### Container Başlamıyor

```bash
# Logları kontrol et
./core/cli/stackvo.sh logs <container-name>

# Container detaylarını incele
docker inspect stackvo-<container-name>

# Konfigürasyonu yeniden üret
./core/cli/stackvo.sh generate
./core/cli/stackvo.sh down
./core/cli/stackvo.sh up
```

### Port Çakışması

```bash
# Hangi portu kullanan container'ı bul
docker ps --format "table {{.Names}}\t{{.Ports}}"

# .env dosyasında port değiştir
nano .env

# Yeniden üret ve başlat
./core/cli/stackvo.sh generate
./core/cli/stackvo.sh restart
```

### Permission Hatası

```bash
# Docker grubuna kullanıcı ekle
sudo usermod -aG docker $USER
newgrp docker

# Veya sudo ile çalıştır
sudo ./core/cli/stackvo.sh up
```

---

