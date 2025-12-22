# CLI Komutları Referansı

Stackvo CLI komutlarının tam referansı. Bu sayfa, generate, up, down, restart, ps, logs, pull, doctor, install ve uninstall komutlarının syntax'ını, parametrelerini, seçeneklerini ve kullanım örneklerini detaylı olarak açıklamaktadır. Ayrıca environment variables, exit codes ve Docker Compose eşdedeğerleri de içermektedir.

## Kurulum

```bash
./cli/stackvo.sh install
```

Kurulumdan sonra `stackvo` komutu sistem genelinde kullanılabilir.

---

## Komutlar

### generate

Konfigürasyon dosyalarını üretir.

**Syntax:**
```bash
./cli/stackvo.sh generate [MODE] [OPTIONS]
```

**Modes:**
- (boş) - Tümünü üret
- `projects` - Sadece projeleri üret
- `services` - Sadece servisleri üret

**Options:**
- `--uninstall-tools` - Tools konfigürasyonlarını kaldır

**Örnekler:**
```bash
# Tümünü üret
./cli/stackvo.sh generate

# Sadece projeleri üret
./cli/stackvo.sh generate projects

# Sadece servisleri üret
./cli/stackvo.sh generate services

# Tools'u kaldır
./cli/stackvo.sh generate --uninstall-tools
```

**Çıktı Dosyaları:**
- `generated/stackvo.yml`
- `generated/docker-compose.dynamic.yml`
- `generated/docker-compose.projects.yml`
- `core/traefik/dynamic/routes.yml`
- `core/generated/configs/*`

---

### up

Tüm servisleri başlatır.

**Syntax:**
```bash
./cli/stackvo.sh up [OPTIONS]
```

**Options:**
- `-d, --detach` - Arka planda çalıştır (varsayılan)
- `--build` - Image'ları yeniden build et
- `--force-recreate` - Container'ları yeniden oluştur

**Örnekler:**
```bash
# Tüm servisleri başlat
./cli/stackvo.sh up

# Build ile başlat
./cli/stackvo.sh up --build

# Force recreate
./cli/stackvo.sh up --force-recreate
```

**Eşdeğer Docker Compose:**
```bash
docker compose -f generated/stackvo.yml \
  -f generated/docker-compose.dynamic.yml \
  -f generated/docker-compose.projects.yml \
  up -d
```

---

### down

Tüm servisleri durdurur.

**Syntax:**
```bash
./cli/stackvo.sh down [OPTIONS]
```

**Options:**
- `-v, --volumes` - Volume'ları da sil
- `--remove-orphans` - Orphan container'ları kaldır

**Örnekler:**
```bash
# Servisleri durdur
./cli/stackvo.sh down

# Volume'larla birlikte durdur
./cli/stackvo.sh down -v

# Orphan'ları kaldır
./cli/stackvo.sh down --remove-orphans
```

---

### restart

Servisleri yeniden başlatır.

**Syntax:**
```bash
./cli/stackvo.sh restart [SERVICE...]
```

**Örnekler:**
```bash
# Tüm servisleri yeniden başlat
./cli/stackvo.sh restart

# Belirli servisleri yeniden başlat
docker compose -f generated/stackvo.yml \
  -f generated/docker-compose.dynamic.yml \
  restart mysql redis
```

---

### ps

Çalışan servisleri listeler.

**Syntax:**
```bash
./cli/stackvo.sh ps [OPTIONS]
```

**Options:**
- `-a, --all` - Tüm container'ları göster (durdurulmuş olanlar dahil)
- `--format` - Çıktı formatı

**Örnekler:**
```bash
# Çalışan servisleri listele
./cli/stackvo.sh ps

# Tüm container'ları listele
./cli/stackvo.sh ps -a

# Custom format
./cli/stackvo.sh ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
```

**Çıktı:**
```
NAME                      STATUS              PORTS
stackvo-traefik         Up 2 hours          0.0.0.0:80->80/tcp
stackvo-mysql           Up 2 hours          0.0.0.0:3306->3306/tcp
stackvo-redis           Up 2 hours          0.0.0.0:6379->6379/tcp
```

---

### logs

Container loglarını görüntüler.

**Syntax:**
```bash
./cli/stackvo.sh logs [OPTIONS] [SERVICE...]
```

**Options:**
- `-f, --follow` - Logları canlı izle
- `--tail=N` - Son N satırı göster
- `--timestamps` - Zaman damgası ekle
- `--since` - Belirli bir zamandan sonraki loglar

**Örnekler:**
```bash
# Tüm logları göster
./cli/stackvo.sh logs

# MySQL loglarını izle
./cli/stackvo.sh logs -f mysql

# Son 100 satır
./cli/stackvo.sh logs --tail=100 mysql

# Zaman damgalı
./cli/stackvo.sh logs --timestamps mysql

# Son 1 saatteki loglar
./cli/stackvo.sh logs --since=1h mysql
```

---

### pull

Docker image'larını çeker.

**Syntax:**
```bash
./cli/stackvo.sh pull [SERVICE...]
```

**Örnekler:**
```bash
# Tüm image'ları çek
./cli/stackvo.sh pull

# Belirli image'ları çek
docker compose -f generated/stackvo.yml \
  -f generated/docker-compose.dynamic.yml \
  pull mysql redis
```

---

### doctor

Sistem sağlık kontrolü yapar.

**Syntax:**
```bash
stackvo doctor
```

**Kontroller:**
- ✓ Docker kurulu mu?
- ✓ Docker Compose kurulu mu?
- ✓ Docker daemon çalışıyor mu?
- ✓ Gerekli portlar açık mı?
- ✓ `.env` dosyası var mı?
- ✓ SSL sertifikaları var mı?
- ✓ `generated/` dizini var mı?

**Örnek Çıktı:**
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

Stackvo CLI'yi sisteme kurar.

**Syntax:**
```bash
./cli/stackvo.sh install
```

**Ne yapar:**
- `/usr/local/bin/stackvo` sembolik link oluşturur
- CLI'yi sistem genelinde kullanılabilir yapar

**Gereksinimler:**
- Sudo yetkisi

---

### uninstall

Stackvo'u kaldırır.

**Syntax:**
```bash
./cli/stackvo.sh uninstall
```

**Ne yapar:**
1. Tüm container'ları durdurur
2. Volume'ları siler (onay ister)
3. Network'ü siler
4. CLI sembolik linkini kaldırır

**Uyarı:** Bu işlem geri alınamaz!

---

## Environment Variables

CLI davranışını kontrol eden environment variable'lar:

### STACKVO_VERBOSE

Detaylı çıktı.

```bash
STACKVO_VERBOSE=true ./cli/stackvo.sh generate
```

### STACKVO_DRY_RUN

Komutları çalıştırmadan göster.

```bash
STACKVO_DRY_RUN=true ./cli/stackvo.sh generate
```

### ENV_FILE

Farklı .env dosyası kullan.

```bash
ENV_FILE=.env.production ./cli/stackvo.sh generate
```

---

## Exit Codes

| Code | Açıklama |
|------|----------|
| 0 | Başarılı |
| 1 | Genel hata |
| 2 | Kullanım hatası |
| 126 | Komut çalıştırılamadı |
| 127 | Komut bulunamadı |
| 130 | Ctrl+C ile iptal edildi |

---

