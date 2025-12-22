# Troubleshooting

Yaygın sorunlar ve çözümleri. Bu sayfa, Docker sorunlarından (daemon, permission, port çakışması) generator ve network sorunlarına, SSL/TLS ve container sorunlarından database ve web server sorunlarına, CLI ve volume sorunlarından acil durum senaryolarına kadar tüm yaygın sorunları ve adım adım çözümlerini detaylı olarak açıklamaktadır. Her sorun için semptom ve çözüm örnekleri içerir.

---

## Genel Sorun Giderme

### Sistem Kontrolü

```bash
# Stackvo doctor
stackvo doctor

# Docker kontrolü
docker --version
docker compose --version
docker ps

# Logları kontrol et
cat core/generator.log
```

---

## Docker Sorunları

### Docker daemon çalışmıyor

**Semptom:**
```
Cannot connect to the Docker daemon
```

**Çözüm:**
```bash
# Linux
sudo systemctl start docker
sudo systemctl enable docker

# macOS
open -a Docker

# WSL2
sudo service docker start
```

### Permission hatası

**Semptom:**
```
permission denied while trying to connect to the Docker daemon socket
```

**Çözüm:**
```bash
# Kullanıcıyı docker grubuna ekle
sudo usermod -aG docker $USER
newgrp docker

# Veya sudo ile çalıştır
sudo ./cli/stackvo.sh up
```

### Port çakışması

**Semptom:**
```
Bind for 0.0.0.0:3306 failed: port is already allocated
```

**Çözüm:**
```bash
# Hangi process kullanıyor?
sudo lsof -i :3306

# .env'de port değiştir
nano .env
# HOST_PORT_MYSQL=3307

./cli/stackvo.sh generate
./cli/stackvo.sh restart
```

---

## Generator Sorunları

### Generate hatası

**Semptom:**
```
Error generating docker-compose files
```

**Çözüm:**
```bash
# Verbose mode
STACKVO_VERBOSE=true ./cli/stackvo.sh generate

# Logları kontrol et
cat core/generator.log

# Template kontrolü
ls -la core/compose/
ls -la core/templates/
```

### stackvo.json parse hatası

**Semptom:**
```
Error parsing stackvo.json
```

**Çözüm:**
```bash
# JSON syntax kontrolü
cat projects/myproject/stackvo.json | jq .

# Örnek geçerli format
{
  "name": "myproject",
  "domain": "myproject.loc",
  "php": {"version": "8.2"},
  "webserver": "nginx",
  "document_root": "public"
}
```

---

## Network Sorunları

### Container'lar birbirini görmüyor

**Semptom:**
```
Could not connect to stackvo-mysql
```

**Çözüm:**
```bash
# Network kontrolü
docker network inspect stackvo-net

# Container network'e bağlı mı?
docker inspect stackvo-mysql | grep -A 10 Networks

# Ping testi
docker exec stackvo-php ping stackvo-mysql

# Network yeniden oluştur
./cli/stackvo.sh down
docker network rm stackvo-net
./cli/stackvo.sh generate
./cli/stackvo.sh up
```

### DNS çözümleme sorunu

**Semptom:**
```
Name or service not known
```

**Çözüm:**
```bash
# Container içinden DNS testi
docker exec stackvo-php nslookup stackvo-mysql
docker exec stackvo-php cat /etc/resolv.conf

# Docker DNS restart
sudo systemctl restart docker
```

---

## SSL/TLS Sorunları

### SSL sertifikası hatası

**Semptom:**
```
SSL certificate problem: self signed certificate
```

**Çözüm:**
```bash
# Sertifikaları yeniden oluştur
./cli/utils/generate-ssl-certs.sh

# Tarayıcıda sertifikayı kabul et
# Chrome: Advanced → Proceed to site
# Firefox: Advanced → Accept the Risk
```

### Traefik SSL hatası

**Semptom:**
```
Traefik cannot find SSL certificates
```

**Çözüm:**
```bash
# Sertifika yolunu kontrol et
ls -la core/certs/

# Traefik config kontrol et
cat core/traefik/traefik.yml

# Traefik restart
docker restart stackvo-traefik
```

---

## Container Sorunları

### Container başlamıyor

**Semptom:**
```
Container exited with code 1
```

**Çözüm:**
```bash
# Logları kontrol et
docker logs stackvo-mysql

# Container detayları
docker inspect stackvo-mysql

# Yeniden oluştur
docker compose up -d --force-recreate stackvo-mysql
```

### Container sürekli restart oluyor

**Semptom:**
```
Container is restarting continuously
```

**Çözüm:**
```bash
# Son 100 log satırı
docker logs --tail=100 stackvo-mysql

# Health check
docker inspect --format='{{.State.Health.Status}}' stackvo-mysql

# Container'ı durdur ve logları incele
docker stop stackvo-mysql
docker logs stackvo-mysql
```

---

## Database Sorunları

### MySQL bağlantı hatası

**Semptom:**
```
SQLSTATE[HY000] [2002] Connection refused
```

**Çözüm:**
```bash
# Container çalışıyor mu?
docker ps | grep mysql

# Bağlantı bilgileri
Host: stackvo-mysql  # NOT localhost!
Port: 3306             # Internal port
User: stackvo
Password: stackvo

# Network testi
docker exec stackvo-php nc -zv stackvo-mysql 3306
```

### PostgreSQL authentication hatası

**Semptom:**
```
FATAL: password authentication failed
```

**Çözüm:**
```bash
# .env kontrolü
cat .env | grep POSTGRES

# Doğru credentials
Host: stackvo-postgres
Port: 5432
User: stackvo
Password: root  # .env'deki POSTGRES_PASSWORD
```

### MongoDB connection timeout

**Semptom:**
```
MongoNetworkError: connection timed out
```

**Çözüm:**
```bash
# Container kontrolü
docker ps | grep mongo

# Connection string
mongodb://root:root@stackvo-mongo:27017/dbname?authSource=admin

# Network testi
docker exec stackvo-php nc -zv stackvo-mongo 27017
```

---

## Web Server Sorunları

### 404 Not Found

**Semptom:**
```
404 Not Found - nginx
```

**Çözüm:**
```bash
# Document root kontrolü
docker exec stackvo-myproject-web ls -la /var/www/html/public

# Nginx config
docker exec stackvo-myproject-web cat /etc/nginx/conf.d/default.conf

# Nginx syntax test
docker exec stackvo-myproject-web nginx -t

# Nginx reload
docker exec stackvo-myproject-web nginx -s reload
```

### 502 Bad Gateway

**Semptom:**
```
502 Bad Gateway - nginx
```

**Çözüm:**
```bash
# PHP-FPM çalışıyor mu?
docker ps | grep php

# PHP-FPM logları
docker logs stackvo-myproject-php

# FastCGI bağlantısı
docker exec stackvo-myproject-web nc -zv myproject-php 9000

# PHP-FPM restart
docker restart stackvo-myproject-php
```

### Permission denied

**Semptom:**
```
Permission denied: /var/www/html/storage
```

**Çözüm:**
```bash
# Host'ta permissions
sudo chown -R $USER:$USER projects/myproject

# Container içinde
docker exec stackvo-myproject-php chown -R www-data:www-data /var/www/html
docker exec stackvo-myproject-php chmod -R 775 /var/www/html/storage
```

---

## CLI Sorunları

### Command not found

**Semptom:**
```
stackvo: command not found
```

**Çözüm:**
```bash
# CLI kur
./cli/stackvo.sh install

# Veya tam yol kullan
./cli/stackvo.sh generate
```

### Script execution hatası

**Semptom:**
```
Permission denied: ./cli/stackvo.sh
```

**Çözüm:**
```bash
# Executable yap
chmod +x cli/stackvo.sh
chmod +x cli/commands/*.sh
chmod +x cli/lib/generators/*.sh
```

---

## Volume Sorunları

### Data kaybı

**Semptom:**
```
All database data is lost after restart
```

**Çözüm:**
```bash
# Volume'ları kontrol et
docker volume ls | grep stackvo

# Volume inspect
docker volume inspect stackvo_mysql-data

# Backup al
docker run --rm \
  -v stackvo_mysql-data:/data \
  -v $(pwd):/backup \
  ubuntu tar czf /backup/mysql-backup.tar.gz /data
```

### Volume mount hatası

**Semptom:**
```
Error response from daemon: invalid mount config
```

**Çözüm:**
```bash
# Absolute path kullan
volumes:
  - /absolute/path/to/projects:/var/www/html

# Relative path yerine
volumes:
  - ./projects:/var/www/html  # ❌ Yanlış
```

---

## Acil Durum

### Tüm sistemi sıfırla

```bash
# 1. Tüm container'ları durdur
./cli/stackvo.sh down -v

# 2. Network'ü sil
docker network rm stackvo-net

# 3. Generated dosyaları sil
rm -rf generated/*

# 4. Yeniden oluştur
./cli/stackvo.sh generate
./cli/stackvo.sh up
```

### Backup'tan geri yükle

```bash
# MySQL
docker exec -i stackvo-mysql mysql -u root -proot < backup.sql

# PostgreSQL
docker exec -i stackvo-postgres psql -U stackvo < backup.sql

# Volume
docker run --rm \
  -v stackvo_mysql-data:/data \
  -v $(pwd):/backup \
  ubuntu tar xzf /backup/mysql-backup.tar.gz -C /
```

---

## Hala Çözülmedi mi?

1. **GitHub Issues:** [Sorun bildir](https://github.com/stackvo/stackvo/issues/new)
2. **Discussions:** [Tartışmalara katıl](https://github.com/stackvo/stackvo/discussions)
3. **Support:** [Destek al](support.md)

**Issue açarken:**
- Hata mesajını ekleyin
- `stackvo doctor` çıktısını paylaşın
- Logları ekleyin
- Environment bilgilerini verin
