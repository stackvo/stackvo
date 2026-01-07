# FAQ - Sık Sorulan Sorular

Stackvo hakkında en sık sorulan sorular ve cevapları. Bu sayfa, genel sorulardan kurulum ve kullanıma, troubleshooting'den performance optimizasyonuna, güvenlikten servislere, Web UI'dan backup ve güncelleme işlemlerine kadar geniş bir yelpazede soruları kapsamaktadır. Hızlı çözümler ve pratik örnekler içerir.

---

## Genel Sorular

### Stackvo nedir?

Stackvo, Docker tabanlı, tamamen özelleştirilebilir ve modüler bir geliştirme ortamı yönetim sistemidir. 40+ servisi destekler ve pure Bash ile yazılmıştır.

### Stackvo ücretsiz mi?

Evet, Stackvo tamamen ücretsiz ve açık kaynaklıdır (MIT License).

### Hangi işletim sistemlerinde çalışır?

- Linux (Ubuntu, Debian, CentOS, Arch)
- macOS
- Windows (WSL2)

---

## Kurulum

### Docker kurulu değilse ne yapmalıyım?

[Installation Guide](../installation/index.md) sayfasındaki adımları takip edin.

### Kurulum sırasında hata alıyorum

```bash
# Sistem kontrolü
stackvo doctor

# Logları kontrol et
cat core/generator.log
```

### Port çakışması hatası alıyorum

`.env` dosyasında portları değiştirin:

```bash
HOST_PORT_POSTGRES=5433
HOST_PORT_PERCONA=3308
```

---

## Kullanım

### Yeni proje nasıl oluştururum?

```bash
# 1. Proje dizini
mkdir -p projects/myproject/public

# 2. stackvo.json
cat > projects/myproject/stackvo.json <<EOF
{
  "name": "myproject",
  "domain": "myproject.loc",
  "php": {"version": "8.2"},
  "webserver": "nginx",
  "document_root": "public"
}
EOF

# 3. Generate ve start
./core/cli/stackvo.sh generate projects
./core/cli/stackvo.sh up

# 4. Hosts
echo "127.0.0.1  myproject.loc" | sudo tee -a /etc/hosts
```

### Servis nasıl aktif ederim?

`.env` dosyasını düzenleyin:

```bash
SERVICE_REDIS_ENABLE=true
SERVICE_REDIS_VERSION=7.0
```

Sonra:

```bash
./core/cli/stackvo.sh generate
./core/cli/stackvo.sh up
```

### PHP versiyonunu nasıl değiştiririm?

`stackvo.json` dosyasında:

```json
{
  "php": {
    "version": "8.3"
  }
}
```

Sonra:

```bash
./core/cli/stackvo.sh generate projects
./core/cli/stackvo.sh restart
```

---

## Troubleshooting

### Container başlamıyor

```bash
# Logları kontrol et
docker logs stackvo-mysql

# Yeniden oluştur
./core/cli/stackvo.sh down
./core/cli/stackvo.sh generate
./core/cli/stackvo.sh up
```

### 404 hatası alıyorum

```bash
# Document root kontrolü
docker exec stackvo-myproject-web ls -la /var/www/html/public

# Nginx config kontrolü
docker exec stackvo-myproject-web nginx -t
```

### Database bağlantı hatası

```bash
# Container çalışıyor mu?
docker ps | grep mysql

# Network kontrolü
docker exec stackvo-php ping stackvo-mysql

# Bağlantı bilgileri
Host: stackvo-mysql
Port: 3306
User: stackvo
Password: stackvo
```

---

## Performance

### Sistem yavaş çalışıyor

```bash
# Resource kullanımı
docker stats

# Gereksiz container'ları kaldır
docker system prune -a
```

### Build süresi uzun

```bash
# Cache kullan
docker compose build --parallel

# Image'ları önceden çek
./core/cli/stackvo.sh pull
```

---

## Güvenlik

### Production'da kullanabilir miyim?

Evet, ancak:
- Güçlü şifreler kullanın
- SSL/TLS aktif edin
- Firewall kuralları ekleyin
- Gereksiz portları kapatın

### Şifreleri nasıl değiştiririm?

`.env` dosyasında:

```bash
SERVICE_MYSQL_ROOT_PASSWORD=$(openssl rand -base64 32)
SERVICE_RABBITMQ_DEFAULT_PASS=$(openssl rand -base64 32)
```

---

## Servisler

### Hangi veritabanları destekleniyor?

- MySQL (5.6 - 8.1)
- MariaDB (10.6)
- PostgreSQL (9.6 - 16)
- MongoDB (4.0 - 7.0)
- Cassandra
- Percona
- CouchDB
- Couchbase

### Redis Cluster nasıl kurarım?

Şu anda tek node Redis destekleniyor. Cluster için custom konfigürasyon gerekir.

### Elasticsearch nasıl kullanırım?

```bash
# .env
SERVICE_ELASTICSEARCH_ENABLE=true
SERVICE_KIBANA_ENABLE=true

./core/cli/stackvo.sh generate
./core/cli/stackvo.sh up
```

Erişim:
- Elasticsearch: http://localhost:9200
- Kibana: https://kibana.stackvo.loc

---

## Web UI

### Web UI'a erişemiyorum

```bash
# Container kontrolü
docker ps | grep stackvo-ui

# Hosts kontrolü
cat /etc/hosts | grep stackvo.loc

# Yeniden başlat
docker restart stackvo-ui
```

### API çalışmıyor

```bash
# PHP logları
docker logs stackvo-ui

# Permissions
docker exec stackvo-ui ls -la /var/www/html/api/
```

---

## Backup

### Veritabanı backup nasıl alırım?

**MySQL:**
```bash
docker exec stackvo-mysql mysqldump -u root -proot --all-databases > backup.sql
```

**PostgreSQL:**
```bash
docker exec stackvo-postgres pg_dumpall -U stackvo > backup.sql
```

**MongoDB:**
```bash
docker exec stackvo-mongo mongodump --username root --password root --out /backup
```

### Volume backup nasıl alırım?

```bash
docker run --rm \
  -v stackvo_mysql-data:/data \
  -v $(pwd):/backup \
  ubuntu tar czf /backup/mysql-backup.tar.gz /data
```

---

## Güncelleme

### Stackvo nasıl güncellenir?

```bash
# Pull
git pull origin main

# Yeniden generate
./core/cli/stackvo.sh generate

# Restart
./core/cli/stackvo.sh restart
```

### Image'lar nasıl güncellenir?

```bash
# Tüm image'ları güncelle
./core/cli/stackvo.sh pull

# Yeniden başlat
./core/cli/stackvo.sh up --force-recreate
```

---

## Diğer

### Birden fazla proje çalıştırabilir miyim?

Evet, sınırsız proje çalıştırabilirsiniz.

### Custom domain kullanabilir miyim?

Evet, `stackvo.json` dosyasında:

```json
{
  "domain": "myapp.local"
}
```

`/etc/hosts` dosyasına ekleyin:

```
127.0.0.1  myapp.local
```

### SSL sertifikası nasıl oluşturulur?

```bash
./core/cli/utils/generate-ssl-certs.sh
```

---

## Hala Sorunuz mu Var?

- [GitHub Discussions](https://github.com/stackvo/stackvo/discussions)
- [GitHub Issues](https://github.com/stackvo/stackvo/issues)
- [Support](support.md)
